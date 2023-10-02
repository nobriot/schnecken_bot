use log::*;

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::model::game_state::GamePhase;
use crate::model::game_state::GameState;
use crate::model::game_state::GameStatus;
use crate::model::moves::*;
use crate::model::piece::Color;
use crate::model::tables::zobrist::BoardHash;

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
  // GameState struct saved with a board position.
  game_states: Arc<Mutex<HashMap<BoardHash, GameState>>>,
  // List of moves available from a board position
  move_lists: Arc<Mutex<HashMap<BoardHash, Vec<Move>>>>,
  // List of variations available from a position
  variations: Arc<Mutex<HashMap<BoardHash, HashMap<Move, BoardHash>>>>,
  // Evaluation for a given board configuration
  evals: Arc<Mutex<HashMap<BoardHash, f32>>>,
  // Game Status of an actual board.
  statuses: Arc<Mutex<HashMap<BoardHash, GameStatus>>>,
  // GamePhases saved with each board configuration
  phases: Arc<Mutex<HashMap<BoardHash, GamePhase>>>,
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
      game_states: Arc::new(Mutex::new(HashMap::new())),
      move_lists: Arc::new(Mutex::new(HashMap::new())),
      variations: Arc::new(Mutex::new(HashMap::new())),
      evals: Arc::new(Mutex::new(HashMap::new())),
      statuses: Arc::new(Mutex::new(HashMap::new())),
      phases: Arc::new(Mutex::new(HashMap::new())),
      tree: Arc::new(Mutex::new(HashMap::new())),
      killer_moves: Arc::new(Mutex::new(HashSet::new())),
    }
  }

  // ---------------------------------------------------------------------------
  // Generic cache functions

  /// Returns the number of game states saved in the cache.
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  ///
  /// ### Return value
  ///
  /// Number of GameState objects saved in the EngineCache
  ///
  pub fn len(&self) -> usize {
    return self.game_states.lock().unwrap().len();
  }

  /// Erases everything in the cache
  ///
  pub fn clear(&self) {
    self.game_states.lock().unwrap().clear();
    self.move_lists.lock().unwrap().clear();
    self.variations.lock().unwrap().clear();
    self.evals.lock().unwrap().clear();
    self.statuses.lock().unwrap().clear();
    self.phases.lock().unwrap().clear();
    self.tree.lock().unwrap().clear();
    self.killer_moves.lock().unwrap().clear();
  }

  // ---------------------------------------------------------------------------
  // Game State cached data

  /// Checks if a board position has a cached GameState object
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board_hash` :           Hash value for the board configuration
  ///
  /// ### Return value
  ///
  /// True if the board hash has a PositionCache in the EngineCache. False otherwise
  ///
  pub fn has_game_state(&self, board_hash: &BoardHash) -> bool {
    return self.game_states.lock().unwrap().contains_key(board_hash);
  }

  /// Sets  the associated GameState object to a board position
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board_hash` :      Hash value for the board configuration
  /// * `game_state` :      GameState object to save
  ///
  ///
  pub fn set_game_state(&self, board_hash: BoardHash, game_state: &GameState) {
    self
      .game_states
      .lock()
      .unwrap()
      .insert(board_hash, game_state.clone());
  }

  /// Gets the cached GameState object for a board position
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board_hash` :           Hash value for the board configuration
  ///
  /// ### Return value
  ///
  /// A clone of the game state object for the cached boardhash
  ///
  pub fn get_game_state(&self, board_hash: &BoardHash) -> GameState {
    if !self.has_game_state(board_hash) {
      error!("Looked up a gameState not present in the cache. Returning default");
      return GameState::default();
    }
    self
      .game_states
      .lock()
      .unwrap()
      .get(board_hash)
      .unwrap()
      .clone()
  }

  // ---------------------------------------------------------------------------
  // Move lists cached data

  /// Checks if a board position has a known move list
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board_hash` :           Hash value for the board configuration
  ///
  /// ### Return value
  ///
  /// True if the board hash a known move list in the EngineCache. False otherwise
  ///
  pub fn has_move_list(&self, board_hash: &BoardHash) -> bool {
    return self.move_lists.lock().unwrap().contains_key(board_hash);
  }

  /// Sets the associated Move list to a board position
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board_hash` :      Hash value for the board configuration
  /// * `game_state` :      GameState object to save
  ///
  ///
  pub fn set_move_list(&self, board_hash: BoardHash, move_list: &Vec<Move>) {
    self
      .move_lists
      .lock()
      .unwrap()
      .insert(board_hash, move_list.clone());
  }

  /// Gets the cached Move List for a board position
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board_hash` :      Hash value for the board configuration
  ///
  /// ### Return value
  ///
  /// A clone of the Move List cached for the board position
  ///
  pub fn get_move_list(&self, board_hash: &BoardHash) -> Vec<Move> {
    if !self.has_move_list(board_hash) {
      error!("Looked up a MoveList not present in the cache. Returning empty move list");
      return Vec::new();
    }
    self
      .move_lists
      .lock()
      .unwrap()
      .get(board_hash)
      .unwrap()
      .clone()
  }

  // ---------------------------------------------------------------------------
  // Variations cached data

  /// Checks if the cache knows the resulting variation of applying a move
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board_hash` :      Hash value for the board configuration
  /// * `chess_move` :      Move to apply on the board.
  ///
  /// ### Return value
  ///
  /// True if the board hash a known variation using that movein the EngineCache.
  /// False otherwise
  ///
  pub fn has_variation(&self, board_hash: &BoardHash, chess_move: &Move) -> bool {
    let board_variations = self.variations.lock().unwrap();

    if !board_variations.contains_key(board_hash) {
      return false;
    }

    if !board_variations[board_hash].contains_key(chess_move) {
      return false;
    }

    true
  }

  /// Adds the result of applying a move to a board in the cache.
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board_hash` :      Hash value for the board configuration
  /// * `chess_move` :      Move to apply on the board.
  /// * `resulting_board` : New board hash after applying the move.
  ///
  ///
  pub fn add_variation(
    &self,
    board_hash: &BoardHash,
    chess_move: &Move,
    resulting_board: &BoardHash,
  ) {
    let mut variations = self.variations.lock().unwrap();

    if !variations.contains_key(board_hash) {
      variations.insert(*board_hash, HashMap::new());
    }

    let position_variations = variations.get_mut(board_hash).unwrap();
    position_variations.insert(*chess_move, *resulting_board);
  }

  /// Gets the cached the result of applying a move to a board.
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board_hash` :      Hash value for the board configuration
  /// * `chess_move` :      Move to apply on the board.
  ///
  /// ### Return value
  ///
  /// BoardHash of the new board configuration. Returns 0 if the variation is unknown.
  ///
  pub fn get_variation(&self, board_hash: &BoardHash, chess_move: &Move) -> BoardHash {
    let variations = self.variations.lock().unwrap();

    if !variations.contains_key(board_hash) || !variations[board_hash].contains_key(chess_move) {
      return 0;
    }
    return variations[board_hash][chess_move];
  }

  // ---------------------------------------------------------------------------
  // Evaluation cached data

  /// Checks if a board position has a known static position evaluation
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board_hash` :      Hash value for the board configuration
  ///
  /// ### Return value
  ///
  /// True if the board hash a known eval in the EngineCache. False otherwise
  ///
  pub fn has_eval(&self, board_hash: &BoardHash) -> bool {
    return self.evals.lock().unwrap().contains_key(board_hash);
  }

  /// Sets the associated evaluation to a board position
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board_hash` :      Hash value for the board configuration
  /// * `eval` :            Evaluation value to save
  ///
  ///
  pub fn set_eval(&self, board_hash: BoardHash, eval: f32) {
    self.evals.lock().unwrap().insert(board_hash, eval);
  }

  /// Gets the cached eval for a board position
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board_hash` :      Hash value for the board configuration
  ///
  /// ### Return value
  ///
  /// The evaluation of the board. Returns 0 if the evaluation is unknown.
  ///
  pub fn get_eval(&self, board_hash: &BoardHash) -> f32 {
    if !self.has_eval(board_hash) {
      error!("Looked up an eval without any value for board. Returning 0");
      return 0.0;
    }
    *self.evals.lock().unwrap().get(board_hash).unwrap_or(&0.0)
  }

  // ---------------------------------------------------------------------------
  // GameStatus cached data

  /// Checks if a board position has a known game status
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board_hash` :      Hash value for the board configuration
  ///
  /// ### Return value
  ///
  /// True if the board hash a known GameStatus in the EngineCache. False otherwise
  ///
  pub fn has_status(&self, board_hash: &BoardHash) -> bool {
    return self.statuses.lock().unwrap().contains_key(board_hash);
  }

  /// Sets the associated game status to a board position
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board_hash` :      Hash value for the board configuration
  /// * `status` :          GameStatus value to save
  ///
  pub fn set_status(&self, board_hash: BoardHash, status: GameStatus) {
    self.statuses.lock().unwrap().insert(board_hash, status);
  }

  /// Gets the cached GameStatus for a board position
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board_hash` :      Hash value for the board configuration
  ///
  /// ### Return value
  ///
  /// The gamestatus of the board. Returns Ongoing if the status is unknown.
  ///
  pub fn get_status(&self, board_hash: &BoardHash) -> GameStatus {
    if !self.has_status(board_hash) {
      error!("Looked up a GameStatus without any value for board. Returning Ongoing");
      return GameStatus::Ongoing;
    }
    self
      .statuses
      .lock()
      .unwrap()
      .get(board_hash)
      .unwrap_or(&GameStatus::Ongoing)
      .clone()
  }

  // ---------------------------------------------------------------------------
  // GamePhase cached data

  /// Checks if a board position has a known game phase
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board_hash` :      Hash value for the board configuration
  ///
  /// ### Return value
  ///
  /// True if the board hash a known GamePhase in the EngineCache. False otherwise
  ///
  pub fn has_game_phase(&self, board_hash: &BoardHash) -> bool {
    return self.phases.lock().unwrap().contains_key(board_hash);
  }

  /// Sets the associated game phase to a board position
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board_hash` :      Hash value for the board configuration
  /// * `phase` :          GameStatus value to save
  ///
  pub fn set_game_phase(&self, board_hash: BoardHash, phase: GamePhase) {
    self.phases.lock().unwrap().insert(board_hash, phase);
  }

  /// Gets the cached GamePhase for a board position
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board_hash` :      Hash value for the board configuration
  ///
  /// ### Return value
  ///
  /// The phase associated with of the board. Returns Endgame if the phase is unknown.
  ///
  pub fn get_game_phase(&self, board_hash: &BoardHash) -> GamePhase {
    if !self.has_game_phase(board_hash) {
      error!("Looked up a GamePhase without any value for board. Returning Endgame");
      return GamePhase::Endgame;
    }
    *self
      .phases
      .lock()
      .unwrap()
      .get(board_hash)
      .unwrap_or(&GamePhase::Endgame)
  }

  // ---------------------------------------------------------------------------
  // Tree cached data

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

  /// Clears the alpha/beta stored data
  ///
  /// ### Arguments
  ///
  /// * `self` :        EngineCache
  ///
  ///
  pub fn clear_alpha_beta_values(&self) {
    self.tree.lock().unwrap().clear();
  }

  // ---------------------------------------------------------------------------
  // Position independant cached data

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

  /// Functions used to compare 2 moves by their resulting position evaluation
  ///
  /// ### Arguments
  ///
  /// * `cache`:     EngineCache to use to look-up assets like Killer Moves
  /// * `board_hash` Reference to a BoardHash in the cache for which to compare the moves
  /// * `color`      Side to play. It will order ascending for black, descending for white
  /// * `a`          Move A
  /// * `b`          Move B
  ///
  /// ### Return value
  ///
  /// Ordering telling if B is Greater, Equal or Less than A
  ///
  fn compare_moves_by_cache_eval(
    &self,
    board_hash: &BoardHash,
    color: Color,
    a: &Move,
    b: &Move,
  ) -> Ordering {
    let board_a = self.get_variation(board_hash, a);
    let board_b = self.get_variation(board_hash, b);

    match (self.has_eval(&board_a), self.has_eval(&board_b)) {
      (false, false) => return Ordering::Equal,
      (true, false) => return Ordering::Less,
      (false, true) => return Ordering::Greater,
      (_, _) => {},
    }

    let board_a_eval = self.get_eval(&board_a);
    let board_b_eval = self.get_eval(&board_b);

    let (greater, less) = match color {
      Color::White => (Ordering::Less, Ordering::Greater),
      Color::Black => (Ordering::Greater, Ordering::Less),
    };

    if board_a_eval > board_b_eval {
      return greater;
    } else if board_a_eval < board_b_eval {
      return less;
    }
    Ordering::Equal
  }

  /// Sorts the list of moves based the evaluation of the resulting positions
  ///
  /// ### Arguments
  ///
  /// * `cache`:     EngineCache to use to look-up assets like Killer Moves
  /// * `board_hash` BoardHash for which the moves should be ordered
  /// * `color`      The side to play on the boardhash
  ///
  pub fn sort_moves_by_eval(&self, board_hash: &BoardHash, color: Color) {
    if !self.has_move_list(board_hash) {
      return;
    }
    if let Some(move_list) = self.move_lists.lock().unwrap().get_mut(board_hash) {
      move_list.sort_by(|a, b| self.compare_moves_by_cache_eval(board_hash, color, a, b));
    } else {
      error!("Error sorting move list in the cache for board {board_hash}");
    }
  }
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {

  use super::*;
  use crate::model::game_state::GameState;

  #[test]
  fn test_game_state_data() {
    let engine_cache: EngineCache = EngineCache::new();

    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    let game_state = GameState::from_fen(fen);

    // Empty cache:
    assert_eq!(0, engine_cache.len());
    assert_eq!(false, engine_cache.has_game_state(&game_state.board.hash));

    engine_cache.set_game_state(game_state.board.hash, &game_state);
    assert_eq!(true, engine_cache.has_game_state(&game_state.board.hash));

    let new_state = engine_cache.get_game_state(&game_state.board.hash);
    assert_eq!(new_state.move_count, game_state.move_count);
    assert_eq!(new_state.board, game_state.board);
    assert_eq!(new_state.board.checks(), game_state.board.checks());

    // Clear the cache:
    assert_eq!(1, engine_cache.len());
    engine_cache.clear();
    assert_eq!(0, engine_cache.len());
    assert_eq!(false, engine_cache.has_game_state(&game_state.board.hash));
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

    engine_cache.clear_alpha_beta_values();
    assert_eq!(f32::MIN, engine_cache.get_alpha(&game_state.board.hash));
    assert_eq!(f32::MAX, engine_cache.get_beta(&game_state.board.hash));
  }

  #[test]
  fn test_sorting_moves_by_eval_1() {
    use crate::engine::evaluate_board;
    let engine_cache: EngineCache = EngineCache::new();

    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    let game_state = GameState::from_fen(fen);

    // Save a move list
    engine_cache.set_move_list(game_state, &game_state.get_moves());

    for m in engine_cache.get_move_list(&game_state.board) {
      let mut new_game_state = game_state.clone();
      new_game_state.apply_move(&m);
      engine_cache.add_variation(&game_state, &m, &new_game_state);
      evaluate_board(&engine_cache, &new_game_state);
    }

    // Now try to sort move list by eval:
    engine_cache.sort_moves_by_eval(&game_state, Color::Black);

    let mut last_eval = f32::MIN;
    for m in engine_cache.get_move_list(&game_state.board) {
      let new_board = engine_cache.get_variation(&game_state, &m);
      let new_eval = engine_cache.get_eval(&new_board);
      println!("Move: {} - Eval : {}", m.to_string(), new_eval);
      assert!(last_eval <= new_eval);
      last_eval = new_eval;
    }

    // Try again with White:
    println!("----------------------------------------------------------------");
    engine_cache.sort_moves_by_eval(&game_state, Color::White);

    let mut last_eval = f32::MAX;
    for m in engine_cache.get_move_list(&game_state.board) {
      let new_board = engine_cache.get_variation(&game_state, &m);
      let new_eval = engine_cache.get_eval(&new_board);
      println!("Move: {} - Eval : {}", m.to_string(), new_eval);
      assert!(last_eval >= new_eval);
      last_eval = new_eval;
    }

    // Try again with some moves not evaluated:
    println!("----------------------------------------------------------------");
    engine_cache.clear();
    engine_cache.set_move_list(game_state, &game_state.get_moves());
    let mut i = 0;
    for m in engine_cache.get_move_list(&game_state.board) {
      let mut new_game_state = game_state.clone();
      new_game_state.apply_move(&m);
      engine_cache.add_variation(&game_state, &m, &new_game_state);
      evaluate_board(&engine_cache, &new_game_state);
      i += 1;
      if i > 12 {
        break;
      }
    }

    engine_cache.sort_moves_by_eval(&game_state, Color::White);
    let mut last_eval = f32::MAX;
    for m in engine_cache.get_move_list(&game_state.board) {
      let new_board = engine_cache.get_variation(&game_state, &m);
      let new_eval = if engine_cache.has_eval(&new_board) {
        engine_cache.get_eval(&new_board)
      } else {
        f32::MIN
      };
      println!("Move: {} - Eval : {}", m.to_string(), new_eval);
      assert!(last_eval >= new_eval);
      last_eval = new_eval;
    }

    // Try again with some moves not evaluated for Black:
    println!("----------------------------------------------------------------");
    engine_cache.sort_moves_by_eval(&game_state, Color::Black);
    let mut last_eval = f32::MIN;
    for m in engine_cache.get_move_list(&game_state.board) {
      let new_board = engine_cache.get_variation(&game_state, &m);
      let new_eval = if engine_cache.has_eval(&new_board) {
        engine_cache.get_eval(&new_board)
      } else {
        f32::MAX
      };
      println!("Move: {} - Eval : {}", m.to_string(), new_eval);
      assert!(last_eval <= new_eval);
      last_eval = new_eval;
    }
  }

  #[test]
  fn test_sorting_moves_by_eval_2() {
    use crate::engine::evaluate_board;
    let engine_cache: EngineCache = EngineCache::new();

    let fen = "r1bqk2r/ppppbp1p/8/3Bp1pQ/3nP3/3P4/PPPN1PPP/R3K1NR w KQq - 1 8";
    let game_state = GameState::from_fen(fen);

    // Save a move list
    engine_cache.set_move_list(game_state.board.hash, &game_state.get_moves());

    for m in engine_cache.get_move_list(&game_state.board.hash) {
      let mut new_game_state = game_state.clone();
      new_game_state.apply_move(&m);
      engine_cache.add_variation(&game_state.board.hash, &m, &new_game_state.board.hash);
      evaluate_board(&engine_cache, &new_game_state);
    }

    // Now try to sort move list by eval:
    engine_cache.sort_moves_by_eval(&game_state.board.hash, game_state.board.side_to_play);

    let mut last_eval = f32::MAX;
    for m in engine_cache.get_move_list(&game_state.board.hash) {
      let new_board = engine_cache.get_variation(&game_state.board.hash, &m);
      let new_eval = engine_cache.get_eval(&new_board);
      println!("Move: {} - Eval : {}", m.to_string(), new_eval);
      assert!(last_eval >= new_eval);
      last_eval = new_eval;
    }
  }
}
