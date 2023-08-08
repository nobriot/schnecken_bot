pub mod cache;
pub mod core;
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

pub struct Engine {
  position: GameState,
  /// State of the analysis for the game state.
  analysis: Analysis,
  /// Position cache, used to speed up processing
  cache: EngineCache,
  /// Engine options
  options: Options,
  /// Whether the engine is active of not
  search: Arc<Mutex<bool>>,
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
      search: Arc::new(Mutex::new(false)),
    }
  }

  /// Checks if the engine is resolving a position
  ///
  /// ### Return value
  ///
  /// * True if searching a position, False otherwise
  ///
  pub fn is_searching(&self) -> bool {
    return *self.search.lock().unwrap();
  }

  /// Helper function that sets the "search" bool value in the engine
  ///
  /// ### Arguments
  ///
  /// * `search`: The new value to apply to search
  ///
  fn set_search(&self, search: bool) {
    let mut s = self.search.lock().unwrap();
    *s = search;
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
    self.position = GameState::from_fen(START_POSITION_FEN);
    self.analysis.reset();
    self.cache.clear();
    self.set_search(false);
  }

  /// Sets a new position
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

  /// Apply a move to the current position
  pub fn apply_move(&mut self, chess_move: &str) {
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
    // Save the max depth
    if self.is_searching() {
      // we are already searching.
      debug!("We are already evaluating the position.");
      return;
    }
    self.set_search(true);

    // Start searching
    for i in 1..self.options.max_depth {
      self.evaluate_positions(self.position.clone(), 1, i, Instant::now());
    }

    // We are done
    self.set_search(false);
  }

  /// Starts analyzing the current position
  ///
  /// Analysis will continue until stopped.
  pub fn stop(&self) {
    self.set_search(false);
  }

  /// Returns the best move
  pub fn get_best_move(&self) -> Move {
    let score = self.cache.get_eval(&self.position.board.hash);
    if score.is_none() {
      println!("Warn: No score available.");
      return Move::default();
    }

    let score = score.unwrap();
    println!(
      "Score for position {}: {}",
      &self.position.board.hash, score
    );
    let variations = self.cache.get_variations(&self.position.board.hash);

    for (m, board_hash) in variations {
      if let Some(eval) = self.cache.get_eval(&board_hash) {
        println!("Score for move {}: {}", m, eval);
        if eval == score {
          return m;
        }
      }
    }

    println!("Warn: Cannot find matching score.");
    return Move::default();
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

    for (m, board_hash) in variations {
      if let Some(eval) = self.cache.get_eval(&board_hash) {
        println!("Score for move {}: {}", m, eval);
      } else {
        println!("Score for move {}: None", m);
      }
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

    let mut game_state_a = game_state.clone();
    game_state_a.apply_move(a, false);
    let mut game_state_b = game_state.clone();
    game_state_b.apply_move(b, false);

    match (game_state_a.checks, game_state_b.checks) {
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

    // Single checks
    match (game_state_a.checks, game_state_b.checks) {
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
  /// * `game_state`: Game state to start from in the evaluation tree
  /// * `depth`:      Current depth at which we are in the search
  /// * `max_depth`:  Depth at which to stop
  /// * `start_time`: Time at which we started resolving the chess position
  ///
  fn evaluate_positions(
    &self,
    game_state: GameState,
    depth: usize,
    max_depth: usize,
    start_time: Instant,
  ) {
    if (depth > max_depth) || !self.is_searching() || self.has_been_searching_too_long(start_time) {
      return;
    }

    // Check that we know the moves, so it is safe to unwrap after that:
    Engine::find_move_list(&self.cache, &game_state);

    for m in self.cache.get_move_list(&game_state.board.hash).unwrap() {
      let mut new_game_state = game_state.clone();
      new_game_state.apply_move(&m, false);

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

      // Check if we did not evaluate already:
      if let None = self.cache.get_eval(&new_game_state.board.hash) {
        // Position evaluation:
        let (score, _) = evaluate_position(&new_game_state);
        // FIXME: This will save some 3 fold repetitions and stuff like that in the tree.
        self.cache.set_eval(&new_game_state.board.hash, score);
      }

      // We just found a checkmate, stop looking at other lines
      let score = self.cache.get_eval(&new_game_state.board.hash).unwrap();
      if score.abs() == 200.0 {
        // This is an interesting move, we save it as killer move
        self.cache.add_killer_move(&m);
        break;
      }

      // Recurse until we get to the bottom.
      self.evaluate_positions(new_game_state, depth + 1, max_depth, start_time);
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
    // Save the best child eval to the node above:
    self.cache.set_eval(&game_state.board.hash, best_eval);
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
      search: Arc::new(Mutex::new(false)),
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

    let expected_move = Move::from_string("c1b2");
    assert_eq!(expected_move, engine.get_best_move());
  }

  #[test]
  fn engine_select_find_best_defensive_move() {
    // Only good defense is : h8f8
    let mut engine = Engine::new();
    engine.set_position("r1bqk2r/ppppbp1p/2n5/3Bp1pQ/4P3/3P4/PPPN1PPP/R3K1NR b KQq - 0 7");
    engine.set_maximum_depth(3);
    engine.go();

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

    let expected_move = Move::from_string("a7a8Q");
    assert_eq!(expected_move, engine.get_best_move());
  }
}
