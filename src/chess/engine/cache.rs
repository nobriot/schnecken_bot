use lazy_static::lazy_static;
use log::*;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::chess::model::board::Board;
use crate::chess::model::game_state::GamePhase;
use crate::chess::model::game_state::GameStatus;
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
  // Whether we reached a board based game-over
  pub status: Option<GameStatus>,
}

#[derive(Debug, Clone)]
pub struct EvalTree {
  // White's min guaranteed
  pub alpha: f32,
  // Black's max guaranteed
  pub beta: f32,
}

// -----------------------------------------------------------------------------
// Default implementations for our EvalTree

impl Default for EvalTree {
  fn default() -> Self {
    EvalTree {
      alpha: f32::MIN,
      beta: f32::MAX,
    }
  }
}

#[derive(Clone, Debug)]
pub struct EngineCache {
  // List of position/board properties that we cache and do not recompute. Including static position eval
  positions: Arc<Mutex<HashMap<BoardHash, PositionCache>>>,
  // Tree analysis, including alpha/beta
  tree: Arc<Mutex<HashMap<BoardHash, EvalTree>>>,
  // List of killer moves that we've met recently during the analysis
  killer_moves: Arc<Mutex<HashSet<Move>>>,
}

impl EngineCache {
  /// Instantiate a new EngineCache object
  ///
  pub fn new() -> Self {
    EngineCache {
      positions: Arc::new(Mutex::new(HashMap::new())),
      tree: Arc::new(Mutex::new(HashMap::new())),
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
    self.tree.lock().unwrap().clear();
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
  pub fn has_position_key(&self, board: &BoardHash) -> bool {
    return self.positions.lock().unwrap().contains_key(board);
  }

  /// Checks if a position has a EvalTree entry
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
  pub fn has_tree_key(&self, board: &BoardHash) -> bool {
    return self.tree.lock().unwrap().contains_key(board);
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
    if !self.has_position_key(board) {
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
    if !self.has_position_key(board) {
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
    if !self.has_position_key(board) {
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
    if !self.has_position_key(board) {
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
    if !self.has_position_key(board) {
      self.add(board, PositionCache::default());
    }

    if let Some(entry) = self.positions.lock().unwrap().get_mut(board) {
      entry.checks = Some(checks);
    } else {
      error!("Error updating checks in the cache for hash {board}");
    }
  }

  /// Gets the game status based on the board configuration
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board_hash` :  Board configuration to look up in the cache
  ///
  /// ### Return value
  ///
  /// GameStatus that has been saved in the cache, else None.
  ///
  pub fn get_game_status(&self, board_hash: &BoardHash) -> Option<GameStatus> {
    return self
      .positions
      .lock()
      .unwrap()
      .get(board_hash)
      .unwrap_or(&PositionCache::default())
      .status;
  }

  /// Sets the game status based on the board configuration
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board_hash` :  Board configuration to look up
  /// * `game_status` : Game status for the board configuration
  ///
  pub fn set_game_status(&self, board_hash: &BoardHash, game_status: GameStatus) {
    if !self.has_position_key(board_hash) {
      self.add(board_hash, PositionCache::default());
    }

    if let Some(entry) = self.positions.lock().unwrap().get_mut(board_hash) {
      entry.status = Some(game_status);
    } else {
      error!("Error updating game status in the cache for hash {board_hash}");
    }
  }

  // ---------------------------------------------------------------------------
  // Alpha-Beta pruning data

  /// Gets the alpha value for the board configuration
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :  Board configuration to look up
  ///
  /// ### Return value
  ///
  /// Alpha value that has been cached, f32::MIN if none.
  ///
  pub fn get_alpha(&self, board: &BoardHash) -> f32 {
    return self
      .tree
      .lock()
      .unwrap()
      .get(board)
      .unwrap_or(&EvalTree::default())
      .alpha;
  }

  /// Sets the alpha value for the board configuration.
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :  Board configuration to look up
  /// * `alpha` :  Alpha value to save
  ///
  ///
  pub fn set_alpha(&self, board: &BoardHash, alpha: f32) {
    if !self.has_tree_key(board) {
      self
        .tree
        .lock()
        .unwrap()
        .insert(*board, EvalTree::default());
    }

    if let Some(entry) = self.tree.lock().unwrap().get_mut(board) {
      entry.alpha = alpha;
    } else {
      error!("Error setting alpha value in the cache for hash {board}");
    }
  }

  /// Sets the alpha value for the board configuration only if the previous value is higher.
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :  Board configuration to look up
  /// * `alpha` :  Alpha value to compare with the current, replase the current only if it is higher
  ///
  ///
  pub fn update_alpha(&self, board: &BoardHash, alpha: f32) {
    if !self.has_tree_key(board) {
      self.set_alpha(board, alpha);
      return;
    }

    if let Some(entry) = self.tree.lock().unwrap().get_mut(board) {
      if entry.alpha < alpha {
        entry.alpha = alpha;
      }
    } else {
      error!("Error updating alpha value in the cache for hash {board}");
    }
  }

  /// Gets the beta value for the board configuration
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :  Board configuration to look up
  ///
  /// ### Return value
  ///
  /// Beta value that has been cached, f32::MIN if none.
  ///
  pub fn get_beta(&self, board: &BoardHash) -> f32 {
    return self
      .tree
      .lock()
      .unwrap()
      .get(board)
      .unwrap_or(&EvalTree::default())
      .beta;
  }

  /// Sets the beta value for the board configuration.
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :  Board configuration to look up
  /// * `beta` :   Beta value to save
  ///
  ///
  pub fn set_beta(&self, board: &BoardHash, beta: f32) {
    if !self.has_tree_key(board) {
      self
        .tree
        .lock()
        .unwrap()
        .insert(*board, EvalTree::default());
    }

    if let Some(entry) = self.tree.lock().unwrap().get_mut(board) {
      entry.beta = beta;
    } else {
      error!("Error updating beta value in the cache for hash {board}");
    }
  }

  /// Updates the alpha value for the board configuration only
  /// if the previous value is lower.
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :  Board configuration to look up
  /// * `beta` :  Beta value to compare with the current, replase the current only if it is lower
  ///
  ///
  pub fn update_beta(&self, board: &BoardHash, beta: f32) {
    if !self.has_tree_key(board) {
      self.set_beta(board, beta);
      return;
    }

    if let Some(entry) = self.tree.lock().unwrap().get_mut(board) {
      if entry.beta > beta {
        entry.beta = beta;
      }
    } else {
      error!("Error updating beta value in the cache for hash {board}");
    }
  }

  /// Checks if alpha >= beta for a position, in which case the branch should be pruned
  ///
  /// ### Arguments
  ///
  /// * `self` :        EngineCache
  /// * `board_hash` :  Board configuration to look up
  ///
  /// ### Return value
  ///
  /// True if the board position should be pruned, false otherwise
  ///
  pub fn is_pruned(&self, board_hash: &BoardHash) -> bool {
    if !self.has_tree_key(board_hash) {
      return false;
    }

    if let Some(entry) = self.tree.lock().unwrap().get_mut(board_hash) {
      return entry.alpha >= entry.beta;
    } else {
      error!("Error comparing alpha/beta values in the cache for hash {board_hash}");
    }
    false
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

  /// Gets the new board hash from a position after applying a move
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :           Hash value for the board configuration
  /// * `chess_move` :      Chess move to apply
  ///
  /// ### Return value
  ///
  /// Optional BoardHash of the resulting continuation
  ///
  pub fn get_variation(&self, board_hash: &BoardHash, chess_move: &Move) -> Option<BoardHash> {
    let p = &self.positions.lock().unwrap();
    let default = PositionCache::default();
    let variations = &p.get(board_hash).unwrap_or(&default).variations;

    if variations.contains_key(chess_move) {
      return Some(variations[chess_move]);
    }
    None
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
  fn test_cache_has_position_key() {
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
    assert_eq!(true, engine_cache.has_position_key(&game_state.board.hash));

    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/7P b - - 8 43";
    let game_state = GameState::from_fen(fen);
    assert_eq!(false, engine_cache.has_position_key(&game_state.board.hash));
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
      status: None,
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

  #[test]
  fn test_alpha_beta_cache() {
    let engine_cache: EngineCache = EngineCache::new();

    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    let game_state = GameState::from_fen(fen);

    assert_eq!(f32::MIN, engine_cache.get_alpha(&game_state.board.hash));
    assert_eq!(f32::MAX, engine_cache.get_beta(&game_state.board.hash));
    assert_eq!(false, engine_cache.is_pruned(&game_state.board.hash));

    let test_alpha: f32 = 3.0;
    engine_cache.set_alpha(&game_state.board.hash, test_alpha);
    assert_eq!(test_alpha, engine_cache.get_alpha(&game_state.board.hash));
    assert_eq!(f32::MAX, engine_cache.get_beta(&game_state.board.hash));
    assert_eq!(false, engine_cache.is_pruned(&game_state.board.hash));

    let test_beta: f32 = -343.3;
    engine_cache.set_beta(&game_state.board.hash, test_beta);
    assert_eq!(test_alpha, engine_cache.get_alpha(&game_state.board.hash));
    assert_eq!(test_beta, engine_cache.get_beta(&game_state.board.hash));
    assert_eq!(true, engine_cache.is_pruned(&game_state.board.hash));

    // These values won't be accepted, less good than the previous
    engine_cache.update_beta(&game_state.board.hash, 0.0);
    engine_cache.update_alpha(&game_state.board.hash, 0.0);
    assert_eq!(test_alpha, engine_cache.get_alpha(&game_state.board.hash));
    assert_eq!(test_beta, engine_cache.get_beta(&game_state.board.hash));
    assert_eq!(true, engine_cache.is_pruned(&game_state.board.hash));

    // These values won't be accepted, less good than the previous
    engine_cache.set_alpha(&game_state.board.hash, 0.0);
    engine_cache.set_beta(&game_state.board.hash, 1.0);
    assert_eq!(0.0, engine_cache.get_alpha(&game_state.board.hash));
    assert_eq!(1.0, engine_cache.get_beta(&game_state.board.hash));
    assert_eq!(false, engine_cache.is_pruned(&game_state.board.hash));
  }
}
