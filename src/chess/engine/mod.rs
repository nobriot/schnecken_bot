pub mod books;
pub mod cache;
pub mod development;
pub mod eval;
pub mod nnue;
pub mod square_affinity;
pub mod test;

use log::*;
use rand::seq::SliceRandom;
use std::cmp::min;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// Same module (engine)
use self::cache::engine_cache::EngineCache;
use self::cache::evaluation_table::EvaluationCache;
use self::eval::position::*;
use books::*;
use nnue::NNUE;

// Chess model
use super::model::game_state::GameState;
use super::model::game_state::GameStatus;
use super::model::game_state::START_POSITION_FEN;
use super::model::moves::Move;
use super::model::piece::Color;
use crate::model::board::Board;

// -----------------------------------------------------------------------------
// Constants
pub const NNUE_FILE: &str = "engine/nnue/net.nuue";

// -----------------------------------------------------------------------------
// Type definitions

pub type SearchResult = Vec<(Vec<Move>, f32)>;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum PlayStyle {
  /// Normal play style for the engine
  #[default]
  Normal,
  /// Engine will try to play very safe lines. Kind of good if the opponent is
  /// Stronger and we just want to draw
  Conservative,
  /// Try spectacular sacrifices to get to the king.
  Aggressive,
  /// Use this with weaker opponents, to play dangerous/provocative lines
  /// like the bongcloud.
  Provocative,
}

impl FromStr for PlayStyle {
  type Err = ();

  fn from_str(input: &str) -> Result<PlayStyle, Self::Err> {
    match input.to_lowercase().as_str() {
      "normal" => Ok(PlayStyle::Normal),
      "conservative" => Ok(PlayStyle::Conservative),
      "aggressive" => Ok(PlayStyle::Aggressive),
      "provocative" => Ok(PlayStyle::Provocative),
      _ => Err(()),
    }
  }
}

#[derive(Clone, Debug, Default)]
pub struct Options {
  /// Whether this engine is used with the UCI interface and it
  /// should print information when searching
  pub uci: bool,
  /// Continue thinking even if we found a winning sequence.
  pub ponder: bool,
  /// Maximum depth to go at
  pub max_depth: usize,
  /// time in miliseconds to spend on a calculation
  /// Set to 0 for no limit
  pub max_time: usize,
  /// Number of threads to use for the search.
  pub max_threads: usize,
  /// Number of threads to use for the search.
  pub use_nnue: bool,
  /// Debug mode : The engine will print additional info (info string <debug string>)
  /// if this is set to true
  pub debug: bool,
  /// Set the play style of the engine.
  pub style: PlayStyle,
  /// Number of best lines that the engine will return.
  pub multi_pv: usize,
}

#[derive(Clone, Debug)]
struct Analysis {
  /// After the search, the nth best lines will be saved in this vector.
  pub best_lines: Arc<Mutex<SearchResult>>,
  /// Represent how deep the analysis is/was
  pub depth: Arc<Mutex<usize>>,
  /// Represent how deep the analysis is/was
  pub selective_depth: Arc<Mutex<usize>>,
}

#[derive(Clone, Debug)]
struct EngineState {
  /// Indicates if the engine is active at resolving positions
  pub active: Arc<Mutex<bool>>,
  /// Indicates that we want the engine to stop resolving positions
  pub stop_requested: Arc<Mutex<bool>>,
  /// Indicates when the engine was requested to start searching
  pub start_time: Arc<Mutex<Instant>>,
}

impl Analysis {
  /// Resets the analysis
  pub fn reset(&self) {
    self.best_lines.lock().unwrap().clear();
    self.set_selective_depth(0);
    self.set_depth(0);
  }

  /// Saves the nth best continuations in the analysis.best_lines
  ///
  /// ### Arguments
  ///
  /// * `self`:    Analysis struct reference
  /// * `result`:  Sorted vector with best variations.
  ///
  ///
  pub fn update_result(&self, result: &SearchResult) {
    let mut pvs = self.best_lines.lock().unwrap();
    *pvs = result.clone();
  }

  /// Sets the depth we have reached during the analysis
  ///
  /// ### Arguments
  ///
  /// * `self`:   Instance of the Chess Engine
  /// * `depth`:  New depth to set.
  ///
  ///
  pub fn set_depth(&self, depth: usize) {
    let mut analysis_depth = self.depth.lock().unwrap();
    *analysis_depth = depth;
  }

