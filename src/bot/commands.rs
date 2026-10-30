use crate::bot::state::BotStateRef;
use crate::logger;
use log::*;

// Constants
const EXIT_COMMAND: &str = "exit";
const QUIT_COMMAND: &str = "quit";
const Q_COMMAND: &str = "q";
const HELP_COMMAND: &str = "help";
const PLAY_COMMAND: &str = "play";
const P_COMMAND: &str = "p";
const LOG_COMMAND: &str = "log";
const EMPTY_COMMAND: &str = "";

// -----------------------------------------------------------------------------
// Trait definition
pub trait BotCommands {
  /// Function that reads some text input and executes a bot command
  ///
  /// # Arguments
  ///
  /// * `self` -            Object reference for which we implement the method
  /// * `input` -           Text input / Command to interpret
  fn execute_command(self, input: &str);
}

// -----------------------------------------------------------------------------
// Helper functions
fn print_help() {
  println!("Welcome ! You can use the following commands:\n\n");
  println!("{} or {} - Challenges a bot we follow (or a new bot with a similar rating)",
           PLAY_COMMAND, P_COMMAND);
  println!("{} - Exits the program - keep ongoing games alive", EXIT_COMMAND);
  println!("{} or {} - Exits the program - Aborts/resigns ongoing games",
           QUIT_COMMAND, Q_COMMAND);
  println!("{} <level> - Sets the log level: off, error, warn, info, debug, trace",
           LOG_COMMAND);
  println!("{} - Displays the help", HELP_COMMAND);
}

fn set_log_level(level: &str) {
  match level.parse::<LevelFilter>() {
    Ok(level) => {
      logger::set_log_level(level);
      println!("Log level set to {level}");
    },
    Err(_) => {
      println!("Unknown log level '{level}', use one of: off, error, warn, info, debug, trace")
    },
  }
}

// -----------------------------------------------------------------------------
// Implementation
impl BotCommands for BotStateRef {
  fn execute_command(self, input: &str) {
    // Remember to trim, it will also remove the newline
    match input.trim() as &str {
      PLAY_COMMAND | P_COMMAND => {
        tokio::spawn(async { self.challenge_somebody().await });
      },
      EXIT_COMMAND => {
        self.request_exit(false);
      },
      QUIT_COMMAND | Q_COMMAND => {
        self.request_exit(true);
      },
      HELP_COMMAND => print_help(),
      EMPTY_COMMAND => {},
      input => match input.split_once(' ') {
        Some((LOG_COMMAND, level)) => set_log_level(level.trim()),
        _ => print_help(),
      },
    }
  }
}
