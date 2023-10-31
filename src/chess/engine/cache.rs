use log::*;

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::model::board::*;
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
  age: Arc<Mutex<HashMap<Board, usize>>>,
  // List of moves available from a board position
  move_lists: Arc<Mutex<HashMap<Board, Vec<Move>>>>,
  // List of variations available from a position
  variations: Arc<Mutex<HashMap<GameState, HashMap<Move, GameState>>>>,
  // Evaluation for a given board configuration
  evals: Arc<Mutex<HashMap<Board, f32>>>,
  // Game Status of an actual board.
  statuses: Arc<Mutex<HashMap<GameState, GameStatus>>>,
  // List of killer moves that we've met recently during the analysis
  killer_moves: Arc<Mutex<HashSet<Move>>>,
}

impl EngineCache {
  /// Instantiate a new EngineCache object
  ///
  pub fn new() -> Self {
    EngineCache {
      age: Arc::new(Mutex::new(HashMap::new())),
      move_lists: Arc::new(Mutex::new(HashMap::new())),
      variations: Arc::new(Mutex::new(HashMap::new())),
      evals: Arc::new(Mutex::new(HashMap::new())),
      statuses: Arc::new(Mutex::new(HashMap::new())),
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
    self.age.lock().unwrap().clear();
    self.move_lists.lock().unwrap().clear();
    self.variations.lock().unwrap().clear();
    self.evals.lock().unwrap().clear();
    self.statuses.lock().unwrap().clear();
    self.killer_moves.lock().unwrap().clear();
  }

  /// Purges board data that is older than the age limit
  ///
  /// ### Arguments
  ///
  /// * `self`:         Reference to a cache
  /// * `age_limit`:    Minimum age to have in the cache.
  ///
  pub fn purge(&self, age_limit: usize) {
    debug!("Purging cache with age: {age_limit}");
    let ages = self.age.lock().unwrap();
    let mut board_to_remove = Vec::new();

    for (board, _) in ages.iter().filter(|(_, a)| **a >= age_limit) {
      let new_board = board.clone();
      board_to_remove.push(new_board);
    }
    drop(ages);

    for board in board_to_remove {
      self.age.lock().unwrap().remove(&board);
      self.move_lists.lock().unwrap().remove(&board);
      self.evals.lock().unwrap().remove(&board);
    }

    // Same procedure for game states
    let mut game_states_to_remove = Vec::new();

    let statuses = self.statuses.lock().unwrap();
    for (game_state, _) in statuses.iter().filter(|(g, _)| g.move_count >= age_limit) {
      let g_clone = game_state.clone();
      game_states_to_remove.push(g_clone);
    }
    drop(statuses);

    for game_state in game_states_to_remove {
      self.statuses.lock().unwrap().remove(&game_state);
    }

    self.killer_moves.lock().unwrap().clear();
  }

  // ---------------------------------------------------------------------------
  // Age of a board - i.e. on which move number did we first meet the board config

  /// Checks if a board position has a known age
  ///
  /// ### Arguments
  ///
  /// * `self` :        EngineCache
  /// * `board` :       Board configuration to look up in the cache
  ///
  /// ### Return value
  ///
  /// True if the board has a known age
  ///
  pub fn has_age(&self, board: &Board) -> bool {
    return self.age.lock().unwrap().contains_key(board);
  }

  /// Sets the age of a board
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :           Board configuration to look up in the cache
  /// * `age` :             Move number at which we first met the board configuration
  ///
  ///
  pub fn set_age(&self, board: &Board, age: usize) {
    self.age.lock().unwrap().insert(*board, age);
  }

  /// Gets the age of a board
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :           Board configuration to look up in the cache
  ///
  /// ### Return value
  ///
  /// Move number at which we first met the board. 0 if we never met the board.
  ///
  pub fn get_age(&self, board: &Board) -> usize {
    *self.age.lock().unwrap().get(board).unwrap_or(&0)
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
  pub fn clear_variations(&self) {
    self.variations.lock().unwrap().clear();
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

impl Drop for EngineCache {
  fn drop(&mut self) {
    self.clear();
    debug!("Dropping EngineCache");
  }
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {

  use super::*;
  use crate::model::game_state::GameState;

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
      engine_cache.set_eval(&new_game_state.board, evaluate_board(&new_game_state));
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
      engine_cache.set_eval(&new_game_state.board, evaluate_board(&new_game_state));
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
      engine_cache.set_eval(&new_game_state.board, evaluate_board(&new_game_state));
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
}
