// Submodules
pub mod account;
pub mod challenges;
pub mod game;
pub mod users;

// Other crates
use log::*;

use futures_util::StreamExt;
use reqwest;
use serde_json::Value as JsonValue;

// From the same module:
use crate::lichess::helpers;

// Constants
const API_BASE_URL: &'static str = "https://lichess.org/api/";

// Type definitions
#[derive(Debug, Clone)]
pub struct LichessApi {
  /// Reqwest client use to send HTTP/HTTPS requests
  pub client: reqwest::Client,
  /// Lichess API token, giving us access to an account and some permissions
  pub token: String,
}

/// Trait for an object that can be invoked using the Event Streams
pub trait EventStreamHandler {
  /// Receives an incoming event on a Bot Account
  ///
  /// See https://lichess.org/api#tag/Bot/operation/apiStreamEvent
  /// for the JSON payload
  ///
  /// ### Arguments
  ///
  /// * `self`       Reference to the object streaming the events
  /// * `json_value` JSON object with the event details.
  ///
  fn event_stream_handler(&self, json_value: JsonValue);
}

/// Trait for an object that can be invoked using the Game Streams
pub trait GameStreamHandler {
  /// Receives an incoming game event on a Bot Account
  ///
  /// https://lichess.org/api#tag/Bot/operation/botGameStream
  /// for the JSON payload
  ///
  /// ### Arguments
  ///
  /// * `self`       Reference to the object streaming the game events
  /// * `json_value` JSON object with the event details.
  /// * `game_id`    Game ID
  ///
  fn game_stream_handler(&self, json_value: JsonValue, game_id: String);
}

impl LichessApi {
  //----------------------------------------------------------------------------
  // Private functions

  /// Sends a GET request to a given Endpoint
  async fn get(&self, api_endpoint: &str) -> Result<reqwest::Response, reqwest::Error> {
    debug!("Lichess GET request at {}{}", API_BASE_URL, api_endpoint);
    self
      .client
      .get(format!("{}{}", API_BASE_URL, api_endpoint))
      .header("Authorization", format!("Bearer {}", self.token))
      .header("Accept", "application/x-ndjson")
      .send()
      .await
  }

  /// Sends a POST request to a given Endpoint
  async fn post(
    &self,
    api_endpoint: &str,
    body: &str,
  ) -> Result<reqwest::Response, reqwest::Error> {
    debug!("Lichess POST request at {}{}", API_BASE_URL, api_endpoint);
    self
      .client
      .post(format!("{}{}", API_BASE_URL, api_endpoint))
      .header("Authorization", format!("Bearer {}", self.token))
      .header("Accept", "application/x-ndjson")
      .header("Content-Type", "application/x-www-form-urlencoded")
      .body(body.to_string())
      .send()
      .await
  }

  //----------------------------------------------------------------------------
  // Public functions

  /// Generic HTTPS Get request to Lichess.
  ///
  /// ### Arguments
  ///
  /// * `api_endpoint` Endpoint for the API, e.g. `"account/playing"` to do a
  ///   Get to `https://lichess.org/api/account/playing`
  ///
  /// ### Returns
  ///
  /// Result with a JSON value received in the API response.
  ///
  pub async fn lichess_get(&self, api_endpoint: &str) -> Result<JsonValue, ()> {
    let response_result = self.get(api_endpoint).await;

    if let Err(error) = response_result {
      warn!("Error issuing a Get request to Lichess {}", error);
      return Err(());
    }

    let response_text_result = response_result.unwrap().text().await;

    if let Err(error) = response_text_result {
      warn!(
        "Error reading the payload from Get request to Lichess {}",
        error
      );
      return Err(());
    }

    let json_value_result = serde_json::from_str(&response_text_result.unwrap());
    let json_object = match json_value_result {
      Ok(object) => object,
      Err(error) => {
        warn!(
          "Error parsing JSON from the Lichess Response for API call {api_endpoint}. Error:{error}"
        );
        return Err(());
      },
    };

    debug!("Lichess get answer: {}", json_object);
    Ok(json_object)
  }

