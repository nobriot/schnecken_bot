use chess::engine::config::play_style::PlayStyle;
use chess::engine::Engine;
use chess::model::game_state::START_POSITION_FEN;
use lichess;
use lichess::types::{GameStart, Title};
use log::*;

/// Looks at the game data (which kind of opponent, time control, start
/// position) and configures the engine accordingly.
pub fn configure_engine(game: &GameStart) -> Engine {
  // We are not using the uci interface internally
  let mut engine = Engine::new(false);

  // Cache table is kinda always the same value, regardless of the game, and
  // opponent
  engine.resize_cache_tables(1024);

  // Configure the start position
  let start_fen = game.fen.as_deref().unwrap_or(START_POSITION_FEN);
  engine.set_position(&start_fen);

  // Adjust the level of difficulty based on the rating of the opponent, if they
  // are human
  if !game.opponent_is_bot() && game.opponent.title.is_none() {
    if game.opponent.rating < 1300 {
      engine.options.max_depth = 1;
      engine.options.play_style = PlayStyle::Provocative;
    } else if game.opponent.rating < 1500 {
      engine.options.max_depth = 2;
      engine.options.play_style = PlayStyle::Provocative;
    } else if game.opponent.rating < 1700 {
      engine.options.max_depth = 3;
      engine.options.play_style = PlayStyle::Provocative;
    } else if game.opponent.rating < 1800 {
      engine.options.max_depth = 4;
      engine.options.play_style = PlayStyle::Provocative;
    } else if game.opponent.rating < 1900 {
      engine.options.max_depth = 5;
    } else if game.opponent.rating < 2000 {
      engine.options.max_depth = 6;
    }
  }

  info!("Engine configuration for game {}: {:?}",
        game.game_id, engine.options);

  engine
}
