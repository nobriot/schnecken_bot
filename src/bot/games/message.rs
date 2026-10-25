use lichess;

#[derive(Debug)]
pub enum GameMessage {
  /// Starts a game and allocates all the resources for playing a game on
  /// Lichess.
  #[allow(dead_code)] // Not sent yet
  Start(lichess::types::GameStart),
  /// Full game information, sent first by the game stream (also when
  /// rejoining an ongoing game): we set the initial position and replay all
  /// the moves, so that the engine knows the game history (repetitions)
  Full(lichess::types::GameFull),
  /// Updates the game state, plays moves if it is our turn
  Update(lichess::types::GameState),
  /// Notifies that the game is over (based on what the server says)
  #[allow(dead_code)] // Not sent yet
  End(lichess::types::GameState),
  /// There was just a message in the chat
  ChatMessage(lichess::types::ChatMessage),
  /// Notifies of an opponent gone event. Bool indicates if the opponent is
  /// gone, or back
  OpponentGone(Option<u64>),
  /// Resigns the game (with a goodbye message) and stops the game loop,
  /// because the program shuts down (`quit` command)
  Terminate,
  /// Stops the game loop without resigning, because the program shuts down
  /// (`exit` command). The game stays open on Lichess.
  Leave,
  /// Just a nop message to check everything is alright
  #[allow(dead_code)] // Not sent yet
  Nop,
  /// Resigns the game and stops it
  Resign,
}
