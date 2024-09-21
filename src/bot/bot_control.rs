// Type definitions

/// Trying to mess around with other bots in the chat
///
#[derive(Copy, Debug, Clone, Eq, PartialEq, Default)]
pub enum BotKnownCommands {
  /// This represents !help
  #[default]
  Help,
  /// This represents !wait
  Wait,
}

/// Trying to mess around with other bots in the chat
///
#[derive(Copy, Debug, Clone, Eq, PartialEq, Default)]
pub enum BotControlState {
  /// The bot has not told us anything yet.
  #[default]
  NotStarted,
  /// The bot has told us we can use !help
  HelpAnnounced,
  /// The bot has told us what commands we can use.
  SupportedCommands,
}

impl BotControlState {
  /// Tries to control the bot again based on the last state.
  pub fn next_step(self) {}
}
