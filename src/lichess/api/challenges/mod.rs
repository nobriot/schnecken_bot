// Internal crates
use crate::lichess::api::LichessApi;
use crate::lichess::types::Clock;

// External crates
use log::*;
//use serde_json::Value as JsonValue;
use urlencoding::encode;

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
  /// * `clock`:  Clock settings (set clock.initial for time, and clock.increment for the increment)
  ///
  /// ### Returns
  ///
  /// Result
  ///
  pub async fn send_challenge(&self, player: &str, clock: &Clock) -> Result<(), ()> {
    let api_endpoint: String = format!("challenge/{}", player);
    let body_parameters = format!(
      "rated=true&clock.limit={}&clock.increment={}&color=random&variant=standard",
      clock.initial, clock.increment
    );
    if self
      .lichess_post(&api_endpoint, body_parameters.as_str())
      .await
      .is_ok()
    {
      Ok(())
    } else {
      Err(())
    }
  }
}
