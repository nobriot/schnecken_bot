use lazy_static::lazy_static;
use log::*;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::chess::model::board::Board;
use crate::chess::model::game_state::GamePhase;
use crate::chess::model::moves::*;
use crate::chess::model::tables::zobrist::BoardHash;

// How large do we want the cache to grow before we purge it.
const DEFAULT_CACHE_MAX_SIZE: usize = 100_000_000;

#[derive(Debug, Clone, Default)]
pub struct PositionCache {
  // List of moves available for a position
  pub move_list: Option<Vec<Move>>,
  // Evaluation for a position
  pub eval: Option<f32>,
  // game phase for a position
  pub game_phase: Option<GamePhase>,
  // List of variations from a position if a move is played
  pub variations: HashMap<Move, BoardHash>,
  // Number of checks for the position
  pub checks: Option<u8>,
}

pub struct EngineCache {
  positions: Arc<Mutex<HashMap<BoardHash, PositionCache>>>,
  killer_moves: Arc<Mutex<HashSet<Move>>>,
}

impl EngineCache {
  /// Instantiate a new EngineCache object
  ///
  pub fn new() -> Self {
    EngineCache {
      positions: Arc::new(Mutex::new(HashMap::new())),
      killer_moves: Arc::new(Mutex::new(HashSet::new())),
    }
  }

  // ---------------------------------------------------------------------------
  // Position cached data

  /// Adds a PositionCache object to a give board configuration
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :           Hash value for the board configuration
  /// * `position_cache` :  Position cache for the board configuration
  ///
  pub fn add(&self, board: &BoardHash, position_cache: PositionCache) {
    if self.len() > DEFAULT_CACHE_MAX_SIZE {
      // Ideally we should purge old entries
      error!("Clearing the cache due to large size.");
      self.clear();
    }
    self
      .positions
      .lock()
      .unwrap()
      .insert(*board, position_cache);
  }

  /// Returns the number of positions saved in the cache.
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  ///
  /// ### Return value
  ///
  /// Number of positions saved with a PositionCache in the EngineCache
  ///
  pub fn len(&self) -> usize {
    return self.positions.lock().unwrap().len();
  }

  /// Erases all the position caches and killer moves
  ///
  pub fn clear(&self) {
    self.positions.lock().unwrap().clear();
    self.killer_moves.lock().unwrap().clear();
  }

  /// Checks if a position has a PositionCache entry
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :           Hash value for the board configuration
  ///
  /// ### Return value
  ///
  /// True if the board hash has a PositionCache in the EngineCache. False otherwise
  ///
  pub fn has_key(&self, board: &BoardHash) -> bool {
    return self.positions.lock().unwrap().contains_key(board);
  }

  /// Checks if a position has a known move list
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :           Hash value for the board configuration
  ///
  /// ### Return value
  ///
  /// True if the board hash has a move list in the PositionCache in the EngineCache. False otherwise
  ///
  pub fn has_move_list(&self, board: &BoardHash) -> bool {
    return self
      .positions
      .lock()
      .unwrap()
      .get(board)
      .unwrap_or(&PositionCache::default())
      .move_list
      .is_some();
  }

  /// Retrieves the list of legal moves for a position, if present in the EngineCache
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :           Hash value for the board configuration
  ///
  /// ### Return value
  ///
  /// Optional Vector of moves for the board position
  ///
  pub fn get_move_list(&self, board: &BoardHash) -> Option<Vec<Move>> {
    return self
      .positions
      .lock()
      .unwrap()
      .get(board)
      .unwrap_or(&PositionCache::default())
      .move_list
      .clone();
  }

  /// Configures the list of legal moves for a position in the EngineCache
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :           Hash value for the board configuration
  /// * `move_list` :       List of moves to save in the PositionCache
  ///
  pub fn set_move_list(&self, board: &BoardHash, move_list: &[Move]) {
    if !self.has_key(board) {
      self.add(board, PositionCache::default());
    }

    if let Some(entry) = self.positions.lock().unwrap().get_mut(board) {
      entry.move_list = Some(move_list.to_owned());
    } else {
      error!("Error updating move list in the cache for board {board}");
    }
  }

  /// Retrieves the evaluation for a position, if present in the EngineCache
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :           Hash value for the board configuration
  ///
  /// ### Return value
  ///
  /// Optional evaluation value
  ///
  pub fn get_eval(&self, board: &BoardHash) -> Option<f32> {
    return self
      .positions
      .lock()
      .unwrap()
      .get(board)
      .unwrap_or(&PositionCache::default())
      .eval;
  }

