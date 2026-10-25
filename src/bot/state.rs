use super::games::game::Game;
use super::matchmaking;
use crate::bot::games::games::BotGames;
use crate::bot::games::handle::GameHandle;
use lichess::api::LichessApi;
use lichess::error::LichessError;
use lichess::types::{Clock, User};
// Other libraries from our repo
use log::*;
use rand::Rng;
use serde_json::Value as JsonValue;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;

// -----------------------------------------------------------------------------
// Constants
const DEFAULT_USERNAME: &str = "schnecken_bot";
/// Minimum interval between two lookups of new opponents in the online bots
/// list, so that we do not spam bots we do not know with challenges.
const ONLINE_BOTS_LOOKUP_INTERVAL: Duration = Duration::from_secs(30 * 60);
/// How many online bots to fetch when looking for new opponents
const ONLINE_BOTS_TO_FETCH: usize = 200;

// -----------------------------------------------------------------------------
// Types

pub type BotStateRef = &'static BotState;

// -----------------------------------------------------------------------------
// Main BotState

pub struct BotState {
  /// Lichess API
  pub api:           &'static LichessApi,
  /// Cache of our lichess username
  username:          String,
  /// List of ongoing games
  games:             BotGames,
  /// Timestamp of the last game we played
  last_game:         Arc<Mutex<std::time::Instant>>,
  /// Bool value indicating if the bot should exit
  exit:              Arc<Mutex<bool>>,
  /// Engine configuration from config file / CLI
  pub engine_config: crate::config::EngineConfig,
  /// IDs of the users we follow on Lichess
  following:         Mutex<Vec<String>>,
  /// Last time we looked for new opponents in the online bots list
  last_bots_lookup:  Mutex<Option<Instant>>,
}

// We pass bot state references accross threads
unsafe impl Sync for BotState {}

impl BotState {
  /// Instantiates a new bot state, using a given api token for Lichess
  /// (to identify itself in the games, challenges, etc.)
  pub async fn new(api_token: &str, engine_config: crate::config::EngineConfig) -> BotStateRef {
    let api: &'static _ = Box::leak(Box::new(LichessApi::new(api_token)));
    let bot_games = BotGames::new(api);

    // Find out our username with the API token:
    let mut username = String::from(DEFAULT_USERNAME);

    let account_info = api.get_profile().await;
    if let Ok(json) = account_info
       && let Some(id) = json["id"].as_str()
    {
      username = String::from(id);
    }

    let bot_state_ref: &'static _ =
      Box::leak(Box::new(BotState { api,
                                    username,
                                    games: bot_games,
                                    last_game: Arc::new(Mutex::new(std::time::Instant::now())),
                                    exit: Arc::new(Mutex::new(false)),
                                    engine_config,
                                    following: Mutex::new(Vec::new()),
                                    last_bots_lookup: Mutex::new(None) }));
    bot_state_ref
  }

  /// Checks if the bot was configured to exit.
  pub fn should_exit(&self) -> bool {
    let exit = self.exit.lock().unwrap();
    *exit
  }

  /// Indicates the bot that it should stop and exit everything.
  pub fn request_exit(&self, resign: bool) {
    self.games.terminate_all(resign);
    let mut exit = self.exit.lock().unwrap();
    *exit = true;
  }

