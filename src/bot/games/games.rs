// Internal
use super::handle::GameHandle;
use super::message::GameMessage;
use lichess::api::LichessApi;
use lichess::traits::GameStreamHandler;
// External
use log::*;
use std::sync::{Arc, Mutex};

// -----------------------------------------------------------------------------
// Constants
const NUMBER_OF_SIMULTANEOUS_GAMES: usize = 4;

#[derive(Debug)]
pub struct BotGames {
  games: Arc<Mutex<Vec<Arc<GameHandle>>>>,
  api:   &'static LichessApi,
}

impl BotGames {
  /// Creates a new `BotGames` instance with a predefined capacity
  /// for simultaneous games.
  ///
  /// # Returns
  ///
  /// A new instance of `BotGames`.
  pub fn new(api: &'static LichessApi) -> Self {
    let games = Vec::with_capacity(NUMBER_OF_SIMULTANEOUS_GAMES);
    let games = Arc::new(Mutex::new(games));
    Self { games, api }
  }

  /// Checks if the current set of games has reached its capacity.
  pub fn is_full(&self) -> bool {
    let games = self.games.lock().unwrap();
    games.len() >= NUMBER_OF_SIMULTANEOUS_GAMES
  }

  /// Checks if we are playing
  pub fn is_empty(&self) -> bool {
    let games = self.games.lock().unwrap();
    games.is_empty()
  }

  /// Number of ongoing games
  pub fn len(&self) -> usize {
    let games = self.games.lock().unwrap();
    games.len()
  }

  /// Adds a new game to the set of games if there is capacity.
  ///
  /// # Arguments
  ///
  /// * `game` - A `lichess::types::GameStart` instance representing the game to
  ///   be added.
  pub fn add(&self, game_handle: GameHandle) {
    let mut games = self.games.lock().unwrap();
    if games.len() >= NUMBER_OF_SIMULTANEOUS_GAMES {
      error!("Error: Cannot add more games. The set of games is full.");
      return;
    }

    // Start the game stream
    let api = self.api.clone();
    let handle = game_handle.clone();
    let _ = tokio::spawn(async move {
      let _ =
        api.stream_game_state_with_callback(&handle.id, &handle, GameHandle::game_stream_handler)
           .await;
    });

    games.push(Arc::new(game_handle));
  }

  /// Removes a game from the set of games based on the game ID.
  ///
  /// # Arguments
  ///
  /// * `game_id` - A string slice representing the ID of the game to be
  ///   removed.
  pub fn remove(&self, game_id: &str) {
    let mut games = self.games.lock().unwrap();
    games.retain(|handle| handle.id != game_id);
  }

  /// Remove finished games from our list that we do not need anymore.
  pub fn purge(&mut self) {
    let mut games = self.games.lock().unwrap();
    games.retain(|handle| !handle.is_over());
  }

  pub fn terminate_all(&self) {
    let games = self.games.lock().unwrap();
    for handle in games.iter() {
      let _ = handle.tx.send(GameMessage::Terminate);
    }
  }

  /// Checks if any of the players we like is online and sends a challenge.
  pub fn on_game_update(&self, game_state: lichess::types::GameState, game_id: &str) {
    let handle = self.get_handle(game_id);
    if handle.is_none() {
      error!("Error: Cannot find game with ID {}", game_id);
      return;
    }
    let handle = handle.unwrap();
    let _ = handle.tx.send(GameMessage::Update(game_state));
  }

  /// Gets a game handle based on the game ID.
  pub fn get_handle(&self, game_id: &str) -> Option<Arc<GameHandle>> {
    let games = self.games.lock().unwrap();
    games.iter().find(|handle| handle.id == game_id).cloned()
  }
}
