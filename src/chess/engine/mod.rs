pub mod cache;
pub mod core;
pub mod development;
pub mod eval;
pub mod square_affinity;
pub mod theory;

use std::collections::HashMap;

// Same module (engine)
use cache::EngineCache;
use eval::position::evaluate_position;

// Chess model
use super::model::game_state::GameState;
use super::model::game_state::START_POSITION_FEN;
use super::model::moves::Move;
use super::model::piece::Color;

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
  search: bool,
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
        max_depth: 1,
        max_time: 0,
      },
      search: false,
    }
  }

  /// Resets the engine to a default state.
  /// Same as Engine::Default() or Engine::new()
  pub fn reset(&mut self) {
    self.position = GameState::from_fen(START_POSITION_FEN);
    self.analysis.reset();
    self.cache.clear();
    self.search = false;
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
  /// Analysis will continue until stopped.
  pub fn go(&mut self) {
    // Save the max depth
    if self.search {
      // we are already searching. Just change the max depth
      return;
    }
    self.search = true;

    Engine::evaluate_positions(
      &self.cache,
      self.position.clone(),
      1,
      self.options.max_depth,
    );

    // We are done
    self.search = false;
  }

  /// Starts analyzing the current position
  ///
  /// Analysis will continue until stopped.
  pub fn stop(&mut self) {}

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
  pub fn set_maximum_depth(&mut self, max_depth: usize) {
    self.options.max_depth = max_depth;
  }

  /// Sets a timelimit on how long we search
  pub fn set_search_time_limit(&mut self, max_time: usize) {
    self.options.max_time = max_time;
  }

  //----------------------------------------------------------------------------
  // Engine Evaluation

  /// Evaluate a position with the configured engine options
  ///
  fn evaluate_positions(
    cache: &EngineCache,
    game_state: GameState,
    depth: usize,
    max_depth: usize,
  ) {
    if depth > max_depth {
      return;
    }

    // Check that we know the moves:
    if !cache.has_move_list(&game_state.board.hash) {
      let mut new_game_state = game_state.clone();
      let move_list = new_game_state.get_moves();
      cache.set_move_list(&game_state.board.hash, &move_list);
    }

    for m in cache.get_move_list(&game_state.board.hash).unwrap() {
      let mut new_game_state = game_state.clone();
      new_game_state.apply_move(&m, false);

      // Add the variation to the parent position
      cache.add_variation(&game_state.board.hash, &m, &new_game_state.board.hash);

      // Game phase
      if let Some(game_phase) = cache.get_game_phase(&new_game_state.board.hash) {
        new_game_state.game_phase = Some(game_phase);
      } else {
        new_game_state.update_game_phase();
        if let Some(phase) = new_game_state.game_phase {
          cache.set_game_phase(&new_game_state.board.hash, phase);
        }
      }

      // List of moves (again, we have to check if we have legal moves from this position)
      // Check that we know the moves:
      if !cache.has_move_list(&new_game_state.board.hash) {
        let mut gs = new_game_state.clone();
        let move_list = gs.get_moves();
        cache.set_move_list(&new_game_state.board.hash, &move_list);
        new_game_state.set_moves(move_list);
      }

      // Check if we did not evaluate already:
      if let None = cache.get_eval(&new_game_state.board.hash) {
        // Position evaluation:
        let (score, _) = evaluate_position(&new_game_state);
        // FIXME: This will save some 3 fold repetitions and stuff like that in the tree.
        cache.set_eval(&new_game_state.board.hash, score);
      }

      // We just found a checkmate, stop looking at other lines
      let score = cache.get_eval(&new_game_state.board.hash).unwrap();
      if score.abs() == 200.0 {
        cache.add_killer_move(&m);
        // TODO: Add to killer moves
        break;
      }

      // Recurse until we get to the bottom.
      Engine::evaluate_positions(cache, new_game_state, depth + 1, max_depth);
    }

    // Back propagate from children nodes
    let variations = cache.get_variations(&game_state.board.hash);
    let mut best_eval: f32 = match game_state.board.side_to_play {
      Color::White => f32::MIN,
      Color::Black => f32::MAX,
    };
    for (_, board_hash) in variations {
      if let Some(eval) = cache.get_eval(&board_hash) {
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
    cache.set_eval(&game_state.board.hash, best_eval);
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
      search: false,
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
}
