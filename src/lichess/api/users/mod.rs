use crate::lichess::api::LichessApi;
use serde_json::Value as JsonValue;

use log::*;

impl LichessApi {
  /// Checks if a player is online
  ///
  /// ### Arguments
  ///
  /// * `game_id` Game ID to abort
  ///
  /// ### Returns
  ///
  /// True if the player is online, false otherwise
  ///
  pub async fn is_online(&self, user_id: &str) -> bool {
    let endpoint: String = String::from(format!("users/status?ids={}", user_id));
    let result = self.lichess_get(&endpoint).await;

    if let Err(error) = result {
      warn!("Error parsing the result for is_online API. {:#?}", error);
      return false;
    }

    let json_object: JsonValue = result.unwrap();
    return json_object[0]["online"].as_bool().unwrap_or(false);
  }
}
