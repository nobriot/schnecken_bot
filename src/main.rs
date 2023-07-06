// External crates
use log::*;
use std::fs;

// Local modules
mod bot;
mod chess;
mod lichess;

// Constants:
const USERNAME_FILENAME: &str = "/assets/username.txt";

// Main function
fn main() {
  env_logger::init();
  let rt = tokio::runtime::Runtime::new().unwrap();

  match rt.block_on(main_loop()) {
    Ok(_) => info!("Exiting successfully."),
    Err(_) => error!("An error ocurred"),
  };
}

async fn main_loop() -> Result<(), ()> {
  // Check that the Lichess Token is okay:
  if lichess::api::get_api().token.len() == 0 {
    error!("Error reading the API token. Make sure that you have added a token file.");
    return Err(());
  }
  info!("Lichess API token loaded successfully");

  // Read our username from the text file, default on schnecken_bot if text file is not readable.
  // TODO: We should ask Lichess with account info for retrieving our username
  let username = fs::read_to_string(String::from(env!("CARGO_MANIFEST_DIR")) + USERNAME_FILENAME)
    .unwrap_or(String::from("schnecken_bot"));

  // Starts the bot, it will stream incoming events
  info!("Starting the Lichess bot... ");
  info!("Watch it at: https://lichess.org/@/{username}");
  let schnecken_bot = bot::state::BotState::new(username.as_str());
  schnecken_bot.start();

  // Start 2 tasks: one that checks stream events, one that send challenges when we have been idle for a while
  //tokio::spawn(async { lichess::api::stream_incoming_events(&stream_event_handler).await });
  //tokio::spawn(async { lichess::api::send_challenges_with_interval(3600).await });

  loop {
    // Read command line inputs for ever, until we have to exit
    let mut exit_requested: bool = false;
    if let Err(_) = bot::commands::read_user_commands(&mut exit_requested) {
      error!("Error reading user input");
    }
    if true == exit_requested {
      info!("Exiting the Lichess bot... ");
      break;
    }
  }

  // End the main loop.
  Ok(())
}
