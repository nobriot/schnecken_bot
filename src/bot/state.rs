use super::games::game::Game;
use crate::bot::games::games::BotGames;
use crate::bot::games::handle::GameHandle;
use lichess::api::LichessApi;
use lichess::types::Clock;
// Other libraries from our repo
use log::*;
use rand::Rng;
use serde_json::Value as JsonValue;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;

// -----------------------------------------------------------------------------
// Constants
const DEFAULT_USERNAME: &str = "schnecken_bot";
const LICHESS_PLAYERS: &str =
  include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/players_we_like.txt"));

// -----------------------------------------------------------------------------
// Types

pub type BotStateRef = &'static BotState;

// -----------------------------------------------------------------------------
// Main BotState

pub struct BotState {
  /// Lichess API
  pub api:   &'static LichessApi,
  /// Cache of our lichess username
  username:  String,
  /// List of ongoing games
  games:     BotGames,
  /// Timestamp of the last game we played
  last_game: Arc<Mutex<std::time::Instant>>,
  /// Bool value indicating if the bot should exit
  exit:      Arc<Mutex<bool>>,
}

// We pass bot state references accross threads
unsafe impl Sync for BotState {}

impl BotState {
  /// Instantiates a new bot state, using a given api token for Lichess
  /// (to identify itself in the games, challenges, etc.)
  pub async fn new(api_token: &str) -> BotStateRef {
    let api: &'static _ = Box::leak(Box::new(LichessApi::new(api_token)));
    let bot_games = BotGames::new(api);

    // Find out our username with the API token:
    let mut username = String::from(DEFAULT_USERNAME);

    let account_info = api.get_profile().await;
    if account_info.is_ok() {
      let json = account_info.unwrap();
      if json["id"].as_str().is_some() {
        username = String::from(json["id"].as_str().unwrap());
      }
    }

    let bot_state_ref: &'static _ =
      Box::leak(Box::new(BotState { api,
                                    username,
                                    games: bot_games,
                                    last_game: Arc::new(Mutex::new(std::time::Instant::now())),
                                    exit: Arc::new(Mutex::new(false)) }));
    bot_state_ref
  }

  /// Checks if the bot was configured to exit.
  pub fn should_exit(&self) -> bool {
    let exit = self.exit.lock().unwrap();
    *exit
  }

  /// Indicates the bot that it should stop and exit everything.
  pub fn request_exit(&self, resign: bool) {
    if resign {
      self.games.terminate_all();
    }
    let mut exit = self.exit.lock().unwrap();
    *exit = true;
  }

  /// Update last_game time-stamp
  pub fn update_last_game_timestamp(&self) {
    let mut last_game = self.last_game.lock().unwrap();
    *last_game = std::time::Instant::now();
  }

  /// Get the bot started with its activity on Lichess
  /// Will spawn a thread handling incoming stream events.
  pub fn start(self: BotStateRef) {
    info!("Starting the Lichess bot... ");
    info!("Watch it at: https://lichess.org/@/{}", self.username);
    // Start streaming incoming events
    // Okay this is quite ugly, not sure how to do better :-D

    let bot_ref: BotStateRef = self;
    let handle = tokio::spawn(async move {
      bot_ref.api
             .stream_incoming_events_with_callback(bot_ref, Self::stream_incoming_events)
             .await
    });

    tokio::spawn(async { self.restart_incoming_streams(handle).await });

    // Start a thread that sends challenges with a given interval:
    tokio::spawn(async { self.send_challenges_with_interval(7200).await });
  }

