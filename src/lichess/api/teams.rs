// Internal crates
use crate::api::LichessApi;
use crate::error::LichessError;
// External crates
use log::*;

impl LichessApi {
  /// Gets the IDs of the teams a user is a member of.
  ///
  /// ### Arguments
  ///
  /// * `username` Username / user ID of the player
  ///
  /// ### Returns
  ///
  /// Result with the list of team IDs
  pub async fn get_user_teams(&self, username: &str) -> Result<Vec<String>, LichessError> {
    let json = self.lichess_get(&format!("team/of/{username}")).await?;
    let Some(teams) = json.as_array() else {
      warn!("Unexpected answer from Lichess for the teams of {username}: {json}");
      return Err(LichessError);
    };
    Ok(teams.iter().filter_map(|team| team["id"].as_str().map(String::from)).collect())
  }
}
