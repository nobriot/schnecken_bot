use log::*;
use serde_json::Value as JsonValue;
use std::fs;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

use crate::chess;
use crate::chess::model::game_state::START_POSITION_FEN;
use crate::chess::model::piece::Color;
use crate::lichess;
use crate::lichess::api::*;

// -----------------------------------------------------------------------------
//  Macros

/// Executes an async function synchronously
macro_rules! execute_sync {
  ($func:expr) => {
    let _ = Runtime::new().unwrap().block_on($func);
  };
}

// Constants
const DEFAULT_USERNAME: &str = "schnecken_bot";
const LICHESS_PLAYERS_FILE_NAME: &str = "/assets/players_we_like.txt";

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
  /// * `json_value` - JSON payload received in the HTTP stream.
  ///
  fn on_incoming_challenge(&self, json_value: JsonValue) {
    // Check if it is a challenge generated by us.
    let challenger_id = json_value["challenger"]["id"]
      .as_str()
      .unwrap_or(self.username.as_str());
    if challenger_id == self.username.as_str() {
      debug!("Ignoring notification of our own challenge");
      return;
    }

    debug!("Incoming challenge JSON: {}", json_value);
    let challenger = json_value["challenger"]["name"]
      .as_str()
      .unwrap_or("Unknown challenger");
    let challenger_rating = json_value["challenger"]["rating"].as_u64().unwrap_or(0);
    let variant = json_value["variant"]["key"]
      .as_str()
      .unwrap_or("Unknown variant");
    let challenge_id = json_value["id"].as_str().unwrap_or("UnknownID").to_owned();
    let time_control_type = json_value["timeControl"]["type"]
      .as_str()
      .unwrap_or("unknown")
      .to_owned();

    info!("{challenger} would like to play with us! Challenge {challenge_id}");
    info!("{} is rated {} ", challenger, challenger_rating);

    // We do not play non-standard for now
    if variant != "standard" {
      info!("Ignoring challenge for variant {variant}. We play only standard for now.");
      let api_clone = self.api.clone();
      tokio::spawn(async move {
        api_clone
          .decline_challenge(&challenge_id, lichess::types::DECLINE_VARIANT)
          .await
      });
      return;
    }

    // We do not play infinitely long games either
    if time_control_type != "clock" {
      info!("Ignoring non-real-time challenge.");
      let api_clone = self.api.clone();
      tokio::spawn(async move {
        api_clone
          .decline_challenge(&challenge_id, lichess::types::DECLINE_TIME_CONTROL)
          .await
      });
      return;
    }

    // Do not take several games at a time for now:
    if self.games.lock().unwrap().len() > 0 {
      info!("Ignoring challenge as we are already playing");
      let api_clone = self.api.clone();
      tokio::spawn(async move {
        api_clone
          .decline_challenge(&challenge_id, lichess::types::DECLINE_LATER)
          .await
      });
      return;
    }

    // Else we just accept.
    let api = self.api.clone();
    tokio::spawn(async move { api.accept_challenge(&challenge_id).await });
  }

  //----------------------------------------------------------------------------
  // Game Stream handlers

  async fn play_on_game(&self, game_id: &str, game_state: JsonValue) -> Result<(), ()> {
    // Double check that the game is still alive and it's our turn
    let json = self.api.get_ongoing_games().await?;

    if json["nowPlaying"].as_array().is_none() {
      warn!("Cannot find the 'nowPlaying' array in ongoing games");
      return Err(());
    }

    let json_game_array = json["nowPlaying"].as_array().unwrap();
    let mut is_my_turn: bool = false;
    let mut seconds_left: u64 = 0;
    for json_game in json_game_array {
      let current_game_id = json_game["gameId"].as_str().unwrap();
      if current_game_id == game_id {
        is_my_turn = json_game["isMyTurn"].as_bool().unwrap_or(true);
        seconds_left = json_game["secondsLeft"].as_u64().unwrap_or(20);
        break;
      }
    }

    if false == is_my_turn {
      info!("Not our turn. Now relying on the stream to tell us when to play for game {game_id}");
      return Ok(());
    }

    info!("Trying to find a move for game id {game_id}");

    let moves = game_state["moves"].as_str().unwrap_or("Unknown move list");
    let mut increment_ms = game_state["winc"].as_f64().unwrap_or(0.0);
    if increment_ms > 60_000.0 {
      increment_ms = 60_000.0
    }
    let mut game_state = chess::model::game_state::GameState::default();
    game_state.apply_move_list(moves);

    let suggested_time_ms;
    if seconds_left < 10 {
      // Play as quick as possible if we have less than 10 seconds left
      suggested_time_ms = 30.0;
    } else {
      suggested_time_ms = (seconds_left as f64 / 90.0) * 1000.0 + increment_ms;
    }

    if let Ok(chess_move) =
      &chess::engine::core::play_move(&mut game_state, suggested_time_ms as u64)
    {
      info!("Playing move {} for game id {}", chess_move, game_id);
      self.api.make_move(game_id, chess_move, false).await;
    } else {
      warn!("Can't find a move... Let's offer draw");
      self.api.make_move(game_id, "", true).await;
    }

    Ok(())
  }

  // ------------------------
  // Others
  async fn already_playing(&self) -> bool {
    warn!("FIXME: This should be cached in the bot state");
    let json_response: JsonValue;
    if let Ok(json) = self.api.lichess_get("account/playing").await {
      json_response = json;
    } else {
      warn!("Error checking if we are already playing");
      return false;
    }

    if json_response["nowPlaying"].as_array().is_none() {
      warn!("Cannot find the 'nowPlaying' array in ongoing games");
      return false;
    }

    let json_game_array = json_response["nowPlaying"].as_array().unwrap();

    return json_game_array.len() > 0;
  }

  /// Checks if any of the players we like is online and sends a challenge.
  pub async fn play(&self) {
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
        //FIXME: Serialize this with serde JSON too.
        self.on_incoming_challenge(json_value["challenge"].to_owned());
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
    info!("Incoming stream event for Game ID {game_id}");
    if json_value["type"].as_str().is_none() {
      error!("No type for incoming stream event.");
      return;
    }

    match json_value["type"].as_str().unwrap() {
      "gameFull" => {
        info!("Full game state!");
        let clone = self.clone();
        tokio::spawn(async move {
          clone
            .play_on_game(&game_id.clone(), json_value["state"].clone())
            .await
        });
      },
      "gameState" => {
        info!("Game state update received.");
        debug!("{}", json_value);
        let clone = self.clone();
        tokio::spawn(async move {
          clone
            .play_on_game(&game_id.clone(), json_value.clone())
            .await
        });
      },
      "chatLine" => {
        info!("Incoming Message!");
        debug!("{}", json_value);
      },
      "opponentGone" => {
        let gone = json_value["gone"].as_bool().unwrap_or(false);
        if gone == true {
          info!("Opponent gone! We'll just claim victory now, you chicken!");
          let claim_win = json_value["claimWinInSeconds"].as_u64();
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
