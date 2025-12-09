use crate::api::LichessApi;
use log::*;
use serde_json::Value as JsonValue;

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
  /// True if the player is online, false otherwise
  pub async fn get_crosstable(&self,
                              user_id_1: &str,
                              user_id_2: &str,
                              matchup: bool)
                              -> Option<(f64, f64)> {
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

    Some((user_1_score, user_2_score))
  }
}
