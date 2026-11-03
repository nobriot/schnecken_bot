pub mod books;
pub mod cache;
pub mod config;
pub mod eval;
pub mod game_history;
pub mod nnue;
pub mod search;
pub mod search_result;
pub mod tables;
pub mod time_management;

mod comments;
#[cfg(test)]
pub mod tests;

// Same module (engine)
use self::cache::engine_cache::EngineCache;
use self::cache::evaluation_table::EvaluationCache;
use self::eval::position::*;
use self::game_history::GameHistory;
use self::search_result::SearchResult;
// Chess model
use super::model::game_state::GameState;
use super::model::game_state::{GameStatus, START_POSITION_FEN};
use super::model::moves::Move;
use super::model::piece::{Color, Piece};
use crate::engine::search_result::VariationWithEval;
use crate::model::board::Board;
use crate::model::containers::move_list::MoveList;
use books::*;
use config::options::*;
use config::play_style::*;
use log::*;
use nnue::NNUE;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use time_management::*;

// -----------------------------------------------------------------------------
// Constants
pub const NNUE_FILE: &str = "engine/nnue/net.nnue";
pub const NUMBER_OF_MOVES_IN_SEARCH_RESULTS: usize = 30;
/// How many plies deeper than the current search depth we can go when
/// following checks.
const MAX_CHECK_EXTENSION: usize = 4;
/// Search depth when there is only one legal move: enough to see if a mate
/// follows, while still playing the move almost instantly.
const SINGLE_MOVE_SEARCH_DEPTH: usize = 3;
/// Alpha has to exceed beta by this many centipawns before we prune: we have
/// low trust in the eval accuracy.
const SEARCH_PRUNING_MARGIN: Score = 50;
/// The search checks the clock once every that many calls to `search()`
/// (must be a power of 2): reading the time at every node is expensive. Most
/// calls are just above the leaves, so this is about a thousand leaves.
const TIME_CHECK_SEARCH_INTERVAL: usize = 32;
/// Maximum number of plies played in the quiescence search, after the leaves
/// of the search: long capture sequences are rare, and checks could go on.
const QUIESCENCE_MAX_PLY: usize = 6;
/// Delta pruning in the quiescence search: captures are skipped when winning
/// the captured piece plus this margin still cannot improve the score.
const QUIESCENCE_DELTA_MARGIN: Score = 200;

// -----------------------------------------------------------------------------
// Type definitions

// TODO: I need to break this file down into simpler/more independent modules

/// Evaluation for a position
#[derive(Clone, Debug)]
pub enum Eval {
  /// Mating sequence.
  /// Mate(-1) for black to mate in 1 half move
  /// Mate(4) for white to mate in 4 plys
  /// Mate(0) for Game Over, either black or white won
  Mate(i8),
  /// Score in centipawns for the position
  Score(i16),
}

#[derive(Clone, Debug)]
struct Analysis {
  /// After the search, the nth best lines will be saved in this vector.
  pub result:          Arc<Mutex<SearchResult>>,
  /// Represent how deep the analysis is/was
  pub depth:           Arc<AtomicUsize>,
  /// Represent how deep the analysis is/was
  pub selective_depth: Arc<AtomicUsize>,
  /// Represents how many nodes we visited in the search
  pub nodes_visited:   Arc<AtomicUsize>,
}

#[derive(Clone, Debug)]
struct EngineState {
  /// Indicates if the engine is active at resolving positions
  pub active:         Arc<AtomicBool>,
  /// Indicates that we want the engine to stop resolving positions
  pub stop_requested: Arc<AtomicBool>,
  /// Indicates when the engine was requested to start searching
  pub start_time:     Arc<Mutex<Instant>>,
  /// Number of calls to `search()`, used to check the clock periodically
  pub search_calls:   Arc<AtomicUsize>,
  /// Maximum time (milliseconds) for the current search, 0 for no limit
  pub hard_limit_ms:  Arc<AtomicUsize>,
}

/// Stops the searches of an engine from another thread
#[derive(Clone, Debug)]
pub struct EngineStopHandle {
  stop_requested: Arc<AtomicBool>,
}

impl EngineStopHandle {
  /// Stops the ongoing search, if any. A search started afterwards is not
  /// stopped.
  pub fn stop(&self) {
    self.stop_requested.store(true, Ordering::Release);
  }
}

impl Analysis {
  /// Resets the analysis
  pub fn reset(&self) {
    self.result.lock().unwrap().clear();
    self.set_selective_depth(0);
    self.set_depth(0);
    self.set_nodes_visited(0);
  }

  /// Saves the nth best continuations in the analysis.best_lines
  ///
  /// ### Arguments
  ///
  /// * `self`:    Analysis struct reference
  /// * `result`:  Sorted vector with best variations.
  pub fn update_result(&self, result: SearchResult) {
    let mut pvs = self.result.lock().unwrap();
    *pvs = result;
  }

  /// Sets the depth we have reached during the analysis
  ///
  /// ### Arguments
  ///
  /// * `self`:   Instance of the Chess Engine
  /// * `depth`:  New depth to set.
  pub fn set_depth(&self, depth: usize) {
    self.depth.store(depth, Ordering::Relaxed);
  }