  /// Checks if the stream_incoming_events has died and restarts it if that's
  /// the case.
  ///
  /// ### Arguments
  ///
  /// * `handle` Thread handle that is supported to stream incoming streams
  /// * `bot`    Reference to the bot, so that we can use the API
  async fn restart_incoming_streams(self: BotStateRef, mut handle: JoinHandle<Result<(), ()>>) {
    // Start streaming incoming events again if it stopped
    loop {
      tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

      // Check if the thread has finished executing
      if handle.is_finished() {
        warn!("Event stream died! Restarting it");
        // The thread has finished, restart it
        handle = tokio::spawn(async {
          self.api
              .stream_incoming_events_with_callback(self, BotState::stream_incoming_events)
              .await
        });
      } else if !self.api.is_online(&self.username).await {
        // The thread rarely dies, however, sometimes the HTTP stream stops and we do
        // not receive chunks anymore. Look up if the bot appears offline, and
        // if so, restart the incoming event stream
        warn!("Bot seems offline, restarting event stream");
        handle.abort(); // This will trigger the is_finished() to be to true at
                        // the next iteration.
      }
    }
  }

  async fn send_challenges_with_interval(self: BotStateRef, interval: u64) {
    // Start streaming incoming events again if it stopped
    loop {
      tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

      if !self.games.is_empty() {
        let mut last_game = self.last_game.lock().unwrap();
        *last_game = std::time::Instant::now();
        continue;
      }

      // Check if we have not played in a while, and if not, send a challenge
      let last_game = self.last_game.lock().unwrap();
      if last_game.elapsed().as_secs() > interval {
        info!("Let's challenge somebody");
        let bot_ref: BotStateRef = self;
        tokio::spawn(async move { bot_ref.challenge_somebody().await });
      }
    }
  }

  //----------------------------------------------------------------------------
  // Stream handlers

  /// Handles incoming gameStart events
  ///
  /// ### Arguments
  ///
  /// * `json_value` - JSON payload received in the HTTP stream.
  fn on_game_start(&self, game: lichess::types::GameStart) {
    // Update the last game time-stamp
    self.update_last_game_timestamp();

    // Create a game handle and start the game
    let game_handle: GameHandle = Game::new(game, self.api);
    self.games.add(game_handle);
  }

  /// Handles incoming gameFinish events
  ///
  /// ### Arguments
  ///
  /// * `json_value` - JSON payload received in the HTTP stream.
  fn on_game_end(&self, game: lichess::types::GameStart) {
    // Update the last game time-stamp
    self.update_last_game_timestamp();

    // Remove the game from the list of games
    self.games.remove(&game.game_id);
  }

  /// Handles incoming gameStart events
  ///
  /// ### Arguments
  ///
  /// * `challenge` - Challenge object received from Lichess.
  async fn on_incoming_challenge(self: BotStateRef, challenge: lichess::types::Challenge) {
    // Check if it is a challenge generated by us.
    if challenge.challenger.id == self.username.as_str() {
      debug!("Ignoring notification of our own challenge");
      return;
    }

    debug!("Incoming challenge : {:?}", challenge);
    info!("{} would like to play with us! Challenge {}",
          challenge.challenger.name, challenge.id);
    info!("{} is rated {} ",
          challenge.challenger.name, challenge.challenger.rating);

    // Just print our crosstable
    let crosstable =
      self.api.get_crosstable(self.username.as_str(), &challenge.challenger.id, false).await;

    if let Some(scores) = crosstable {
      info!("Crosstable {} - {} vs {} - {}",
            self.username.as_str(),
            scores.0,
            scores.1,
            challenge.challenger.id);
    }

    // We do not play non-standard for now
    if challenge.variant.key != lichess::types::VariantKey::Standard {
      info!("Ignoring challenge for variant {:?}. We play only standard for now.",
            challenge.variant);

      tokio::spawn(async move {
        self.api.decline_challenge(&challenge.id, lichess::types::DECLINE_VARIANT).await
      });
      return;
    }

    // If we play other bots, it should be rated
    if !challenge.rated
       && challenge.challenger.title.is_some()
       && challenge.challenger.title.as_ref().unwrap() == "BOT"
    {
      info!("Ignoring casual challenge from another bot {:?}. We play only rated here.",
            challenge.challenger);
      tokio::spawn(async move {
        self.api.decline_challenge(&challenge.id, lichess::types::DECLINE_RATED).await
      });
      return;
    }

    // We do not play infinitely long games either
    if challenge.time_control.control_type != lichess::types::TimeControlType::Clock {
      info!("Ignoring non-real-time challenge.");
      tokio::spawn(async move {
        self.api.decline_challenge(&challenge.id, lichess::types::DECLINE_TIME_CONTROL).await
      });
      return;
    }

    // Do not accept we are playing at capacity
    if self.games.is_full() {
      info!("Ignoring challenge as we are already playing too many games");
      tokio::spawn(async move {
        self.api.decline_challenge(&challenge.id, lichess::types::DECLINE_LATER).await
      });
      return;
    }

    // Else we just accept.
    tokio::spawn(async move { self.api.accept_challenge(&challenge.id).await });
  }

