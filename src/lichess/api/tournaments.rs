// Internal crates
use crate::api::LichessApi;
use crate::error::LichessError;
use crate::types::ArenaTournament;
// External crates
use log::*;

impl LichessApi {
  /// Gets the arena tournaments of a team, last starting first.
  ///
  /// ### Arguments
  ///
  /// * `team_id` ID of the team, e.g. `"daily-bot-tournaments"`
  /// * `status`  Only tournaments in this state: `"created"`, `"started"` or
  ///   `"finished"`
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
  /// * `pair_me_asap`  If the tournament is started, asks Lichess to pair us
  ///   even though we are not connected to the tournament page. This expires
  ///   after one minute, so it has to be sent again regularly.
  ///
  /// ### Returns
  ///
  /// Result indicating if Lichess accepted the request
  pub async fn join_arena(&self,
                          tournament_id: &str,
                          pair_me_asap: bool)
                          -> Result<(), LichessError> {
    let api_endpoint = format!("tournament/{tournament_id}/join");
    let json = self.lichess_post(&api_endpoint, &format!("pairMeAsap={pair_me_asap}")).await?;
    if json["ok"].as_bool() == Some(true) {
      Ok(())
    } else {
      warn!("Lichess refused to join tournament {tournament_id}: {json} (does the API token have \
             the tournament:write scope?)");
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
