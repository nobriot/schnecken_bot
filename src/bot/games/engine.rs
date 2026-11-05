use chess::engine::Engine;
use chess::engine::config::play_style::PlayStyle;
use chess::model::game_state::START_POSITION_FEN;
use lichess;
use lichess::types::GameStart;
use log::*;

/// Against humans, we play a bit quicker than the engine would like to
pub const HUMAN_TIME_FACTOR: f32 = 0.75;
/// Against humans, we never think longer than this on a move (milliseconds)
pub const HUMAN_MAX_MOVE_TIME_MS: usize = 40_000;
/// We never think longer than this on our first move (milliseconds): Lichess
/// aborts the game if we take too long
pub const FIRST_MOVE_MAX_TIME_MS: usize = 3_000;

/// Checks if the engine level is adjusted for this game, and can be changed by
/// the opponent with `max-depth`: casual games against untitled humans.
#[inline(always)]
pub fn is_depth_adjustable(game: &GameStart) -> bool {
  !game.rated && !game.opponent_is_bot() && game.opponent.title.is_none()
}

/// Looks at the game data (which kind of opponent, time control, start
/// position) and configures the engine accordingly.
pub fn configure_engine(game: &GameStart, config: &crate::config::EngineConfig) -> Engine {
  // We are not using the uci interface internally
  let mut engine = Engine::new(false);

  // Cache table size from config
  engine.resize_cache_tables(config.cache_table_size);

  // Apply play style from config as default
  if let Some(ref style_str) = config.play_style {
    if let Ok(style) = style_str.parse::<PlayStyle>() {
      engine.options.play_style = style;
    } else {
      warn!("Unknown play style in config: {style_str}");
    }
  }

  // Configure the start position
  let start_fen = game.fen.as_deref().unwrap_or(START_POSITION_FEN);
  engine.set_position(start_fen);

  if !game.opponent_is_bot() {
    engine.options.time_factor = HUMAN_TIME_FACTOR;
  }

  // Adjust the level of difficulty based on the rating of the opponent, if they
  // are human
  if is_depth_adjustable(game) {
    if game.opponent.rating < 1300 {
      engine.options.max_depth = 3;
      engine.options.play_style = PlayStyle::Provocative;
    } else if game.opponent.rating < 1500 {
      engine.options.max_depth = 4;
      engine.options.play_style = PlayStyle::Provocative;
    } else if game.opponent.rating < 1700 {
      engine.options.max_depth = 5;
      engine.options.play_style = PlayStyle::Provocative;
    } else if game.opponent.rating < 1800 {
      engine.options.max_depth = 6;
      engine.options.play_style = PlayStyle::Provocative;
    } else if game.opponent.rating < 1900 {
      engine.options.max_depth = 7;
    } else if game.opponent.rating < 2000 {
      engine.options.max_depth = 8;
    }
  }

  info!("Engine configuration for game {}: {:?}", game.game_id, engine.options);

  engine
}

/// Maximum time we can spend on a move, 0 for no maximum
///
/// ### Arguments
///
/// * `opponent_is_bot`:  Whether we are playing against a bot
/// * `flagging_time_ms`: Time per move when we try to flag the opponent, if we do
/// * `first_move`:       Whether this is our first move of the game
pub fn max_move_time_ms(opponent_is_bot: bool,
                        flagging_time_ms: Option<usize>,
                        first_move: bool)
                        -> usize {
  let human_max_time = (!opponent_is_bot).then_some(HUMAN_MAX_MOVE_TIME_MS);
  let first_move_max_time = first_move.then_some(FIRST_MOVE_MAX_TIME_MS);
  [human_max_time, flagging_time_ms, first_move_max_time].into_iter()
                                                         .flatten()
                                                         .min()
                                                         .unwrap_or(0)
}