  // ------------------------
  // Others

  /// Checks if any of the players we like is online and sends a challenge.
  pub async fn challenge_somebody(&self) {
    let clock_setting = rand::thread_rng().gen_range(0..40);
    let clock: Clock = match clock_setting {
      0..=15 => Clock { initial:   60,
                        increment: 0,
                        totaltime: None, },
      16..=35 => Clock { initial:   180,
                         increment: 0,
                         totaltime: None, },
      _ => Clock { initial:   600,
                   increment: 0,
                   totaltime: None, },
    };

    let players: Vec<&str> = LICHESS_PLAYERS.lines().collect();

    for username in players {
      // TODO: Shuffle the list correctly
      if rand::thread_rng().gen_range(0..2) == 0 {
        continue;
      }
      if self.api.is_online(username).await {
        info!("{username} is online. Sending a challenge!");
        if let Err(()) = self.api.send_challenge(username, &clock).await {
          info!("Error sending a challenge to {username}");
          continue;
        }
        break;
      }
    }
  }

  pub fn stream_incoming_events(self: BotStateRef, json_value: JsonValue) {
    if json_value["type"].as_str().is_none() {
      error!("No type for incoming stream event. JSON: {json_value}");

      if let Some(error) = json_value["error"].as_str() {
        if error.contains("token") {
          error!("Token error. Exiting the bot.");
          self.request_exit(true);
        }
      }
      return;
    }

    debug!("Event Stream payload: \n{}", json_value);

    match json_value["type"].as_str().unwrap() {
      "gameStart" => {
        info!("New game Started!");
        let result: Result<lichess::types::GameStart, serde_json::Error> =
          serde_json::from_value(json_value["game"].clone());
        if let Err(error) = result {
          warn!("Error deserializing GameStart event data !! {:?}", error);
          println!("JSON object: {}", json_value["game"]);
        } else {
          let game_start = result.unwrap();
          // debug!("Parsed data: {:?}", game_start);
          self.on_game_start(game_start);
        }
      },
      "gameFinish" => {
        info!("Game finished! ");
        let result: Result<lichess::types::GameStart, serde_json::Error> =
          serde_json::from_value(json_value["game"].clone());
        if let Err(error) = result {
          warn!("Error deserializing gameFinish event data !! {:?}", error);
          warn!("JSON object: {}", json_value["game"]);
        } else {
          let game_end = result.unwrap();
          // debug!("Parsed data: {:?}", game_end);
          self.on_game_end(game_end);
        }
      },
      "challenge" => {
        info!("Incoming challenge!");
        let result: Result<lichess::types::Challenge, serde_json::Error> =
          serde_json::from_value(json_value["challenge"].clone());
        if let Err(error) = result {
          warn!("Error deserializing Challenge event data !! {:?}", error);
          warn!("JSON object: {}", json_value["challenge"]);
        } else {
          let bot_ref: BotStateRef = self;
          tokio::spawn(async move { bot_ref.on_incoming_challenge(result.unwrap()).await });
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
