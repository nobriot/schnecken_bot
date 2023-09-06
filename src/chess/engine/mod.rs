pub mod cache;
pub mod development;
pub mod eval;
pub mod square_affinity;
pub mod theory;

use log::*;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// Same module (engine)
use cache::EngineCache;
use eval::position::evaluate_position;

// Chess model
use super::model::game_state::GameState;
use super::model::game_state::GameStatus;
use super::model::game_state::START_POSITION_FEN;
use super::model::moves::Move;
use super::model::piece::Color;
use super::model::piece::*;
use super::model::tables::zobrist::BoardHash;

#[derive(Clone, Debug, Default)]
pub struct Options {
  /// Continue thinking even if we found a winning sequence.
  pub ponder: bool,
  /// Maximum depth to go at
  pub max_depth: usize,
  /// time in miliseconds to spend on a calculation
  /// Set to 0 for no limit
  pub max_time: usize,
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
  pub scores: HashMap<Move, Eval>,
  pub depth: usize,
}

#[derive(Clone, Debug, Default)]
struct EngineState {
  /// Indicates if the engine is active at resolving positions
  pub active: Arc<Mutex<bool>>,
  /// Indicates that we want the engine to stop resolving positions
  pub stop_requested: Arc<Mutex<bool>>,
}

impl Analysis {
  /// Resets the analysis
  pub fn reset(&mut self) {
    self.scores.clear();
  }

  /// Initializes the list of scores for a move_list
  pub fn init_scores(&mut self, move_list: &Vec<Move>) {
    self.scores.clear();
    for m in move_list {
      self.scores.insert(*m, Eval::default());
    }
  }
}

