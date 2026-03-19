use chess::engine::Engine;
use chess::engine::config::play_style::PlayStyle;
use chess::model::game_state::START_POSITION_FEN;
use lichess;
use lichess::types::GameStart;
use log::*;

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

  // Adjust the level of difficulty based on the rating of the opponent, if they
  // are human
  if !game.rated && !game.opponent_is_bot() && game.opponent.title.is_none() {
    if game.opponent.rating < 1300 {
      engine.options.max_depth = 2;
      engine.options.play_style = PlayStyle::Provocative;
    } else if game.opponent.rating < 1500 {
      engine.options.max_depth = 3;
      engine.options.play_style = PlayStyle::Provocative;
    } else if game.opponent.rating < 1700 {
      engine.options.max_depth = 4;
      engine.options.play_style = PlayStyle::Provocative;
    } else if game.opponent.rating < 1800 {
      engine.options.max_depth = 5;
      engine.options.play_style = PlayStyle::Provocative;
    } else if game.opponent.rating < 1900 {
      engine.options.max_depth = 6;
    } else if game.opponent.rating < 2000 {
      engine.options.max_depth = 7;
    }
  }

  info!("Engine configuration for game {}: {:?}", game.game_id, engine.options);

  engine
}
