use log::*;
use serde_json::Value as JsonValue;
use std::fs;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;

use crate::chess;
use crate::chess::model::board::Move;
use crate::chess::model::game_state::START_POSITION_FEN;
use crate::chess::model::piece::Color;
use crate::lichess;
use crate::lichess::api::*;

// -----------------------------------------------------------------------------
// Constants
const DEFAULT_USERNAME: &str = "schnecken_bot";
const LICHESS_PLAYERS_FILE_NAME: &str = "/assets/players_we_like.txt";
/// How many games do we want to play simultaneously ?
const NUMBER_OF_SIMULTANEOUS_GAMES: usize = 2;

// -----------------------------------------------------------------------------
// Types

#[derive(Debug, Clone)]
pub struct BotState {
  pub api: LichessApi,
  pub username: String,
  pub games: Arc<Mutex<Vec<BotGame>>>,
}

#[derive(Debug, Clone)]
pub struct GameClock {
  pub white_time: usize,
  pub white_increment: usize,
  pub black_time: usize,
  pub black_increment: usize,
}

#[derive(Debug, Clone)]
pub struct BotGame {
  /// Color played by the bot in the ongoing game
  pub color: Color,
  /// Start FEN
  pub start_fen: String,
  /// Short Lichess Game ID, used in URLs
  pub id: String,
  /// Whether it got started, ever
  pub has_moved: bool,
  /// If it is our turn or not
  pub is_my_turn: bool,
  /// list of moves with algebraic notation: "e2e4 e7e5 d2d4 f8b4 .."
  pub move_list: String,
  pub rated: bool,
  pub clock: GameClock,
}

impl BotState {
  /// Instantiates a new bot state, using a given api token for Lichess
  /// (to identify itself in the games, challenges, etc.)
  pub async fn new(api_token: &str) -> Self {
    let api = LichessApi {
      client: reqwest::Client::new(),
      token: String::from(api_token),
    };

    // Find out our username with the API token:
    let username = api
      .get_lichess_username()
      .await
      .unwrap_or(String::from(DEFAULT_USERNAME));

    BotState {
      api: api,
      username: username,
      games: Arc::new(Mutex::new(Vec::new())),
    }
  }

  /// Get the bot started with its activity on Lichess
  /// Will spawn a thread handling incoming stream events.
  pub fn start(&mut self) {
    info!("Starting the Lichess bot... ");
    info!("Watch it at: https://lichess.org/@/{}", self.username);
    // Start streaming incoming events
    // Okay this is quite ugly, not sure how to do better :-D
    let api_clone = self.api.clone();
    let bot_clone = self.clone();
    let handle = tokio::spawn(async move { api_clone.stream_incoming_events(&bot_clone).await });

    let bot_clone = self.clone();
    tokio::spawn(async move { BotState::restart_incoming_streams(handle, &bot_clone).await });

    // Start a thread that sends challenges with a given interval:
    // tokio::spawn(async { lichess::api::send_challenges_with_interval(3600).await });
  }

  /// Checks if the stream_incoming_events has died and restarts it if that's the case.
  ///
  /// ### Arguments
  ///
  /// * `handle` Thread handle that is supported to stream incoming streams
  /// * `bot`    Reference to the bot, so that we can use the API
  ///
  async fn restart_incoming_streams(mut handle: JoinHandle<Result<(), ()>>, bot: &BotState) {
    // Start streaming incoming events again if it stopped
    loop {
      tokio::time::sleep(tokio::time::Duration::from_millis(4000)).await;

      // Check if the thread has finished executing
      if handle.is_finished() == true {
        warn!("Event stream died! Restarting it");
        // The thread has finished, restart it
        let api_clone = bot.api.clone();
        let bot_clone = bot.clone();
        handle = tokio::spawn(async move { api_clone.stream_incoming_events(&bot_clone).await });
      }
    }
  }

  pub fn add_game(&self, game: BotGame) {
    // Wait to get our Mutex:
    let mut binding = self.games.lock().unwrap();
    let games: &mut Vec<BotGame> = binding.as_mut();

    for i in 0..games.len() {
      if games[i].id == game.id {
        debug!("Game ID {} already in the cache. Ignoring", game.id);
        return;
      }
    }
    debug!("Adding Game ID {} in the cache", &game.id);
    let game_id = game.id.clone();
    games.push(game);
    // Stream the game in a separate thread.
    let api_clone = self.api.clone();
    let bot_clone = self.clone();
    tokio::spawn(async move { api_clone.stream_game_state(&bot_clone, &game_id).await });
  }