impl Default for Analysis {
  fn default() -> Self {
    Analysis {
      scores: HashMap::new(),
      depth: 0,
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
  options: Options,
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
      options: Options {
        ponder: false,
        max_depth: 0,
        max_time: 0,
      },
      state: EngineState {
        active: Arc::new(Mutex::new(false)),
        stop_requested: Arc::new(Mutex::new(false)),
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
    if self.options.max_time == 0 {
      return false;
    }
    (Instant::now() - start_time) > Duration::from_millis(self.options.max_time as u64)
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
    self.position = GameState::from_fen(fen);

    if let Some(move_list) = self.cache.get_move_list(&self.position.board.hash) {
      self.analysis.init_scores(&move_list);
    } else {
      let board = self.position.board.clone();
      let move_list = self.position.get_moves();
      self.cache.set_move_list(&board.hash, &move_list);
      self.analysis.init_scores(&move_list);
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
    self
      .position
      .apply_move(&Move::from_string(chess_move), false);
    self.analysis.reset();

    if let Some(move_list) = self.cache.get_move_list(&self.position.board.hash) {
      self.position.set_moves(&move_list);
      self.analysis.init_scores(&move_list);
    } else {
      let board = self.position.board.clone();
      let move_list = self.position.get_moves();
      self.cache.set_move_list(&board.hash, &move_list);
      self.analysis.init_scores(&move_list);
    }
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

    // Start searching
    let start_time = Instant::now();
    let mut i = 1;

    while !self.has_been_searching_too_long(start_time) && !self.stop_requested() {
      // Fist iteration solves all the positions, then after we look at captures only, then all, and repeat
      if i != 1 {
        self.evaluate_positions(&self.position.clone(), true, 1, i, start_time);
      }

      if self.stop_requested() {
        break;
      }

      self.evaluate_positions(&self.position.clone(), false, 1, i, start_time);
      //println!("Depth {i} completed");
      //self.print_evaluations();
      i += 1;

      if self.options.max_depth > 0 && i > self.options.max_depth {
        break;
      }
    }

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
    let score = self.cache.get_eval(&self.position.board.hash);
    if score.is_none() {
      println!("Warn: No score available.");
      return self.position.move_list.as_ref().unwrap()[0];
    }

    let score = score.unwrap();
    let variations = self.cache.get_variations(&self.position.board.hash);

    for (m, board_hash) in variations {
      if let Some(eval) = self.cache.get_eval(&board_hash) {
        //println!("Score for move {}: {}", m, eval);
        if eval == score {
          return m;
        }
      }
    }

    println!("Warn: Cannot find matching score.");
    return self.position.move_list.as_ref().unwrap()[0];
  }

  /// Returns a string of the best move continuation (e.g. d1c3 c2c8 f2g3)
  /// based on the board, using the engine cache.
  ///
  /// ### Arguments
  ///
  /// * self:   Used to look up the engine cache
  /// * board:  Position to start from
  ///
  /// ### Return value
  ///
  /// String containing the list of best moves found by the engine
  ///
  pub fn get_line_string(&self, board_hash: BoardHash, side_to_play: Color) -> String {
    let line_string = String::new();

    let variations = self.cache.get_variations(&board_hash);
    if variations.is_empty() {
      return line_string;
    }

    let mut best_eval: f32 = match side_to_play {
      Color::White => f32::MIN,
      Color::Black => f32::MAX,
    };
    let mut best_move = Move::default();
    let mut best_new_board: BoardHash = 0;

    for (m, board_hash) in variations {
      if let Some(eval) = self.cache.get_eval(&board_hash) {
        match side_to_play {
          Color::White => {
            if eval > best_eval {
              best_eval = eval;
              best_move = m;
              best_new_board = board_hash;
            }
          },
          Color::Black => {
            if eval < best_eval {
              best_eval = eval;
              best_move = m;
              best_new_board = board_hash;
            }
          },
        }
      }
    }

    return best_move.to_string()
      + " "
      + self
        .get_line_string(best_new_board, Color::opposite(side_to_play))
        .as_str();
  }

  /// Prints the evaluation result in the console
  ///
  pub fn print_evaluations(&self) {
    let score = self.cache.get_eval(&self.position.board.hash);
    if score.is_none() {
      println!(
        "Warn: No score available for position. {}",
        &self.position.board.hash
      );
      return;
    }

    let score = score.unwrap();
    println!(
      "Score for position {}: {}",
      &self.position.board.hash, score
    );
    let variations = self.cache.get_variations(&self.position.board.hash);

    let mut i = 0;
    for (m, board_hash) in variations {
      if let Some(eval) = self.cache.get_eval(&board_hash) {
        println!(
          "Line {:<2}: Eval {:<10} - {} {}",
          i,
          eval,
          m,
          self.get_line_string(
            board_hash,
            Color::opposite(self.position.board.side_to_play)
          )
        );
      } else {
        println!("Line {:<2}: Eval <None> - {}", i, m);
      }
      i += 1;
    }
  }

  //----------------------------------------------------------------------------
  // Engine Options

  /// Configures if the engine should ponder when it finds a winning sequence
  /// (i.e. continue calculating alterative lines)
  ///
  pub fn set_ponder(&mut self, ponder: bool) {
    self.options.ponder = ponder;
  }

  /// Configure the depth at which to search with the engine.
  ///
  /// ### Arguments
  ///
  /// * `max_depth`: Maximum amount of time, in milliseconds, to spend resolving a position
  ///
  pub fn set_maximum_depth(&mut self, max_depth: usize) {
    self.options.max_depth = max_depth;
  }

  /// Sets a timelimit in ms on how long we search
  ///
  /// ### Arguments
  ///
  /// * `max_time`: Maximum amount of time, in milliseconds, to spend resolving a position
  ///
  pub fn set_search_time_limit(&mut self, max_time: usize) {
    self.options.max_time = max_time;
  }

  //----------------------------------------------------------------------------
  // Position calculations

  /// Looks at the cache and makes sure we have a move list known for
  /// the position / Game State
  ///
  /// ### Arguments
  ///
  /// * cache:      EngineCache where the move list is stored at the end.
  /// * game_state: GameState to evalute for a move-list
  ///
  fn find_move_list(cache: &EngineCache, game_state: &GameState) {
    // Check that we know the moves:
    if !cache.has_move_list(&game_state.board.hash) {
      let mut new_game_state = game_state.clone();
      let mut move_list = new_game_state.get_moves().clone();

      // Sort the moves based on interesting-ness
      move_list.sort_by(|a, b| Engine::compare_moves(cache, &new_game_state, a, b));
      cache.set_move_list(&game_state.board.hash, &move_list);
    }
  }

  /// Looks at the board and checks what's the game status (ongoing, win, draw)
  /// and saves it in the cache
  ///
  /// ### Arguments
  ///
  /// * cache:      EngineCache where the move list is stored at the end.
  /// * game_state: GameState to evaluate for a Draw/Win/Ongoing
  ///
  fn find_game_status(cache: &EngineCache, game_state: &GameState) {
    if cache.get_game_status(&game_state.board.hash).is_some() {
      return;
    }

    let game_status = game_state.get_game_status();
    cache.set_game_status(&game_state.board.hash, game_status);
  }

  /// Looks at the cache and makes sure we have known game phrase for a position
  /// Else it gets computed and saved to the cache
  ///
  /// The game_state object also gets modified
  ///
  /// ### Arguments
  ///
  /// * cache:      EngineCache where the game phase is stored at the end.
  /// * game_state: GameState to determine the game phase and save it to the cache.
  ///
  fn find_game_phase(cache: &EngineCache, game_state: &mut GameState) {
    if let Some(game_phase) = cache.get_game_phase(&game_state.board.hash) {
      game_state.game_phase = Some(game_phase);
    } else {
      game_state.update_game_phase();
      if let Some(phase) = game_state.game_phase {
        cache.set_game_phase(&game_state.board.hash, phase);
      }
    }
  }

  /// Looks at the cache and makes sure we know the number of checks for a board position
  ///
  /// ### Arguments
  ///
  /// * cache:      EngineCache where the number of checks is stored.
  /// * game_state: GameState indicating a board configuration
  ///
  fn save_checks(cache: &EngineCache, game_state: &GameState) {
    if cache.get_checks(&game_state.board.hash).is_none() {
      cache.set_checks(&game_state.board.hash, game_state.checks);
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

    let a_checks = if cache.get_variation(&game_state.board.hash, a).is_none() {
      let mut game_state_a = game_state.clone();
      game_state_a.apply_move(a, false);
      cache.set_checks(&game_state_a.board.hash, game_state_a.checks);
      game_state_a.checks
    } else {
      let variation = cache.get_variation(&game_state.board.hash, a).unwrap();
      cache.get_checks(&variation).unwrap_or(0)
    };

    let b_checks = if cache.get_variation(&game_state.board.hash, b).is_none() {
      let mut game_state_b = game_state.clone();
      game_state_b.apply_move(b, false);
      cache.set_checks(&game_state_b.board.hash, game_state_b.checks);
      game_state_b.checks
    } else {
      let variation = cache.get_variation(&game_state.board.hash, b).unwrap();
      cache.get_checks(&variation).unwrap_or(0)
    };

    match (a_checks, b_checks) {
      (2, 2) => {},
      (2, _) => return Ordering::Less,
      (_, 2) => return Ordering::Greater,
      (_, _) => {},
    }

    match (a.promotion, b.promotion) {
      (BLACK_QUEEN | WHITE_QUEEN, BLACK_QUEEN | WHITE_QUEEN) => {},
      (BLACK_QUEEN | WHITE_QUEEN, _) => return Ordering::Less,
      (_, BLACK_QUEEN | WHITE_QUEEN) => return Ordering::Greater,
      (_, _) => {},
    }

    let a_captured_value =
      Piece::material_value_from_u8(game_state.board.squares[a.dest as usize]).abs();
    let b_captured_value =
      Piece::material_value_from_u8(game_state.board.squares[b.dest as usize]).abs();

    if a_captured_value > b_captured_value {
      return Ordering::Less;
    } else if a_captured_value < b_captured_value {
      return Ordering::Greater;
    }

    if a_captured_value > b_captured_value {
      return Ordering::Less;
    } else if a_captured_value < b_captured_value {
      return Ordering::Greater;
    }

    // Single checks
    match (a_checks, b_checks) {
      (1, _) => return Ordering::Less,
      (_, 1) => return Ordering::Greater,
      (_, _) => {},
    }

    // We like castling in general:
    let a_castle = game_state.board.is_castle(a);
    let b_castle = game_state.board.is_castle(b);
    match (a_castle, b_castle) {
      (true, false) => return Ordering::Less,
      (false, true) => return Ordering::Greater,
      (_, _) => {},
    }

    return Ordering::Equal;
  }

  //----------------------------------------------------------------------------
  // Engine Evaluation

  /// Evaluate a position with the configured engine options
  ///
  /// ### Arguments
  ///
  /// * `self`: Engine to use to store all the calculations
  /// * `game_state`:    Game state to start from in the evaluation tree
  /// * `captures_only`: Set this bit to evaluate captures only
  /// * `depth`:      Current depth at which we are in the search
  /// * `max_depth`:  Depth at which to stop
  /// * `start_time`: Time at which we started resolving the chess position
  ///
  fn evaluate_positions(
    &self,
    game_state: &GameState,
    captures_only: bool,
    depth: usize,
    max_depth: usize,
    start_time: Instant,
  ) {
    if self.stop_requested() || self.has_been_searching_too_long(start_time) {
      return;
    }

    if !captures_only && (depth > max_depth) {
      //info!("Reached maximum depth. Stopping search");
      return;
    }

    if !captures_only && self.cache.is_pruned(&game_state.board.hash) {
      println!("Skipping {} as it is pruned", game_state.board.hash);
      return;
    }

    // Check that we know the moves, so it is safe to unwrap after that:
    Engine::find_move_list(&self.cache, &game_state);

    for m in self.cache.get_move_list(&game_state.board.hash).unwrap() {
      // Skip non-capture if we are resolving captures only
      if captures_only && !game_state.board.is_move_a_capture(&m) {
        continue;
      }

      let mut new_game_state = game_state.clone();
      new_game_state.apply_move(&m, false);

      if self.cache.get_eval(&new_game_state.board.hash).is_some() {
        // Move forward, we already processed this
        self.evaluate_positions(
          &new_game_state,
          captures_only,
          depth + 1,
          max_depth,
          start_time,
        );
        continue;
      }

      // We just computed the number of checks for a position, save it.
      Engine::save_checks(&self.cache, &new_game_state);

      // Add the variation to the parent position
      self
        .cache
        .add_variation(&game_state.board.hash, &m, &new_game_state.board.hash);

      // Game phase
      Engine::find_game_phase(&self.cache, &mut new_game_state);

      // Check the new moves from the new position, so that we can detect game-over situations
      Engine::find_move_list(&self.cache, &new_game_state);
      new_game_state.set_moves(
        &self
          .cache
          .get_move_list(&new_game_state.board.hash)
          .unwrap(),
      );

      Engine::find_game_status(&self.cache, &new_game_state);

      // Check if we did not evaluate already:
      if let None = self.cache.get_eval(&new_game_state.board.hash) {
        // Position evaluation:
        let (score, _) = evaluate_position(&new_game_state);
        // FIXME: This will save some 3 fold repetitions and stuff like that in the tree.
        self.cache.set_eval(&new_game_state.board.hash, score);
      }

      // Get the alpha/beta result ppropagated upwards.
      match game_state.board.side_to_play {
        Color::White => self.cache.set_alpha(
          &game_state.board.hash,
          self
            .cache
            .get_eval(&new_game_state.board.hash)
            .unwrap_or(f32::MIN),
        ),
        Color::Black => self.cache.set_beta(
          &game_state.board.hash,
          self
            .cache
            .get_eval(&new_game_state.board.hash)
            .unwrap_or(f32::MAX),
        ),
      }

      // We just found a checkmate/draw
      let new_game_status = self
        .cache
        .get_game_status(&new_game_state.board.hash)
        .unwrap_or_default();
      match new_game_status {
        GameStatus::Ongoing => {},
        _ => self.cache.add_killer_move(&m),
      }

      // No need to look at other moves in this variation if we found a checkmate for the side to play:
      if new_game_status == GameStatus::WhiteWon || new_game_status == GameStatus::BlackWon {
        break;
      }

      // Recurse until we get to the bottom.
      self.evaluate_positions(
        &new_game_state,
        captures_only,
        depth + 1,
        max_depth,
        start_time,
      );
    }

    // Back propagate from children nodes
    let variations = self.cache.get_variations(&game_state.board.hash);
    let mut best_eval: f32 = match game_state.board.side_to_play {
      Color::White => f32::MIN,
      Color::Black => f32::MAX,
    };
    for (_, board_hash) in variations {
      if let Some(eval) = self.cache.get_eval(&board_hash) {
        match game_state.board.side_to_play {
          Color::White => {
            if eval > best_eval {
              best_eval = eval;
            }
          },
          Color::Black => {
            if eval < best_eval {
              best_eval = eval;
            }
          },
        }
      }
    }

    // Before back-propagating, check if we are in the capture only mode,
    // and if so and the evaluation swinged, try to refute a bad capture
    if captures_only {
      if let Some(eval) = self.cache.get_eval(&game_state.board.hash) {
        if (eval - best_eval).abs() > 2.0 {
          // Recurse until we get to the bottom.
          println!(
            "Trying to refute capture on position {}",
            game_state.to_fen()
          );
          self.evaluate_positions(&game_state, false, 1, 2, start_time);
          // Recursing will take care of the back-propagation, so return here
          return;
        }
      }
    }

    // Save the best child eval to the node above:
    if best_eval != f32::MIN && best_eval != f32::MAX {
      self.cache.set_eval(&game_state.board.hash, best_eval);
    }

    // Sort the children moves according to their evaluation:
    // FIXME: This does not work
    // &self.cache.sort_moves_by_eval(&game_state.board.hash);
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
      },
      options: Options {
        ponder: false,
        max_depth: 1,
        max_time: 0,
      },
    }
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
    engine.set_maximum_depth(4);
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
    engine.set_maximum_depth(3);
    engine.go();

    engine.print_evaluations();
    let expected_move = Move::from_string("c1b2");
    assert_eq!(expected_move, engine.get_best_move());
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
    let expected_move = Move::from_string("h8f8");
    assert_eq!(expected_move, engine.get_best_move());
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
    engine.set_position("8/P7/4kN2/4P3/1K3P2/4P3/8/8 w - - 7 76");
    engine.set_maximum_depth(20);
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
    // 1000 kNPS would be nice. Right now we are at a very low number LOL
    assert!(
      engine.cache.len() > 1_000_000,
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

    let expected_move = Move::from_string("c1b2");
    assert_eq!(expected_move, engine.get_best_move());
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
    use crate::chess::model::board::Board;
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
}
