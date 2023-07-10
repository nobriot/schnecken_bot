// External crates
use anyhow::{anyhow, Result};
use log::*;
use std::fs;
use std::io;

// Local modules
mod bot;
mod chess;
mod lichess;

// Constants:
const API_TOKEN_FILE_NAME: &str = "/assets/lichess_api_token.txt";

// Main function
fn main() {
  env_logger::init();
  let rt = tokio::runtime::Runtime::new().unwrap();

  match rt.block_on(main_loop()) {
    Ok(_) => info!("Exiting successfully."),
    Err(_) => error!("An error ocurred"),
  };
}

async fn main_loop() -> Result<()> {
  // Check that the Lichess Token is okay:
  let api_token =
    fs::read_to_string(String::from(env!("CARGO_MANIFEST_DIR")) + API_TOKEN_FILE_NAME)?;
  if api_token.len() == 0 {
    error!("Error reading the API token. Make sure that you have added a token file.");
    return Err(anyhow!("Missing API Token"));
  }
  info!("Lichess API token loaded successfully");

  // Starts the bot, it will stream incoming events
  let mut schnecken_bot = bot::state::BotState::new(api_token.as_str()).await;
  schnecken_bot.start();

  loop {
    use crate::bot::commands::BotCommands;
    // Read command line inputs for ever, until we have to exit
    let mut input = String::new();
    let mut exit_requested: bool = false;
    io::stdin().read_line(&mut input)?;

    schnecken_bot.execute_command(input.trim(), &mut exit_requested);

    if true == exit_requested {
      info!("Exiting the Lichess bot... ");
      break;
    }
  }

  // End the main loop.
  Ok(())
}
