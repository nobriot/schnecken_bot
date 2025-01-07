use lichess;

#[derive(Debug)]
pub enum GameMessage {
  /// Starts a game and allocates all the resources for playing a game on
  /// Lichess.
  Start(lichess::types::GameStart),
  /// Updates the game state, plays moves if it is our turn
  Update(lichess::types::GameState),
  /// Notifies that the game is over (based on what the server says)
  End(lichess::types::GameState),
  /// Notifiies of an opponent gone event. Bool indicates if the opponent is
  /// gone, or back
  OpponentGone(Option<u64>),
  /// Terminates the game loop, typically because the program wants to shut down
  /// But leaves the game open (no resignation)
  Terminate,
  /// Just a nop message to check everything is alright
  Nop,
  /// Resigns the game and stops it
  Resign,
}
