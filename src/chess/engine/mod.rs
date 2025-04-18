pub mod books;
pub mod cache;
pub mod config;
pub mod eval;
pub mod game_history;
pub mod nnue;
pub mod search;
pub mod search_result;
pub mod tables;

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
use super::model::piece::Color;
use crate::engine::search_result::VariationWithEval;
use crate::model::board::Board;
use books::*;
use config::options::*;
use config::play_style::*;
use log::*;
use nnue::NNUE;
use rand::seq::SliceRandom;
use std::cmp::min;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// -----------------------------------------------------------------------------
// Constants
pub const NNUE_FILE: &str = "engine/nnue/net.nnue";
pub const NUMBER_OF_MOVES_IN_SEARCH_RESULTS: usize = 30;

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
  pub depth:           Arc<Mutex<usize>>,
  /// Represent how deep the analysis is/was
  pub selective_depth: Arc<Mutex<usize>>,
  /// Represents how many nodes we visited in the search
  pub nodes_visited:   Arc<Mutex<usize>>,
}

#[derive(Clone, Debug)]
struct EngineState {
  /// Indicates if the engine is active at resolving positions
  pub active:         Arc<Mutex<bool>>,
  /// Indicates that we want the engine to stop resolving positions
  pub stop_requested: Arc<Mutex<bool>>,
  /// Indicates when the engine was requested to start searching
  pub start_time:     Arc<Mutex<Instant>>,
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
    let mut analysis_depth = self.depth.lock().unwrap();
    *analysis_depth = depth;
  }

  /// Increments the depth we have reached during the analysis
  ///
  /// ### Arguments
  ///
  /// * `self`:   Instance of the Chess Engine
  pub fn increment_depth(&self) {
    let mut analysis_depth = self.depth.lock().unwrap();
    *analysis_depth += 1;
  }

  /// Decrements the depth we have reached during the analysis
  ///
  /// ### Arguments
  ///
  /// * `self`:   Instance of the Chess Engine
  pub fn decrement_depth(&self) {
    let mut analysis_depth = self.depth.lock().unwrap();
    if *analysis_depth > 0 {
      *analysis_depth -= 1;
    }
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
    *self.depth.lock().unwrap()
  }

  /// Sets the selective depth we have reached during the analysis
  ///
  /// ### Arguments
  ///
  /// * `self`:   Instance of the Chess Engine
  /// * `selective_depth`:  New depth to set.
  pub fn set_selective_depth(&self, depth: usize) {
    let mut selective_depth = self.selective_depth.lock().unwrap();
    *selective_depth = depth;
  }

  /// Updates the selective depth if the new value is higher than the current
  /// value.
  ///
  /// ### Arguments
  ///
  /// * `self`:   Instance of the Chess Engine
  /// * `selective_depth`:  New depth to set.
  pub fn update_selective_depth(&self, depth: usize) {
    let mut selective_depth = self.selective_depth.lock().unwrap();
    if *selective_depth < depth {
      *selective_depth = depth;
    }
  }

  /// Increments the selective depth we have reached during the analysis
  ///
  /// ### Arguments
  ///
  /// * `self`:   Instance of the Chess Engine
  pub fn increment_selective_depth(&self) {
    let mut selective_depth = self.selective_depth.lock().unwrap();
    *selective_depth += 1;
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
    *self.selective_depth.lock().unwrap()
  }

  /// Increments the nodes we visited
  pub fn increment_nodes_visited(&self) {
    let mut analysis_nodes_visited = self.nodes_visited.lock().unwrap();
    *analysis_nodes_visited += 1;
  }

  pub fn get_nodes_visited(&self) -> usize {
    *self.nodes_visited.lock().unwrap()
  }

  pub fn set_nodes_visited(&self, value: usize) {
    let mut analysis_nodes_visited = self.nodes_visited.lock().unwrap();
    *analysis_nodes_visited = value;
  }
}

impl Default for Analysis {
  fn default() -> Self {
    Analysis { result:          Arc::new(Mutex::new(SearchResult::new(1, Color::White))),
               depth:           Arc::new(Mutex::new(0)),
               selective_depth: Arc::new(Mutex::new(0)),
               nodes_visited:   Arc::new(Mutex::new(0)), }
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

type AsyncResult = std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;

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
               state:    EngineState { active:         Arc::new(Mutex::new(false)),
                                       stop_requested: Arc::new(Mutex::new(false)),
                                       start_time:     Arc::new(Mutex::new(Instant::now())), },
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
    return *self.state.active.lock().unwrap();
  }