  pub fn remove_game(&self, game_id: &str) {
    // Wait to get our Mutex:
    let mut binding = self.games.lock().unwrap();
    let games: &mut Vec<BotGame> = binding.as_mut();

    for i in 0..games.len() {
      if games[i].id == game_id {
        debug!("Removing Game ID {} as it is completed", game_id);
        games.remove(i);
        return;
      }
    }
    debug!("Could not removing Game ID {} as it is now known.", game_id);
  }

  //----------------------------------------------------------------------------
  // Stream handlers

  /// Handles incoming gameStart events
  ///
  /// ### Arguments
  ///
  /// * `json_value` - JSON payload received in the HTTP stream.
  ///
  fn on_game_start(&self, game: lichess::types::GameStart) {
    // Write a hello message
    let game_id = game.game_id.clone();
    let api_clone = self.api.clone();
    tokio::spawn(async move {
      api_clone
        .write_in_chat(game_id.as_str(), "Hey! Have fun!")
        .await
    });

    // Game started, we add it to our games and stream the game events
    let bot_game: BotGame = BotGame {
      color: game.color,
      start_fen: game.fen.unwrap_or(String::from(START_POSITION_FEN)),
      id: game.game_id,
      has_moved: game.has_moved,
      is_my_turn: game.is_my_turn,
      move_list: game.last_move.unwrap_or(String::new()),
      rated: game.rated,
      clock: GameClock {
        white_time: game.seconds_left,
        white_increment: 0,
        black_time: game.seconds_left,
        black_increment: 0,
      },
    };

    self.add_game(bot_game);
  }

  /// Handles incoming gameFinish  events
  ///
  /// ### Arguments
  ///
  /// * `json_value` - JSON payload received in the HTTP stream.
  ///
  fn on_game_end(&self, game: lichess::types::GameStart) {
    // We are not playing on this game anymore
    self.remove_game(&game.game_id);

    // Write a goodbye message
    let api_clone = self.api.clone();
    tokio::spawn(async move {
      api_clone
        .write_in_chat(&game.game_id, "Thanks for playing!")
        .await
    });
  }

  /// Handles incoming gameStart events
  ///
  /// ### Arguments
  ///
  /// * `challenge` - Challenge object received from Lichess.
  ///
  fn on_incoming_challenge(&self, challenge: lichess::types::Challenge) {
    // Check if it is a challenge generated by us.
    if challenge.challenger.id == self.username.as_str() {
      debug!("Ignoring notification of our own challenge");
      return;
    }

    debug!("Incoming challenge : {:?}", challenge);
    info!(
      "{} would like to play with us! Challenge {}",
      challenge.challenger.name, challenge.id
    );
    info!(
      "{} is rated {} ",
      challenge.challenger.name, challenge.challenger.rating
    );

    // We do not play non-standard for now
    if challenge.variant.key != lichess::types::VariantKey::Standard {
      info!(
        "Ignoring challenge for variant {:?}. We play only standard for now.",
        challenge.variant
      );
      let api_clone = self.api.clone();
      tokio::spawn(async move {
        api_clone
          .decline_challenge(&challenge.id, lichess::types::DECLINE_VARIANT)
          .await
      });
      return;
    }

    // We do not play infinitely long games either
    if challenge.time_control.control_type != lichess::types::TimeControlType::Clock {
      info!("Ignoring non-real-time challenge.");
      let api_clone = self.api.clone();
      tokio::spawn(async move {
        api_clone
          .decline_challenge(&challenge.id, lichess::types::DECLINE_TIME_CONTROL)
          .await
      });
      return;
    }

    // Do not take several games at a time for now:
    if self.games.lock().unwrap().len() >= NUMBER_OF_SIMULTANEOUS_GAMES {
      info!("Ignoring challenge as we are already playing too many games");
      let api_clone = self.api.clone();
      tokio::spawn(async move {
        api_clone
          .decline_challenge(&challenge.id, lichess::types::DECLINE_LATER)
          .await
      });
      return;
    }

    // Else we just accept.
    let api = self.api.clone();
    tokio::spawn(async move { api.accept_challenge(&challenge.id).await });
  }