  /// Generic HTTPS Post request to Lichess.
  ///
  /// ### Arguments
  ///
  /// * `api_endpoint` Endpoint for the API, e.g. `"account/playing"` to do a
  /// Get to `https://lichess.org/api/account/playing`
  /// * `body` for the POST message
  ///
  /// ### Returns
  ///
  /// Result with a JSON value received in the API response.
  ///
  pub async fn lichess_post(&self, api_endpoint: &str, body: &str) -> Result<JsonValue, ()> {
    let response_result = self.post(api_endpoint, body).await;
    if let Err(e) = response_result {
      warn!("Error issuing a Get request to Lichess {e}");
      return Err(());
    }

    let response_text_result = response_result.unwrap().text().await;

    if let Err(e) = response_text_result {
      warn!("Error reading the payload from Post request to Lichess {e}");
      return Err(());
    }

    //debug!("Lichess post answer: {:?}", response_text_result);
    let json_value_result = serde_json::from_str(&response_text_result.unwrap());
    let json_object = match json_value_result {
      Ok(object) => object,
      Err(error) => {
        warn!(
          "Error parsing JSON from the Lichess Response for API call {api_endpoint}. Error:{error}"
        );
        return Ok(JsonValue::Null);
      },
    };

    debug!("Lichess post answer: {}", json_object);
    Ok(json_object)
  }

  /// Streams incoming events using an object and stream handler.
  /// Refer to https://lichess.org/api/stream/event
  ///
  /// JSON values received on the stream will be passed to the stream_handler function.
  ///
  /// ### Arguments
  ///
  /// * `object` Reference to the object invoking the stream handler (e.g. bot reference)
  /// * `stream_handler` Function to invoke when data has been received on the stream
  ///
  /// ### Returns
  ///
  /// Result indicating if we had error receiving/parsing the event stream.
  ///
  pub async fn stream_incoming_events<T>(self, bot: &T) -> Result<(), ()>
  where
    T: EventStreamHandler,
  {
    let response_result = self.get("stream/event").await;

    if let Err(e) = response_result {
      warn!("Error Streaming events (get) request to Lichess {}", e);
      return Err(());
    }

    let stream = response_result.unwrap().bytes_stream();
    stream
      .for_each(|chunk_response| async {
        if let Err(e) = chunk_response {
          warn!("Error receiving stream? {}", e);
          //return futures::future::ready(());
          return ();
        }

        let chunk = chunk_response.unwrap();
        let string_value: String = String::from_utf8_lossy(&chunk).to_string();
        let json_entries = helpers::parse_string_to_nd_json(&string_value);

        for json_entry in json_entries {
          bot.event_stream_handler(json_entry);
        }
        // Sending 1 byte is usually just the keep-alive message
        if chunk.len() == 1 {
          debug!("Received keep-alive message for event stream");
        }

        ()
      })
      .await;

    Ok(())
  }

  /// Streams incoming game state events using a Game ID and stream handler.
  /// using https://lichess.org/api/bot/game/stream/{gameId}
  ///
  /// JSON values received on the stream will be passed to the stream_handler function.
  ///
  /// ### Arguments
  ///
  /// * `game_id` Game ID to stream
  /// * `stream_handler` Function to invoke when data has been received on the stream
  ///
  /// ### Returns
  ///
  /// Result indicating if we had error receiving/parsing the event stream.
  ///
  pub async fn stream_game_state<T>(self, bot: &T, game_id: &str) -> Result<(), ()>
  where
    T: GameStreamHandler,
  {
    info!("Requesting Lichess to stream Game ID {game_id}");
    let response_result = self.get(&format!("bot/game/stream/{game_id}")).await;

    if let Err(e) = response_result {
      warn!("Error issuing a Get request to Lichess {}", e);
      return Err(());
    }

    let stream = response_result.unwrap().bytes_stream();
    stream
      .for_each(|chunk_response| async {
        if let Err(e) = chunk_response {
          info!("Error receiving stream? {}", e);
          //return futures::future::ready(());
          return ();
        }

        let chunk = chunk_response.unwrap();
        let string_value: String = String::from_utf8_lossy(&chunk).to_string();
        let json_entries = helpers::parse_string_to_nd_json(&string_value);

        for json_entry in json_entries {
          bot.game_stream_handler(json_entry, String::from(game_id));
        }
        // Sending 1 byte is usually just the keep-alive message
        if chunk.len() == 1 {
          debug!("Received keep-alive message for Game State stream");
        }

        //futures::future::ready(())
        ()
      })
      .await;

    info!("Finished to stream game events for game id {game_id}");
    Ok(())
  }
} // impl LichessApi