  /// Increments the depth we have reached during the analysis
  ///
  /// ### Arguments
  ///
  /// * `self`:   Instance of the Chess Engine
  ///
  pub fn increment_depth(&self) {
    let mut analysis_depth = self.depth.lock().unwrap();
    *analysis_depth += 1;
  }

  /// Decrements the depth we have reached during the analysis
  ///
  /// ### Arguments
  ///
  /// * `self`:   Instance of the Chess Engine
  ///
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
  ///
  pub fn get_depth(&self) -> usize {
    self.depth.lock().unwrap().clone()
  }

  /// Sets the selective depth we have reached during the analysis
  ///
  /// ### Arguments
  ///
  /// * `self`:   Instance of the Chess Engine
  /// * `selective_depth`:  New depth to set.
  ///
  ///
  pub fn set_selective_depth(&self, depth: usize) {
    let mut selective_depth = self.selective_depth.lock().unwrap();
    *selective_depth = depth;
  }

  /// Updates the selective depth if the new value is higher than the current value.
  ///
  /// ### Arguments
  ///
  /// * `self`:   Instance of the Chess Engine
  /// * `selective_depth`:  New depth to set.
  ///
  ///
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
  ///
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
  ///
  pub fn get_selective_depth(&self) -> usize {
    self.selective_depth.lock().unwrap().clone()
  }
}

impl Default for Analysis {
  fn default() -> Self {
    Analysis {
      best_lines: Arc::new(Mutex::new(Vec::new())),
      depth: Arc::new(Mutex::new(0)),
      selective_depth: Arc::new(Mutex::new(0)),
    }
  }
}

#[derive(Clone)]
pub struct Engine {
  pub position: GameState,
  /// State of the analysis for the game state.
  analysis: Analysis,
  /// Position cache, used to speed up processing
  cache: EngineCache,
  /// Engine options
  options: Arc<Mutex<Options>>,
  /// Whether the engine is active of not, and if we want to stop it.
  state: EngineState,
  /// NNUE
  nnue: Arc<Mutex<NNUE>>,
}

impl Engine {
  //----------------------------------------------------------------------------
  // Public functions