  /// Sets the evaluation for a position in the EngineCache
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :           Hash value for the board configuration
  /// * `eval` :            Evaluation for the position to save in the PositionCache
  ///
  pub fn set_eval(&self, board: &BoardHash, eval: f32) {
    if !self.has_key(board) {
      self.add(board, PositionCache::default());
    }

    if let Some(entry) = self.positions.lock().unwrap().get_mut(board) {
      entry.eval = Some(eval);
    } else {
      error!("Error updating eval in the cache for hash {board}");
    }
  }

  /// Retrieves the game phase for a position, if present in the EngineCache
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :           Hash value for the board configuration
  ///
  /// ### Return value
  ///
  /// Optional game phase value
  ///
  pub fn get_game_phase(&self, board: &BoardHash) -> Option<GamePhase> {
    return self
      .positions
      .lock()
      .unwrap()
      .get(board)
      .unwrap_or(&PositionCache::default())
      .game_phase;
  }

  /// Sets the game phase for a position in the EngineCache
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :           Hash value for the board configuration
  /// * `game_phase` :      Game phase for the position to save in the PositionCache
  ///
  pub fn set_game_phase(&self, board: &BoardHash, game_phase: GamePhase) {
    if !self.has_key(board) {
      self.add(board, PositionCache::default());
    }

    if let Some(entry) = self.positions.lock().unwrap().get_mut(board) {
      entry.game_phase = Some(game_phase);
    } else {
      error!("Error updating Game Phase in the cache for hash {board}");
    }
  }

  /// Adds a continuation for a position in the EngineCache
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :           Hash value for the board configuration
  /// * `chess_move` :      Chess move applied on the board configuration
  /// * `resulting_board` : Hash value for the board after the chess move applied on the board configuration
  ///
  pub fn add_variation(&self, board: &BoardHash, chess_move: &Move, resulting_board: &BoardHash) {
    if !self.has_key(board) {
      self.add(board, PositionCache::default());
    }

    if let Some(entry) = self.positions.lock().unwrap().get_mut(board) {
      entry.variations.insert(*chess_move, *resulting_board);
    } else {
      error!("Error updating Variations in the cache for board {board}");
    }
  }

  /// Retrives the number of checks for a board configuration
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :  Candidate move to look up in the EngineCache
  ///
  /// ### Return value
  ///
  /// True if the `candidate_move` is present in the list of Killer moves
  ///
  pub fn get_checks(&self, board: &BoardHash) -> Option<u8> {
    return self
      .positions
      .lock()
      .unwrap()
      .get(board)
      .unwrap_or(&PositionCache::default())
      .checks;
  }

  /// Sets the number of checks for a board configuration
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :  Candidate move to look up in the EngineCache
  /// * `checks` : Number of checks to set in the EngineCache for the position
  ///
  pub fn set_checks(&self, board: &BoardHash, checks: u8) {
    if !self.has_key(board) {
      self.add(board, PositionCache::default());
    }

    if let Some(entry) = self.positions.lock().unwrap().get_mut(board) {
      entry.checks = Some(checks);
    } else {
      error!("Error updating eval in the cache for hash {board}");
    }
  }

  // ---------------------------------------------------------------------------
  // Position independant cached data

  /// Gets the list of known continuations for a position in the EngineCache
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :           Hash value for the board configuration
  ///
  /// ### Return value
  ///
  /// HashMap of moves/boards for continuations.
  ///
  pub fn get_variations(&self, board: &BoardHash) -> HashMap<Move, BoardHash> {
    return self
      .positions
      .lock()
      .unwrap()
      .get(board)
      .unwrap_or(&PositionCache::default())
      .variations
      .clone();
  }

  /// Adds a killer move in the EngineCache
  /// This is not dependant on positions, and should be cleared when the engine moves to another position
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `killer_move` :     Killer Move to add in the EngineCache
  ///
  pub fn add_killer_move(&self, killer_move: &Move) {
    self.killer_moves.lock().unwrap().insert(*killer_move);
  }

  /// Removes all killer moves from the EngineCache
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  ///
  pub fn clear_killer_moves(&self) {
    self.killer_moves.lock().unwrap().clear();
  }

  /// Checks if a move is a known killer move in the EngineCache
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `candidate_move` :  Candidate move to look up in the EngineCache
  ///
  /// ### Return value
  ///
  /// True if the `candidate_move` is present in the list of Killer moves
  ///
  pub fn is_killer_move(&self, candidate_move: &Move) -> bool {
    return self.killer_moves.lock().unwrap().contains(candidate_move);
  }
}

lazy_static! {
  static ref ENGINE_CACHE: EngineCache = EngineCache::new();
}

