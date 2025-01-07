// This was a test file, I'll probably remove that later

/*

use anyhow::Result;
use game::Game;
use games::BotGames;
use handle::GameHandle;
use lichess::api::LichessApi;
use lichess::traits::GameStreamHandler;
use log::{debug, error, info};
use tokio::runtime::Runtime;
use tokio::time::{sleep, Duration};

// Local modules
mod game;
mod games;
mod handle;
mod message;

// Constants:
const API_TOKEN: &str = include_str!("../../../assets/lichess_api_token.txt");

// Main function
fn main() {
  env_logger::builder().format_timestamp_millis().init();
  let rt = Runtime::new().unwrap();

  match rt.block_on(main_loop()) {
    Ok(_) => info!("Exiting successfully."),
    Err(_) => error!("An error occurred"),
  };
}

async fn main_loop() -> Result<()> {
  let api: &'static _ = Box::leak(Box::new(LichessApi::new(API_TOKEN)));
  let game_id = "KsAkhvi1";
  let game_1 = Game::new_with_id(game_id, &api);

  api.stream_game_state_with_callback(game_id, &game_1, GameHandle::game_stream_handler)
     .await;
  let mut games = BotGames::new(&api);
  games.add(game_1);

  loop {
    if games.is_empty() {
      break;
    }
  }

  // End the main loop.
  Ok(())
}
 */
