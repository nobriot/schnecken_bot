use crate::lichess;
use std::io;

// Constants
const EXIT_COMMAND: &str = "exit";
const QUIT_COMMAND: &str = "quit";
const HELP_COMMAND: &str = "help";
const PLAY_COMMAND: &str = "play";
const P_COMMAND: &str = "p";
const EMPTY_COMMAND: &str = "";

// Private functions
fn print_help() {
  println!("Welcome ! You can use the following commands:");
  println!(
    "{} or {} - Attempts to play with one of our favorite players",
    PLAY_COMMAND, P_COMMAND
  );
  println!("{} - Exits the program", EXIT_COMMAND);
  println!("{} - Exits the program", QUIT_COMMAND);
  println!("{} - Displays the help", HELP_COMMAND);
}

// Public functions
pub fn read_user_commands(exit_requested: &mut bool) -> Result<(), std::io::Error> {
  // Read the line from the terminal
  let mut input = String::new();
  io::stdin().read_line(&mut input)?;

  // Remember to trim, it will also remove the newline
  match input.trim() as &str {
    PLAY_COMMAND | P_COMMAND => {
      tokio::spawn(async { lichess::play().await });
    },
    EXIT_COMMAND | QUIT_COMMAND => {
      *exit_requested = true;
    },
    HELP_COMMAND => print_help(),
    EMPTY_COMMAND => {},
    _ => print_help(),
  }

  Ok(())
}
