use crate::engine::config::play_style::PlayStyle;

#[derive(Clone, Debug, Default)]
pub struct Options {
  /// Whether this engine is used with the UCI interface and it
  /// should print information when searching
  pub uci: bool,
  /// Continue thinking even if we found a winning sequence.
  pub ponder: bool,
  /// Maximum depth to go at
  pub max_depth: usize,
  /// time in milliseconds to spend on a calculation
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
