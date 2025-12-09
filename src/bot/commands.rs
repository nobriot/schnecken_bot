use crate::bot::state::BotStateRef;

// Constants
const EXIT_COMMAND: &str = "exit";
const QUIT_COMMAND: &str = "quit";
const Q_COMMAND: &str = "q";
const HELP_COMMAND: &str = "help";
const PLAY_COMMAND: &str = "play";
const P_COMMAND: &str = "p";
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
  println!("{} or {} - Attempts to play with one of our favorite players",
           PLAY_COMMAND, P_COMMAND);
  println!("{} - Exits the program - keep ongoing games alive", EXIT_COMMAND);
  println!("{} or {} - Exits the program - Aborts/resigns ongoing games",
           QUIT_COMMAND, Q_COMMAND);
  println!("{} - Displays the help", HELP_COMMAND);
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
      _ => print_help(),
    }
  }
}
