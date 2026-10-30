// Internal crates
use crate::api::LichessApi;
use crate::error::LichessError;
use crate::types::ArenaTournament;
// External crates
use log::*;
use urlencoding::encode;

/// Error answered by Lichess when we join too many new tournaments: about 4
/// per hour and 15 per day for tournaments created by users. Joining a
/// tournament we are already registered to does not count.
const TOO_MANY_JOINS: &str = "You are joining too many tournaments";

/// Why we could not join a tournament
#[derive(Debug, PartialEq, Eq)]
pub enum JoinArenaError {
  /// We joined too many new tournaments recently, try again later
  TooManyJoins,
  /// Lichess refused for another reason (missing team, missing scope, etc.)
  Refused,
  /// The request failed
  Request,
}

impl LichessApi {
  /// Gets the arena tournaments of a team, last starting first.
  ///
  /// ### Arguments
  ///
  /// * `team_id` ID of the team, e.g. `"daily-bot-tournaments"`
  /// * `status`  Only tournaments in this state: `"created"`, `"started"` or `"finished"`
  ///
  /// ### Returns
  ///
  /// Result with the list of arena tournaments
  pub async fn get_team_arenas(&self,
                               team_id: &str,
                               status: &str)
                               -> Result<Vec<ArenaTournament>, LichessError> {
    let json_values =
      self.lichess_get_nd_json(&format!("team/{team_id}/arena?status={status}")).await?;
    Ok(Self::parse_arenas(json_values))
  }

  /// Joins an arena tournament (also unpauses it if we paused it).
  /// Requires the `tournament:write` scope for the API token.
  ///
  /// ### Arguments
  ///
  /// * `tournament_id` ID of the arena tournament
  /// * `pair_me_asap`  If the tournament is started, asks Lichess to pair us even though we are not
  ///   connected to the tournament page. This expires after one minute, so it has to be sent again
  ///   regularly.
  /// * `team`          Team we play for, required for team battles. We must be a member of that
  ///   team.
  ///
  /// ### Returns
  ///
  /// Result indicating if Lichess accepted the request
  pub async fn join_arena(&self,
                          tournament_id: &str,
                          pair_me_asap: bool,
                          team: Option<&str>)
                          -> Result<(), JoinArenaError> {
    let api_endpoint = format!("tournament/{tournament_id}/join");
    let mut body = format!("pairMeAsap={pair_me_asap}");
    if let Some(team) = team {
      body.push_str(&format!("&team={}", encode(team)));
    }
    let json =
      self.lichess_post(&api_endpoint, &body).await.map_err(|_| JoinArenaError::Request)?;
    if json["ok"].as_bool() == Some(true) {
      Ok(())
    } else if json["error"].as_str() == Some(TOO_MANY_JOINS) {
      Err(JoinArenaError::TooManyJoins)
    } else if let Some(error) = json["error"].as_str() {
      warn!("Lichess refused to join tournament {tournament_id}: {error}");
      Err(JoinArenaError::Refused)
    } else {
      warn!("Lichess refused to join tournament {tournament_id}: {json} (does the API token have \
             the tournament:write scope?)");
      Err(JoinArenaError::Refused)
    }
  }

  /// Leaves a future arena tournament, or pauses an ongoing one (we can join
  /// again later, points are kept).
  /// Note: leaving a future tournament deletes our registration, joining it
  /// again counts as a new join (see `JoinArenaError::TooManyJoins`).
  /// Requires the `tournament:write` scope for the API token.
  ///
  /// ### Arguments
  ///
  /// * `tournament_id` ID of the arena tournament
  ///
  /// ### Returns
  ///
  /// Result indicating if Lichess accepted the request
  pub async fn withdraw_arena(&self, tournament_id: &str) -> Result<(), LichessError> {
    let json = self.lichess_post(&format!("tournament/{tournament_id}/withdraw"), "").await?;
    if json["ok"].as_bool() == Some(true) {
      Ok(())
    } else {
      debug!("Lichess refused to withdraw from tournament {tournament_id}: {json}");
      Err(LichessError)
    }
  }

  /// Parses a list of JSON values into arena tournaments, skipping the ones
  /// that cannot be parsed.
  pub(crate) fn parse_arenas(json_values: Vec<serde_json::Value>) -> Vec<ArenaTournament> {
    json_values.into_iter()
               .filter_map(|json| match serde_json::from_value::<ArenaTournament>(json) {
                 Ok(arena) => Some(arena),
                 Err(error) => {
                   warn!("Could not parse arena tournament from Lichess: {error}");
                   None
                 },
               })
               .collect()
  }
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use super::*;
  use crate::api::account::parse_token_scopes;
  use crate::types::{ARENA_STATUS_FINISHED, GameStart, VariantKey};