  //----------------------------------------------------------------------------
  // Game Stream handlers

  async fn play_on_game(&self, game_id: &str) -> Result<(), ()> {
    let mut binding = self.games.lock().unwrap();
    let games: &mut Vec<BotGame> = binding.as_mut();

    let mut game_index = games.len() + 1;
    for i in 0..games.len() {
      if games[i].id == game_id {
        game_index = i;
        break;
      }
    }

    let game = games.get_mut(game_index);
    if game.is_none() {
      return Err(());
    }

    let game = game.unwrap();

    if game.is_my_turn == false {
      info!("Not our turn for Game ID {game_id}. Waiting.");
      return Ok(());
    }

    info!("Trying to find a move for game id {game_id}");
    let moves = &game.move_list;
    let (time_left, mut increment_ms) = match game.color {
      Color::White => (game.clock.white_time, game.clock.white_increment),
      Color::Black => (game.clock.black_time, game.clock.black_increment),
    };

    if increment_ms > 60_000 {
      increment_ms = 60_000;
    }
    let mut game_state = chess::model::game_state::GameState::default();
    game_state.apply_move_list(moves);

    let suggested_time_ms;
    if time_left < 10_000 {
      // Play as quick as possible if we have less than 10 seconds left
      suggested_time_ms = 30.0;
    } else {
      suggested_time_ms = (time_left as f64 / 90.0) + (increment_ms) as f64;
    }

    if let Ok(chess_move) =
      &chess::engine::core::play_move(&mut game_state, suggested_time_ms as u64)
    {
      info!("Playing move {} for game id {}", chess_move, game_id);
      let api_clone = self.api.clone();
      let game_id_clone = String::from(game_id);
      let chess_move_clone = String::from(chess_move);
      tokio::spawn(async move {
        api_clone
          .make_move(&game_id_clone, &chess_move_clone, false)
          .await
      });
      return Ok(());
    } else {
      warn!("Can't find a move... Let's offer draw");
      let api_clone = self.api.clone();
      let game_id_clone = String::from(game_id);
      tokio::spawn(async move { api_clone.make_move(&game_id_clone, "", true).await });
      return Ok(());
    }
  }

  // ------------------------
  // Others

  /// Checks if any of the players we like is online and sends a challenge.
  pub fn update_game_and_play(&self, game_state: lichess::types::GameState, game_id: &str) {
    let mut binding = self.games.lock().unwrap();
    let games: &mut Vec<BotGame> = binding.as_mut();

    let mut game_index = games.len() + 1;
    for i in 0..games.len() {
      if games[i].id == game_id {
        game_index = i;
        break;
      }
    }

    debug!("Data for GameState: {:?}", game_state);

    if let Some(game) = games.get_mut(game_index) {
      if game_state.status != lichess::types::GameStatus::Started {
        debug!("Game ID {game_id} is not started. Removing it from our list");
        self.remove_game(game_id);
        return;
      }
      game.move_list = game_state.moves;
      game.clock.white_time = game_state.wtime;
      game.clock.white_increment = game_state.winc;
      game.clock.black_time = game_state.btime;
      game.clock.black_increment = game_state.binc;

      // Update whether it is our turn
      let move_count = Move::string_to_vec(game.move_list.as_str()).len();
      match game.color {
        Color::White => {
          game.is_my_turn = move_count % 2 == 0;
        },
        Color::Black => {
          game.is_my_turn = move_count % 2 == 1;
        },
      }

      if game.is_my_turn == true {
        let clone = self.clone();
        let game_id_clone = String::from(game_id);
        tokio::spawn(async move { clone.play_on_game(&game_id_clone).await });
        return;
      }
    } else {
      //warn!("Cannot find our cached game for Game ID {game_id}. Won't play");
      return;
    }
  }

