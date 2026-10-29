// External crates
use anyhow::Result;
use clap::Parser;
use log::*;
use std::io;
use tokio::time::{Duration, sleep};

// Local modules
mod args;
mod bot;
mod config;
mod logger;
mod tee_writer;

// Main function
fn main() {
  let cli = args::Args::parse();
  logger::init_logger(cli.log_file.as_deref());
  let api_token = match config::resolve_token(cli.api_token) {
    Ok(token) => token,
    Err(e) => {
      error!("Failed to resolve API token: {e}");
      std::process::exit(1);
    },
  };

  let mut engine_config = config::load_engine_config();
  if let Some(size) = cli.cache_table_size {
    engine_config.cache_table_size = size;
  }
  if let Some(style) = cli.play_style {
    engine_config.play_style = Some(style);
  }
  info!("Engine config: {engine_config:?}");

  let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();

  match rt.block_on(main_loop(&api_token, engine_config)) {
    Ok(_) => info!("Exiting successfully."),
    Err(_) => error!("An error occurred"),
  };
}

async fn main_loop(api_token: &str, engine_config: config::EngineConfig) -> Result<()> {
  // Starts the bot, it will stream incoming events
  let schnecken_bot = bot::state::BotState::new(api_token, engine_config).await;
  schnecken_bot.start();

  loop {
    use bot::commands::BotCommands;
    // Read command line inputs for ever, until we have to exit
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    schnecken_bot.execute_command(input.trim());

    sleep(Duration::from_millis(100)).await;
    if schnecken_bot.should_exit() {
      info!("Exiting the Lichess bot... ");
      break;
    }
  }

  // Let the games send their last requests (resign, chat) while the runtime is
  // still alive: once it shuts down, new HTTP requests fail.
  schnecken_bot.wait_for_games(Duration::from_secs(15)).await;

  // End the main loop.
  Ok(())
}
