use std::fmt;

/// Error returned by the Lichess API calls.
/// The details of what went wrong are logged where the error happens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LichessError;

impl fmt::Display for LichessError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Lichess API error")
  }
}

impl std::error::Error for LichessError {}
