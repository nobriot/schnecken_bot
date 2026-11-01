use crate::scopes::Scope;
use std::fmt;

/// Error returned by the Lichess API calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LichessError {
  /// The request failed or Lichess refused it. The details of what went wrong
  /// are logged where the error happens.
  Request,
  /// The request was not sent: our API token lacks this scope
  MissingScope(Scope),
}

impl fmt::Display for LichessError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      LichessError::Request => write!(f, "Lichess API error"),
      LichessError::MissingScope(scope) => write!(f, "Missing Lichess API scope: {scope}"),
    }
  }
}

impl std::error::Error for LichessError {}
