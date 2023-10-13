pub mod cache;
pub mod development;
pub mod engine_test;
pub mod eval;
pub mod square_affinity;
pub mod theory;

use log::*;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

// Same module (engine)
use self::eval::position::{evaluate_board, is_game_over};
use crate::model::board::Board;
use crate::model::moves::Promotion;
use cache::EngineCache;

// Chess model
use super::model::game_state::GameState;
use super::model::game_state::GameStatus;
use super::model::game_state::START_POSITION_FEN;
use super::model::moves::Move;
use super::model::piece::Color;
use super::model::piece::*;

#[derive(Clone, Debug, Default)]
pub struct Options {
  /// Continue thinking even if we found a winning sequence.
  pub ponder: bool,
  /// Maximum depth to go at
  pub max_depth: usize,
  /// time in miliseconds to spend on a calculation
  /// Set to 0 for no limit
  pub max_time: usize,
  /// Number of threads to use for the search.
  pub max_threads: usize,
}

#[derive(Clone, Debug, Default)]
pub struct Eval {
  pub score: Option<f32>,
  pub selective_depth: usize,
  pub game_over: bool,
}

#[derive(Clone, Debug)]
struct Analysis {
  /// Scores assigned to each move starting from the start position
  pub scores: Arc<Mutex<HashMap<Move, f32>>>,
  /// Represent how deep the analysis is/was
  pub depth: Arc<Mutex<usize>>,
}

#[derive(Clone, Debug, Default)]
struct EngineState {
  /// Indicates if the engine is active at resolving positions
  pub active: Arc<Mutex<bool>>,
  /// Indicates that we want the engine to stop resolving positions
  pub stop_requested: Arc<Mutex<bool>>,
  /// List of active thread handles in the engine
  pub threads: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl Analysis {
  /// Resets the analysis
  pub fn reset(&self) {
    self.scores.lock().unwrap().clear();
  }

  /// Sets the depth we have reached during the analysis
  ///
  /// ### Arguments
  ///
  /// * `self`:   Instance of the Chess Engine
  /// * `depth`:  New depth to set.
  ///
  ///
  pub fn update_result(&self, result: HashMap<Move, f32>) {
    let mut score = self.scores.lock().unwrap();

    for (m, eval) in result {
      if !score.get(&m).unwrap_or(&f32::NAN).is_nan() && eval.is_nan() {
        continue;
      }
      score.insert(m, eval);
    }
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
}

impl Default for Analysis {
  fn default() -> Self {
    Analysis {
      scores: Arc::new(Mutex::new(HashMap::new())),
      depth: Arc::new(Mutex::new(0)),
    }
  }
}

#[derive(Clone, Debug)]
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
}

impl Engine {
  //----------------------------------------------------------------------------
  // Public functions