pub fn get_engine_cache() -> &'static EngineCache {
  &ENGINE_CACHE
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {

  use super::*;
  use crate::chess::model::game_state::GameState;

  #[test]
  fn test_cache_has_key() {
    let engine_cache: EngineCache = EngineCache::new();

    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    let game_state = GameState::from_fen(fen);
    engine_cache.add(&game_state.board.hash, PositionCache::default());

    // Same position, different move number
    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 46";
    let game_state = GameState::from_fen(fen);
    engine_cache.add(&game_state.board.hash, PositionCache::default());

    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 0 0";
    let game_state = GameState::from_fen(fen);
    assert_eq!(true, engine_cache.has_key(&game_state.board.hash));

    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/7P b - - 8 43";
    let game_state = GameState::from_fen(fen);
    assert_eq!(false, engine_cache.has_key(&game_state.board.hash));
  }

  #[test]
  fn test_cache_get_set_data() {
    let engine_cache: EngineCache = EngineCache::new();

    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    let game_state = GameState::from_fen(fen);

    // Empty cache:
    assert_eq!(0, engine_cache.len());
    assert_eq!(false, engine_cache.has_move_list(&game_state.board.hash));
    assert_eq!(None, engine_cache.get_move_list(&game_state.board.hash));
    assert_eq!(None, engine_cache.get_checks(&game_state.board.hash));

    // Now add the data:
    let position_cache = PositionCache {
      move_list: Some(Vec::new()),
      eval: Some(20.0),
      game_phase: Some(GamePhase::Opening),
      variations: HashMap::new(),
      checks: Some(2),
    };

    engine_cache.add(&game_state.board.hash, position_cache);
    assert_eq!(1, engine_cache.len());

    // Read the cache
    assert_eq!(
      Vec::<Move>::new(),
      engine_cache.get_move_list(&game_state.board.hash).unwrap()
    );
    assert_eq!(Some(20.0), engine_cache.get_eval(&game_state.board.hash));
    assert_eq!(
      Some(GamePhase::Opening),
      engine_cache.get_game_phase(&game_state.board.hash)
    );
    assert_eq!(Some(2), engine_cache.get_checks(&game_state.board.hash));

    // Add manually:
    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/7P b - - 8 43";
    let game_state = GameState::from_fen(fen);
    assert_eq!(None, engine_cache.get_move_list(&game_state.board.hash));
    assert_eq!(None, engine_cache.get_eval(&game_state.board.hash));
    assert_eq!(None, engine_cache.get_game_phase(&game_state.board.hash));

    // First the move list
    let mut move_list = Vec::new();
    move_list.push(Move::from_string("h3g7"));
    move_list.push(Move::from_string("a1a8Q"));

    assert_eq!(false, engine_cache.has_move_list(&game_state.board.hash));
    engine_cache.set_move_list(&game_state.board.hash, &move_list);
    assert_eq!(true, engine_cache.has_move_list(&game_state.board.hash));

    assert_eq!(
      move_list,
      engine_cache.get_move_list(&game_state.board.hash).unwrap()
    );
    assert_eq!(None, engine_cache.get_eval(&game_state.board.hash));
    assert_eq!(None, engine_cache.get_game_phase(&game_state.board.hash));

    // Then the eval:
    engine_cache.set_eval(&game_state.board.hash, 99.9);
    assert_eq!(
      move_list,
      engine_cache.get_move_list(&game_state.board.hash).unwrap()
    );
    assert_eq!(Some(99.9), engine_cache.get_eval(&game_state.board.hash));
    assert_eq!(None, engine_cache.get_game_phase(&game_state.board.hash));

    // Finally the game phase
    engine_cache.set_game_phase(&game_state.board.hash, GamePhase::Endgame);
    assert_eq!(
      move_list,
      engine_cache.get_move_list(&game_state.board.hash).unwrap()
    );
    assert_eq!(Some(99.9), engine_cache.get_eval(&game_state.board.hash));
    assert_eq!(
      Some(GamePhase::Endgame),
      engine_cache.get_game_phase(&game_state.board.hash)
    );

    // Clear the cache:
    engine_cache.clear();
    assert_eq!(0, engine_cache.len());
    assert_eq!(None, engine_cache.get_move_list(&game_state.board.hash));
    assert_eq!(false, engine_cache.has_move_list(&game_state.board.hash));
    assert_eq!(None, engine_cache.get_eval(&game_state.board.hash));
    assert_eq!(None, engine_cache.get_game_phase(&game_state.board.hash));
    assert_eq!(None, engine_cache.get_checks(&game_state.board.hash));
  }
}