  /// Gets a new engine
  ///
  pub fn new() -> Self {
    initialize_chess_books();
    let nnue_path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), NNUE_FILE);

    let mut engine = Engine {
      position: GameState::default(),
      analysis: Analysis::default(),
      cache: EngineCache::new(),
      options: Arc::new(Mutex::new(Options {
        uci: true,
        ponder: false,
        max_depth: 20,
        max_time: 0,
        max_threads: 16,
        use_nnue: false,
        debug: false,
        style: PlayStyle::Normal,
        multi_pv: 3,
      })),
      state: EngineState {
        active: Arc::new(Mutex::new(false)),
        stop_requested: Arc::new(Mutex::new(false)),
        start_time: Arc::new(Mutex::new(Instant::now())),
      },
      nnue: Arc::new(Mutex::new(
        NNUE::load(nnue_path.as_str()).unwrap_or_default(),
      )),
    };

    engine.set_position(START_POSITION_FEN);
    engine
  }

  /// Checks if the engine is resolving a position
  ///
  /// ### Return value
  ///
  /// * True if searching a position, False otherwise
  ///
  pub fn is_active(&self) -> bool {
    return *self.state.active.lock().unwrap();
  }

  /// Helper function that sets the "active" bool value in the engine
  ///
  /// ### Arguments
  ///
  /// * `active`: The new value to apply to active
  ///
  fn set_engine_active(&self, active: bool) {
    let mut s = self.state.active.lock().unwrap();
    *s = active;
  }

  /// Checks if the engine has been requested to stop evaluating
  ///
  /// ### Return value
  ///
  /// * True if the engine should stop searching positions, False otherwise
  ///
  pub fn stop_requested(&self) -> bool {
    return *self.state.stop_requested.lock().unwrap();
  }

  /// Helper function that sets the "stop_requested" bool value in the engine
  ///
  /// ### Arguments
  ///
  /// * `stop_requested`: The new value to apply to stop_requested
  ///
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
  ///
  fn has_been_searching_too_long(&self) -> bool {
    let max_time = self.options.lock().unwrap().max_time;
    if max_time == 0 {
      return false;
    }
    let start_time = self.get_start_time();
    (Instant::now() - start_time) > Duration::from_millis(max_time as u64)
  }

  /// Clears the cache of the engine.
  ///
  /// Note: You should not invoke this function when the engine is active/searching.
  ///
  pub fn clear_cache(&self) {
    self.cache.clear();
  }

  /// Clears and resize the cache table size.
  ///
  /// Note: You should not invoke this function when the engine is active/searching.
  ///
  /// ### Arguments
  ///
  /// * self : Engine reference
  /// * capacity_mb : Size in MB to use for the engine cache tables (there are 2 of them).
  ///
  pub fn resize_cache_tables(&self, capacity_mb: usize) {
    self.cache.resize_tables(capacity_mb);
  }

  /// Resets the engine to a default state.
  /// Same as Engine::Default() or Engine::new()
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
  ///
  pub fn set_position(&mut self, fen: &str) {
    self.reset();
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
  ///
  pub fn apply_move(&mut self, chess_move: &str) {
    if self.is_active() {
      self.stop();
    }
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

    // Make sure we know the move list:
    Engine::find_move_list(&self.cache, &self.position.board);

    // First check if we are in a known book position. If yes, just return the known list
    let book_entry = get_book_moves(
      &self.position.board,
      self.options.lock().unwrap().style == PlayStyle::Provocative,
    );
    if book_entry.is_some() {
      info!("Known position, returning book moves");
      let mut move_list = book_entry.unwrap();
      move_list.shuffle(&mut rand::thread_rng());
      let mut result: SearchResult = Vec::new();
      for m in &move_list {
        let mut new_game_state = self.position.clone();
        new_game_state.apply_move(m);
        self.cache.set_eval(
          &new_game_state.board,
          EvaluationCache {
            game_status: GameStatus::Ongoing,
            eval: 0.0,
            depth: 1,
          },
        );
        result.push((vec![*m], 0.0));
      }
      self.analysis.update_result(&result);

      // We are done
      self.print_uci_info();
      self.print_uci_best_move();
      self.set_stop_requested(false);
      self.set_engine_active(false);
      return;
    }

    // If we have only one legal move, we should just give it a score and play it instantaneously.
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
        evaluation_cache = EvaluationCache {
          game_status,
          eval,
          depth: 1,
        };
        self.cache.set_eval(&game_state.board, evaluation_cache);
      }
      let result = vec![(vec![moves[0]], evaluation_cache.eval)];
      self.analysis.update_result(&result);
      self.analysis.set_depth(evaluation_cache.depth);
      self.analysis.set_selective_depth(evaluation_cache.depth);

      self.print_uci_info();
      self.print_uci_best_move();
      self.set_stop_requested(false);
      self.set_engine_active(false);
      return;
    }

    // Fill up a dummy result based on previous state, in case the main search does not reach a new depth
    let mut temporary_result: SearchResult = Vec::new();
    for m in &moves {
      let mut game_state = self.position.clone();
      game_state.apply_move(&m);
      let evaluation_cache = self.cache.get_eval(&game_state.board).unwrap_or_default();
      temporary_result.push((vec![*m], evaluation_cache.eval));
    }
    self.analysis.update_result(&temporary_result);

    // Main search
    while !self.has_been_searching_too_long() && !self.stop_requested() {
      self.analysis.increment_depth();
      self.analysis.increment_selective_depth();

      // Try to search for the current depth
      let result = self.search(
        &self.position.clone(),
        1,
        self.analysis.get_depth(),
        f32::MIN,
        f32::MAX,
      );

      if self.has_been_searching_too_long() || self.stop_requested() {
        // Toss away unfinished depths
        self.analysis.decrement_depth();
        break;
      }

      // Depth completed - print UCI result if needed
      self.analysis.update_result(&result);
      self.print_uci_info();

      // If the best move is just winning for us, stop searching unless requested to.
      if Engine::best_move_is_mating_sequence(self.position.board.side_to_play, &result)
        && self.options.lock().unwrap().ponder == false
      {
        debug!("Winning sequence found! Stopping search");
        break;
      }

      let max_depth = self.options.lock().unwrap().max_depth;
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
  pub fn get_best_move(&self) -> Move {
    let analysis = self.analysis.best_lines.lock().unwrap();

    if analysis.is_empty() || analysis[0].0.is_empty() {
      return Move::default();
    }

    return analysis[0].0[0];
  }

  /// Prints information to stdout for the GUI using UCI protocol
  /// Nothing will be sent if the UCI option is not set in the engine
  ///
  #[inline]
  pub fn print_uci_info(&self) {
    if self.get_uci() == false {
      return;
    }

    let best_lines = self.analysis.best_lines.lock().unwrap().clone();
    let depth = self.analysis.get_depth();
    let selective_depth = self.analysis.get_selective_depth();
    let start_time = self.get_start_time();
    let multi_pv_setting = self.options.lock().unwrap().multi_pv;

    for i in 0..min(multi_pv_setting, best_lines.len()) {
      let eval = best_lines[i].1;
      let line = best_lines[i].0.clone();
      let mut line_string = String::new();
      for m in &line {
        line_string += m.to_string().as_str();
        line_string += " ";
      }
      let score_string = if eval.abs() > 100.0 {
        format!("score mate {}", ((eval.signum() * 200.0) - eval) as isize)
      } else {
        format!("score cp {}", (eval * 100.0) as isize)
      };
      let multi_pv_string = if multi_pv_setting > 1 {
        String::from(format!(" multipv {} ", i + 1))
      } else {
        String::from(format!(""))
      };
      println!(
        "info {} depth {} seldepth {} nodes {} time {}{}pv {}",
        score_string,
        depth,
        selective_depth,
        self.cache.len(),
        (Instant::now() - start_time).as_millis(),
        multi_pv_string,
        line_string,
      );
    }
  }

  /// Prints tyhe best move
  ///
  #[inline]
  pub fn print_uci_best_move(&self) {
    if self.get_uci() {
      println!("bestmove {}", self.get_best_move());
    }
  }

  pub fn print_debug(&self, debug_info: &str) {
    if self.options.lock().unwrap().debug {
      println!("info string {}", debug_info);
    }
  }

  /// Returns the full analysis
  pub fn get_analysis(&self) -> SearchResult {
    return self.analysis.best_lines.lock().unwrap().clone();
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
  ///
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

    return best_move.to_string()
      + " "
      + self.get_line_string(&best_new_state, Color::opposite(side_to_play), ttl - 1).as_str();
  }

  /// Prints the evaluation result in the console
  ///
  pub fn print_evaluations(&self) {
    let lines = self.analysis.best_lines.lock().unwrap();

    let move_list = self.cache.get_move_list(&self.position.board);
    let mut i = 0;

    let position_eval = if lines.is_empty() { f32::NAN } else { lines[0].1 };

    if move_list.is_none() {
      println!("Cannot print results, we do not even know the move list!");
      return;
    }
    let move_list = move_list.unwrap();

    println!(
      "Score for position {}: {}",
      self.position.to_fen(),
      position_eval
    );
    for m in move_list {
      let mut used_lines = false;
      let mut new_state = self.position.clone();
      new_state.apply_move(&m);
      let eval_cache = self.cache.get_eval(&new_state.board).unwrap_or_default();
      let mut eval = eval_cache.eval;
      for line_moves in lines.iter() {
        if !line_moves.0.is_empty() && line_moves.0[0] == m {
          eval = line_moves.1;
          used_lines = true;
          println!(
            "Line {:<2}: Eval {:<7.2} @ depth {} - {}",
            i,
            eval,
            line_moves.0.len(),
            Move::vec_to_string(&line_moves.0)
          );
          break;
        }
      }

      if !used_lines {
        let ttl = if self.analysis.get_depth() > 0 { self.analysis.get_depth() - 1 } else { 0 };

        println!(
          "Line {:<2}: Eval {:<7.2} @ depth {} - {} {}",
          i,
          eval,
          eval_cache.depth,
          m,
          self.get_line_string(
            &new_state,
            Color::opposite(self.position.board.side_to_play),
            ttl,
          )
        );
      }

      i += 1;
    }
  }

  //----------------------------------------------------------------------------
  // Engine State

  /// Captures the timestamp of now in an analysis in the engine state
  ///
  fn set_start_time(&self) {
    *self.state.start_time.lock().unwrap() = Instant::now();
  }

  /// Retrives the start time
  ///
  /// ### Arguments
  ///
  /// * `max_depth`: Maximum amount of time, in milliseconds, to spend resolving a position
  ///
  pub fn get_start_time(&self) -> Instant {
    *self.state.start_time.lock().unwrap()
  }

  //----------------------------------------------------------------------------
  // Engine Options

  /// Configures if the engine should ponder when it finds a winning sequence
  /// (i.e. continue calculating alternative lines)
  ///
  pub fn set_ponder(&self, ponder: bool) {
    self.options.lock().unwrap().ponder = ponder;
  }

  /// Configure the depth at which to search with the engine.
  ///
  /// ### Arguments
  ///
  /// * `max_depth`: Maximum amount of time, in milliseconds, to spend resolving a position
  ///
  pub fn set_maximum_depth(&self, max_depth: usize) {
    self.options.lock().unwrap().max_depth = max_depth;
  }

  /// Sets a timelimit in ms on how long we search
  ///
  /// ### Arguments
  ///
  /// * `max_time`: Maximum amount of time, in milliseconds, to spend resolving a position
  ///
  pub fn set_search_time_limit(&self, max_time: usize) {
    self.options.lock().unwrap().max_time = max_time;
  }

  /// Sets the number of threads to use during a search
  ///
  /// ### Arguments
  ///
  /// * `max_threads`: Maximum number of threads that will be used during the search.
  ///
  pub fn set_number_of_threads(&self, max_threads: usize) {
    self.options.lock().unwrap().max_threads = max_threads;
  }

  /// Helper function that sets the "uci" bool value in the engine options
  ///
  /// ### Arguments
  ///
  /// * `uci`: The new value to apply to uci
  ///
  pub fn set_uci(&self, uci: bool) {
    self.options.lock().unwrap().uci = uci;
  }

  /// Helper function that sets the "uci" bool value in the engine options
  ///
  /// ### Arguments
  ///
  /// * `uci`: The new value to apply to uci
  ///
  pub fn get_uci(&self) -> bool {
    return self.options.lock().unwrap().uci;
  }

  /// Helper function that sets the "use_nnue" bool value in the engine options
  ///
  /// ### Arguments
  ///
  /// * `use_nnue`: The new value to apply to use_nnue
  ///
  pub fn set_use_nnue(&self, nnue: bool) {
    self.options.lock().unwrap().use_nnue = nnue;
  }

  /// Helper function that sets the "debug" bool value in the engine options
  /// If debug is set to true, it will print "info string <debug_strings>"
  /// once in a while.
  ///
  /// ### Arguments
  ///
  /// * `enabled`: Set this value to enable or disable debug information
  ///
  pub fn set_debug(&self, enabled: bool) {
    self.options.lock().unwrap().debug = enabled;
  }

  /// Helper function that sets the "play style" value in the engine options
  ///
  /// ### Arguments
  ///
  /// * `play_style`: Value to set for the play style.
  ///
  pub fn set_play_style(&self, play_style: PlayStyle) {
    self.options.lock().unwrap().style = play_style;
  }

  /// Helper function that sets the "multi_pv" value in the engine options
  ///
  /// ### Arguments
  ///
  /// * `multi_pv`: Value to set for the multi_pv.
  ///
  pub fn set_multi_pv(&self, value: usize) {
    self.options.lock().unwrap().multi_pv = value;
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
  ///
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
  ///
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

  /// Searchs and evaluate a position with the configured engine options
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
  ///
  fn search(
    &self,
    game_state: &GameState,
    depth: usize,
    max_depth: usize,
    mut alpha: f32,
    mut beta: f32,
  ) -> SearchResult {
    if self.stop_requested() || self.has_been_searching_too_long() {
      return Vec::new();
    }

    if depth > max_depth {
      //println!("Reached maximum depth {max_depth}. Stopping search");
      return Vec::new();
    }

    // Check that we know the moves
    Engine::find_move_list(&self.cache, &game_state.board);
    let mut moves = self.cache.get_move_list(&game_state.board).unwrap();
    let mut best_continuation: Vec<Move>;
    let mut result: HashMap<Move, (Vec<Move>, f32)> = HashMap::new();

    for m in &moves {
      // println!("Move: {} - alpha-beta: {}/{}", m.to_string(), alpha, beta);
      // Here we have low trust in eval accuracy, so it has to be more than
      // good gap between alpha and beta before we prune.
      if (alpha - 0.5) > beta {
        // TODO: Test this a bit better, I think we are pruning stuff that should not get prunned.
        //println!("Skipping {} as it is pruned {}/{}",game_state.to_fen(), alpha, beta);
        break;
      }

      // If we are looking at a capture, make sure that we analyze possible
      // recaptures by increasing temporarily the maximum depth
      let mut max_line_depth = max_depth;
      if depth == max_depth && m.is_piece_capture() {
        if depth < self.analysis.get_depth() + 3 {
          max_line_depth = max_depth + 1;
          self.analysis.update_selective_depth(max_line_depth);
          //println!("Continuing to depth {max_line_depth}");
        }
      }

      let mut new_game_state = game_state.clone();
      new_game_state.apply_move(&m);

      // Check if we just repeated the position too much or did not make progress.
      let draw = can_declare_draw(&new_game_state);
      if draw != GameStatus::Ongoing {
        self.cache.set_eval(
          &new_game_state.board,
          EvaluationCache {
            game_status: draw,
            eval: 0.0,
            depth: 1,
          },
        );
        Engine::update_alpha_beta(game_state.board.side_to_play, 0.0, &mut alpha, &mut beta);
        result.insert(*m, (Vec::new(), 0.0));
        continue;
      }

      // Check if we already looked at this position.
      let mut eval_cache = self.cache.get_eval(&new_game_state.board).unwrap_or_default();
      if eval_cache.depth > 0 && depth >= max_line_depth {
        // Nothing to do, we already looked at this position.
        // FIXME: If the position appears in another variation but leads to a draw, e.g. 3 fold repetitions, we won't detect it and skip it.
        // Get the alpha/beta result propagated upwards.
        Engine::update_alpha_beta(
          game_state.board.side_to_play,
          eval_cache.eval,
          &mut alpha,
          &mut beta,
        );
        result.insert(*m, (Vec::new(), eval_cache.eval));
        continue;
      }

      if eval_cache.depth == 0 {
        eval_cache.game_status = is_game_over(&self.cache, &new_game_state.board);
      };

      // No need to look at other moves in this variation if we found a checkmate for the side to play:
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

        // FIXME: We should make this a bit smarter, go one level up to save the good move
        // Also if there is an eval swing, not just checkmate.
        self.cache.add_killer_move(&m);
        Engine::update_alpha_beta(game_state.board.side_to_play, eval, &mut alpha, &mut beta);
        result.insert(*m, (Vec::new(), eval));
        eval_cache.eval = eval;
        self.cache.set_eval(&new_game_state.board, eval_cache);
        // Don't look at other moves when we found a checkmate
        break;
      }

      // Search more if the game is not over.
      if eval_cache.game_status == GameStatus::Ongoing {
        if depth < max_line_depth {
          /*
          // Recurse until we get to the bottom, spin 1 thread per move at the first level.
          let self_clone = self.clone();
          handles.push(std::thread::spawn(move || {self_clone.search(&new_game_state, depth + 1, max_line_depth, start_time) }));
          */
          let sub_result = self.search(&new_game_state, depth + 1, max_line_depth, alpha, beta);
          if sub_result.is_empty() {
            return sub_result;
          }
          (best_continuation, eval) =
            Engine::get_best_result_from_subresult(new_game_state.board.side_to_play, &sub_result);
          eval = decrement_eval_if_mating_sequence(eval);

          result.insert(*m, (best_continuation, eval));
          Engine::update_alpha_beta(game_state.board.side_to_play, eval, &mut alpha, &mut beta);
        } else if eval_cache.game_status == GameStatus::Ongoing && depth >= max_line_depth {
          // Evaluate our position
          eval = evaluate_board(&new_game_state);

          // FIXME:  NNUE eval is still too slow, we should implement incremental updates
          if depth > 10 && self.options.lock().unwrap().use_nnue == true {
            let nnue_eval = self.nnue.lock().unwrap().eval(&new_game_state);
            //println!("board: {} - Eval: {} - NNUE Eval: {} - final eval {}",new_game_state.to_fen(), eval,nnue_eval,eval * 0.5 + nnue_eval * 0.5);
            eval = eval * 0.5 + nnue_eval * 0.5;
          }

          result.insert(*m, (Vec::new(), eval));
          Engine::update_alpha_beta(game_state.board.side_to_play, eval, &mut alpha, &mut beta);
        }
      } else {
        // Here the game is no longer ongoing (draw, etc.)
        Engine::update_alpha_beta(game_state.board.side_to_play, eval, &mut alpha, &mut beta);
        result.insert(*m, (Vec::new(), eval));
      }

      // Save the intermediate result in the transposition table
      eval_cache.eval = eval;
      eval_cache.depth = max_line_depth - depth + 1;
      self.cache.set_eval(&new_game_state.board, eval_cache);
    } // for m in &moves

    // Sort the children moves according to their evaluation:
    moves
      .sort_by(|a, b| Engine::compare_by_result_eval(game_state.board.side_to_play, a, b, &result));
    self.cache.set_move_list(&game_state.board, &moves);

    // Save in the cache table:
    let mut pv = game_state.clone();
    pv.apply_move(&moves[0]);
    let mut best_move_eval = self.cache.get_eval(&pv.board).unwrap_or_default();

    best_move_eval.depth += 1;
    best_move_eval.eval = decrement_eval_if_mating_sequence(best_move_eval.eval);
    self.cache.set_eval(&game_state.board, best_move_eval);

    // Build a result:
    let mut search_result: SearchResult = Vec::new();
    for m in moves {
      if !result.contains_key(&m) {
        continue;
      }

      let mut line = vec![m];
      for m in &result[&m].0 {
        line.push(*m);
      }
      let line_eval = result[&m].1;
      search_result.push((line, line_eval));
    }

    return search_result;
  }

  /// Finds the best move among the list of moves/evals stored in a subresult,
  /// using the side to play
  ///
  /// ### Arguments
  ///
  /// * `color`:      Side to play
  /// * `subresult`:  Sub result
  ///
  /// ### Returns
  ///
  /// Pair with the best move / best eval.
  /// (Move::default, f32::NAN) if there are no data in the subresult
  ///
  fn get_best_result_from_subresult(color: Color, subresult: &SearchResult) -> (Vec<Move>, f32) {
    if subresult.is_empty() {
      return (Vec::new(), f32::NAN);
    }

    let mut best_moves = Vec::new();
    let mut best_result = match color {
      Color::White => f32::MIN,
      Color::Black => f32::MAX,
    };

    for (m, eval) in subresult {
      if eval.is_nan() {
        continue;
      }

      match color {
        Color::White => {
          if *eval > best_result {
            best_result = *eval;
            best_moves = m.clone();
          }
        },
        Color::Black => {
          if *eval < best_result {
            best_result = *eval;
            best_moves = m.clone();
          }
        },
      }
    }

    (best_moves, best_result)
  }

  /// Checks the best move in the result and check if it is a winning sequence
  /// for the color indicated in argument
  ///
  #[inline]
  fn best_move_is_mating_sequence(color: Color, result: &SearchResult) -> bool {
    if result.is_empty() {
      return false;
    }

    let eval = result[0].1;
    match color {
      Color::White => eval > 150.0,
      Color::Black => eval < -150.0,
    }
  }

  /// Sorts the list of moves based on the data in the result
  ///
  /// ### Arguments
  ///
  /// * `Color`:     Side to play, for sorting
  /// * `a`          Move A
  /// * `b`          Move B
  /// * `result`     HashMap with f32 evaluations
  ///
  /// ### Return value
  ///
  /// Ordering telling if B is Greater, Equal or Less than A
  ///
  fn compare_by_result_eval(
    color: Color,
    a: &Move,
    b: &Move,
    result: &HashMap<Move, (Vec<Move>, f32)>,
  ) -> Ordering {
    let (greater, less, default) = match color {
      Color::White => (Ordering::Less, Ordering::Greater, (Vec::new(), f32::MIN)),
      Color::Black => (Ordering::Greater, Ordering::Less, (Vec::new(), f32::MAX)),
    };

    let a_eval = result.get(a).unwrap_or(&default).1;
    let b_eval = result.get(b).unwrap_or(&default).1;

    if a_eval > b_eval {
      return greater;
    } else if a_eval < b_eval {
      return less;
    }

    Ordering::Equal
  }
}

impl std::fmt::Display for Engine {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.to_string().as_str())
  }
}

impl Default for Engine {
  fn default() -> Self {
    let nnue_path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), NNUE_FILE);
    Engine {
      position: GameState::default(),
      analysis: Analysis::default(),
      cache: EngineCache::new(),
      state: EngineState {
        active: Arc::new(Mutex::new(false)),
        stop_requested: Arc::new(Mutex::new(false)),
        start_time: Arc::new(Mutex::new(Instant::now())),
      },
      options: Arc::new(Mutex::new(Options {
        uci: true,
        ponder: false,
        max_depth: 20,
        max_time: 0,
        max_threads: 16,
        use_nnue: false,
        debug: false,
        style: PlayStyle::Normal,
        multi_pv: 3,
      })),
      nnue: Arc::new(Mutex::new(
        NNUE::load(nnue_path.as_str()).unwrap_or_default(),
      )),
    }
  }
}