  /// Checks if any of the players we like is online and sends a challenge.
  ///
  pub async fn challenge_somebody(&self) {
    let player_list =
      fs::read_to_string(String::from(env!("CARGO_MANIFEST_DIR")) + LICHESS_PLAYERS_FILE_NAME)
        .unwrap();
    //let parameters = serde_json::json!({ "rated": true, "clock" : {"limit":180,"increment":0}, "color":"random", "variant":"standard" });
    let players = player_list.lines();

    for username in players {
      if self.api.is_online(username).await == true {
        info!("{username} is online. Sending a challenge!");
        if let Err(()) = self.api.send_challenge(username).await {
          info!("Error sending a challenge to {username}");
          continue;
        }
        break;
      }
    }
  }
}

impl EventStreamHandler for BotState {
  /// Handles incoming account/stream events for the bot.
  ///
  /// ### Arguments
  ///
  /// * `json_value` - JSON payload received in the HTTP stream.
  fn event_stream_handler(&self, json_value: JsonValue) {
    if json_value["type"].as_str().is_none() {
      error!("No type for incoming stream event.");
      return;
    }

    debug!("Event Stream payload: \n{}", json_value);

    match json_value["type"].as_str().unwrap() {
      "gameStart" => {
        info!("New game Started!");
        let result: Result<lichess::types::GameStart, serde_json::Error> =
          serde_json::from_value(json_value["game"].clone());
        if result.is_err() {
          let error = result.unwrap_err();
          warn!("Error deserializing GameStart event data !! {:?}", error);
        } else {
          self.on_game_start(result.unwrap());
        }
      },
      "gameFinish" => {
        info!("Game finished! ");
        let result: Result<lichess::types::GameStart, serde_json::Error> =
          serde_json::from_value(json_value["game"].clone());
        if result.is_err() {
          let error = result.unwrap_err();
          warn!("Error deserializing gameFinish event data !! {:?}", error);
        } else {
          self.on_game_end(result.unwrap());
        }
      },
      "challenge" => {
        info!("Incoming challenge!");
        let result: Result<lichess::types::Challenge, serde_json::Error> =
          serde_json::from_value(json_value["challenge"].clone());
        if result.is_err() {
          let error = result.unwrap_err();
          warn!("Error deserializing Challenge event data !! {:?}", error);
        } else {
          self.on_incoming_challenge(result.unwrap());
        }
      },
      "challengeCanceled" => {
        info!("Challenge cancelled ");
      },
      "challengeDeclined" => {
        info!("Challenge declined");
      },
      other => {
        // Ignore other events
        warn!("Received unknown streaming event: {}", other);
      },
    }
  }
}

impl GameStreamHandler for BotState {
  /// Handles incoming game events for the bot.
  ///
  /// https://lichess.org/api#tag/Bot/operation/botGameStream
  /// for the JSON payload
  ///
  /// ### Arguments
  ///
  /// * `json_value` JSON object with the event details.
  /// * `game_id`    Game ID
  ///
  fn game_stream_handler(&self, json_value: JsonValue, game_id: String) {
    debug!("Incoming stream event for Game ID {game_id}");
    if json_value["type"].as_str().is_none() {
      error!("No type for incoming stream event.");
      return;
    }

    match json_value["type"].as_str().unwrap() {
      "gameFull" => {
        debug!("Full game state!");
        let clone = self.clone();
        tokio::spawn(async move { clone.play_on_game(&game_id.clone()).await });
      },
      "gameState" => {
        debug!("Game state update received: {}", json_value);
        let result: Result<lichess::types::GameState, serde_json::Error> =
          serde_json::from_value(json_value);
        if result.is_err() {
          let error = result.unwrap_err();
          warn!("Error deserializing GameState data !! {:?}", error);
        } else {
          self.update_game_and_play(result.unwrap(), game_id.as_str());
        }
      },
      "chatLine" => {
        info!("Incoming Message: {}", json_value);
      },
      "opponentGone" => {
        let gone = json_value["gone"].as_bool().unwrap_or(false);
        if gone == true {
          info!("Opponent gone! We'll just claim victory as soon as possible!");
          if let Some(timeout) = json_value["claimWinInSeconds"].as_u64() {
            let api_clone = self.api.clone();
            tokio::spawn(async move {
              api_clone
                .claim_victory_after_timeout(timeout, &game_id.clone())
                .await
            });
          }
        } else {
          info!("Opponent is back!");
        }
      },
      other => {
        // Ignore other events
        warn!("Received unknown streaming game state: {}", other);
        warn!("{}", json_value);
      },
    }
    //debug!("JSON: {}", json_value);
  }
}