  /// Gets a new engine
  ///
  pub fn new() -> Self {
    Engine {
      position: GameState::from_fen(START_POSITION_FEN),
      analysis: Analysis::default(),
      cache: EngineCache::new(),
      options: Arc::new(Mutex::new(Options {
        ponder: false,
        max_depth: 20,
        max_time: 100,
        max_threads: 30,
      })),
      state: EngineState {
        active: Arc::new(Mutex::new(false)),
        stop_requested: Arc::new(Mutex::new(false)),
        threads: Arc::new(Mutex::new(Vec::new())),
      },
    }
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
  /// ### Arguments
  ///
  /// * `start_time`: Time when we started searching
  ///
  /// ### Return value
  ///
  /// True if the (current_time - start_time) is larger than options.max_time
  /// and max_time is set to a non-zero value.
  ///
  fn has_been_searching_too_long(&self, start_time: Instant) -> bool {
    let max_time = self.options.lock().unwrap().max_time;
    if max_time == 0 {
      return false;
    }
    (Instant::now() - start_time) > Duration::from_millis(max_time as u64)
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
    self.position.apply_move(&Move::from_string(chess_move));
    self.cache.clear_killer_moves();
    self.cache.clear_variations();
    self.analysis.reset();
    self.analysis.set_depth(1);
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

    // Start searching... now
    let start_time = Instant::now();

    // If we have only one legal move, we should just give it a score and play it instantaneously.
    Engine::find_move_list(&self.cache, &self.position.board);
    if self.cache.get_move_list(&self.position.board).len() == 1 {
      debug!("Single or no move available. Just evaluating quickly");
      let mut game_state = self.position.clone();
      game_state.apply_move(&self.cache.get_move_list(&self.position.board)[0]);
      let _ = evaluate_board(&self.cache, &game_state);
      self.set_stop_requested(false);
      self.set_engine_active(false);
      return;
    }

    while !self.has_been_searching_too_long(start_time) && !self.stop_requested() {
      self.analysis.increment_depth();
      println!("Starting depth {}", self.analysis.get_depth());

      let result = self.search(
        &self.position.clone(),
        1,
        self.analysis.get_depth(),
        f32::MIN,
        f32::MAX,
        start_time,
      );

      self.analysis.update_result(result);

      let max_depth = self.options.lock().unwrap().max_depth;
      if max_depth > 0 && self.analysis.get_depth() > max_depth {
        break;
      }
    }

    // Sort one last time the list of moves from the result: (it may have incomplete sorting if we aborted in the middle of a "depth")
    let mut top_level_result: HashMap<Move, f32> = HashMap::new();
    let mut move_list = self.cache.get_move_list(&self.position.board);
    let scores = self.analysis.scores.lock().unwrap();
    for m in &move_list {
      top_level_result.insert(*m, *scores.get(m).unwrap_or(&f32::NAN));
    }
    move_list.sort_by(|a, b| {
      Engine::compare_by_result_eval(self.position.board.side_to_play, a, b, &top_level_result)
    });
    self.cache.set_move_list(&self.position.board, &move_list);

    // We are done
    self.set_stop_requested(false);
    self.set_engine_active(false);
  }

  /// Starts analyzing the current position
  ///
  /// Analysis will continue until stopped.
  pub fn stop(&self) {
    self.set_stop_requested(true);
  }

  /// Returns the best move
  pub fn get_best_move(&self) -> Move {
    return self.cache.get_move_list(&self.position.board)[0];
  }

  /// Returns the full analysis
  pub fn get_analysis(&self) -> Vec<(Move, f32)> {
    let mut analysis: Vec<(Move, f32)> = Vec::new();
    let move_list = self.cache.get_move_list(&self.position.board);
    let scores = self.analysis.scores.lock().unwrap();

    for m in move_list {
      analysis.push((m, *scores.get(&m).unwrap_or(&f32::NAN)));
    }

    analysis
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
    if ttl == 0 {
      return String::new();
    }
    let line_string = String::new();

    if !self.cache.has_move_list(&game_state.board) {
      return line_string;
    }

    if self.cache.has_status(game_state) && self.cache.get_status(game_state) != GameStatus::Ongoing
    {
      return line_string;
    }

    let move_list = self.cache.get_move_list(&game_state.board);
    if move_list.is_empty() {
      return line_string;
    }
    let best_move = move_list[0];
    let mut best_new_state = game_state.clone();
    best_new_state.apply_move(&best_move);
    if !self.cache.has_eval(&best_new_state.board) {
      return line_string;
    }

    return best_move.to_string()
      + " "
      + self
        .get_line_string(&best_new_state, Color::opposite(side_to_play), ttl - 1)
        .as_str();
  }

  /// Prints the evaluation result in the console
  ///
  pub fn print_evaluations(&self) {
    let scores = self.analysis.scores.lock().unwrap();

    let move_list = self.cache.get_move_list(&self.position.board);
    let mut i = 0;

    println!(
      "Score for position {}: {}",
      self.position.to_fen(),
      scores.get(&move_list[0]).unwrap_or(&f32::NAN)
    );
    for m in move_list {
      let mut new_state = self.position.clone();
      new_state.apply_move(&m);
      let eval = scores.get(&m).unwrap_or(&f32::NAN);
      println!(
        "Line {:<2}: Eval {:<10.2} - {} {}",
        i,
        eval,
        m,
        self.get_line_string(
          &new_state,
          Color::opposite(self.position.board.side_to_play),
          self.analysis.get_depth() + 1,
        )
      );
      i += 1;
    }
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

  /// Sorts the list of moves based on their interesting-ness
  ///
  /// It will sort using the following ordering:
  /// 1. Double checks
  /// 2. Queen promotions
  /// 3. Captures (ordered by captured material)
  /// 4. Checks
  /// 5. Castling
  /// 6. All the rest
  ///
  /// ### Arguments
  ///
  /// * `cache`:     EngineCache to use to look-up assets like Killer Moves
  /// * `game_state` Reference to a GameState on which we want to sort the moves
  /// * `a`          Move A
  /// * `b`          Move B
  ///
  /// ### Return value
  ///
  /// Ordering telling if B is Greater, Equal or Less than A
  ///
  pub fn compare_moves(
    cache: &EngineCache,
    game_state: &GameState,
    a: &Move,
    b: &Move,
  ) -> Ordering {
    // Check if any move is a killer move
    match (cache.is_killer_move(a), cache.is_killer_move(b)) {
      (true, false) => return Ordering::Less,
      (false, true) => return Ordering::Greater,
      (_, _) => {},
    }

    // Check we know the a continuation:
    let a_game_state = if !cache.has_variation(&game_state, a) {
      let mut new_game_state = game_state.clone();
      new_game_state.apply_move(a);
      cache.add_variation(&game_state, a, &new_game_state);
      new_game_state
    } else {
      cache.get_variation(&game_state, a)
    };

    // Check we know the b continuation:
    let b_game_state = if !cache.has_variation(&game_state, b) {
      let mut new_game_state = game_state.clone();
      new_game_state.apply_move(b);
      cache.add_variation(&game_state, b, &new_game_state);
      new_game_state
    } else {
      cache.get_variation(&game_state, b)
    };

    match (a_game_state.board.checks(), b_game_state.board.checks()) {
      (2, 2) => {},
      (2, _) => return Ordering::Less,
      (_, 2) => return Ordering::Greater,
      (_, _) => {},
    }

    match (a.promotion(), b.promotion()) {
      (
        Promotion::BlackQueen | Promotion::WhiteQueen,
        Promotion::BlackQueen | Promotion::WhiteQueen,
      ) => {},
      (Promotion::BlackQueen | Promotion::WhiteQueen, _) => return Ordering::Less,
      (_, Promotion::BlackQueen | Promotion::WhiteQueen) => return Ordering::Greater,
      (_, _) => {},
    }

    let a_captured_value =
      Piece::material_value_from_u8(game_state.board.pieces.get(a.dest() as u8));
    let b_captured_value =
      Piece::material_value_from_u8(game_state.board.pieces.get(b.dest() as u8));

    if a_captured_value > b_captured_value {
      return Ordering::Less;
    } else if a_captured_value < b_captured_value {
      return Ordering::Greater;
    } else if a_captured_value > 0.0 {
      let a_captor = Piece::material_value_from_u8(game_state.board.pieces.get(a.src() as u8));
      let b_captor = Piece::material_value_from_u8(game_state.board.pieces.get(b.src() as u8));

      if a_captor < b_captor {
        return Ordering::Less;
      } else if a_captor > b_captor {
        return Ordering::Greater;
      }
    }

    // We like castling in general:
    match (a.is_castle(), b.is_castle()) {
      (true, false) => return Ordering::Less,
      (false, true) => return Ordering::Greater,
      (_, _) => {},
    }

    // Single checks
    match (a_game_state.board.checks(), b_game_state.board.checks()) {
      (1, _) => return Ordering::Less,
      (_, 1) => return Ordering::Greater,
      (_, _) => {},
    }

    return Ordering::Equal;
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
    start_time: Instant,
  ) -> HashMap<Move, f32> {
    if self.stop_requested() || self.has_been_searching_too_long(start_time) {
      return HashMap::new();
    }

    if depth > max_depth {
      println!("Reached maximum depth {max_depth}. Stopping search");
      return HashMap::new();
    }

    // Check that we know the moves
    Engine::find_move_list(&self.cache, &game_state.board);
    let mut moves = self.cache.get_move_list(&game_state.board);
    let mut result: HashMap<Move, f32> = HashMap::new();

    for m in &moves {
      // Here we have low trust in eval accuracy, so it has to be more than
      // good gap between alpha and beta before we prune.
      //if (alpha - 3.0) > beta {
      // TODO: Find why this does not seem to improve anything.
      //println!("Skipping {} as it is pruned {}/{}",game_state.to_fen(), alpha, beta);
      //break;
      //}

      // If we are looking at a capture, make sure that we analyze possible
      // recaptures by increasing temporarily the maximum depth
      //let mut max_line_depth = max_depth;
      if depth == max_depth && m.is_capture() {
        //max_line_depth = max_depth + 1;
        //println!("Continuing to depth {max_line_depth}");
      }

      let mut new_game_state = game_state.clone();
      new_game_state.apply_move(&m);

      let game_status = if !self.cache.has_status(&new_game_state) {
        is_game_over(&self.cache, &new_game_state)
      } else {
        self.cache.get_status(&new_game_state)
      };

      // No need to look at other moves in this variation if we found a checkmate for the side to play:
      let mut eval = f32::NAN;
      match game_status {
        GameStatus::WhiteWon => {
          self.cache.add_killer_move(&m);
          eval = 200.0;
        },
        GameStatus::BlackWon => {
          self.cache.add_killer_move(&m);
          eval = -200.0;
        },
        GameStatus::Stalemate => {
          eval = 0.0;
        },
        GameStatus::Draw => {
          eval = 0.0;
        },
        GameStatus::Ongoing => {},
      }

      if game_status == GameStatus::Ongoing && depth < max_depth {
        // Recurse until we get to the bottom, spin 1 thread per move at the first level.

        /*
        let self_clone = self.clone();
        handles.push(std::thread::spawn(move || {self_clone.search(&new_game_state, depth + 1, max_line_depth, start_time) }));
        */
        let sub_result = self.search(
          &new_game_state,
          depth + 1,
          max_depth,
          alpha,
          beta,
          start_time,
        );
        //println!("SUb result: {:#?}", sub_result);
        eval = match new_game_state.board.side_to_play {
          Color::White => Engine::get_best_eval_for_white(&sub_result),
          Color::Black => Engine::get_best_eval_for_black(&sub_result),
        };
      } else if game_status == GameStatus::Ongoing && depth >= max_depth {
        // We are at the end of the depth, let's statically evaluate the board and return this eval
        eval = if !self.cache.has_eval(&new_game_state.board) {
          // Position evaluation: (will be saved in the cache automatically)
          evaluate_board(&self.cache, &new_game_state)
        } else {
          self.cache.get_eval(&new_game_state.board)
        };
      }

      // Now that we have an eval for sure, save it
      if !eval.is_nan() {
        result.insert(*m, eval);
        // Get the alpha/beta result propagated upwards.
        match game_state.board.side_to_play {
          Color::White => {
            if alpha < eval {
              alpha = eval;
            }
          },
          Color::Black => {
            if beta > eval {
              beta = eval;
            }
          },
        }
      }

      // Don't look at other moves when we found a checkmate:
      if game_status == GameStatus::WhiteWon || game_status == GameStatus::BlackWon {
        break;
      }
    } // for m in &moves

    // Sort the children moves according to their evaluation:
    /*
     */
    moves
      .sort_by(|a, b| Engine::compare_by_result_eval(game_state.board.side_to_play, a, b, &result));
    self.cache.set_move_list(&game_state.board, &moves);
    return result;
  }

  /// TODO: Write description
  ///
  /// ### Arguments
  ///
  /// * `result`:
  ///
  /// ### Returns
  ///
  /// f32::NAN if there are no data in the result
  /// f32 with the best evaluation for white
  ///
  fn get_best_eval_for_white(result: &HashMap<Move, f32>) -> f32 {
    if result.is_empty() {
      return f32::NAN;
    }

    let mut best_result = f32::MIN;
    for (_, eval) in result {
      if !eval.is_nan() && *eval > best_result {
        best_result = *eval;
      }
    }
    best_result
  }

  /// TODO: Write description
  ///
  /// ### Arguments
  ///
  /// * `result`:
  ///
  /// ### Returns
  ///
  /// f32::NAN if there are no data in the result
  /// f32 with the best evaluation for white
  ///
  fn get_best_eval_for_black(result: &HashMap<Move, f32>) -> f32 {
    if result.is_empty() {
      return f32::NAN;
    }

    let mut best_result = f32::MAX;
    for (_, eval) in result {
      if !eval.is_nan() && *eval < best_result {
        best_result = *eval;
      }
    }
    best_result
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
    result: &HashMap<Move, f32>,
  ) -> Ordering {
    let (greater, less, default) = match color {
      Color::White => (Ordering::Less, Ordering::Greater, f32::MIN),
      Color::Black => (Ordering::Greater, Ordering::Less, f32::MAX),
    };

    let a_eval = result.get(a).unwrap_or(&default);
    let b_eval = result.get(b).unwrap_or(&default);

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
    Engine {
      position: GameState::default(),
      analysis: Analysis::default(),
      cache: EngineCache::new(),
      state: EngineState {
        active: Arc::new(Mutex::new(false)),
        stop_requested: Arc::new(Mutex::new(false)),
        threads: Arc::new(Mutex::new(Vec::new())),
      },
      options: Arc::new(Mutex::new(Options {
        ponder: false,
        max_depth: 1,
        max_time: 0,
        max_threads: 30,
      })),
    }
  }
}

impl Drop for Engine {
  fn drop(&mut self) {
    debug!("Dropping Engine!")
  }
}

//------------------------------------------------------------------------------
// Tests
#[cfg(test)]
mod tests {

  use super::*;
  #[test]
  fn engine_select_best_move_checkmate_in_one() {
    // This is a forced checkmate in 1:
    let mut engine = Engine::new();
    engine.set_position("1n4nr/5ppp/1N6/1P2p3/1P6/4kP2/1B1NP1PP/R3KB1R w KQ - 1 36");
    engine.set_maximum_depth(2);
    engine.go();

    // println!("engine analysis: {:#?}", engine.analysis.scores);
    engine.print_evaluations();
    let expected_move = Move::from_string("b6d5");
    assert_eq!(expected_move, engine.get_best_move());
  }

  #[test]
  fn engine_select_best_move_checkmate_in_one_for_black() {
    // This is a forced checkmate in 1 for black:
    let mut engine = Engine::new();
    engine.set_position("8/8/2p1pkp1/p3p3/P1P1P1P1/6q1/7q/3K4 b - - 2 55");
    engine.set_maximum_depth(2);
    engine.go();

    //println!("engine analysis: {:#?}", engine.analysis.scores);
    engine.print_evaluations();
    let expected_move = Move::from_string("g3g1");
    assert_eq!(expected_move, engine.get_best_move());
  }

  #[test]
  fn engine_select_best_move_checkmate_in_two() {
    // This is a forced checkmate in 2: c1b2 d4e3 b6d5
    let mut engine = Engine::new();
    engine.set_position("1n4nr/5ppp/1N6/1P2p3/1P1k4/5P2/1p1NP1PP/R1B1KB1R w KQ - 0 35");
    engine.set_search_time_limit(5000);
    engine.set_maximum_depth(3);
    engine.go();

    engine.print_evaluations();
    let expected_move = "c1b2";
    assert_eq!(expected_move, engine.get_best_move().to_string());
    let analysis = engine.get_analysis();
    assert_eq!(analysis[0].1, 200.0);
  }

  #[test]
  fn engine_select_find_best_defensive_move() {
    // Only good defense is : h8f8
    let mut engine = Engine::new();
    engine.set_position("r1bqk2r/ppppbp1p/2n5/3Bp1pQ/4P3/3P4/PPPN1PPP/R3K1NR b KQq - 0 7");
    engine.set_search_time_limit(5000);
    engine.set_maximum_depth(8);
    engine.go();

    engine.print_evaluations();
    let expected_move = "h8f8";
    assert_eq!(expected_move, engine.get_best_move().to_string());
  }

  #[test]
  fn engine_save_the_last_knight() {
    // Game: https://lichess.org/iavzLpKc
    let mut engine = Engine::new();
    engine.set_position("4r1k1/1p6/7p/p4p2/Pb1p1P2/1PN3P1/2P1P1K1/r7 w - - 0 34");
    engine.set_maximum_depth(20);
    engine.set_search_time_limit(7863);
    engine.go();

    let good_moves = [Move::from_string("c3b5"), Move::from_string("c3d5")];
    let engine_move = engine.get_best_move();
    engine.print_evaluations();
    if !good_moves.contains(&engine_move) {
      assert!(
        false,
        "Expected either c3b5 or c3d5, but instead we have {}",
        engine_move.to_string()
      );
    }
  }

  #[test]
  fn engine_promote_this_pawn() {
    let mut engine = Engine::new();
    engine.set_position("8/P7/4kN2/4P3/1K3P2/4P3/8/8 w - - 7 76");
    engine.set_maximum_depth(20);
    engine.set_search_time_limit(855);
    engine.go();

    engine.print_evaluations();
    let expected_move = Move::from_string("a7a8Q");
    assert_eq!(expected_move, engine.get_best_move());
  }

  #[test]
  fn engine_go_and_stop() {
    let mut engine = Engine::new();
    engine.set_position("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    engine.set_maximum_depth(0);
    engine.set_search_time_limit(0);
    engine.set_ponder(true);

    let engine_clone = engine.clone();
    let handle = std::thread::spawn(move || engine_clone.go());

    std::thread::sleep(std::time::Duration::from_millis(10));

    assert_eq!(true, engine.is_active());
    std::thread::sleep(std::time::Duration::from_millis(1000));
    assert_eq!(true, engine.is_active());
    engine.stop();
    assert_eq!(true, engine.is_active());

    std::thread::sleep(std::time::Duration::from_millis(50));
    assert_eq!(false, engine.is_active());

    assert_eq!(true, handle.is_finished());
  }

  #[test]
  fn engine_bench_positions_per_second() {
    let mut engine = Engine::new();
    engine.set_position("4r1k1/1p6/7p/p4p2/Pb1p1P2/1PN3P1/2P1P1K1/r7 w - - 0 34");
    engine.set_search_time_limit(1000);
    engine.go();

    println!("Engine cache length: {}", engine.cache.len());
    // 100 kNPS would be nice. Right now we are at a very low number LOL
    assert!(
      engine.cache.len() > 100_000,
      "Number of NPS for engine analysis: {}",
      engine.cache.len()
    );
  }

  #[test]
  fn save_the_bishop() {
    /*
     [2023-06-26T13:51:05Z DEBUG schnecken_bot::lichess::api] Lichess get answer: {"nowPlaying":[{"color":"white","fen":"2kr1b1r/ppp2ppp/2nqp3/3n1BP1/8/3P1N1P/PPP1PP2/R1BQK2R w KQ - 0 12","fullId":"AHbg0nGCsiMN","gameId":"AHbg0nGC","hasMoved":true,"isMyTurn":true,"lastMove":"e7e6","opponent":{"id":"sargon-1ply","rating":1233,"username":"BOT sargon-1ply"},"perf":"blitz","rated":true,"secondsLeft":160,"source":"friend","speed":"blitz","status":{"id":20,"name":"started"},"variant":{"key":"standard","name":"Standard"}}]}
     [2023-06-26T13:51:05Z INFO  schnecken_bot] Trying to find a move for game id AHbg0nGC
     [2023-06-26T13:51:05Z INFO  schnecken_bot::chess::engine::core] Using 1777 ms to find a move
     Line 0 Eval: -1.8000004 - f5e6 d6e6 e2e4
     Line 1 Eval: -4.4000006 - f3g1 e6f5
     Line 2 Eval: -16.820002 - c2c3 f8e7
     Line 3 Eval: -17.800003 - a2a3 f8e7
     Line 4 Eval: -17.860003 - f3e5 f8e7
    */
    let mut engine = Engine::new();
    engine.set_position("2kr1b1r/ppp2ppp/2nqp3/3n1BP1/8/3P1N1P/PPP1PP2/R1BQK2R w KQ - 0 12");
    engine.set_search_time_limit(2000);
    engine.go();
    engine.print_evaluations();
    let expected_move = Move::from_string("f5e4");
    assert_eq!(
      expected_move,
      engine.get_best_move(),
      "Come on, the only good move is f5e4"
    );
  }

  #[test]
  fn test_dont_hang_pieces_1() {
    /* Got this in a game, hanging a knight, after thinking for 16_000 ms :
     Line 0 Eval: 0.79999995 - f8h6 d5e4 d7d5 e4d3
     Line 1 Eval: -0.30000085 - e4f6 d5d3
     Line 2 Eval: 2.3999996 - b7b5 d5e4 d7d5 e4d3 e7e5 b1c3
     Line 3 Eval: 2.5499997 - b7b6 d5e4 d7d5 e4d3 e7e5 b1c3
     Line 4 Eval: 3.2999995 - c6b8 d5e4 d7d5 e4d3 b8c6 b1c3
    */
    let mut engine = Engine::new();
    engine.set_position("r1bqkb1r/1ppppp1p/p1n5/3Q4/4n3/5N2/PPPP1PPP/RNB1KB1R b KQkq - 0 7");
    engine.set_search_time_limit(3000);
    engine.go();
    engine.print_evaluations();

    let best_move = engine.get_best_move().to_string();

    if "e4f6" != best_move && "e4d6" != best_move {
      assert!(
        false,
        "Should have been either e4f6 or e4d6, instead we have: {best_move}"
      );
    }
  }

  #[test]
  fn test_dont_hang_pieces_2() {
    /*
      https://lichess.org/zcQesp7F#69
      Here we blundered a rook playing e2f2
      2k5/pp5p/2p3p1/8/1PpP4/P5KP/4r2P/8 b - - 1 35
      Using 1355 ms to find a move
      Line 0 Eval: -9.860003 - e2f2 g3f2 c8b8 f2g1 c4c3 g1g2 c3c2 g2g1 c2c1Q
      Line 1 Eval: -9.250003 - e2e5 d4e5 c8b8 g3g2 c4c3 e5e6 c3c2 e6e7 c2c1Q
      Line 2 Eval: -7.820003 - e2a2 g3f3 a2a3 f3g2
      Line 3 Eval: -8.105003 - e2h2 g3g4 h2e2
      Line 4 Eval: -7.9150023 - e2d2 b4b5 d2d4
      [2023-05-12T06:06:18Z INFO  schnecken_bot] Playing move e2f2 for game id zcQesp7F
    */

    let mut engine = Engine::new();
    engine.set_position("2k5/pp5p/2p3p1/8/1PpP4/P5KP/4r2P/8 b - - 1 35");
    engine.set_search_time_limit(1000);
    engine.go();
    engine.print_evaluations();
    let not_expected_move = Move::from_string("e2f2");
    assert!(
      not_expected_move != engine.get_best_move(),
      "e2f2 should not be played!!"
    );
  }

  // From game : https://lichess.org/SKF7qgMu -
  // Did not capture the knight, it was very obvious to capture.
  // Spent 2450 ms to come up with this crap: e5f5
  #[test]
  fn save_the_queen() {
    let mut engine = Engine::new();
    engine.set_position("rnbqk2r/pp3ppp/2pbpn2/3pQ3/B3P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 6");
    engine.set_search_time_limit(2450);
    engine.go();
    engine.print_evaluations();

    let game_state1 =
      GameState::from_fen("rnbqk2r/pp3ppp/2pQpn2/3p4/B3P3/8/PPPP1PPP/RNB1K1NR b KQkq - 0 6");
    println!(
      "Static intermediate:  {}",
      engine.cache.get_eval(&game_state1.board)
    );

    let game_state =
      GameState::from_fen("rnb1k2r/pp3ppp/2pqpn2/3p4/B3P3/8/PPPP1PPP/RNB1K1NR w KQkq - 0 7");
    println!(
      "Static from cache:  {}",
      engine.cache.get_eval(&game_state.board)
    );

    let static_eval = evaluate_board(&engine.cache, &game_state);
    println!("Static eval: {static_eval}");
    assert_eq!(true, engine.cache.has_eval(&game_state.board));

    let best_move = engine.get_best_move().to_string();
    if "e5g5" != best_move && "e5d4" != best_move && "e5c3" != best_move {
      assert!(
        false,
        "Should have been either e5g5, e5d4 or e5c3, instead we have: {best_move}"
      );
    }
  }

  // From game : https://lichess.org/47V8eE5x -
  // Did not capture the knight, it was very obvious to capture.
  // Spent 2900 ms to come up with this crap: d7d5
  #[test]
  fn capture_the_damn_knight_1() {
    let mut engine = Engine::new();
    engine.set_position("rnb2r1k/pppp2pp/5N2/8/1bB5/8/PPPPQPPP/RNB1K2R b KQ - 0 9");
    engine.set_search_time_limit(2900);
    engine.go();
    engine.print_evaluations();

    let best_move = engine.get_best_move().to_string();
    if "f8f6" != best_move && "g7f6" != best_move {
      assert!(
        false,
        "Should have been either f8f6 or g7f6, instead we have: {best_move}"
      );
    }
  }

  #[test]
  fn evaluate_checkmate_with_castle() {
    let mut engine = Engine::new();
    engine.set_position("8/8/8/8/2nN4/1q6/ppP1NPPP/1k2K2R w K - 0 1");
    engine.set_search_time_limit(10);
    engine.go();
    engine.print_evaluations();

    assert_eq!("e1g1", engine.get_best_move().to_string());
  }

  // Game https://lichess.org/Xjgkf4pp seemed really off. Testing some of the positions here
  #[test]
  fn test_select_pawn_capture() {
    let mut engine = Engine::new();
    engine.set_position("r2q1rk1/1pp1ppbp/p2p1np1/P7/6bP/R1N1Pn2/1PPP1PP1/2BQKB1R w K - 0 11");
    engine.set_search_time_limit(2000);
    engine.go();
    engine.print_evaluations();

    assert_eq!("g2f3", engine.get_best_move().to_string());
  }

  #[test]
  fn test_select_best_move_checkmate_in_two() {
    // This is a forced checkmate in 2: c1b2 d4e3 b6d5
    let mut engine = Engine::new();
    engine.set_position("1n4nr/5ppp/1N6/1P2p3/1P1k4/5P2/1p1NP1PP/R1B1KB1R w KQ - 0 35");
    engine.set_search_time_limit(5000);
    engine.go();
    engine.print_evaluations();

    let expected_move = "c1b2";
    assert_eq!(expected_move, engine.get_best_move().to_string());
  }

  #[test]
  fn test_select_best_move_checkmate_in_one() {
    // This is a forced checkmate in 1:
    let mut engine = Engine::new();
    engine.set_position("1n4nr/5ppp/1N6/1P2p3/1P6/4kP2/1B1NP1PP/R3KB1R w KQ - 1 36");
    engine.set_search_time_limit(5000);
    engine.go();
    engine.print_evaluations();
    let expected_move = Move::from_string("b6d5");
    assert_eq!(expected_move, engine.get_best_move());
  }

  #[test]
  fn test_avoid_threefold_repetitions() {
    use crate::model::board::Board;
    /* Looks like we had a permutation bug that lead us into some 3-fold repetitions
     [2023-07-04T12:36:47Z INFO  schnecken_bot::chess::engine::core] Using 1211 ms to find a move
       Line 0 Eval: 10.71348 - d1e2 / Permutation
       Line 1 Eval: 6.581044 - h2h3 / Permutation
       Line 2 Eval: 6.461045 - g3g2 / Permutation
       Line 3 Eval: 6.431045 - a1b1 / Permutation
       Line 4 Eval: 6.391044 - g3g1 / Permutation
    */

    let mut engine = Engine::new();
    engine.set_position("r7/1p4p1/5p1p/b3n1k1/p3P1P1/PbN3R1/1P1K3P/R1BB4 w - - 10 45");
    engine.set_search_time_limit(1200);
    engine
      .position
      .last_positions
      .push_back(Board::from_fen("r7/1p4p1/5p1p/b3n1k1/p3P1P1/PbN3R1/1P1K3P/R1BB4").hash);
    engine
      .position
      .last_positions
      .push_back(Board::from_fen("r7/1p4p1/5p1p/b3n1k1/p1b1P1P1/P1N3R1/1P1K3P/R1BB4").hash);
    engine
      .position
      .last_positions
      .push_back(Board::from_fen("r7/1p4p1/5p1p/b3n1k1/p1b1P1P1/P1N3R1/1P1KB2P/R1B5").hash);
    engine
      .position
      .last_positions
      .push_back(Board::from_fen("r7/1p4p1/5p1p/b3n1k1/p3P1P1/PbN3R1/1P1KB2P/R1B5").hash);
    engine
      .position
      .last_positions
      .push_back(Board::from_fen("r7/1p4p1/5p1p/b3n1k1/p3P1P1/PbN3R1/1P1K3P/R1BB4").hash);
    engine
      .position
      .last_positions
      .push_back(Board::from_fen("r7/1p4p1/5p1p/b3n1k1/p1b1P1P1/P1N3R1/1P1K3P/R1BB4").hash);
    engine
      .position
      .last_positions
      .push_back(Board::from_fen("r7/1p4p1/5p1p/b3n1k1/p1b1P1P1/P1N3R1/1P1KB2P/R1B5").hash);
    engine
      .position
      .last_positions
      .push_back(Board::from_fen("r7/1p4p1/2n2p1p/b5k1/p1b1P1P1/P1N3R1/1P1KB2P/R1B5").hash);

    engine.go();
    engine.print_evaluations();
    assert!(engine.get_best_move() != Move::from_string("d1e2"));
  }

  #[test]
  fn test_only_one_legal_move() {
    let mut engine = Engine::new();
    engine.set_position("5k2/R6P/8/2PKB3/1P6/1P1P1N2/5PP1/R7 b - - 0 67");
    engine.set_search_time_limit(942);

    engine.go();
    engine.print_evaluations();
    let analysis = engine.get_analysis();
    assert!(!analysis.is_empty());
    assert!(engine.get_best_move() == Move::from_string("f8e8"));
  }

  #[test]
  fn capture_the_bishop() {
    let mut engine = Engine::new();
    engine.set_position("rnbqk1nr/pp3ppp/2p5/1Q1p4/1b1Pp3/2N2N2/PPP1PPPP/R1B1KB1R w KQkq - 0 6");
    engine.set_search_time_limit(1875);
    engine.go();
    engine.print_evaluations();
    let analysis = engine.get_analysis();
    assert!(!analysis.is_empty());
    assert!(engine.get_best_move().to_string() == "b5b4");
  }

  #[test]
  fn endgame_evaluation_search() {
    let mut engine = Engine::new();
    engine.set_position("1K6/2Q5/8/8/8/3k4/8/8 w - - 0 1");
    engine.set_search_time_limit(800);
    engine.go();
    engine.print_evaluations();
    let analysis = engine.get_analysis();

    // 26 moves.
    assert_eq!(analysis.len(), 26);
    let bad_moves = vec![
      "c7c4", "c7c3", "c7c2", "c7d8", "c7c8", "c7b7", "c7a7", "c7e7", "c7f7", "c7d7", "c7g7",
      "c7h7", "b8a8", "b8a7",
    ];
    assert!(!bad_moves.contains(&engine.get_best_move().to_string().as_str()));
  }

  #[test]
  #[allow(non_snake_case)]
  fn evaluate_real_game_0BYxLu3V_example_1() {
    // https://lichess.org/0BYxLu3V has plently of blunders.
    //
    let mut engine = Engine::new();
    engine.set_position("r1b1kbnr/pppp1p1p/4pqp1/8/3nP3/2NQ1N2/PPPP1PPP/R1B1KB1R b KQkq - 7 6");
    engine.set_search_time_limit(1897);
    engine.go();
    engine.print_evaluations();
    let analysis = engine.get_analysis();
    assert!(!analysis.is_empty());
    assert!(engine.get_best_move() != Move::from_string("f8d6"));
  }

  #[test]
  #[allow(non_snake_case)]
  fn evaluate_real_game_0BYxLu3V_example_2() {
    // https://lichess.org/0BYxLu3V has plently of blunders.
    //
    let mut engine = Engine::new();
    engine.set_position("r1b1k1nr/pppp1p1p/3bpqp1/8/3QP3/2N2N2/PPPP1PPP/R1B1KB1R b KQkq - 0 7");
    engine.set_search_time_limit(1870);
    engine.go();
    engine.print_evaluations();
    let analysis = engine.get_analysis();
    assert!(!analysis.is_empty());
    assert!(engine.get_best_move() != Move::from_string("d6e5"));
  }

  #[test]
  fn evaluate_real_game_no8g7oup_example() {
    // https://lichess.org/no8g7oup
    //
    let mut engine = Engine::new();
    engine.set_position("r4rk1/2p5/p2pq2p/1p4p1/3Qb1n1/2N5/PPn1K1PP/R1B2B1R b - - 1 22");
    engine.set_search_time_limit(423);
    engine.go();
    engine.print_evaluations();
    let analysis = engine.get_analysis();
    assert!(!analysis.is_empty());
    assert!(engine.get_best_move().to_string() == "c2d4");
  }

  #[test]
  #[allow(non_snake_case)]
  fn evaluate_real_game_ov5SZJLX_example() {
    // https://lichess.org/ov5SZJLX
    // Engine came up with this:
    // Depth 2 completed
    // Score for position rn2kbnr/ppp1pppp/3q4/3p4/P7/2N1P2N/1PPP1PPP/R1BbKB1R w KQkq - 0 5: 21.355005
    // Line 0 : Eval 21.355005  - f1b5 d6c6
    // Line 1 : Eval -6.16      - e1d1 d6h2
    // Line 2 : Eval -6.4399996 - c3d1 d6h2
    // Line 3 : Eval -8.295     - f1d3 d1c2
    // Line 4 : Eval -8.605     - d2d4 d1c

    let mut engine = Engine::new();
    engine.set_position("rn2kbnr/ppp1pppp/3q4/3p4/P7/2N1P2N/1PPP1PPP/R1BbKB1R w KQkq - 0 5");
    engine.set_search_time_limit(6426);
    engine.go();
    engine.print_evaluations();
    let analysis = engine.get_analysis();
    assert!(!analysis.is_empty());
    assert!(analysis[0].1 < -5.0);
  }

  #[ignore]
  #[test]
  fn test_sorting_moves_without_eval() {
    let engine_cache: EngineCache = EngineCache::new();

    let fen = "r1bqk2r/pp3ppp/n1pbpn2/3pQ3/B3P3/5N2/PPPP1PPP/RNB1K2R w KQkq - 6 7";
    let game_state = GameState::from_fen(fen);

    let mut engine = Engine::new();
    Engine::find_move_list(&engine.cache, &game_state.board);

    for m in engine.cache.get_move_list(&game_state.board) {
      println!("Move: {}", m.to_string());
    }

    assert_eq!(
      Move::from_string("e5d6"),
      engine.cache.get_move_list(&game_state.board)[0]
    );
    assert_eq!(
      Move::from_string("e5f6"),
      engine.cache.get_move_list(&game_state.board)[1]
    );
    assert_eq!(
      Move::from_string("e4d5"),
      engine.cache.get_move_list(&game_state.board)[2]
    );
    assert_eq!(
      Move::from_string("a4c6"),
      engine.cache.get_move_list(&game_state.board)[3]
    );
    assert_eq!(
      Move::from_string("e5e6"),
      engine.cache.get_move_list(&game_state.board)[4]
    );
  }
}
