pub mod cache;
pub mod core;
pub mod development;
pub mod eval;
pub mod square_affinity;
pub mod theory;

// Same module (engine)
use cache::EngineCache;

// Chess model
use super::model::moves::Move;
use super::model::game_state::GameState;
use super::model::game_state::START_POSITION_FEN;

#[derive(Clone, Debug)]
pub struct Options {
  /// Continue thinking even if we found a winning sequence.
  pub ponder: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Eval {
  pub score: Option<f32>,
  pub depth: Option<usize>,
  pub selective_depth: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct ChessLine {
  pub position: GameState,
  /// Move from the previous position to get to that position
  pub chess_move: Move,
  /// Derived positions applying all possible moves from this position.
  pub variations: Vec<ChessLine>,
  /// Evaluation data
  pub eval: Eval,
  pub game_over: bool,
}

impl Default for ChessLine {
  fn default() -> Self {
    ChessLine {
      position: GameState::default(),
      chess_move: Move::default(),
      variations: Vec::new(),
      eval: Eval::default(),
      game_over: false,
    }
  }
}

#[derive(Clone, Debug)]
struct Analysis {
  /// Whether we are active at calculating or not
  active: bool,
  /// Highest depth reached while analysing the position
  tree: ChessLine,
}

impl Analysis {
  /// Resets the analysis
  pub fn reset(&mut self) {
    self.tree = ChessLine::default();
    self.active = false;
  }

  /// Checks if we are working on the analysis
  pub fn is_active(&self) -> bool {
    self.active
  }
}

impl Default for Analysis {
  fn default() -> Self {
    Analysis {
      tree: ChessLine::default(),
      active: false,
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
}

impl Engine {
  /// Resets the engine to a default state.
  /// Same as Engine::Default
  pub fn reset(&mut self) {
    self.position = GameState::from_fen(START_POSITION_FEN);
    self.analysis.reset();
    self.cache.clear();
  }

  /// Sets a new position
  pub fn set_position(&mut self, fen: &str) {
    self.reset();
    self.position = GameState::from_fen(fen);
  }

  /// Apply a move to the current position
  pub fn apply_move(&mut self, chess_move: &str) {
    let m = Move::from_string(chess_move);
    todo!("Apply the move int he analysis tree too");
  }

  /// Starts analyzing the current position
  ///
  /// Analysis will continue until stopped.
  pub fn go(&mut self) {}

  /// Starts analyzing the current position
  ///
  /// Analysis will continue until stopped.
  pub fn stop(&mut self) {}

  /// Returns the best move
  pub fn get_best_move() -> Move {
    todo!();
  }
  //----------------------------------------------------------------------------
  // Engine Options

  /// Returns the best move
  pub fn set_ponder(&mut self, ponder: bool) {
    self.options.ponder = ponder;
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
      options: Options { ponder: false },
    }
  }
}
