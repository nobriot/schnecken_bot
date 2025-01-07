// Internal crates
use crate::api::LichessApi;

// External crates
use serde_json::Value as JsonValue;

impl LichessApi {
  /// Read public information about our Lichess profile.
  /// Queries using the `account` API endpoint.
  ///
  /// ### Returns
  ///
  /// Result with JSON value (containing account information) in case of success.
  ///
  pub async fn get_profile(&self) -> Result<JsonValue, ()> {
    self.lichess_get("account").await
  }

  /// Fetches our username from Lichess
  /// Queries using the `account` API endpoint.
  ///
  /// ### Returns
  ///
  /// Result with JSON value (containing account information) in case of success.
  ///
  pub async fn get_lichess_username(&self) -> Result<String, ()> {
    let json = self.lichess_get("account").await?;

    if json["id"].as_str().is_none() {
      Err(())
    } else {
      Ok(String::from(json["id"].as_str().unwrap()))
    }
  }

  /// Checks the ongoing games.
  /// Queries using the `account/playing` API endpoint.
  ///
  /// ### Returns
  ///
  /// Result with JSON value in case of success.
  ///
  pub async fn get_ongoing_games(&self) -> Result<JsonValue, ()> {
    self.lichess_get("account/playing").await
  }
}