  /// Increments the depth we have reached during the analysis
  ///
  /// ### Arguments
  ///
  /// * `self`:   Instance of the Chess Engine
  pub fn increment_depth(&self) {
    self.depth.fetch_add(1, Ordering::Relaxed);
  }

  /// Decrements the depth we have reached during the analysis
  ///
  /// ### Arguments
  ///
  /// * `self`:   Instance of the Chess Engine
  pub fn decrement_depth(&self) {
    // Only the thread running go() changes the depth: no need for an atomic
    // read-modify-write.
    let depth = self.depth.load(Ordering::Relaxed);
    self.depth.store(depth.saturating_sub(1), Ordering::Relaxed);
  }

  /// Increments the depth we have reached during the analysis
  ///
  /// ### Arguments
  ///
  /// * `self`:   Instance of the Chess Engine
  ///
  /// ### Return value
  ///
  /// The value contained in the analysis depth.
  pub fn get_depth(&self) -> usize {
    self.depth.load(Ordering::Relaxed)
  }

  /// Sets the selective depth we have reached during the analysis
  ///
  /// ### Arguments
  ///
  /// * `self`:   Instance of the Chess Engine
  /// * `selective_depth`:  New depth to set.
  pub fn set_selective_depth(&self, depth: usize) {
    self.selective_depth.store(depth, Ordering::Relaxed);
  }

  /// Updates the selective depth if the new value is higher than the current
  /// value.
  ///
  /// ### Arguments
  ///
  /// * `self`:   Instance of the Chess Engine
  /// * `selective_depth`:  New depth to set.
  pub fn update_selective_depth(&self, depth: usize) {
    self.selective_depth.fetch_max(depth, Ordering::Relaxed);
  }

  /// Increments the selective depth we have reached during the analysis
  ///
  /// ### Arguments
  ///
  /// * `self`:   Instance of the Chess Engine
  pub fn increment_selective_depth(&self) {
    self.selective_depth.fetch_add(1, Ordering::Relaxed);
  }

  /// Increments the depth we have reached during the analysis
  ///
  /// ### Arguments
  ///
  /// * `self`:   Instance of the Chess Engine
  ///
  /// ### Return value
  ///
  /// The value contained in the analysis depth.
  pub fn get_selective_depth(&self) -> usize {
    self.selective_depth.load(Ordering::Relaxed)
  }

  /// Increments the nodes we visited
  pub fn increment_nodes_visited(&self) {
    self.nodes_visited.fetch_add(1, Ordering::Relaxed);
  }

  pub fn get_nodes_visited(&self) -> usize {
    self.nodes_visited.load(Ordering::Relaxed)
  }

  pub fn set_nodes_visited(&self, value: usize) {
    self.nodes_visited.store(value, Ordering::Relaxed);
  }
}

impl Default for Analysis {
  fn default() -> Self {
    Analysis { result:          Arc::new(Mutex::new(SearchResult::new(1, Color::White))),
               depth:           Arc::new(AtomicUsize::new(0)),
               selective_depth: Arc::new(AtomicUsize::new(0)),
               nodes_visited:   Arc::new(AtomicUsize::new(0)), }
  }
}

#[derive(Clone)]
pub struct Engine {
  pub position: GameState,
  /// State of the analysis for the game state.
  analysis:     Analysis,
  /// Position cache, used to speed up processing
  cache:        EngineCache,
  /// Engine options
  pub options:  EngineOptions,
  /// Whether the engine is active of not, and if we want to stop it.
  state:        EngineState,
  /// NNUE
  nnue:         Arc<Mutex<NNUE>>,
  /// Game History
  history:      GameHistory,
}

// type AsyncResult = std::result::Result<(), Box<dyn std::error::Error + Send +
// Sync>>;

impl Engine {
  //----------------------------------------------------------------------------
  // Public functions

