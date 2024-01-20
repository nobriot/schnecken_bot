use std::cmp::Ordering;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use super::evaluation_table::{EvaluationCache, EvaluationCacheTable};
use super::move_list_cache_table::MoveListCacheTable;
use crate::model::board::*;
use crate::model::game_state::GameState;
use crate::model::moves::*;
use crate::model::piece::Color;

#[derive(Clone)]
pub struct EngineCache {
  // List of moves available from a board position
  move_lists: Arc<Mutex<MoveListCacheTable>>,
  // Evaluation for a given board configuration (GameStatus, Eval and depth)
  evals: Arc<Mutex<EvaluationCacheTable>>,
  // List of killer moves that we've met recently during the analysis
  killer_moves: Arc<Mutex<HashSet<Move>>>,
}

impl EngineCache {
  /// Instantiate a new EngineCache object
  ///
  ///
  pub fn new() -> Self {
    EngineCache {
      move_lists: Arc::new(Mutex::new(MoveListCacheTable::new(10))),
      evals: Arc::new(Mutex::new(EvaluationCacheTable::new(10))),
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
    return self.evals.lock().unwrap().len();
  }

  /// Erases everything in the cache
  ///
  pub fn clear(&self) {
    self.move_lists.lock().unwrap().clear();
    self.killer_moves.lock().unwrap().clear();
    self.evals.lock().unwrap().clear();
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
    self.move_lists.lock().unwrap().get(board.hash).is_some()
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
    self.move_lists.lock().unwrap().add(board.hash, move_list);
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
  pub fn get_move_list(&self, board: &Board) -> Option<Vec<Move>> {
    self.move_lists.lock().unwrap().get(board.hash)
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
    return self.evals.lock().unwrap().get(board.hash).is_some();
  }

  /// Sets the associated evaluation to a board position
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `board` :           Board configuration to look up in the cache
  /// * `eval` :            Evaluation value to save
  /// * `depth` :            Depth at which we evaluated the board
  ///
  pub fn set_eval(&self, board: &Board, eval_data: EvaluationCache) {
    self.evals.lock().unwrap().add(board.hash, eval_data);
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
  pub fn get_eval(&self, board: &Board) -> Option<EvaluationCache> {
    self.evals.lock().unwrap().get(board.hash)
  }

  /// Clear all the evaluation table
  /// Use this if e.g. starting a new game and you want to be sure to avoid
  /// board hash collisions.
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  ///
  pub fn clear_evals(&self) {
    self.evals.lock().unwrap().clear();
  }

  /// Clears and resizes the cache tables. (both for evals and move lists)
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `capacity_mb`:      New size for the tables, in MB.
  ///
  ///
  pub fn resize_tables(&self, capacity_mb: usize) {
    self.evals.lock().unwrap().resize(capacity_mb);
    self.move_lists.lock().unwrap().resize(capacity_mb);
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
    let mut game_state_a = game_state.clone();
    game_state_a.apply_move(a);
    let mut game_state_b = game_state.clone();
    game_state_b.apply_move(b);

    let board_a_eval = self.get_eval(&game_state_a.board).unwrap_or_default();
    let board_b_eval = self.get_eval(&game_state_b.board).unwrap_or_default();

    match (board_a_eval.eval.is_nan(), board_b_eval.eval.is_nan()) {
      (true, true) => return Ordering::Equal,
      (true, _) => return Ordering::Greater,
      (_, true) => return Ordering::Less,
      (_, _) => {},
    }

    let (greater, less) = match color {
      Color::White => (Ordering::Less, Ordering::Greater),
      Color::Black => (Ordering::Greater, Ordering::Less),
    };

    if board_a_eval.eval > board_b_eval.eval {
      return greater;
    } else if board_a_eval.eval < board_b_eval.eval {
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
    let mut move_list = self.get_move_list(&game_state.board).unwrap();
    move_list.sort_by(|a, b| self.compare_moves_by_cache_eval(game_state, color, a, b));
    self.set_move_list(&game_state.board, &move_list);
  }
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {

  use super::*;
  use crate::model::game_state::{GameState, GameStatus};

  #[test]
  fn test_sorting_moves_by_eval_1() {
    use crate::engine::evaluate_board;
    let engine_cache: EngineCache = EngineCache::new();

    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    let game_state = GameState::from_fen(fen);

    // Save a move list
    engine_cache.set_move_list(&game_state.board, &game_state.get_moves());
    let move_list = engine_cache.get_move_list(&game_state.board).unwrap();

    for m in &move_list {
      let mut new_game_state = game_state.clone();
      new_game_state.apply_move(&m);
      let evaluation_cache = EvaluationCache {
        game_status: GameStatus::Ongoing,
        eval: evaluate_board(&new_game_state),
        depth: 1,
      };
      engine_cache.set_eval(&new_game_state.board, evaluation_cache);
    }

    // Now try to sort move list by eval:
    engine_cache.sort_moves_by_eval(&game_state, Color::Black);
    let move_list = engine_cache.get_move_list(&game_state.board).unwrap();

    let mut last_eval = f32::MIN;
    for m in &move_list {
      let mut new_game_state = game_state.clone();
      new_game_state.apply_move(&m);
      let new_eval = engine_cache.get_eval(&new_game_state.board).unwrap_or_default();
      println!("Move: {} - Eval : {}", m.to_string(), new_eval.eval);
      assert!(last_eval <= new_eval.eval);
      last_eval = new_eval.eval;
    }

    // Try again with White:
    println!("----------------------------------------------------------------");
    engine_cache.sort_moves_by_eval(&game_state, Color::White);
    let move_list = engine_cache.get_move_list(&game_state.board).unwrap();

    let mut last_eval = f32::MAX;
    for m in &move_list {
      let mut new_game_state = game_state.clone();
      new_game_state.apply_move(&m);
      let new_eval = engine_cache.get_eval(&new_game_state.board).unwrap_or_default();
      println!("Move: {} - Eval : {}", m.to_string(), new_eval.eval);
      assert!(last_eval >= new_eval.eval);
      last_eval = new_eval.eval;
    }

    // Try again with some moves not evaluated:
    println!("----------------------------------------------------------------");
    engine_cache.clear();
    engine_cache.set_move_list(&game_state.board, &game_state.get_moves());
    let mut i = 0;
    for m in &move_list {
      let mut new_game_state = game_state.clone();
      new_game_state.apply_move(&m);
      let evaluation_cache = EvaluationCache {
        game_status: GameStatus::Ongoing,
        eval: evaluate_board(&new_game_state),
        depth: 1,
      };
      engine_cache.set_eval(&new_game_state.board, evaluation_cache);
      i += 1;
      if i > 12 {
        break;
      }
    }

    engine_cache.sort_moves_by_eval(&game_state, Color::White);
    let move_list = engine_cache.get_move_list(&game_state.board).unwrap();

    let mut last_eval = f32::MAX;
    for m in &move_list {
      let mut new_game_state = game_state.clone();
      new_game_state.apply_move(&m);
      let eval_cache = engine_cache.get_eval(&new_game_state.board).unwrap_or_default();
      let new_eval = if eval_cache.depth > 0 { eval_cache.eval } else { f32::MIN };
      println!("Move: {} - Eval : {}", m.to_string(), new_eval);
      assert!(last_eval >= new_eval);
      last_eval = new_eval;
    }

    // Try again with some moves not evaluated for Black:
    println!("----------------------------------------------------------------");
    engine_cache.sort_moves_by_eval(&game_state, Color::Black);
    let move_list = engine_cache.get_move_list(&game_state.board).unwrap();
    let mut last_eval = f32::MIN;
    for m in &move_list {
      let mut new_game_state = game_state.clone();
      new_game_state.apply_move(&m);
      let eval_cache = engine_cache.get_eval(&new_game_state.board).unwrap_or_default();
      let new_eval = if eval_cache.depth != 0 { eval_cache.eval } else { f32::MAX };
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
    let move_list = engine_cache.get_move_list(&game_state.board).unwrap();

    for m in &move_list {
      let mut new_game_state = game_state.clone();
      new_game_state.apply_move(&m);
      let evaluation_cache = EvaluationCache {
        game_status: GameStatus::Ongoing,
        eval: evaluate_board(&new_game_state),
        depth: 1,
      };
      engine_cache.set_eval(&new_game_state.board, evaluation_cache);
    }

    // Now try to sort move list by eval:
    engine_cache.sort_moves_by_eval(&game_state, game_state.board.side_to_play);
    let move_list = engine_cache.get_move_list(&game_state.board).unwrap();

    let mut last_eval = f32::MAX;
    for m in &move_list {
      let mut new_game_state = game_state.clone();
      new_game_state.apply_move(&m);
      let new_eval = engine_cache.get_eval(&new_game_state.board).unwrap_or_default();
      println!("Move: {} - Eval : {}", m.to_string(), new_eval.eval);
      assert!(last_eval >= new_eval.eval);
      last_eval = new_eval.eval;
    }
  }
}
