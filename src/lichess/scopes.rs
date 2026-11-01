//! OAuth scopes of Lichess API tokens, and which endpoints need them.

use serde_json::Value as JsonValue;
use std::fmt;

/// OAuth scope needed by some Lichess API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
  BotPlay,
  BoardPlay,
  ChallengeWrite,
  FollowRead,
  FollowWrite,
  TournamentWrite,
}

impl Scope {
  /// Name of the scope in the Lichess API, e.g. `bot:play`
  pub fn as_str(&self) -> &'static str {
    match self {
      Scope::BotPlay => "bot:play",
      Scope::BoardPlay => "board:play",
      Scope::ChallengeWrite => "challenge:write",
      Scope::FollowRead => "follow:read",
      Scope::FollowWrite => "follow:write",
      Scope::TournamentWrite => "tournament:write",
    }
  }
}

impl fmt::Display for Scope {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

/// Reads the scopes of a token in a `token/test` answer.
///
/// ### Arguments
///
/// * `json`  Answer from `token/test`: `{ "<token>": { "scopes": "a,b" } }`
/// * `token` Token we want the scopes of
///
/// ### Returns
///
/// The list of scopes, None if the token is unknown or invalid.
pub fn parse_token_scopes(json: &JsonValue, token: &str) -> Option<Vec<String>> {
  let scopes = json[token]["scopes"].as_str()?;
  Some(scopes.split(',').map(str::trim).filter(|s| !s.is_empty()).map(String::from).collect())
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_token_scopes() {
    let json = serde_json::json!({
      "lip_abc": { "userId": "schnecken_bot", "scopes": "bot:play,challenge:write", "expires": null },
      "lip_empty": { "userId": "schnecken_bot", "scopes": "", "expires": null },
      "lip_invalid": null,
    });
    assert_eq!(Some(vec!["bot:play".to_string(), "challenge:write".to_string()]),
               parse_token_scopes(&json, "lip_abc"));
    assert_eq!(Some(Vec::new()), parse_token_scopes(&json, "lip_empty"));
    assert_eq!(None, parse_token_scopes(&json, "lip_invalid"));
    assert_eq!(None, parse_token_scopes(&json, "lip_unknown"));
  }
}
