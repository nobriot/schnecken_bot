// Internal crates
use crate::api::LichessApi;
use crate::error::LichessError;
use crate::types::User;
// External crates
use log::*;

impl LichessApi {
  /// Gets the list of users we are following.
  /// Requires the `follow:read` scope for the API token.
  ///
  /// ### Returns
  ///
  /// Result with the list of users we are following
  pub async fn get_following(&self) -> Result<Vec<User>, LichessError> {
    let json_values = self.lichess_get_nd_json("rel/following").await?;
    Ok(Self::parse_users(json_values))
  }

  /// Follows a player.
  /// Requires the `follow:write` scope for the API token.
  ///
  /// ### Arguments
  ///
  /// * `username` Username/ user_id of the player to follow
  ///
  /// ### Returns
  ///
  /// Result indicating if Lichess accepted the request
  pub async fn follow_user(&self, username: &str) -> Result<(), LichessError> {
    info!("Following {username}");
    self.relation_request(&format!("rel/follow/{username}")).await
  }

  /// Unfollows a player.
  /// Requires the `follow:write` scope for the API token.
  ///
  /// ### Arguments
  ///
  /// * `username` Username/ user_id of the player to unfollow
  ///
  /// ### Returns
  ///
  /// Result indicating if Lichess accepted the request
  pub async fn unfollow_user(&self, username: &str) -> Result<(), LichessError> {
    info!("Unfollowing {username}");
    self.relation_request(&format!("rel/unfollow/{username}")).await
  }

  /// Sends a follow/unfollow request and checks that Lichess answered `ok`
  async fn relation_request(&self, api_endpoint: &str) -> Result<(), LichessError> {
    let json = self.lichess_post(api_endpoint, "").await?;
    if json["ok"].as_bool() == Some(true) {
      Ok(())
    } else {
      warn!("Lichess refused {api_endpoint}: {json} (does the API token have the follow:write \
             scope?)");
      Err(LichessError)
    }
  }

  /// Parses a list of JSON values into users, skipping the ones that cannot be
  /// parsed.
  pub(crate) fn parse_users(json_values: Vec<serde_json::Value>) -> Vec<User> {
    json_values.into_iter()
               .filter_map(|json| match serde_json::from_value::<User>(json) {
                 Ok(user) => Some(user),
                 Err(error) => {
                   warn!("Could not parse user from Lichess: {error}");
                   None
                 },
               })
               .collect()
  }
}