  #[test]
  fn test_parse_arenas() {
    // Example from the Lichess API documentation, plus an invalid line
    let json = vec![serde_json::json!({
                      "id": "LojvlQOo",
                      "createdBy": "natebrady23",
                      "system": "arena",
                      "minutes": 60,
                      "clock": { "limit": 300, "increment": 3 },
                      "rated": false,
                      "fullName": "Return of the BBB No. 140 Arena",
                      "nbPlayers": 12,
                      "variant": { "key": "standard", "short": "Std", "name": "Standard" },
                      "startsAt": 1762914600000u64,
                      "finishesAt": 1762918200000u64,
                      "status": 30,
                      "perf": { "key": "blitz", "name": "Blitz", "position": 1, "icon": ")" },
                      "teamMember": "bradys-blunder-buddies",
                    }),
                    serde_json::json!({ "id": "broken" })];

    let arenas = LichessApi::parse_arenas(json);
    assert_eq!(1, arenas.len());
    let arena = &arenas[0];
    assert_eq!("LojvlQOo", arena.id);
    assert_eq!("Return of the BBB No. 140 Arena", arena.full_name);
    assert!(!arena.rated);
    assert_eq!(VariantKey::Standard, arena.variant.key);
    assert_eq!(300, arena.clock.limit);
    assert_eq!(3, arena.clock.increment);
    assert_eq!(1762914600000, arena.starts_at);
    assert_eq!(1762918200000, arena.finishes_at);
    assert_eq!(ARENA_STATUS_FINISHED, arena.status);
    assert_eq!(None, arena.bots_allowed);
    assert!(arena.team_battle.is_none());
  }

  #[test]
  fn test_parse_team_battle() {
    // Line from the daily-bot-tournaments team arena list
    let json = vec![serde_json::json!({
                      "id": "YGyJEbiw",
                      "createdBy": "sybsuper",
                      "system": "arena",
                      "minutes": 120,
                      "clock": { "limit": 60, "increment": 0 },
                      "rated": true,
                      "fullName": "Team Battle Team Battle",
                      "nbPlayers": 0,
                      "variant": { "key": "standard", "short": "Std", "name": "Standard" },
                      "startsAt": 1793098800000u64,
                      "finishesAt": 1793106000000u64,
                      "status": 10,
                      "perf": { "key": "bullet", "name": "Bullet", "position": 0, "icon": "T" },
                      "botsAllowed": true,
                      "teamBattle": {
                        "teams": ["chess-pro-elnur-uktamov", "daily-bot-tournaments"],
                        "nbLeaders": 5
                      },
                    })];

    let arenas = LichessApi::parse_arenas(json);
    assert_eq!(1, arenas.len());
    assert_eq!(Some(true), arenas[0].bots_allowed);
    assert_eq!(vec!["chess-pro-elnur-uktamov", "daily-bot-tournaments"],
               arenas[0].team_battle.as_ref().unwrap().teams);
  }

  #[test]
  fn test_game_start_tournament_id() {
    let json = serde_json::json!({
      "gameId": "5IrD6Gzz",
      "fullId": "5IrD6GzzAbCd",
      "color": "white",
      "fen": "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
      "hasMoved": false,
      "isMyTurn": true,
      "lastMove": "",
      "opponent": { "id": "maia1", "username": "BOT maia1", "rating": 1500 },
      "rated": true,
      "speed": "blitz",
      "secondsLeft": 180,
      "rating": 1650,
      "tournamentId": "hL7vMrFQ",
    });
    let game: GameStart = serde_json::from_value(json).unwrap();
    assert_eq!(Some("hL7vMrFQ".to_string()), game.tournament_id);
    assert_eq!(Some(1650), game.rating);

    // Casual games have neither
    let json = serde_json::json!({
      "gameId": "5IrD6Gzz",
      "color": "black",
      "hasMoved": true,
      "isMyTurn": false,
      "opponent": { "id": "maia1", "username": "BOT maia1", "rating": 1500 },
      "rated": false,
      "speed": "bullet",
      "secondsLeft": 60,
    });
    let game: GameStart = serde_json::from_value(json).unwrap();
    assert_eq!(None, game.tournament_id);
    assert_eq!(None, game.rating);
  }

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
