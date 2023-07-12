// Internal crates
use crate::lichess::api::LichessApi;

// External crates
use log::*;
//use serde_json::Value as JsonValue;
use urlencoding::encode;

// Constants
// Reasons for declining a challenge
pub const DECLINE_GENERIC: &str = "generic";
pub const DECLINE_TOO_FAST: &str = "tooFast";
pub const DECLINE_TOO_SLOW: &str = "tooSlow";
pub const DECLINE_RATED: &str = "rated";
pub const DECLINE_CASUAL: &str = "casual";
pub const DECLINE_STANDARD: &str = "standard";
pub const DECLINE_NOT_BOTS: &str = "noBot";
pub const DECLINE_ONLY_BOTS: &str = "onlyBot";
pub const DECLINE_VARIANT: &str = "variant";
pub const DECLINE_LATER: &str = "later";
pub const DECLINE_TIME_CONTROL: &str = "timeControl";

impl LichessApi {
  /// Attempts to accept an incoming challenge
  ///
  /// ### Parameters
  ///
  /// * `challenge_id`: The challenge ID to accept
  ///
  /// ### Returns
  ///
  /// Result
  ///
  pub async fn accept_challenge(&self, challenge_id: &str) -> Result<(), ()> {
    info!("Accepting challenge ID {challenge_id}");
    let api_endpoint: String = format!("challenge/{}/accept", challenge_id);
    if self.lichess_post(&api_endpoint, "").await.is_err() {
      return Err(());
    }

    Ok(())
  }

  /// Attempts to decline an incoming challenge with a reason
  /// See the `DECLINE_GENERIC`, DECLINE_... constants
  ///
  /// ### Parameters
  ///
  /// * `challenge_id`: The challenge ID to accept
  /// * `reason`:       The reason for declining.
  ///
  /// ### Returns
  ///
  /// Result
  ///
  pub async fn decline_challenge(&self, challenge_id: &str, reason: &str) -> Result<(), ()> {
    info!("Declining challenge ID {challenge_id}");
    let api_endpoint: String = format!("challenge/{}/decline", challenge_id);
    let body: String = format!("reason={}", encode(reason));

    if self.lichess_post(&api_endpoint, &body).await.is_ok() {
      Ok(())
    } else {
      Err(())
    }
  }

  /// Sends a challenge to another player
  ///
  /// ### Parameters
  ///
  /// * `player`: Username of the player to challenge
  ///
  /// ### Returns
  ///
  /// Result
  ///
  pub async fn send_challenge(&self, player: &str) -> Result<(), ()> {
    let api_endpoint: String = format!("challenge/{}", player);
    // Let's hardcode this to 3+0 for now.
    // FIXME: We should have Clock parameter.
    let body_parameters =
      "rated=true&clock.limit=180&clock.increment=0&color=random&variant=standard";
    if self
      .lichess_post(&api_endpoint, body_parameters)
      .await
      .is_ok()
    {
      Ok(())
    } else {
      Err(())
    }
  }
}
