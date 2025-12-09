use crate::engine::config::play_style::PlayStyle;

#[derive(Clone, Debug)]
pub struct EngineOptions {
  /// Whether this engine is used with the UCI interface and it
  /// should print information when searching
  pub uci:             bool,
  /// Continue thinking even if we found a winning sequence.
  pub ponder:          bool,
  /// Maximum depth to go at
  pub max_depth:       usize,
  /// time in milliseconds to spend on a calculation
  /// Set to 0 for no limit / automatic
  pub max_search_time: usize,
  /// Number of threads to use for the search.
  pub max_threads:     usize,
  /// Number of threads to use for the search.
  pub use_nnue:        bool,
  /// Debug mode : The engine will print additional info (info string <debug
  /// string>) if this is set to true
  pub debug:           bool,
  /// Set the play style of the engine.
  pub play_style:      PlayStyle,
  /// Number of best lines that the engine will return.
  pub multi_pv:        usize,
}

impl Default for EngineOptions {
  fn default() -> Self {
    EngineOptions { uci:             true,
                    ponder:          false,
                    max_depth:       20,
                    max_search_time: 0,
                    max_threads:     16,
                    use_nnue:        false,
                    debug:           false,
                    play_style:      PlayStyle::Normal,
                    multi_pv:        3, }
  }
}
