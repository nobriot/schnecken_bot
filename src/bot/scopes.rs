//! OAuth scopes the bot needs on its Lichess API token.

/// A scope we need, and what it is used for
pub struct RequiredScope {
  pub scope:    &'static str,
  pub used_for: &'static str,
}

/// Scopes the bot needs
pub const REQUIRED_SCOPES: &[RequiredScope] = &[RequiredScope { scope:    "bot:play",
                                                                used_for: "playing games", },
                                                RequiredScope { scope:    "challenge:write",
                                                                used_for: "sending and answering challenges", },
                                                RequiredScope { scope:    "follow:read",
                                                                used_for: "finding opponents among the users we follow", },
                                                RequiredScope { scope:    "follow:write",
                                                                used_for: "following/unfollowing opponents", },
                                                RequiredScope { scope:    "tournament:write",
                                                                used_for: "joining tournaments", }];

/// Scope needed to join tournaments
pub const TOURNAMENT_SCOPE: &str = "tournament:write";

/// Returns the required scopes missing from the list of scopes of our token
pub fn missing_scopes(granted: &[String]) -> Vec<&'static RequiredScope> {
  REQUIRED_SCOPES.iter()
                 .filter(|required| !granted.iter().any(|s| s == required.scope))
                 .collect()
}

/// Builds an error message explaining which scopes are missing, what does not
/// work without them, and how to get a token with all the scopes.
pub fn missing_scopes_message(missing: &[&RequiredScope]) -> String {
  let missing_list: Vec<String> =
    missing.iter().map(|s| format!("{} ({})", s.scope, s.used_for)).collect();
  let url_scopes: Vec<String> =
    REQUIRED_SCOPES.iter().map(|s| format!("scopes[]={}", s.scope)).collect();
  format!("The Lichess API token is missing scopes: {}. These features will not work. Create a new \
           token at https://lichess.org/account/oauth/token/create?{}&description=schnecken_bot \
           and update ~/.config/schnecken_bot/config.toml (or pass it with --api-token / \
           LICHESS_API_TOKEN).",
          missing_list.join(", "),
          url_scopes.join("&"))
}
