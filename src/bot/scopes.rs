//! OAuth scopes the bot needs on its Lichess API token.

use lichess::scopes::Scope;

/// A scope we need, and what it is used for
pub struct RequiredScope {
  pub scope:    Scope,
  pub used_for: &'static str,
}

/// Scopes the bot needs
pub const REQUIRED_SCOPES: &[RequiredScope] = &[RequiredScope { scope:    Scope::BotPlay,
                                                                used_for: "playing games", },
                                                RequiredScope { scope:    Scope::ChallengeWrite,
                                                                used_for: "sending and answering challenges", },
                                                RequiredScope { scope:    Scope::FollowRead,
                                                                used_for: "finding opponents among the users we follow", },
                                                RequiredScope { scope:    Scope::FollowWrite,
                                                                used_for: "following/unfollowing opponents", },
                                                RequiredScope { scope:    Scope::TournamentWrite,
                                                                used_for: "joining tournaments", }];

/// Scopes the bot can do without, losing only a minor feature
pub const OPTIONAL_SCOPES: &[RequiredScope] = &[RequiredScope { scope:    Scope::BoardPlay,
                                                                used_for: "claiming victory when the opponent left", }];

/// Returns the required scopes missing from the list of scopes of our token
pub fn missing_scopes(granted: &[String]) -> Vec<&'static RequiredScope> {
  missing_from(REQUIRED_SCOPES, granted)
}

/// Returns the optional scopes missing from the list of scopes of our token
pub fn missing_optional_scopes(granted: &[String]) -> Vec<&'static RequiredScope> {
  missing_from(OPTIONAL_SCOPES, granted)
}

fn missing_from(scopes: &'static [RequiredScope],
                granted: &[String])
                -> Vec<&'static RequiredScope> {
  scopes.iter()
        .filter(|wanted| !granted.iter().any(|s| s == wanted.scope.as_str()))
        .collect()
}

/// Builds an error message explaining which scopes are missing, what does not
/// work without them, and how to get a token with all the scopes.
pub fn missing_scopes_message(missing: &[&RequiredScope]) -> String {
  let missing_list: Vec<String> =
    missing.iter().map(|s| format!("{} ({})", s.scope, s.used_for)).collect();
  let url_scopes: Vec<String> = REQUIRED_SCOPES.iter()
                                               .chain(OPTIONAL_SCOPES)
                                               .map(|s| format!("scopes[]={}", s.scope))
                                               .collect();
  format!("The Lichess API token is missing scopes: {}. These features will not work. Create a new \
           token at https://lichess.org/account/oauth/token/create?{}&description=schnecken_bot \
           and update ~/.config/schnecken_bot/config.toml (or pass it with --api-token / \
           LICHESS_API_TOKEN).",
          missing_list.join(", "),
          url_scopes.join("&"))
}
