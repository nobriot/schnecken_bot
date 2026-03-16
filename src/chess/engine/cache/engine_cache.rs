use super::evaluation_table::{EvaluationCache, EvaluationCacheTable};
use super::move_list_cache_table::MoveListCacheTable;
use crate::model::board::*;
use crate::model::containers::move_list::MoveList;
use crate::model::moves::*;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct EngineCache {
  // List of moves available from a board position
  move_lists:   Arc<Mutex<MoveListCacheTable>>,
  // Evaluation for a given board configuration (GameStatus, Eval and depth)
  evals:        Arc<Mutex<EvaluationCacheTable>>,
  // List of killer moves that we've met recently during the analysis
  killer_moves: Arc<Mutex<Vec<Move>>>,
}

impl EngineCache {
  /// Instantiate a new EngineCache object
  pub fn new() -> Self {
    EngineCache { move_lists:   Arc::new(Mutex::new(MoveListCacheTable::new(10))),
                  evals:        Arc::new(Mutex::new(EvaluationCacheTable::new(10))),
                  killer_moves: Arc::new(Mutex::new(Vec::new())), }
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
  pub fn len(&self) -> usize {
    return self.evals.lock().unwrap().len();
  }

  pub fn is_empty(&self) -> bool {
    return self.evals.lock().unwrap().is_empty();
  }

  /// Erases everything in the cache
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
  /// True if the GameState a known move list in the EngineCache. False
  /// otherwise
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
  pub fn set_move_list(&self, board: &Board, move_list: &[Move]) {
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
  #[inline]
  pub fn get_move_list(&self, board: &Board) -> Option<MoveList> {
    let table = self.move_lists.lock().unwrap();
    let entry = table.get(board.hash);

    Some(MoveList::new_from_slice(entry?))
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
  pub fn clear_evals(&self) {
    self.evals.lock().unwrap().clear();
  }

  /// Clears and resizes the cache tables. (both for evals and move lists)
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `capacity_mb`:      New size for the tables, in MB.
  pub fn resize_tables(&self, capacity_mb: usize) {
    self.evals.lock().unwrap().resize(capacity_mb);
    self.move_lists.lock().unwrap().resize(capacity_mb);
  }

  // ---------------------------------------------------------------------------
  // Position independant cached data

  /// Adds a killer move in the EngineCache
  /// This is not dependant on positions, and should be cleared when the engine
  /// moves to another position
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
  /// * `killer_move` :     Killer Move to add in the EngineCache
  pub fn add_killer_move(&self, killer_move: &Move) {
    let mut moves = self.killer_moves.lock().unwrap();
    if !moves.contains(killer_move) {
      moves.push(*killer_move);
    }
  }

  /// Removes all killer moves from the EngineCache
  ///
  /// ### Arguments
  ///
  /// * `self` :            EngineCache
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
  pub fn is_killer_move(&self, candidate_move: &Move) -> bool {
    return self.killer_moves.lock().unwrap().contains(candidate_move);
  }
}

impl Default for EngineCache {
  fn default() -> Self {
    EngineCache::new()
  }
}
