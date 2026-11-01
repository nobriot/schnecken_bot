// Internal crates
use crate::api::LichessApi;
use crate::error::LichessError;
use crate::scopes::parse_token_scopes;
// External crates
use log::*;
use serde_json::Value as JsonValue;

impl LichessApi {
  /// Read public information about our Lichess profile.
  /// Queries using the `account` API endpoint.
  ///
  /// ### Returns
  ///
  /// Result with JSON value (containing account information) in case of
  /// success.
  pub async fn get_profile(&self) -> Result<JsonValue, LichessError> {
    self.lichess_get("account").await
  }

  /// Fetches our username from Lichess
  /// Queries using the `account` API endpoint.
  ///
  /// ### Returns
  ///
  /// Result with JSON value (containing account information) in case of
  /// success.
  pub async fn get_lichess_username(&self) -> Result<String, LichessError> {
    let json = self.lichess_get("account").await?;

    if json["id"].as_str().is_none() {
      Err(LichessError::Request)
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
  pub async fn get_ongoing_games(&self) -> Result<JsonValue, LichessError> {
    self.lichess_get("account/playing").await
  }

  /// Gets the OAuth scopes of our API token, e.g. `bot:play`.
  /// Queries using the `token/test` API endpoint.
  ///
  /// ### Returns
  ///
  /// Result with the list of scopes, or an error if the token is invalid or
  /// Lichess could not be reached.
  pub async fn get_token_scopes(&self) -> Result<Vec<String>, LichessError> {
    // Not using lichess_post: it logs the answer, which contains the token
    let response = self.post("token/test", &self.token).await.map_err(|error| {
                     warn!("Error testing the API token with Lichess: {error:?}");
                     LichessError::Request
                   })?;
    let text = response.text().await.map_err(|error| {
                 warn!("Error reading the API token test answer from Lichess: {error:?}");
                 LichessError::Request
               })?;
    let json: JsonValue = serde_json::from_str(&text).map_err(|error| {
                            warn!("Error parsing the API token test answer from Lichess: {error}");
                            LichessError::Request
                          })?;
    let Some(scopes) = parse_token_scopes(&json, &self.token) else {
      warn!("Lichess does not recognize our API token, cannot read its scopes");
      return Err(LichessError::Request);
    };
    Ok(scopes)
  }
}
