use log::*;

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::model::board::*;
use crate::model::game_state::GamePhase;
use crate::model::game_state::GameState;
use crate::model::game_state::GameStatus;
use crate::model::moves::*;
use crate::model::piece::Color;

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
  // List of moves available from a board position
  move_lists: Arc<Mutex<HashMap<Board, Vec<Move>>>>,
  // List of variations available from a position
  variations: Arc<Mutex<HashMap<GameState, HashMap<Move, GameState>>>>,
  // Evaluation for a given board configuration
  evals: Arc<Mutex<HashMap<Board, f32>>>,
  // Game Status of an actual board.
  statuses: Arc<Mutex<HashMap<GameState, GameStatus>>>,
  // GamePhases saved with each board configuration
  phases: Arc<Mutex<HashMap<Board, GamePhase>>>,
  // Tree analysis, including alpha/beta
  tree: Arc<Mutex<HashMap<GameState, EvalTree>>>,
  // List of killer moves that we've met recently during the analysis
  killer_moves: Arc<Mutex<HashSet<Move>>>,
}

impl EngineCache {
  /// Instantiate a new EngineCache object
  ///
  pub fn new() -> Self {
    EngineCache {
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
    return self.move_lists.lock().unwrap().len();
  }

  /// Erases everything in the cache
  ///
  pub fn clear(&self) {
    self.move_lists.lock().unwrap().clear();
    self.variations.lock().unwrap().clear();
    self.evals.lock().unwrap().clear();
    self.statuses.lock().unwrap().clear();
    self.phases.lock().unwrap().clear();
    self.tree.lock().unwrap().clear();
    self.killer_moves.lock().unwrap().clear();
  }

  // ---------------------------------------------------------------------------
  // Move lists cached data

  /// Checks if a board position has a known move list
  ///
  /// ### Arguments
  ///
  /// * `self` :        EngineCache
  /// * `board` :       Board configuration to look up in the cache
  ///
  /// ### Return value
  ///
  /// True if the GameState a known move list in the EngineCache. False otherwise
  ///
  pub fn has_move_list(&self, board: &Board) -> bool {
    return self.move_lists.lock().unwrap().contains_key(board);
  }

  /// Sets the associated Move list to a board position
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :           Board configuration to look up in the cache
  /// * `move_list` :       Move list to save for the GameState
  ///
  ///
  pub fn set_move_list(&self, board: &Board, move_list: &Vec<Move>) {
    self
      .move_lists
      .lock()
      .unwrap()
      .insert(*board, move_list.clone());
  }

  /// Gets the cached Move List for a board position
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :           Board configuration to look up in the cache
  ///
  /// ### Return value
  ///
  /// A clone of the Move List cached for the GameState / board
  ///
  pub fn get_move_list(&self, board: &Board) -> Vec<Move> {
    if !self.has_move_list(board) {
      error!("Looked up a MoveList not present in the cache. Returning empty move list");
      return Vec::new();
    }
    self.move_lists.lock().unwrap().get(board).unwrap().clone()
  }

  // ---------------------------------------------------------------------------
  // Variations cached data

  /// Checks if the cache knows the resulting variation of applying a move
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `game_state` :      GameState to look up in the cache
  /// * `chess_move` :      Move to apply on the board.
  ///
  /// ### Return value
  ///
  /// True if the board hash a known variation using that movein the EngineCache.
  /// False otherwise
  ///
  pub fn has_variation(&self, game_state: &GameState, chess_move: &Move) -> bool {
    let board_variations = self.variations.lock().unwrap();

    if !board_variations.contains_key(game_state) {
      return false;
    }

    if !board_variations[game_state].contains_key(chess_move) {
      return false;
    }

    true
  }

  /// Adds the result of applying a move to a board in the cache.
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `game_state` :      GameState to look up in the cache
  /// * `chess_move` :      Move to apply on the board.
  /// * `resulting_board` : New GameState after applying the move.
  ///
  ///
  pub fn add_variation(
    &self,
    game_state: &GameState,
    chess_move: &Move,
    resulting_board: &GameState,
  ) {
    let mut variations = self.variations.lock().unwrap();

    if !variations.contains_key(game_state) {
      variations.insert(game_state.clone(), HashMap::new());
    }

    let position_variations = variations.get_mut(game_state).unwrap();
    position_variations.insert(*chess_move, resulting_board.clone());
  }

  /// Gets the cached the result of applying a move to a board.
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `game_state` :      Hash value for the board configuration
  /// * `chess_move` :      Move to apply on the board.
  ///
  /// ### Return value
  ///
  /// GameState of the new board configuration. Returns 0 if the variation is unknown.
  ///
  pub fn get_variation(&self, game_state: &GameState, chess_move: &Move) -> GameState {
    let variations = self.variations.lock().unwrap();

    if !variations.contains_key(game_state) || !variations[game_state].contains_key(chess_move) {
      warn!("Queried a non-existing GameState in the cache. Returning default");
      return GameState::default();
    }
    return variations[game_state][chess_move].clone();
  }

  // ---------------------------------------------------------------------------
  // Evaluation cached data

  /// Checks if a board position has a known static position evaluation
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :           Board configuration to look up in the cache
  ///
  /// ### Return value
  ///
  /// True if the board hash a known eval in the EngineCache. False otherwise
  ///
  pub fn has_eval(&self, board: &Board) -> bool {
    return self.evals.lock().unwrap().contains_key(board);
  }

  /// Sets the associated evaluation to a board position
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :           Board configuration to look up in the cache
  /// * `eval` :            Evaluation value to save
  ///
  ///
  pub fn set_eval(&self, board: &Board, eval: f32) {
    self.evals.lock().unwrap().insert(*board, eval);
  }

  /// Gets the cached eval for a board position
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :           Board configuration to look up in the cache
  ///
  /// ### Return value
  ///
  /// The evaluation of the board. Returns 0 if the evaluation is unknown.
  ///
  pub fn get_eval(&self, board: &Board) -> f32 {
    if !self.has_eval(board) {
      return f32::NAN;
    }
    *self.evals.lock().unwrap().get(board).unwrap_or(&f32::NAN)
  }

  // ---------------------------------------------------------------------------
  // GameStatus cached data

  /// Checks if a board position has a known game status
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `game_state` :      GameState to look up in the cache
  ///
  /// ### Return value
  ///
  /// True if the GameState a known GameStatus in the EngineCache. False otherwise
  ///
  pub fn has_status(&self, game_state: &GameState) -> bool {
    return self.statuses.lock().unwrap().contains_key(game_state);
  }

  /// Sets the associated game status to a board position
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `game_state` :      GameState to look up in the cache
  /// * `status` :          GameStatus value to save
  ///
  pub fn set_status(&self, game_state: &GameState, status: GameStatus) {
    self
      .statuses
      .lock()
      .unwrap()
      .insert(game_state.clone(), status);
  }

  /// Gets the cached GameStatus for a board position
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `game_state` :      GameState to look up in the cache
  ///
  /// ### Return value
  ///
  /// The gamestatus of the board. Returns Ongoing if the status is unknown.
  ///
  pub fn get_status(&self, game_state: &GameState) -> GameStatus {
    if !self.has_status(game_state) {
      error!("Looked up a GameStatus without any value for board. Returning Ongoing");
      return GameStatus::Ongoing;
    }
    self
      .statuses
      .lock()
      .unwrap()
      .get(game_state)
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
  /// * `board` :           Board configuration to look up in the cache
  ///
  /// ### Return value
  ///
  /// True if the board GameState a known GamePhase in the EngineCache. False otherwise
  ///
  pub fn has_game_phase(&self, board: &Board) -> bool {
    return self.phases.lock().unwrap().contains_key(board);
  }

  /// Sets the associated game phase to a board position
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :           Board configuration to look up in the cache
  /// * `phase` :           GameStatus value to save
  ///
  pub fn set_game_phase(&self, board: &Board, phase: GamePhase) {
    self.phases.lock().unwrap().insert(*board, phase);
  }

  /// Gets the cached GamePhase for a board position
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :           Board configuration to look up in the cache
  ///
  /// ### Return value
  ///
  /// The phase associated with of the board. Returns Endgame if the phase is unknown.
  ///
  pub fn get_game_phase(&self, board: &Board) -> GamePhase {
    if !self.has_game_phase(board) {
      error!("Looked up a GamePhase without any value for board. Returning Endgame");
      return GamePhase::Endgame;
    }
    *self
      .phases
      .lock()
      .unwrap()
      .get(board)
      .unwrap_or(&GamePhase::Endgame)
  }

  // ---------------------------------------------------------------------------
  // Tree cached data

  /// Checks if a position has a EvalTree entry
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `game_state` :      GameState to look up in the cache
  ///
  /// ### Return value
  ///
  /// True if the gameState has a PositionCache in the EngineCache. False otherwise
  ///
  pub fn has_tree_key(&self, game_state: &GameState) -> bool {
    return self.tree.lock().unwrap().contains_key(game_state);
  }

  // ---------------------------------------------------------------------------
  // Alpha-Beta pruning data

  /// Gets the alpha value for the board configuration
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `game_state` :      GameState to look up in the cache
  ///
  /// ### Return value
  ///
  /// Alpha value that has been cached, f32::MIN if none.
  ///
  pub fn get_alpha(&self, game_state: &GameState) -> f32 {
    return self
      .tree
      .lock()
      .unwrap()
      .get(game_state)
      .unwrap_or(&EvalTree::default())
      .alpha;
  }

  /// Sets the alpha value for the board configuration.
  ///
  /// ### Arguments
  ///
  /// * `self` :        EngineCache
  /// * `game_state` :  GameState to look up in the cache
  /// * `alpha` :       Alpha value to save
  ///
  ///
  pub fn set_alpha(&self, game_state: &GameState, alpha: f32) {
    if !self.has_tree_key(game_state) {
      self
        .tree
        .lock()
        .unwrap()
        .insert(game_state.clone(), EvalTree::default());
    }

    if let Some(entry) = self.tree.lock().unwrap().get_mut(game_state) {
      entry.alpha = alpha;
    } else {
      error!(
        "Error setting alpha value in the cache for game state: {}",
        game_state.to_fen()
      );
    }
  }

  /// Sets the alpha value for the board configuration
  /// only if the previous value is lower.
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `game_state` :      GameState to look up in the cache
  /// * `alpha` :           Alpha value to compare with the current, replase the current only if it is higher
  ///
  ///
  pub fn update_alpha(&self, game_state: &GameState, alpha: f32) {
    if !self.has_tree_key(game_state) {
      self.set_alpha(game_state, alpha);
      return;
    }

    if let Some(entry) = self.tree.lock().unwrap().get_mut(game_state) {
      if entry.alpha < alpha {
        entry.alpha = alpha;
      }
    } else {
      error!(
        "Error updating alpha value in the cache for game state {}",
        game_state.to_fen()
      );
    }
  }

  /// Gets the beta value for the GameState
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `game_state` :      GameState to look up in the cache
  ///
  /// ### Return value
  ///
  /// Beta value that has been cached, f32::MIN if none.
  ///
  pub fn get_beta(&self, game_state: &GameState) -> f32 {
    return self
      .tree
      .lock()
      .unwrap()
      .get(game_state)
      .unwrap_or(&EvalTree::default())
      .beta;
  }

  /// Sets the beta value for the board configuration.
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `game_state` :      GameState to look up in the cache
  /// * `beta` :   Beta value to save
  ///
  ///
  pub fn set_beta(&self, game_state: &GameState, beta: f32) {
    if !self.has_tree_key(game_state) {
      self
        .tree
        .lock()
        .unwrap()
        .insert(game_state.clone(), EvalTree::default());
    }

    if let Some(entry) = self.tree.lock().unwrap().get_mut(game_state) {
      entry.beta = beta;
    } else {
      error!(
        "Error updating beta value in the cache for GameState {}",
        game_state.to_fen()
      );
    }
  }

  /// Updates the beta value for the board configuration only
  /// if the previous value is higher.
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `game_state` :      GameState to look up in the cache
  /// * `beta` :  Beta value to compare with the current, replase the current only if it is lower
  ///
  ///
  pub fn update_beta(&self, game_state: &GameState, beta: f32) {
    if !self.has_tree_key(game_state) {
      self.set_beta(game_state, beta);
      return;
    }

    if let Some(entry) = self.tree.lock().unwrap().get_mut(game_state) {
      if entry.beta > beta {
        entry.beta = beta;
      }
    } else {
      error!(
        "Error updating beta value in the cache for GameState {}",
        game_state.to_fen()
      );
    }
  }

  /// Checks if alpha >= beta for a position, in which case the branch should be pruned
  ///
  /// ### Arguments
  ///
  /// * `self` :        EngineCache
  /// * `game_state` :  GameState to look up in the cache
  ///
  /// ### Return value
  ///
  /// True if the GameState should be pruned, false otherwise
  ///
  pub fn is_pruned(&self, game_state: &GameState) -> bool {
    if !self.has_tree_key(game_state) {
      return false;
    }

    if let Some(entry) = self.tree.lock().unwrap().get_mut(game_state) {
      return entry.alpha >= entry.beta;
    } else {
      error!(
        "Error comparing alpha/beta values in the cache for Game State {}",
        game_state.to_fen()
      );
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
  /// * `game_state` Reference to a GameState in the cache for which to compare the moves
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
    game_state: &GameState,
    color: Color,
    a: &Move,
    b: &Move,
  ) -> Ordering {
    match (
      self.has_variation(game_state, a),
      self.has_variation(game_state, b),
    ) {
      (false, false) => return Ordering::Equal,
      (true, false) => return Ordering::Less,
      (false, true) => return Ordering::Greater,
      (_, _) => {},
    }

    let game_state_a = self.get_variation(game_state, a);
    let game_state_b = self.get_variation(game_state, b);

    let board_a_eval = match self.get_status(&game_state_a) {
      GameStatus::Ongoing => self.get_eval(&game_state_a.board),
      GameStatus::WhiteWon => 200.0,
      GameStatus::BlackWon => -200.0,
      GameStatus::Draw | GameStatus::Stalemate => 0.0,
    };

    let board_b_eval = match self.get_status(&game_state_b) {
      GameStatus::Ongoing => self.get_eval(&game_state_b.board),
      GameStatus::WhiteWon => 200.0,
      GameStatus::BlackWon => -200.0,
      GameStatus::Draw | GameStatus::Stalemate => 0.0,
    };

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
  /// * `game_state` GameState for which the moves should be ordered
  /// * `color`      The side to play on the board
  ///
  pub fn sort_moves_by_eval(&self, game_state: &GameState, color: Color) {
    if !self.has_move_list(&game_state.board) {
      return;
    }
    if let Some(move_list) = self.move_lists.lock().unwrap().get_mut(&game_state.board) {
      move_list.sort_by(|a, b| self.compare_moves_by_cache_eval(game_state, color, a, b));
    } else {
      error!(
        "Error sorting move list in the cache for Game State {}",
        game_state.to_fen()
      );
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
  fn test_alpha_beta_cache() {
    let engine_cache: EngineCache = EngineCache::new();

    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    let game_state = GameState::from_fen(fen);

    assert_eq!(f32::MIN, engine_cache.get_alpha(&game_state));
    assert_eq!(f32::MAX, engine_cache.get_beta(&game_state));
    assert_eq!(false, engine_cache.is_pruned(&game_state));

    let test_alpha: f32 = 3.0;
    engine_cache.set_alpha(&game_state, test_alpha);
    assert_eq!(test_alpha, engine_cache.get_alpha(&game_state));
    assert_eq!(f32::MAX, engine_cache.get_beta(&game_state));
    assert_eq!(false, engine_cache.is_pruned(&game_state));

    let test_beta: f32 = -343.3;
    engine_cache.set_beta(&game_state, test_beta);
    assert_eq!(test_alpha, engine_cache.get_alpha(&game_state));
    assert_eq!(test_beta, engine_cache.get_beta(&game_state));
    assert_eq!(true, engine_cache.is_pruned(&game_state));

    // These values won't be accepted, less good than the previous
    engine_cache.update_beta(&game_state, 0.0);
    engine_cache.update_alpha(&game_state, 0.0);
    assert_eq!(test_alpha, engine_cache.get_alpha(&game_state));
    assert_eq!(test_beta, engine_cache.get_beta(&game_state));
    assert_eq!(true, engine_cache.is_pruned(&game_state));

    // These values won't be accepted, less good than the previous
    engine_cache.set_alpha(&game_state, 0.0);
    engine_cache.set_beta(&game_state, 1.0);
    assert_eq!(0.0, engine_cache.get_alpha(&game_state));
    assert_eq!(1.0, engine_cache.get_beta(&game_state));
    assert_eq!(false, engine_cache.is_pruned(&game_state));

    engine_cache.clear_alpha_beta_values();
    assert_eq!(f32::MIN, engine_cache.get_alpha(&game_state));
    assert_eq!(f32::MAX, engine_cache.get_beta(&game_state));
  }

  #[test]
  fn test_sorting_moves_by_eval_1() {
    use crate::engine::evaluate_board;
    let engine_cache: EngineCache = EngineCache::new();

    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    let game_state = GameState::from_fen(fen);

    // Save a move list
    engine_cache.set_move_list(&game_state.board, &game_state.get_moves());

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
      let new_game_state = engine_cache.get_variation(&game_state, &m);
      let new_eval = engine_cache.get_eval(&new_game_state.board);
      println!("Move: {} - Eval : {}", m.to_string(), new_eval);
      assert!(last_eval <= new_eval);
      last_eval = new_eval;
    }

    // Try again with White:
    println!("----------------------------------------------------------------");
    engine_cache.sort_moves_by_eval(&game_state, Color::White);

    let mut last_eval = f32::MAX;
    for m in engine_cache.get_move_list(&game_state.board) {
      let new_game_state = engine_cache.get_variation(&game_state, &m);
      let new_eval = engine_cache.get_eval(&new_game_state.board);
      println!("Move: {} - Eval : {}", m.to_string(), new_eval);
      assert!(last_eval >= new_eval);
      last_eval = new_eval;
    }

    // Try again with some moves not evaluated:
    println!("----------------------------------------------------------------");
    engine_cache.clear();
    engine_cache.set_move_list(&game_state.board, &game_state.get_moves());
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
      let new_game_state = engine_cache.get_variation(&game_state, &m);
      let new_eval = if engine_cache.has_eval(&new_game_state.board) {
        engine_cache.get_eval(&new_game_state.board)
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
      let new_game_state = engine_cache.get_variation(&game_state, &m);
      let new_eval = if engine_cache.has_eval(&new_game_state.board) {
        engine_cache.get_eval(&new_game_state.board)
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
    engine_cache.set_move_list(&game_state.board, &game_state.get_moves());

    for m in engine_cache.get_move_list(&game_state.board) {
      let mut new_game_state = game_state.clone();
      new_game_state.apply_move(&m);
      engine_cache.add_variation(&game_state, &m, &new_game_state);
      evaluate_board(&engine_cache, &new_game_state);
    }

    // Now try to sort move list by eval:
    engine_cache.sort_moves_by_eval(&game_state, game_state.board.side_to_play);

    let mut last_eval = f32::MAX;
    for m in engine_cache.get_move_list(&game_state.board) {
      let new_game_state = engine_cache.get_variation(&game_state, &m);
      let new_eval = engine_cache.get_eval(&new_game_state.board);
      println!("Move: {} - Eval : {}", m.to_string(), new_eval);
      assert!(last_eval >= new_eval);
      last_eval = new_eval;
    }
  }

  #[test]
  fn test_bench_cache_speed() {
    use super::*;
    use crate::model::board::Board;
    use rand::Rng;
    use std::time::{Duration, Instant};

    let engine_cache: EngineCache = EngineCache::new();

    // Create a bunch of random boards
    const NUMBER_OF_BOARDS: usize = 1_000_000;
    let mut game_states: Vec<GameState> = Vec::with_capacity(NUMBER_OF_BOARDS);
    let mut move_lists: Vec<Vec<Move>> = Vec::with_capacity(NUMBER_OF_BOARDS);
    for _ in 0..NUMBER_OF_BOARDS {
      let current_game = GameState::from_board(&Board::new_random());
      let move_list = current_game.get_moves();
      game_states.push(current_game);
      move_lists.push(move_list);
    }

    let mut rng = rand::thread_rng();
    let mut positions_evaluated = 0;
    let start_time = Instant::now();

    // Spin at it for 1 second
    while Instant::now() < (start_time + Duration::from_millis(1000)) {
      let i = rng.gen_range(0..NUMBER_OF_BOARDS);
      let current_game = &game_states[i];
      if false == engine_cache.has_move_list(&current_game.board) {
        engine_cache.set_move_list(&current_game.board, &move_lists[i]);
      } else {
        let _ = engine_cache.get_move_list(&current_game.board);
      }
      positions_evaluated += 1;
    }

    // 1000 kNPS would be nice.
    assert!(
      positions_evaluated > 1_000_000,
      "Number of NPS for exercising the cache with move lists: {}. Cache length: {}",
      positions_evaluated,
      engine_cache.len()
    );
  }
}