  /// Helper function that sets the "active" bool value in the engine
  ///
  /// ### Arguments
  ///
  /// * `active`: The new value to apply to active
  fn set_engine_active(&self, active: bool) {
    let mut s = self.state.active.lock().unwrap();
    *s = active;
  }

  /// Checks if the engine has been requested to stop evaluating
  ///
  /// ### Return value
  ///
  /// * True if the engine should stop searching positions, False otherwise
  pub fn stop_requested(&self) -> bool {
    return *self.state.stop_requested.lock().unwrap();
  }

  /// Helper function that sets the "stop_requested" bool value in the engine
  ///
  /// ### Arguments
  ///
  /// * `stop_requested`: The new value to apply to stop_requested
  fn set_stop_requested(&self, stop_requested: bool) {
    let mut s = self.state.stop_requested.lock().unwrap();
    *s = stop_requested;
  }

  /// Checks if the engine has been searching a position for too long
  /// It will compare the start time with the current time and the maximum time
  /// set in the engine options
  ///
  /// ### Return value
  ///
  /// True if the (current_time - start_time) is larger than options.max_time
  /// and max_time is set to a non-zero value.
  fn has_been_searching_too_long(&self) -> bool {
    let max_time = self.options.max_search_time;
    if max_time == 0 {
      return false;
    }
    let start_time = self.get_start_time();
    (Instant::now() - start_time) > Duration::from_millis(max_time as u64)
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
  /// * capacity_mb : Size in MB to use for the engine cache tables (there are 2
  ///   of them).
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
  /// * `chess_move`: Notation of the chess move to apply on the current
  ///   position
  pub fn apply_move(&mut self, chess_move: &str) {
    if self.is_active() {
      self.stop();
    }

    // FIXME: THis does not capture evals when opponent applies a move
    // let eval = self.get_best_eval();
    // let mv = self.position.last_moves.last().unwrap_or(&Move::null()).clone();
    // self.history.add(self.position.to_fen(), mv, eval as isize);

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
    self.analysis.set_nodes_visited(0);

    // Make sure we know the move list:
    Engine::find_move_list(&self.cache, &self.position.board);

    // First check if we are in a known book position. If yes, just return the known
    // list
    let play_style = self.options.play_style;
    let book_entry = get_book_moves(&self.position.board, play_style == PlayStyle::Provocative);
    if book_entry.is_some() {
      info!("Known position, returning book moves for {:?} play",
            play_style);
      let mut move_list = book_entry.unwrap();
      let mut rng = rand::thread_rng();
      move_list.shuffle(&mut rng);

      let mut result: SearchResult =
        SearchResult::new(self.options.multi_pv, self.position.board.side_to_play);

      for m in &move_list {
        result.update(VariationWithEval::new_from_move(0.0, *m));
      }
      self.analysis.update_result(result);

      // We are done
      self.print_uci_info();
      self.print_uci_best_move();
      self.set_stop_requested(false);
      self.set_engine_active(false);
      return;
    }

    // If we have only one legal move, we should just give it a score and play it
    // instantaneously.
    let moves = self.cache.get_move_list(&self.position.board).unwrap();
    if moves.len() == 1 {
      debug!("Single or no move available. Just evaluating quickly");
      let mut game_state = self.position.clone();
      game_state.apply_move(&moves[0]);

      let mut evaluation_cache = self.cache.get_eval(&game_state.board).unwrap_or_default();
      if evaluation_cache.depth == 0 {
        let game_status = is_game_over(&self.cache, &game_state.board);
        let mut eval = get_eval_from_game_status(game_status);
        if eval.is_nan() {
          eval = evaluate_board(&game_state);
        }
        evaluation_cache = EvaluationCache { game_status,
                                             eval,
                                             depth: 1 };
        self.cache.set_eval(&game_state.board, evaluation_cache);
      }
      let mut result: SearchResult =
        SearchResult::new(self.options.multi_pv, game_state.board.side_to_play);
      result.update(VariationWithEval::new_from_move(evaluation_cache.eval, moves[0]));

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
    while !self.has_been_searching_too_long() && !self.stop_requested() {
      self.analysis.increment_depth();
      self.analysis.increment_selective_depth();

      // Try to search for the current depth
      let result = self.search(&self.position.clone(),
                               1,
                               self.analysis.get_depth(),
                               f32::MIN,
                               f32::MAX);

      if self.has_been_searching_too_long() || self.stop_requested() || result.is_none() {
        // Toss away unfinished depths
        self.analysis.decrement_depth();
        break;
      }

      // Depth completed - print UCI result if needed
      let result = result.unwrap(); // Safe due to is_none() above
      let best_eval = result.get_eval().unwrap();
      self.analysis.update_result(result);
      self.print_uci_info();

      // If the best move is just winning for us, stop searching unless requested to.
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

  /// Returns the best move saved in the analysis
  pub fn get_best_move(&self) -> Option<Move> {
    let analysis = self.analysis.result.lock().unwrap();
    analysis.get_best_move()
  }

  /// Returns the best eval saved in the analysis
  pub fn get_eval(&self) -> Option<f32> {
    let analysis = self.analysis.result.lock().unwrap();
    analysis.get_eval()
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

    for i in 0..min(multi_pv_setting, result.variations.len()) {
      let eval = result.variations[i].eval;
      let score_string = if eval.abs() > 100.0 {
        format!("score mate {}", ((eval.signum() * 200.0) - eval) as isize)
      } else {
        format!("score cp {}", (eval * 100.0) as isize)
      };
      let multi_pv_string = if multi_pv_setting > 1 {
        String::from(format!(" multipv {} ", i + 1))
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
               result.variations[i].variation,);
    }
  }

  /// Prints the best move
  #[inline]
  pub fn print_uci_best_move(&self) {
    if self.options.uci {
      println!("bestmove {}", self.get_best_move().unwrap_or(Move::null()));
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
    if evaluation_cache.eval.is_nan() {
      return best_move.to_string() + " - Not evaluated";
    }

    best_move.to_string()
    + " "
    + self.get_line_string(&best_new_state, Color::opposite(side_to_play), ttl - 1).as_str()
  }

  /// Prints the evaluation result in the console
  pub fn print_evaluations(&self) {
    let lines = self.analysis.result.lock().unwrap();
    let position_eval = if lines.is_empty() { f32::NAN } else { lines.variations[0].eval };

    println!("Score for position {}: {}\n{}",
             self.position.to_fen(),
             position_eval,
             lines,);
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
  /// * `max_depth`: Maximum amount of time, in milliseconds, to spend resolving
  ///   a position
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
  fn update_alpha_beta(color: Color, eval: f32, alpha: &mut f32, beta: &mut f32) {
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
            mut alpha: f32,
            mut beta: f32)
            -> Option<SearchResult> {
    if self.stop_requested() || self.has_been_searching_too_long() {
      return None;
    }

    if depth > max_depth {
      // println!("Reached maximum depth {max_depth}. Stopping search");
      return None;
    }

    // Check that we know the moves
    Engine::find_move_list(&self.cache, &game_state.board);
    let moves = self.cache.get_move_list(&game_state.board).unwrap();
    let mut result = SearchResult::new(NUMBER_OF_MOVES_IN_SEARCH_RESULTS,
                                       game_state.board.side_to_play);

    for m in moves {
      // println!("Move: {} - alpha-beta: {}/{}", m.to_string(), alpha, beta);
      // Here we have low trust in eval accuracy, so it has to be more than
      // good gap between alpha and beta before we prune.
      if (alpha - 0.5) > beta {
        // TODO: Test this a bit better, I think we are pruning stuff that should not
        // get pruned. println!("Skipping {} as it is pruned
        // {}/{}",game_state.to_fen(), alpha, beta);
        break;
      }

      // If we are looking at a capture, make sure that we analyze possible
      // recaptures by increasing temporarily the maximum depth
      let mut max_line_depth = max_depth;
      if depth == max_depth && m.is_piece_capture() {
        if depth < self.analysis.get_depth() + 3 {
          max_line_depth = max_depth + 1;
          self.analysis.update_selective_depth(max_line_depth);
          // println!("Continuing to depth {max_line_depth}");
        }
      }

      let mut new_game_state = game_state.clone();
      new_game_state.apply_move(&m);

      // Check if we just repeated the position too much or did not make progress.
      let draw = can_declare_draw(&new_game_state);
      if draw != GameStatus::Ongoing {
        self.cache.set_eval(&new_game_state.board, EvaluationCache { game_status: draw,
                                                                     eval:        0.0,
                                                                     depth:       1, });
        Engine::update_alpha_beta(game_state.board.side_to_play, 0.0, &mut alpha, &mut beta);
        result.update(VariationWithEval::new_from_move(0.0, m));
        continue;
      }

      // Check if we already looked at this position.
      let mut eval_cache = self.cache.get_eval(&new_game_state.board).unwrap_or_default();
      if eval_cache.depth > 0 && depth >= max_line_depth {
        // Nothing to do, we already looked at this position.
        // FIXME: If the position appears in another variation but leads to a draw, e.g.
        // 3 fold repetitions, we won't detect it and skip it. Get the alpha/
        // beta result propagated upwards.
        Engine::update_alpha_beta(game_state.board.side_to_play,
                                  eval_cache.eval,
                                  &mut alpha,
                                  &mut beta);
        result.update(VariationWithEval::new_from_move(eval_cache.eval, m));
        continue;
      }

      if eval_cache.depth == 0 {
        eval_cache.game_status = is_game_over(&self.cache, &new_game_state.board);
      };

      // No need to look at other moves in this variation if we found a checkmate for
      // the side to play:
      let mut eval = get_eval_from_game_status(eval_cache.game_status);
      if eval_cache.game_status == GameStatus::WhiteWon
         || eval_cache.game_status == GameStatus::BlackWon
      {
        if is_smothered_mate(&game_state.board, eval_cache.game_status) {
          // Just assign a higher score to smothered mates
          if eval_cache.game_status == GameStatus::WhiteWon {
            eval = 220.0;
          } else {
            eval = -220.0;
          }
        }

        // FIXME: We should make this a bit smarter, go one level up to save the good
        // move Also if there is an eval swing, not just checkmate.
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
          // Evaluate our position
          eval = evaluate_board(&new_game_state);
          self.analysis.increment_nodes_visited();

          // FIXME:  NNUE eval is still too slow, we should implement incremental updates
          if depth > 10 && self.options.use_nnue {
            let nnue_eval = self.nnue.lock().unwrap().eval(&new_game_state);
            // println!("board: {} - Eval: {} - NNUE Eval: {} - final eval
            // {}",new_game_state.to_fen(), eval,nnue_eval,eval * 0.5 + nnue_eval * 0.5);
            eval = eval * 0.5 + nnue_eval * 0.5;
          }

          result.update(VariationWithEval::new_from_move(eval, m));
          Engine::update_alpha_beta(game_state.board.side_to_play, eval, &mut alpha, &mut beta);
        }
      } else {
        // Here the game is no longer ongoing (draw, etc.)
        Engine::update_alpha_beta(game_state.board.side_to_play, eval, &mut alpha, &mut beta);
        result.update(VariationWithEval::new_from_move(eval, m));
      }

      // Save the intermediate result in the transposition table
      if !result.is_empty() {
        eval_cache.eval = result.get_eval().expect("Result is not empty, eval should be valid");
        eval_cache.depth = max_line_depth - depth + 1;
        self.cache.set_eval(&new_game_state.board, eval_cache);
      }
    } // for m in &moves

    if !result.is_empty() {
      // Save in the cache table:

      let best_move = &result.get_best_move().expect("Valid move in non-empty result");
      debug_assert!(!best_move.is_null(), "Best move is NULL.");
      let mut pv = game_state.clone();
      pv.apply_move(best_move);
      let mut best_move_eval = self.cache.get_eval(&pv.board).unwrap_or_default();

      best_move_eval.depth += 1;
      best_move_eval.eval = result.get_eval().expect("valid eval in non-empty result");
      self.cache.set_eval(&game_state.board, best_move_eval);

      // Influence next visit by promoting the multi_pv best moves to be first
      // in the move list
      let mut top_moves = result.get_top_moves();
      Engine::find_move_list(&self.cache, &game_state.board);
      let mut moves = self.cache.get_move_list(&game_state.board).unwrap().to_vec();
      let initial_length = moves.len();
      moves.retain(|&m| !top_moves.contains(&m));
      top_moves.extend(moves);
      top_moves.dedup();

      debug_assert!(initial_length == top_moves.len(),
                    "Reordered moves should be the same length {} -> {}",
                    initial_length,
                    top_moves.len());
      self.cache.set_move_list(&game_state.board, &top_moves);
    }

    // Return our result
    Some(result)
  }

  /// Checks the best move in the result and check if it is a winning sequence
  /// for the color indicated in argument
  #[inline]
  fn best_move_is_mating_sequence(color: Color, eval: f32) -> bool {
    if eval.is_nan() {
      return false;
    }
    match color {
      Color::White => eval > 150.0,
      Color::Black => eval < -150.0,
    }
  }
}

impl Default for Engine {
  fn default() -> Self {
    Self::new(false)
  }
}