  /// Waits for the game loops to finish (e.g. after `request_exit`), so that
  /// their last requests (resign, chat) are sent before the program exits.
  ///
  /// ### Arguments
  ///
  /// * `timeout` - Maximum time to wait
  pub async fn wait_for_games(&self, timeout: Duration) {
    let start = Instant::now();
    while !self.games.all_finished() {
      if start.elapsed() > timeout {
        warn!("Games did not finish within {}s, exiting anyway", timeout.as_secs());
        return;
      }
      tokio::time::sleep(Duration::from_millis(100)).await;
    }
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

    // Clean up the list of users we follow
    tokio::spawn(async { self.update_following().await });

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
  async fn restart_incoming_streams(self: BotStateRef,
                                    mut handle: JoinHandle<Result<(), LichessError>>) {
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
        // The thread rarely dies, however, sometimes the HTTP stream stops and
        // we do not receive chunks anymore. Look up if the bot appears
        // offline, and if so, restart the incoming event stream
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
    let game_handle: GameHandle = Game::new(game, self.api, &self.engine_config);
    self.games.add(game_handle);
  }

  /// Handles incoming gameFinish events
  ///
  /// ### Arguments
  ///
  /// * `json_value` - JSON payload received in the HTTP stream.
  fn on_game_end(self: BotStateRef, game: lichess::types::GameStart) {
    // Update the last game time-stamp
    self.update_last_game_timestamp();

    // Remove the game from the list of games
    self.games.remove(&game.game_id);

    // Check if this opponent is somebody we would like to follow
    let opponent_id = game.opponent.id;
    tokio::spawn(async move { self.consider_following(&opponent_id).await });
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
    info!("{} is rated {} ", challenge.challenger.name, challenge.challenger.rating);

    // Just print our crosstable
    let crosstable =
      self.api.get_crosstable(self.username.as_str(), &challenge.challenger.id, false).await;

    if let Some(crosstable) = crosstable {
      info!("Crosstable {} - {} vs {} - {} ({} games)",
            self.username.as_str(),
            crosstable.user_1_score,
            crosstable.user_2_score,
            challenge.challenger.id,
            crosstable.games);
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

    // Do not accept if we are already playing this opponent
    if self.games.is_playing_opponent(&challenge.challenger.id) {
      info!("Declining challenge from {} — already playing them", challenge.challenger.name);
      tokio::spawn(async move {
        self.api.decline_challenge(&challenge.id, lichess::types::DECLINE_LATER).await
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

  /// Challenges another bot: one of the bots we follow if any is available
  /// (preferably with a similar rating), else from time to time a new bot
  /// with a similar rating from the list of online bots.
  /// We never challenge humans.
  pub async fn challenge_somebody(&self) {
    let clock_setting = rand::rng().random_range(0..40);
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
    let speed = matchmaking::speed_for_clock(&clock);

    let Some(us) = self.get_our_user().await else {
      warn!("Could not get our own profile, not sending challenges");
      return;
    };
    let Some(our_rating) = us.rating(speed) else {
      warn!("We have no {speed:?} rating, not sending challenges");
      return;
    };

    // First, try the bots we follow
    let following = match self.api.get_following().await {
      Ok(following) => following,
      Err(_) => {
        warn!("Could not get the list of users we follow");
        Vec::new()
      },
    };
    self.set_following(&following);

    let followed_bots: Vec<User> =
      following.into_iter()
               .filter(|user| matchmaking::is_challengeable_bot(&us, user))
               .collect();
    let candidates = self.available_bots(followed_bots).await;
    if self.challenge_one_of(candidates, our_rating, &clock).await {
      return;
    }

    // None of them is available, look for new opponents once in a while
    {
      let mut last_lookup = self.last_bots_lookup.lock().unwrap();
      if last_lookup.is_some_and(|t| t.elapsed() < ONLINE_BOTS_LOOKUP_INTERVAL) {
        debug!("None of the bots we follow is available. Not looking for new opponents yet");
        return;
      }
      *last_lookup = Some(Instant::now());
    }

    info!("None of the bots we follow is available, looking for new opponents online");
    let online_bots = match self.api.get_online_bots(ONLINE_BOTS_TO_FETCH).await {
      Ok(bots) => bots,
      Err(_) => {
        warn!("Could not get the list of online bots");
        return;
      },
    };
    let following = self.following.lock().unwrap().clone();
    let new_opponents = matchmaking::select_new_opponents(&us, online_bots, &following, speed);
    let candidates = self.available_bots(new_opponents).await;
    if !self.challenge_one_of(candidates, our_rating, &clock).await {
      info!("Could not find any bot to challenge");
    }
  }

  /// Keeps the bots that are online and not already playing
  async fn available_bots(&self, bots: Vec<User>) -> Vec<User> {
    if bots.is_empty() {
      return bots;
    }
    let ids: Vec<&str> = bots.iter().map(|bot| bot.id.as_str()).collect();
    let Ok(statuses) = self.api.get_users_status(&ids).await else {
      warn!("Could not get the status of the bots");
      return Vec::new();
    };

    bots.into_iter()
        .filter(|bot| {
          statuses.iter().any(|s| s.id.eq_ignore_ascii_case(&bot.id) && s.online && !s.playing)
        })
        .collect()
  }

  /// Tries to challenge one of the candidates, closest rating first.
  ///
  /// ### Returns
  ///
  /// True if a challenge was sent
  async fn challenge_one_of(&self, candidates: Vec<User>, our_rating: u32, clock: &Clock) -> bool {
    let speed = matchmaking::speed_for_clock(clock);
    for bot in matchmaking::challenge_order(candidates, our_rating, speed) {
      // Should never happen, candidates are filtered, but we really do not want
      // to spam humans
      if !bot.is_bot() {
        warn!("Refusing to challenge {}, not a bot", bot.username);
        continue;
      }
      info!("Challenging {} ({:?} rating {:?}, we have {our_rating})",
            bot.username,
            speed,
            bot.rating(speed));
      if self.api.send_challenge(&bot.id, clock).await.is_ok() {
        return true;
      }
      info!("Error sending a challenge to {}", bot.username);
    }
    false
  }

  /// Gets our own user information (ratings, etc.)
  async fn get_our_user(&self) -> Option<User> {
    let json = self.api.get_profile().await.ok()?;
    serde_json::from_value::<User>(json).ok()
  }

  /// Saves the list of IDs of the users we follow
  fn set_following(&self, users: &[User]) {
    *self.following.lock().unwrap() = users.iter().map(|user| user.id.clone()).collect();
  }

  /// Checks if we are following a user
  fn is_following(&self, user_id: &str) -> bool {
    self.following.lock().unwrap().iter().any(|id| id.eq_ignore_ascii_case(user_id))
  }

  /// Cleans up the list of users we follow: unfollows bots whose rating is too
  /// far from ours (never humans), and makes sure we follow the users we
  /// always want to follow.
  async fn update_following(&self) {
    let (Some(us), Ok(following)) = (self.get_our_user().await, self.api.get_following().await)
    else {
      warn!("Could not get our profile or the users we follow. Skipping follow list cleanup");
      return;
    };
    info!("We are following {} users", following.len());

    let mut kept = Vec::new();
    for user in following {
      if matchmaking::should_unfollow(&us, &user) {
        info!("Unfollowing {}, rating too different from ours", user.username);
        if self.api.unfollow_user(&user.id).await.is_ok() {
          continue;
        }
      }
      kept.push(user);
    }
    self.set_following(&kept);

    for user_id in matchmaking::ALWAYS_FOLLOW {
      if !self.is_following(user_id) && self.api.follow_user(user_id).await.is_ok() {
        self.following.lock().unwrap().push(user_id.to_string());
      }
    }
  }

  /// Starts following an opponent if we played many games together and have a
  /// similar rating.
  async fn consider_following(&self, opponent_id: &str) {
    if opponent_id.eq_ignore_ascii_case(&self.username) || self.is_following(opponent_id) {
      return;
    }

    let Some(crosstable) = self.api.get_crosstable(&self.username, opponent_id, false).await else {
      return;
    };
    if crosstable.games <= matchmaking::FOLLOW_MIN_GAMES {
      return;
    }

    let (Some(us), Ok(opponent)) =
      (self.get_our_user().await, self.api.get_user(opponent_id).await)
    else {
      return;
    };
    if matchmaking::should_follow(&us, &opponent, crosstable.games) {
      info!("Following {} ({} games played together)", opponent.username, crosstable.games);
      if self.api.follow_user(&opponent.id).await.is_ok() {
        self.following.lock().unwrap().push(opponent.id);
      }
    }
  }

  pub fn stream_incoming_events(self: BotStateRef, json_value: JsonValue) {
    if json_value["type"].as_str().is_none() {
      error!("No type for incoming stream event. JSON: {json_value}");

      if let Some(error) = json_value["error"].as_str()
         && error.contains("token")
      {
        error!("Token error. Exiting the bot.");
        self.request_exit(true);
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
