use std::io;

// Constants
const EXIT_COMMAND: &str = "exit";
const QUIT_COMMAND: &str = "quit";
const HELP_COMMAND: &str = "help";
const EMPTY_COMMAND: &str = "";

// Private functions
fn print_help() {
    println!("Welcome ! You can use the following commands:");
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
        EXIT_COMMAND | QUIT_COMMAND => {
            *exit_requested = true;
        }
        HELP_COMMAND => print_help(),
        EMPTY_COMMAND => {}
        _ => print_help(),
    }

    Ok(())
}