  /// Gets a new engine
  pub fn new(uci: bool) -> Self {
    initialize_chess_books();
    let nnue_path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), NNUE_FILE);

    let mut engine =
      Engine { position: GameState::default(),
               analysis: Analysis::default(),
               cache:    EngineCache::new(),
               options:  EngineOptions::default(),
               state:    EngineState { active:         Arc::new(AtomicBool::new(false)),
                                       stop_requested: Arc::new(AtomicBool::new(false)),
                                       start_time:     Arc::new(Mutex::new(Instant::now())),
                                       search_calls:   Arc::new(AtomicUsize::new(0)),
                                       hard_limit_ms:  Arc::new(AtomicUsize::new(0)), },
               nnue:     Arc::new(Mutex::new(NNUE::load(nnue_path.as_str()).unwrap_or_default())),
               history:  GameHistory::new(), };

    engine.options.uci = uci;
    engine.set_position(START_POSITION_FEN);
    engine
  }

  /// Checks if the engine is resolving a position
  ///
  /// ### Return value
  ///
  /// * True if searching a position, False otherwise
  pub fn is_active(&self) -> bool {
    self.state.active.load(Ordering::Acquire)
  }

  /// Helper function that sets the "active" bool value in the engine
  ///
  /// ### Arguments
  ///
  /// * `active`: The new value to apply to active
  fn set_engine_active(&self, active: bool) {
    self.state.active.store(active, Ordering::Release);
  }

  /// Checks if the engine has been requested to stop evaluating
  ///
  /// ### Return value
  ///
  /// * True if the engine should stop searching positions, False otherwise
  pub fn stop_requested(&self) -> bool {
    self.state.stop_requested.load(Ordering::Acquire)
  }

  /// Helper function that sets the "stop_requested" bool value in the engine
  ///
  /// ### Arguments
  ///
  /// * `stop_requested`: The new value to apply to stop_requested
  fn set_stop_requested(&self, stop_requested: bool) {
    self.state.stop_requested.store(stop_requested, Ordering::Release);
  }

  /// Checks if the engine has been searching a position for too long
  /// It will compare the start time with the current time and the maximum time
  /// set in the engine options
  ///
  /// ### Return value
  ///
  /// True if the (current_time - start_time) is larger than the hard limit of
  /// the search and the hard limit is set to a non-zero value.
  fn has_been_searching_too_long(&self) -> bool {
    let max_time = self.state.hard_limit_ms.load(Ordering::Relaxed);
    if max_time == 0 {
      return false;
    }
    let start_time = self.get_start_time();
    (Instant::now() - start_time) > Duration::from_millis(max_time as u64)
  }

  /// Computes the time we can spend on the current position, from the clock
  /// set in the options.
  ///
  /// ### Return value
  ///
  /// The time budget, None if no clock is set (`max_search_time` is then the
  /// fixed search time)
  pub fn time_budget(&self) -> Option<TimeBudget> {
    self.options.clock.map(|clock| {
                        time_budget(&clock,
                                    &self.position,
                                    self.options.time_factor,
                                    self.options.max_search_time)
                      })
  }

  /// Clears the cache of the engine.
  ///
  /// Note: You should not invoke this function when the engine is
  /// active/searching.
  pub fn clear_cache(&self) {
    self.cache.clear();
  }

  /// Clears and resize the cache table size.
  ///
  /// Note: You should not invoke this function when the engine is
  /// active/searching.
  ///
  /// ### Arguments
  ///
  /// * self : Engine reference
  /// * capacity_mb : Size in MB to use for the engine cache tables (there are 2 of them).
  pub fn resize_cache_tables(&self, capacity_mb: usize) {
    self.cache.resize_tables(capacity_mb);
  }

  /// Resets the engine to a default state.
  /// Same as Engine::Default() or Engine::new(..)
  pub fn reset(&mut self) {
    self.stop();
    self.position = GameState::from_fen(START_POSITION_FEN);
    self.analysis.reset();
    self.cache.clear();
    self.set_engine_active(false);
  }

  /// Sets a new position
  /// The cache will be fully cleared
  ///
  /// ### Arguments
  ///
  /// * `fen`: FEN notation of the position to set
  pub fn set_position(&mut self, fen: &str) {
    self.reset();
    self.history.clear();
    self.analysis.set_depth(0);
    self.analysis.set_selective_depth(0);

    let game_state = GameState::from_fen(fen);
    self.position = game_state.clone();
    let move_list = self.position.get_moves();

    // Compute move list if not known.
    if !self.cache.has_move_list(&game_state.board) {
      self.cache.set_move_list(&game_state.board, &move_list);
    }
  }

  /// Applies a move from the current position
  /// Invalid moves will be ignored.
  ///
  /// ### Arguments
  ///
  /// * `chess_move`: Notation of the chess move to apply on the current position
  pub fn apply_move(&mut self, chess_move: &str) {
    if self.is_active() {
      self.stop();
    }

    // FIXME: THis does not capture evals when opponent applies a move
    // let eval = self.get_best_eval();
    // let mv = self.position.last_moves.last().unwrap_or(&Move::null()).
    // clone(); self.history.add(self.position.to_fen(), mv, eval as isize);

    self.position.apply_move_from_notation(chess_move);
    self.cache.clear_killer_moves();
    self.analysis.reset();
    self.analysis.decrement_depth();
  }

  /// Starts analyzing the current position
  ///
  /// Analysis will continue until stopped with the `stop()` method
  pub fn go(&self) {
    if self.is_active() {
      // we are already searching.
      debug!("We are already evaluating the position.");
      return;
    }

    // Mark that we are now active and stop is not requested.
    self.set_stop_requested(false);
    self.set_engine_active(true);
    self.set_start_time(); // Capture that we started searching now.
    let budget = self.time_budget();
    let hard_limit_ms = budget.map_or(self.options.max_search_time, |b| b.hard_ms);
    self.state.hard_limit_ms.store(hard_limit_ms, Ordering::Relaxed);
    self.analysis.set_nodes_visited(0);
    self.state.search_calls.store(0, Ordering::Relaxed);

    // Make sure we know the move list:
    Engine::find_move_list(&self.cache, &self.position.board);

    // First check if we are in a known book position. If yes, just return the
    // known list
    let play_style = self.options.play_style;
    let book_entry = get_book_moves(&self.position.board, play_style == PlayStyle::Provocative);
    if let Some(move_list) = book_entry {
      info!("Known position, returning book moves for {:?} play", play_style);

      let mut result: SearchResult =
        SearchResult::new(self.options.multi_pv, self.position.board.side_to_play);

      for m in &move_list {
        result.update(VariationWithEval::new_from_move(0, *m));
      }
      self.analysis.update_result(result);

      // We are done
      self.print_uci_info();
      self.print_uci_best_move();
      self.set_stop_requested(false);
      self.set_engine_active(false);
      return;
    }

    // If we have only one legal move, we should play it instantaneously.
    // We still run a very shallow search to give it a meaningful eval (e.g. a
    // forced mate after it): a static eval would hide it.
    let moves = self.cache.get_move_list(&self.position.board).unwrap();
    if moves.len() == 1 {
      debug!("Single move available. Just evaluating quickly");
      let result =
        self.search(&self.position.clone(), 1, SINGLE_MOVE_SEARCH_DEPTH, -SCORE_INF, SCORE_INF);
      if let Some(result) = result
         && !result.is_empty()
      {
        self.analysis.update_result(result);
        self.analysis.set_depth(SINGLE_MOVE_SEARCH_DEPTH);
        self.print_uci_info();
        self.print_uci_best_move();
        self.set_stop_requested(false);
        self.set_engine_active(false);
        return;
      }

      // No time for the search: just evaluate the position after the move
      let mut game_state = self.position.clone();
      game_state.apply_move(&moves[0]);

      let mut evaluation_cache = self.cache.get_eval(&game_state.board).unwrap_or_default();
      // A cached mate distance is counted from the position after the move:
      // add the ply of the move.
      let mut eval = decrement_mate_score(evaluation_cache.eval);
      if evaluation_cache.depth == 0 {
        let game_status = is_game_over(&self.cache, &game_state.board);
        eval =
          get_eval_from_game_status(game_status).unwrap_or_else(|| evaluate_board(&game_state));
        evaluation_cache = EvaluationCache { game_status, eval, depth: 1 };
        self.cache.set_eval(&game_state.board, evaluation_cache);
      }
      let mut result: SearchResult =
        SearchResult::new(self.options.multi_pv, game_state.board.side_to_play);
      result.update(VariationWithEval::new_from_move_with_depth(eval,
                                                                evaluation_cache.depth,
                                                                moves[0]));

      self.analysis.update_result(result);
      self.analysis.set_depth(evaluation_cache.depth);
      self.analysis.set_selective_depth(evaluation_cache.depth);

      self.print_uci_info();
      self.print_uci_best_move();
      self.set_stop_requested(false);
      self.set_engine_active(false);
      return;
    }

    // Main search
    let side_to_play = self.position.board.side_to_play;
    let legal_moves = moves.len();
    let mut previous_result: Option<SearchResult> = None;
    let mut best_move_stable_depths = 0;
    let mut soft_limit_ms = budget.map(|b| b.soft_ms);
    while !self.has_been_searching_too_long() && !self.stop_requested() {
      self.analysis.increment_depth();
      self.analysis.increment_selective_depth();

      // Try to search for the current depth
      let result =
        self.search(&self.position.clone(), 1, self.analysis.get_depth(), -SCORE_INF, SCORE_INF);

      if self.has_been_searching_too_long() || self.stop_requested() || result.is_none() {
        // Toss away unfinished depths
        self.analysis.decrement_depth();
        break;
      }

      // Depth completed - print UCI result if needed
      let result = result.unwrap(); // Safe due to is_none() above
      let best_eval = result.get_eval().unwrap();

      // Adjust the time we want to spend based on what the search found
      if let Some(budget) = budget {
        let best_move = result.get_best_move();
        best_move_stable_depths =
          if previous_result.as_ref().and_then(|p| p.get_best_move()) == best_move {
            best_move_stable_depths + 1
          } else {
            1
          };
        let stats = IterationStats::new(previous_result.as_ref(),
                                        &result,
                                        self.analysis.get_depth(),
                                        best_move_stable_depths,
                                        legal_moves,
                                        side_to_play);
        soft_limit_ms = Some(adjust_soft_limit(budget.soft_ms, &stats));
        previous_result = Some(result.clone());
      }
      self.analysis.update_result(result);
      self.print_uci_info();

      // If the best move is just winning for us, stop searching unless
      // requested to.
      if Engine::best_move_is_mating_sequence(self.position.board.side_to_play, best_eval)
         && !self.options.ponder
      {
        debug!("Winning sequence found! Stopping search");
        break;
      }

      let max_depth = self.options.max_depth;
      if max_depth > 0 && self.analysis.get_depth() >= max_depth {
        break;
      }

      // The next depth would likely not finish within the soft limit
      if let Some(soft_limit_ms) = soft_limit_ms
         && (Instant::now() - self.get_start_time()).as_millis() as f32
            > soft_limit_ms as f32 * NEXT_DEPTH_SOFT_LIMIT_SHARE
      {
        debug!("Soft limit {soft_limit_ms}ms reached, stopping at depth {}",
               self.analysis.get_depth());
        break;
      }
    }

    if let Some(budget) = budget {
      debug!("Searched {}ms (soft limit {}ms, adjusted {}ms, hard limit {}ms), depth {}",
             (Instant::now() - self.get_start_time()).as_millis(),
             budget.soft_ms,
             soft_limit_ms.unwrap_or(budget.soft_ms),
             budget.hard_ms,
             self.analysis.get_depth());
    }

    // We are done
    self.print_uci_best_move();
    self.set_stop_requested(false);
    self.set_engine_active(false);
  }

  /// Starts analyzing the current position
  ///
  /// Analysis will continue until stopped.
  pub fn stop(&self) {
    self.set_stop_requested(true);
  }

  /// Gets a handle that can stop the searches of this engine from another
  /// thread, without access to the engine itself.
  pub fn stop_handle(&self) -> EngineStopHandle {
    EngineStopHandle { stop_requested: self.state.stop_requested.clone(), }
  }

  /// Returns the best move saved in the analysis
  pub fn get_best_move(&self) -> Option<Move> {
    let analysis = self.analysis.result.lock().unwrap();
    analysis.get_best_move()
  }

  /// Returns the best eval saved in the analysis
  pub fn get_eval(&self) -> Option<Score> {
    let analysis = self.analysis.result.lock().unwrap();
    analysis.get_eval()
  }

  /// Returns the number of nodes (leaf positions) evaluated in the last search
  pub fn get_nodes_visited(&self) -> usize {
    self.analysis.get_nodes_visited()
  }

  /// Prints information to stdout for the GUI using UCI protocol
  /// Nothing will be sent if the UCI option is not set in the engine
  #[inline]
  pub fn print_uci_info(&self) {
    if !self.options.uci {
      return;
    }

    let result = self.analysis.result.lock().unwrap().clone();
    let depth = self.analysis.get_depth();
    let selective_depth = self.analysis.get_selective_depth();
    let nodes_visited = self.analysis.get_nodes_visited();
    let start_time = self.get_start_time();
    let multi_pv_setting = self.options.multi_pv;

    for (i, variation) in result.variations.iter().take(multi_pv_setting).enumerate() {
      let eval = variation.eval;
      let score_string = match mate_distance_in_plies(eval) {
        // UCI mate scores are in moves, from the side to play's point of view
        Some(plies) => {
          let moves = plies.div_ceil(2) as isize;
          let mating = match self.position.board.side_to_play {
            Color::White => eval > 0,
            Color::Black => eval < 0,
          };
          format!("score mate {}", if mating { moves } else { -moves })
        },
        None => format!("score cp {}", eval),
      };
      let multi_pv_string = if multi_pv_setting > 1 {
        format!(" multipv {} ", i + 1)
      } else {
        String::new()
      };
      println!("info {} depth {} seldepth {} nodes {} time {}{}pv {}",
               score_string,
               depth,
               selective_depth,
               nodes_visited,
               (Instant::now() - start_time).as_millis(),
               multi_pv_string,
               variation.variation,);
    }
  }

  /// Prints the best move
  #[inline]
  pub fn print_uci_best_move(&self) {
    if self.options.uci {
      println!("bestmove {}", self.get_best_move().unwrap_or_default());
    }
  }

  pub fn print_debug(&self, debug_info: &str) {
    if self.options.debug {
      println!("info string {}", debug_info);
    }
  }

  /// Prints out the full game history.
  pub fn print_game_summary(&self) {
    println!("Game Summary:\n{}", self.history);
  }

  /// Returns the full analysis
  pub fn get_analysis(&self) -> SearchResult {
    self.analysis.result.lock().unwrap().clone()
  }

  /// Returns a string of the best move continuation (e.g. d1c3 c2c8 f2g3)
  /// based on the board, using the engine cache.
  ///
  /// ### Arguments
  ///
  /// * self:         Used to look up the engine cache
  /// * game_state:   Game State to start from
  ///
  /// ### Return value
  ///
  /// String containing the list of best moves found by the engine
  pub fn get_line_string(&self, game_state: &GameState, side_to_play: Color, ttl: usize) -> String {
    let line_string = String::new();

    let evaluation_cache = self.cache.get_eval(&game_state.board).unwrap_or_default();
    if evaluation_cache.game_status != GameStatus::Ongoing {
      return line_string + format!(" / {:?}", evaluation_cache.game_status).as_str();
    }
    if ttl == 0 {
      return line_string;
    }

    if !self.cache.has_move_list(&game_state.board) {
      return line_string + " - no moves in cache...";
    }

    Engine::find_move_list(&self.cache, &game_state.board);
    let move_list = self.cache.get_move_list(&game_state.board).unwrap();
    if move_list.is_empty() {
      return line_string
             + " - Empty move list ?? (check what happened it should not be GameStatus::OnGoing";
    }
    let best_move = move_list[0];
    let mut best_new_state = game_state.clone();
    best_new_state.apply_move(&best_move);
    if evaluation_cache.depth == 0 {
      return best_move.to_string() + " - Not evaluated";
    }

    best_move.to_string()
    + " "
    + self.get_line_string(&best_new_state, Color::opposite(side_to_play), ttl - 1).as_str()
  }

  /// Prints the evaluation result in the console
  pub fn print_evaluations(&self) {
    let lines = self.analysis.result.lock().unwrap();
    let position_eval = lines.get_eval().map_or(String::from("-"), |e| e.to_string());

    println!("Score for position {}: {}\n{}", self.position.to_fen(), position_eval, lines,);
  }

  //----------------------------------------------------------------------------
  // Engine State

  /// Captures the timestamp of now in an analysis in the engine state
  fn set_start_time(&self) {
    *self.state.start_time.lock().unwrap() = Instant::now();
  }

  /// Retrieves the start time
  ///
  /// ### Arguments
  ///
  /// * `max_depth`: Maximum amount of time, in milliseconds, to spend resolving a position
  pub fn get_start_time(&self) -> Instant {
    *self.state.start_time.lock().unwrap()
  }

  //----------------------------------------------------------------------------
  // Position calculations

  /// Looks at the cache and makes sure we have a move list known for
  /// the position / Game State
  ///
  /// ### Arguments
  ///
  /// * cache:      EngineCache where the move list is stored at the end.
  /// * board:      Board configuration to determine a move list
  fn find_move_list(cache: &EngineCache, board: &Board) {
    // Check that we know the moves:
    if !cache.has_move_list(board) {
      cache.set_move_list(board, &board.get_moves());
    }
  }

  /// Updates the Alpha/Beta values based on the eval and side to play
  ///
  /// ### Arguments
  ///
  /// * color:      Side to move on the board
  /// * eval:       Evaluation after color has moved
  /// * alpha:      Previous alpha value
  /// * beta:       Previous beta value
  #[inline]
  fn update_alpha_beta(color: Color, eval: Score, alpha: &mut Score, beta: &mut Score) {
    match color {
      Color::White => {
        if *alpha < eval {
          *alpha = eval;
        }
      },
      Color::Black => {
        if *beta > eval {
          *beta = eval;
        }
      },
    }
  }

  //----------------------------------------------------------------------------
  // Engine Evaluation

  /// Search and evaluate a position with the configured engine options
  ///
  /// ### Arguments
  ///
  /// * `self`: Engine to use to store all the calculations
  /// * `game_state`:    Game state to start from in the evaluation tree
  /// * `depth`:      Current depth at which we are in the search
  /// * `max_depth`:  Depth at which to stop
  /// * `alpha`:      Alpha value for the Alpha/Beta pruning
  /// * `Beta`:       Beta value for the Alpha/Beta pruning
  /// * `start_time`: Time at which we started resolving the chess position
  fn search(&self,
            game_state: &GameState,
            depth: usize,
            max_depth: usize,
            mut alpha: Score,
            mut beta: Score)
            -> Option<SearchResult> {
    if self.stop_requested() {
      return None;
    }
    let search_calls = self.state.search_calls.fetch_add(1, Ordering::Relaxed);
    if search_calls & (TIME_CHECK_SEARCH_INTERVAL - 1) == 0 && self.has_been_searching_too_long() {
      // Make sure that the rest of the search tree stops right away too.
      self.set_stop_requested(true);
      return None;
    }

    if depth > max_depth {
      // println!("Reached maximum depth {max_depth}. Stopping search");
      return None;
    }

    // Check that we know the moves
    let moves = self.cache.with_move_list(&game_state.board, MoveList::new_from_slice);
    // Depth of the current iteration of the iterative deepening.
    let analysis_depth = self.analysis.get_depth();
    let mut result =
      SearchResult::new(NUMBER_OF_MOVES_IN_SEARCH_RESULTS, game_state.board.side_to_play);
    // Set when we did not look at all the moves (cut-off or interrupted)
    let mut pruned = false;

    for &m in moves.get_moves() {
      // println!("Move: {} - alpha-beta: {}/{}", m.to_string(), alpha, beta);
      if alpha.saturating_sub(SEARCH_PRUNING_MARGIN) > beta {
        // TODO: Test this a bit better, I think we are pruning stuff that
        // should not get pruned. println!("Skipping {} as it is pruned
        // {}/{}",game_state.to_fen(), alpha, beta);
        pruned = true;
        break;
      }

      // Captures at the leaves are resolved by the quiescence search.
      let mut max_line_depth = max_depth;

      let mut new_game_state = game_state.clone();
      new_game_state.apply_move(&m);

      // Checks are forcing, the opponent has few replies: extend the line so
      // that we do not stop the search in the middle of a mating attack.
      if new_game_state.board.checkers != 0 && max_line_depth < analysis_depth + MAX_CHECK_EXTENSION
      {
        max_line_depth += 1;
        self.analysis.update_selective_depth(max_line_depth);
      }

      // Check if we just repeated the position too much or did not make
      // progress.
      let draw = can_declare_draw(&new_game_state);
      if draw != GameStatus::Ongoing {
        // Do not cache this: repetitions / 50 moves depend on the path that led
        // to the board, the same board reached via another line is not a draw.
        Engine::update_alpha_beta(game_state.board.side_to_play, 0, &mut alpha, &mut beta);
        result.update(VariationWithEval::new_repetition_draw(m));
        continue;
      }

      // Check if we already looked at this position.
      let mut eval_cache = self.cache.get_eval(&new_game_state.board).unwrap_or_default();
      if eval_cache.depth > 0 && depth >= max_line_depth {
        // Nothing to do, we already looked at this position.
        // FIXME: If the position appears in another variation but leads to a
        // draw, e.g. 3 fold repetitions, we won't detect it and skip
        // it. Get the alpha/ beta result propagated upwards.
        // The cached mate distance is counted from the child position: add the
        // ply of this move, like `push_move_to_variations` does.
        let eval = decrement_mate_score(eval_cache.eval);
        Engine::update_alpha_beta(game_state.board.side_to_play, eval, &mut alpha, &mut beta);
        result.update(VariationWithEval::new_from_move_with_depth(eval, eval_cache.depth, m));
        continue;
      }

      if eval_cache.depth == 0 {
        eval_cache.game_status = is_game_over(&self.cache, &new_game_state.board);
      };

      // No need to look at other moves in this variation if we found a
      // checkmate for the side to play:
      let mut eval = get_eval_from_game_status(eval_cache.game_status).unwrap_or_default();
      if eval_cache.game_status == GameStatus::WhiteWon
         || eval_cache.game_status == GameStatus::BlackWon
      {
        if get_epic_mate(&new_game_state.board).is_some() {
          // Epic mates (smothered, big mating nets) get a higher score
          eval = eval.signum() * (EPIC_MATE - 1);
        }

        // FIXME: We should make this a bit smarter, go one level up to save the
        // good move Also if there is an eval swing, not just checkmate.
        self.cache.add_killer_move(&m);
        Engine::update_alpha_beta(game_state.board.side_to_play, eval, &mut alpha, &mut beta);
        result.update(VariationWithEval::new_from_move(eval, m));
        eval_cache.eval = eval;
        self.cache.set_eval(&new_game_state.board, eval_cache);
        // Don't look at other moves when we found a checkmate
        break;
      }

      // Search more if the game is not over.
      if eval_cache.game_status == GameStatus::Ongoing {
        if depth < max_line_depth {
          let sub_result = self.search(&new_game_state, depth + 1, max_line_depth, alpha, beta);
          if sub_result.is_none() {
            pruned = true;
            continue;
          }
          let mut sub_result = sub_result.unwrap();
          sub_result.push_move_to_variations(m);
          if !sub_result.is_empty() {
            result.update(sub_result.get(0));
            Engine::update_alpha_beta(game_state.board.side_to_play,
                                      result.get_eval().expect("valid eval"),
                                      &mut alpha,
                                      &mut beta);
          }
        } else if eval_cache.game_status == GameStatus::Ongoing && depth >= max_line_depth {
          // Evaluate our position, resolving the captures first so that we do
          // not stop the line in the middle of an exchange. Like the search,
          // keep the values a bit outside of the alpha/beta window.
          let (low, high) = (alpha.saturating_sub(SEARCH_PRUNING_MARGIN),
                             beta.saturating_add(SEARCH_PRUNING_MARGIN));
          let leaf_eval = self.quiescence_search(&new_game_state, 0, low, high);
          // Mate distances are counted from the leaf: add the ply of the move.
          eval = decrement_mate_score(leaf_eval);

          result.update(VariationWithEval::new_from_move(eval, m));
          Engine::update_alpha_beta(game_state.board.side_to_play, eval, &mut alpha, &mut beta);

          // Save the eval of the leaf, only if it is exact: outside of the
          // window, it is just a bound. (Searched children save their own eval
          // at the end of their search)
          if low < leaf_eval && leaf_eval < high {
            eval_cache.eval = leaf_eval;
            eval_cache.depth = 1;
            self.cache.set_eval(&new_game_state.board, eval_cache);
          }
        }
      } else {
        // Here the game is no longer ongoing (draw, etc.)
        Engine::update_alpha_beta(game_state.board.side_to_play, eval, &mut alpha, &mut beta);
        result.update(VariationWithEval::new_from_move(eval, m));
        eval_cache.eval = eval;
        eval_cache.depth = 1;
        self.cache.set_eval(&new_game_state.board, eval_cache);
      }
    } // for m in &moves

    // Only save the eval when we looked at all the moves. If we pruned, the
    // eval is just a bound for this alpha/beta window and not the value of the
    // position: caching it would leak it into other lines and later searches.
    // Same for an eval coming from a draw by repetition at the end of the best
    // line: it depends on the path, the same board in another line is not a
    // draw.
    let best_line_is_repetition = result.variations.first().is_some_and(|v| v.repetition);
    if !result.is_empty() && !pruned && !best_line_is_repetition {
      let best_move = &result.get_best_move().expect("Valid move in non-empty result");
      debug_assert!(!best_move.is_null(), "Best move is NULL.");
      let mut pv = game_state.clone();
      pv.apply_move(best_move);
      let best_move_depth = self.cache.get_eval(&pv.board).unwrap_or_default().depth;
      let eval = result.get_eval().expect("valid eval in non-empty result");

      // The game status is the one of this board (we are searching it, so it
      // is ongoing), not the one of the board after the best move.
      self.cache.set_eval(&game_state.board, EvaluationCache { game_status: GameStatus::Ongoing,
                                                               eval,
                                                               depth: best_move_depth + 1 });
    }

    if !result.is_empty() {
      // Influence next visit by promoting the multi_pv best moves to be first
      // in the move list
      let top_moves = result.get_top_moves();
      let mut reordered_moves = MoveList::new_from_slice(&top_moves);
      for &m in moves.get_moves() {
        if !top_moves.contains(&m) {
          reordered_moves.add(m);
        }
      }

      debug_assert!(moves.len() == reordered_moves.len(),
                    "Reordered moves should be the same length {} -> {}",
                    moves.len(),
                    reordered_moves.len());
      self.cache.set_move_list(&game_state.board, reordered_moves.get_moves());
    }

    // Return our result
    Some(result)
  }

  /// Static evaluation of a position, at the leaves of the search.
  fn static_eval(&self, game_state: &GameState) -> Score {
    let eval = evaluate_board(game_state);
    self.analysis.increment_nodes_visited();

    // FIXME:  NNUE eval is still too slow, we should implement
    // incremental updates
    if self.options.use_nnue {
      let nnue_eval = self.nnue.lock().unwrap().eval(game_state);
      return ((eval as i32 + nnue_eval as i32) / 2) as Score;
    }
    eval
  }

  /// Evaluates a leaf of the search by playing all the captures until the
  /// position is quiet, so that the eval does not stop in the middle of an
  /// exchange or miss a piece left hanging (horizon effect).
  ///
  /// The side to play can also "stand pat": not capture anything and keep the
  /// static eval of the position. When in check, all the moves are searched
  /// (standing pat is not an option).
  ///
  /// ### Arguments
  ///
  /// * `game_state`: Position to evaluate
  /// * `ply`:        Number of plies already played in the quiescence search
  /// * `alpha`:      Best score White is guaranteed (White's point of view)
  /// * `beta`:       Best score Black is guaranteed (White's point of view)
  ///
  /// ### Return value
  ///
  /// The eval of the position. Mate distances are counted from this position.
  /// It is exact when it is strictly inside the (`alpha`, `beta`) window,
  /// otherwise it is only a bound.
  fn quiescence_search(&self,
                       game_state: &GameState,
                       ply: usize,
                       mut alpha: Score,
                       mut beta: Score)
                       -> Score {
    let board = &game_state.board;
    let in_check = board.checkers != 0;
    let white_to_play = board.side_to_play == Color::White;

    // Moves to look at, most valuable victim first, then least valuable
    // attacker (MVV-LVA): they are the most likely to cause a cut-off.
    // The move lists are not cached: these positions are not searched.
    let mut moves: Vec<(Score, Score, Move)> =
      board.get_moves()
           .into_iter()
           .filter(|m| in_check || m.is_capture())
           .map(|m| {
             let victim = m.get_captured_piece().map_or(0, Piece::material_value_from_type);
             let attacker = Piece::material_value_from_u8(board.pieces.get(m.src() as u8));
             (victim * 8 - attacker / 8, victim, m)
           })
           .collect();
    if in_check && moves.is_empty() {
      // Checkmate: the side to play lost
      return if white_to_play { -MATE } else { MATE };
    }

    let mut best = if white_to_play { -SCORE_INF } else { SCORE_INF };
    if !in_check || ply >= QUIESCENCE_MAX_PLY {
      let stand_pat = self.static_eval(game_state);
      if ply >= QUIESCENCE_MAX_PLY || moves.is_empty() {
        // Note: stalemates (no move at all) are not detected here.
        return stand_pat;
      }
      if white_to_play {
        if stand_pat >= beta {
          return stand_pat;
        }
        alpha = alpha.max(stand_pat);
      } else {
        if stand_pat <= alpha {
          return stand_pat;
        }
        beta = beta.min(stand_pat);
      }
      best = stand_pat;
    }

    moves.sort_unstable_by_key(|&(order, _, _)| -(order as i32));
    for (_, victim, m) in moves {
      // Delta pruning: if even winning the captured piece for free cannot
      // improve our score, the next captures (smaller victims) cannot either.
      if !in_check {
        let gain = victim + QUIESCENCE_DELTA_MARGIN;
        if (white_to_play && best.saturating_add(gain) <= alpha)
           || (!white_to_play && best.saturating_sub(gain) >= beta)
        {
          break;
        }
      }
      let mut new_game_state = game_state.clone();
      new_game_state.apply_move(&m);
      // Mate distances are counted from the child: add the ply of the move.
      let eval =
        decrement_mate_score(self.quiescence_search(&new_game_state, ply + 1, alpha, beta));
      if white_to_play {
        best = best.max(eval);
        alpha = alpha.max(eval);
      } else {
        best = best.min(eval);
        beta = beta.min(eval);
      }
      if alpha >= beta {
        break;
      }
    }

    best
  }

  /// Checks the best move in the result and check if it is a winning sequence
  /// for the color indicated in argument
  #[inline]
  fn best_move_is_mating_sequence(color: Color, eval: Score) -> bool {
    if !is_mate_score(eval) {
      return false;
    }
    match color {
      Color::White => eval > 0,
      Color::Black => eval < 0,
    }
  }
}

impl Default for Engine {
  fn default() -> Self {
    Self::new(false)
  }
}
