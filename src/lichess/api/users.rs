use crate::api::LichessApi;
use crate::error::LichessError;
use crate::types::{Crosstable, User, UserStatus};
use log::*;
use serde_json::Value as JsonValue;

/// Maximum number of user IDs per users/status request
const MAX_IDS_PER_STATUS_REQUEST: usize = 100;

impl LichessApi {
  /// Checks if a player is online
  ///
  /// ### Arguments
  ///
  /// * `user_id` Username/ user_id to lookup
  ///
  /// ### Returns
  ///
  /// True if the player is online, false otherwise
  pub async fn is_online(&self, user_id: &str) -> bool {
    let endpoint: String = format!("users/status?ids={}", user_id);
    let result = self.lichess_get(&endpoint).await;

    if let Err(error) = result {
      warn!("Error parsing the result for is_online API. {:#?}", error);
      return false;
    }

    let json_object: JsonValue = result.unwrap();
    json_object[0]["online"].as_bool().unwrap_or(false)
  }

  /// Gets the real-time status (online, playing) of a list of users
  ///
  /// ### Arguments
  ///
  /// * `user_ids` Usernames/ user_ids to lookup
  ///
  /// ### Returns
  ///
  /// The status of the users, as known by Lichess
  pub async fn get_users_status(&self, user_ids: &[&str]) -> Result<Vec<UserStatus>, LichessError> {
    let mut statuses = Vec::new();
    for ids in user_ids.chunks(MAX_IDS_PER_STATUS_REQUEST) {
      let endpoint: String = format!("users/status?ids={}", ids.join(","));
      let json = self.lichess_get(&endpoint).await?;
      match serde_json::from_value::<Vec<UserStatus>>(json) {
        Ok(mut result) => statuses.append(&mut result),
        Err(error) => {
          warn!("Could not parse users status from Lichess: {error}");
          return Err(LichessError);
        },
      }
    }
    Ok(statuses)
  }

  /// Gets the public information of a user
  ///
  /// ### Arguments
  ///
  /// * `user_id` Username/ user_id to lookup
  ///
  /// ### Returns
  ///
  /// The user information
  pub async fn get_user(&self, user_id: &str) -> Result<User, LichessError> {
    let json = self.lichess_get(&format!("user/{user_id}")).await?;
    serde_json::from_value::<User>(json).map_err(|error| {
                                          warn!("Could not parse user {user_id}: {error}");
                                          LichessError
                                        })
  }

  /// Gets the list of bots currently online
  /// (the API version of https://lichess.org/player/bots)
  ///
  /// ### Arguments
  ///
  /// * `max_bots` How many bots to fetch (Lichess caps it at 512)
  ///
  /// ### Returns
  ///
  /// The list of online bots
  pub async fn get_online_bots(&self, max_bots: usize) -> Result<Vec<User>, LichessError> {
    let json_values = self.lichess_get_nd_json(&format!("bot/online?nb={max_bots}")).await?;
    Ok(Self::parse_users(json_values))
  }

  /// Gets the cross-table between 2 players
  ///
  /// ### Arguments
  ///
  /// * `user_id_1` First user to check
  /// * `user_id_2` Second user to check
  /// * `matchup`:  Use data from the current match rather than all historical
  ///   data.
  ///
  /// ### Returns
  ///
  /// The scores of both players and the number of games between them, if
  /// available
  pub async fn get_crosstable(&self,
                              user_id_1: &str,
                              user_id_2: &str,
                              matchup: bool)
                              -> Option<Crosstable> {
    let endpoint: String = format!("crosstable/{}/{}?matchup={}", user_id_1, user_id_2, matchup);
    let result = self.lichess_get(&endpoint).await;

    if let Err(error) = result {
      warn!("Error parsing the result for crosstable API. {:#?}", error);
      return None;
    }

    let json_object: JsonValue = result.unwrap();
    let scores_option = json_object["users"].as_object();
    if scores_option.is_none() {
      debug!("Could not find crosstable scores in JSON payload: {json_object}");
      return None;
    }

    let scores = scores_option.unwrap();
    let user_1_score = scores[user_id_1].as_f64();
    let user_2_score = scores[user_id_2].as_f64();

    if user_1_score.is_none() || user_2_score.is_none() {
      debug!("Could not find user scores in JSON payload: {json_object}");
      return None;
    }
    let user_1_score = user_1_score.unwrap();
    let user_2_score = user_2_score.unwrap();
    let games = json_object["nbGames"].as_u64().unwrap_or(0);

    Some(Crosstable { user_1_score,
                      user_2_score,
                      games })
  }
}
