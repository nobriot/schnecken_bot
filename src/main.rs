// External crates
use anyhow::{anyhow, Result};
use log::*;
use std::fs;
use std::io;
use tokio::runtime::Runtime;
use tokio::time::{sleep, Duration};

// Local modules
mod bot;

// Constants:
const API_TOKEN_FILE_NAME: &str = "/assets/lichess_api_token.txt";

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
  // Check that the Lichess Token is okay:
  let api_token_file = String::from(env!("CARGO_MANIFEST_DIR")) + API_TOKEN_FILE_NAME;
  let api_token = fs::read_to_string(api_token_file)?;
  if api_token.is_empty() {
    error!("Error reading the API token. Make sure that you have added a token file.");
    return Err(anyhow!("Missing API Token"));
  }
  info!("Lichess API token loaded successfully");

  // Starts the bot, it will stream incoming events
  let schnecken_bot = bot::state::BotState::new(api_token.as_str()).await;
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

  // End the main loop.
  Ok(())
}
