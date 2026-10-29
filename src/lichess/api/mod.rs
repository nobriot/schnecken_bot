// Submodules
pub mod account;
pub mod challenges;
pub mod game;
pub mod games;
pub mod relations;
pub mod teams;
pub mod tournaments;
pub mod users;

// Other crates
// From the same library:
use crate::error::LichessError;
use crate::helpers;
use crate::traits::{EventStreamHandler, GameStreamHandler};
use futures_util::StreamExt;
use log::*;
use reqwest;
use serde_json::Value as JsonValue;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// Constants
static API_BASE_URL: &str = "https://lichess.org/api/";
/// Wait after a first 429 (Too Many Requests) response. Lichess says one
/// minute is enough in most cases: https://lichess.org/api#description/rate-limiting
const RATE_LIMIT_BASE_WAIT: Duration = Duration::from_secs(60);
/// Longest wait when we keep getting rate limited
const RATE_LIMIT_MAX_WAIT: Duration = Duration::from_secs(15 * 60);
/// How many times a request is retried after a 429 response
const RATE_LIMIT_MAX_RETRIES: u32 = 2;

// It is okay to pass the LichessApi references between threads
unsafe impl Sync for LichessApi {}

/// Rate limiting state, shared by all the clones of a LichessApi
#[derive(Debug, Default)]
struct RateLimit {
  /// We should not send requests before this time
  until:       Option<Instant>,
  /// Number of times we got rate limited in a row
  consecutive: u32,
  /// Last time we received a 429 response
  last:        Option<Instant>,
}

// Type definitions
#[derive(Debug, Clone)]
pub struct LichessApi {
  /// Reqwest client use to send HTTP/HTTPS requests
  client:     reqwest::Client,
  /// Lichess API token, giving us access to an account and some permissions
  token:      String,
  /// Rate limiting state, after receiving 429 responses
  rate_limit: Arc<Mutex<RateLimit>>,
}

/// Computes how long to wait after a 429 (Too Many Requests) response.
/// The wait doubles each time we get rate limited in a row, and is never
/// shorter than what Lichess asks for in a `Retry-After` header.
///
/// ### Arguments
///
/// * `consecutive` Number of times we got rate limited in a row (1 for the
///   first time)
/// * `retry_after` Wait requested by Lichess, if any
///
/// ### Returns
///
/// Duration to wait before sending requests again
pub fn rate_limit_wait(consecutive: u32, retry_after: Option<Duration>) -> Duration {
  let exponent = consecutive.saturating_sub(1).min(16);
  let wait = RATE_LIMIT_BASE_WAIT.saturating_mul(1 << exponent).min(RATE_LIMIT_MAX_WAIT);
  wait.max(retry_after.unwrap_or_default())
}

/// Reads the `Retry-After` header (in seconds) of a response, if any
fn parse_retry_after(response: &reqwest::Response) -> Option<Duration> {
  let value = response.headers().get(reqwest::header::RETRY_AFTER)?;
  let seconds = value.to_str().ok()?.trim().parse::<u64>().ok()?;
  Some(Duration::from_secs(seconds))
}

impl LichessApi {
  /// Creates a new instance of a LichessApi, using the API token
  pub fn new(token: &str) -> Self {
    LichessApi { client:     reqwest::Client::new(),
                 token:      token.trim().to_string(),
                 rate_limit: Arc::new(Mutex::new(RateLimit::default())), }
  }

  //----------------------------------------------------------------------------
  // Private functions

  /// Sends a GET request to a given Endpoint
  async fn get(&self, api_endpoint: &str) -> Result<reqwest::Response, reqwest::Error> {
    debug!("Lichess GET request at {}{}", API_BASE_URL, api_endpoint);
    self.send(api_endpoint, || {
          self.client
              .get(format!("{}{}", API_BASE_URL, api_endpoint))
              .header("Authorization", format!("Bearer {}", self.token))
              .header("Accept", "application/x-ndjson")
        })
        .await
  }

  /// Sends a POST request to a given Endpoint
  async fn post(&self,
                api_endpoint: &str,
                body: &str)
                -> Result<reqwest::Response, reqwest::Error> {
    debug!("Lichess POST request at {}{}", API_BASE_URL, api_endpoint);
    self.send(api_endpoint, || {
          self.client
              .post(format!("{}{}", API_BASE_URL, api_endpoint))
              .header("Authorization", format!("Bearer {}", self.token))
              .header("Accept", "application/x-ndjson")
              .header("Content-Type", "application/x-www-form-urlencoded")
              .body(body.to_string())
        })
        .await
  }

  /// Sends a request, handling rate limiting: waits if we were rate limited
  /// recently, and retries after a while if Lichess answers with a 429.
  ///
  /// ### Arguments
  ///
  /// * `api_endpoint` Endpoint of the request, for logging
  /// * `request`      Builds the request to send (called again for retries)
  ///
  /// ### Returns
  ///
  /// The response from Lichess, or an error if we are still rate limited
  /// after all the retries.
  async fn send<F>(&self,
                   api_endpoint: &str,
                   request: F)
                   -> Result<reqwest::Response, reqwest::Error>
    where F: Fn() -> reqwest::RequestBuilder
  {
    let mut retries = 0;
    loop {
      self.wait_if_rate_limited().await;
      let response = request().send().await?;

      if response.status() != reqwest::StatusCode::TOO_MANY_REQUESTS {
        self.rate_limit.lock().unwrap().consecutive = 0;
        return Ok(response);
      }

      let wait = self.on_rate_limited(parse_retry_after(&response));
      if retries >= RATE_LIMIT_MAX_RETRIES {
        warn!("Rate limited by Lichess on {api_endpoint}, giving up after {retries} retries");
        // Turn the 429 into an error, so that callers do not mistake it for a
        // valid answer
        return response.error_for_status();
      }
      retries += 1;
      warn!("Rate limited by Lichess on {api_endpoint}. Retrying in {}s ({retries}/{})",
            wait.as_secs(),
            RATE_LIMIT_MAX_RETRIES);
    }
  }

  /// Waits until the end of the current rate limiting period, if any
  async fn wait_if_rate_limited(&self) {
    let until = self.rate_limit.lock().unwrap().until;
    if let Some(until) = until {
      let now = Instant::now();
      if until > now {
        debug!("Waiting {}s before sending requests to Lichess (rate limited)",
               (until - now).as_secs());
        tokio::time::sleep(until - now).await;
      }
    }
  }

  /// Records that we got a 429 response, and returns how long we have to wait.
  /// Several requests rate limited at the same time only count once, so that a
  /// burst of parallel requests does not escalate the wait.
  fn on_rate_limited(&self, retry_after: Option<Duration>) -> Duration {
    let mut rate_limit = self.rate_limit.lock().unwrap();
    let now = Instant::now();
    let already_waiting = rate_limit.until.is_some_and(|until| until > now);
    if !already_waiting {
      rate_limit.consecutive += 1;
    }

    rate_limit.last = Some(now);

    let wait = rate_limit_wait(rate_limit.consecutive.max(1), retry_after);
    let until = (now + wait).max(rate_limit.until.unwrap_or(now));
    rate_limit.until = Some(until);
    until - now
  }

  //----------------------------------------------------------------------------
  // Public functions

  /// Checks if Lichess rate limited us recently, so that callers can hold off
  /// non-essential requests (e.g. sending challenges) for a while.
  ///
  /// ### Arguments
  ///
  /// * `period` How far back to look for a 429 response
  ///
  /// ### Returns
  ///
  /// True if we are currently waiting for a rate limit to clear, or received a
  /// 429 response within `period`
  pub fn rate_limited_within(&self, period: Duration) -> bool {
    let rate_limit = self.rate_limit.lock().unwrap();
    let now = Instant::now();
    rate_limit.until.is_some_and(|until| until > now)
    || rate_limit.last.is_some_and(|last| now.duration_since(last) < period)
  }

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
  pub async fn lichess_get(&self, api_endpoint: &str) -> Result<JsonValue, LichessError> {
    let response_result = self.get(api_endpoint).await;

    if let Err(error) = response_result {
      warn!("Error issuing a GET request to Lichess: {error:?}");
      return Err(LichessError);
    }

    let response_text_result = response_result.unwrap().text().await;

    if let Err(error) = response_text_result {
      warn!("Error reading the payload from GET request to Lichess: {error:?}");
      return Err(LichessError);
    }

    let json_value_result = serde_json::from_str(&response_text_result.unwrap());
    let json_object = match json_value_result {
      Ok(object) => object,
      Err(error) => {
        warn!("Error parsing JSON from the Lichess Response for API call {api_endpoint}. Error:{error}");
        return Err(LichessError);
      },
    };

    debug!("Lichess get answer: {}", json_object);
    Ok(json_object)
  }

  /// HTTPS Get request to Lichess, for endpoints answering with
  /// [ND-JSON](https://lichess.org/api#description/streaming-with-nd-json)
  /// (one JSON object per line) and closing the stream when done.
  ///
  /// ### Arguments
  ///
  /// * `api_endpoint` Endpoint for the API, e.g. `"rel/following"`
  ///
  /// ### Returns
  ///
  /// Result with the list of JSON values received in the API response.
  pub async fn lichess_get_nd_json(&self,
                                   api_endpoint: &str)
                                   -> Result<Vec<JsonValue>, LichessError> {
    let response = match self.get(api_endpoint).await {
      Ok(response) => response,
      Err(error) => {
        warn!("Error issuing a GET request to Lichess: {error:?}");
        return Err(LichessError);
      },
    };

    let status = response.status();
    let text = match response.text().await {
      Ok(text) => text,
      Err(error) => {
        warn!("Error reading the payload from GET request to Lichess: {error:?}");
        return Err(LichessError);
      },
    };

    if !status.is_success() {
      warn!("Lichess GET request for {api_endpoint} failed with status {status}: {text}");
      return Err(LichessError);
    }

    Ok(helpers::parse_string_to_nd_json(&text))
  }

  /// Generic HTTPS Post request to Lichess.
  ///
  /// ### Arguments
  ///
  /// * `api_endpoint` Endpoint for the API, e.g. `"account/playing"` to do a
  ///   Get to `https://lichess.org/api/account/playing`
  /// * `body` for the POST message
  ///
  /// ### Returns
  ///
  /// Result with a JSON value received in the API response.
  pub async fn lichess_post(&self,
                            api_endpoint: &str,
                            body: &str)
                            -> Result<JsonValue, LichessError> {
    let response_result = self.post(api_endpoint, body).await;
    if let Err(e) = response_result {
      warn!("Error issuing a POST request to Lichess: {e:?}");
      return Err(LichessError);
    }

    let response_text_result = response_result.unwrap().text().await;

    if let Err(e) = response_text_result {
      warn!("Error reading the payload from POST request to Lichess: {e:?}");
      return Err(LichessError);
    }

    // debug!("Lichess post answer: {:?}", response_text_result);
    let json_value_result = serde_json::from_str(&response_text_result.unwrap());
    let json_object = match json_value_result {
      Ok(object) => object,
      Err(error) => {
        warn!("Error parsing JSON from the Lichess Response for API call {api_endpoint}. Error:{error}");
        return Ok(JsonValue::Null);
      },
    };

    debug!("Lichess post answer: {}", json_object);
    Ok(json_object)
  }

  /// Streams incoming events using an object and stream handler.
  /// Refer to https://lichess.org/api/stream/event
  ///
  /// JSON values received on the stream will be passed to the stream_handler
  /// function.
  ///
  /// ### Arguments
  ///
  /// * `object` Reference to the object invoking the stream handler (e.g. bot
  ///   reference)
  /// * `stream_handler` Function to invoke when data has been received on the
  ///   stream
  ///
  /// ### Returns
  ///
  /// Result indicating if we had error receiving/parsing the event stream.
  pub async fn stream_incoming_events<T>(&self, handler: &T) -> Result<(), LichessError>
    where T: EventStreamHandler
  {
    let response_result = self.get("stream/event").await;

    if let Err(e) = response_result {
      warn!("Error streaming events (GET) request to Lichess: {e:?}");
      return Err(LichessError);
    }

    let mut nd_json_buffer = helpers::NdJsonBuffer::default();

    let mut stream = response_result.unwrap().bytes_stream();
    while let Some(chunk_response) = stream.next().await {
      match chunk_response {
        Err(e) => {
          warn!("Error receiving event stream: {e:?}");
          return Err(LichessError);
        },
        Ok(chunk) => {
          let json_entries = nd_json_buffer.push(&chunk);

          for json_entry in json_entries {
            handler.event_stream_handler(json_entry);
          }
          // Sending 1 byte is usually just the keep-alive message
          if chunk.len() == 1 {
            debug!("Received keep-alive message for event stream");
          }
        },
      }
    }

    info!("Finished to stream events");
    Ok(())
  }

  pub async fn stream_incoming_events_with_callback<T>(&self,
                                                       handler: &'static T,
                                                       callback: fn(&'static T, JsonValue))
                                                       -> Result<(), LichessError> {
    let response_result = self.get("stream/event").await;

    if let Err(e) = response_result {
      warn!("Error streaming events (GET) request to Lichess: {e:?}");
      return Err(LichessError);
    }

    let mut nd_json_buffer = helpers::NdJsonBuffer::default();

    let mut stream = response_result.unwrap().bytes_stream();
    while let Some(chunk_response) = stream.next().await {
      match chunk_response {
        Err(e) => {
          warn!("Error receiving event stream: {e:?}");
          return Err(LichessError);
        },
        Ok(chunk) => {
          let json_entries = nd_json_buffer.push(&chunk);

          for json_entry in json_entries {
            callback(handler, json_entry);
          }
          // Sending 1 byte is usually just the keep-alive message
          if chunk.len() == 1 {
            debug!("Received keep-alive message for event stream");
          }
        },
      }
    }

    info!("Finished to stream events");
    Ok(())
  }

  /// Streams incoming game state events using a Game ID and stream handler.
  /// using https://lichess.org/api/bot/game/stream/{gameId}
  ///
  /// JSON values received on the stream will be passed to the stream_handler
  /// function.
  ///
  /// ### Arguments
  ///
  /// * `game_id` Game ID to stream
  /// * `stream_handler` Function to invoke when data has been received on the
  ///   stream
  ///
  /// ### Returns
  ///
  /// Result indicating if we had error receiving/parsing the event stream.
  pub async fn stream_game_state<T>(self, handler: &T, game_id: &str) -> Result<(), LichessError>
    where T: GameStreamHandler
  {
    info!("Requesting Lichess to stream Game ID {game_id}");
    let response_result = self.get(&format!("bot/game/stream/{game_id}")).await;

    if let Err(e) = response_result {
      warn!("Error issuing a GET request to Lichess: {e:?}");
      return Err(LichessError);
    }

    let mut nd_json_buffer = helpers::NdJsonBuffer::default();

    let mut stream = response_result.unwrap().bytes_stream();
    while let Some(chunk_response) = stream.next().await {
      match chunk_response {
        Err(e) => {
          warn!("Error receiving game stream for {game_id}: {e:?}");
          return Err(LichessError);
        },
        Ok(chunk) => {
          let json_entries = nd_json_buffer.push(&chunk);

          for json_entry in json_entries {
            handler.game_stream_handler(json_entry, String::from(game_id));
          }
          // Sending 1 byte is usually just the keep-alive message
          if chunk.len() == 1 {
            debug!("Received keep-alive message for Game State stream");
          }
        },
      }
    }

    info!("Finished to stream game events for game id {game_id}");
    Ok(())
  }

  /// Streams incoming game state events using a Game ID and callback function.
  /// using https://lichess.org/api/bot/game/stream/{gameId}
  ///
  /// JSON values received on the stream will be passed to the stream_handler
  /// function.
  ///
  /// ### Arguments
  ///
  /// * `game_id` Game ID to stream
  /// * `callback` Function to invoke when data has been received on the stream
  ///
  /// ### Returns
  ///
  /// Result indicating if we had error receiving/parsing the event stream.
  pub async fn stream_game_state_with_callback<T>(&self,
                                                  game_id: &str,
                                                  handler: &T,
                                                  callback: fn(&T, JsonValue, String))
                                                  -> Result<(), LichessError> {
    info!("Requesting Lichess to stream Game ID {game_id}");
    let response_result = self.get(&format!("bot/game/stream/{game_id}")).await;

    if let Err(e) = response_result {
      warn!("Error issuing a GET request to Lichess: {e:?}");
      return Err(LichessError);
    }

    let mut nd_json_buffer = helpers::NdJsonBuffer::default();

    let mut stream = response_result.unwrap().bytes_stream();
    while let Some(chunk_response) = stream.next().await {
      match chunk_response {
        Err(e) => {
          warn!("Error receiving game stream for {game_id}: {e:?}");
          return Err(LichessError);
        },
        Ok(chunk) => {
          let json_entries = nd_json_buffer.push(&chunk);

          for json_entry in json_entries {
            callback(handler, json_entry, String::from(game_id));
          }
          // Sending 1 byte is usually just the keep-alive message
          if chunk.len() == 1 {
            debug!("Received keep-alive message for Game State stream");
          }
        },
      }
    }

    info!("Finished to stream game events for game id {game_id}");
    Ok(())
  }
} // impl LichessApi

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_rate_limit_wait() {
    // One minute the first time, then doubling, up to 15 minutes
    assert_eq!(Duration::from_secs(60), rate_limit_wait(1, None));
    assert_eq!(Duration::from_secs(120), rate_limit_wait(2, None));
    assert_eq!(Duration::from_secs(240), rate_limit_wait(3, None));
    assert_eq!(Duration::from_secs(480), rate_limit_wait(4, None));
    assert_eq!(Duration::from_secs(900), rate_limit_wait(5, None));
    assert_eq!(Duration::from_secs(900), rate_limit_wait(100, None));

    // Lichess may ask for longer
    assert_eq!(Duration::from_secs(300), rate_limit_wait(1, Some(Duration::from_secs(300))));
    // But never shorter than our own wait
    assert_eq!(Duration::from_secs(120), rate_limit_wait(2, Some(Duration::from_secs(10))));
  }

  #[test]
  fn test_on_rate_limited() {
    let api = LichessApi::new("token");
    let clone = api.clone();

    // First 429: one minute
    let wait = api.on_rate_limited(None);
    assert!(wait <= Duration::from_secs(60) && wait > Duration::from_secs(59));

    // Parallel requests rate limited during the same period do not escalate
    // the wait, and the state is shared between clones
    let wait = clone.on_rate_limited(None);
    assert!(wait <= Duration::from_secs(60));
    assert_eq!(1, api.rate_limit.lock().unwrap().consecutive);

    // Rate limited again after the wait: wait longer
    api.rate_limit.lock().unwrap().until = Some(Instant::now());
    let wait = api.on_rate_limited(None);
    assert!(wait > Duration::from_secs(119));
    assert_eq!(2, clone.rate_limit.lock().unwrap().consecutive);
  }

  #[test]
  fn test_rate_limited_within() {
    let api = LichessApi::new("token");
    assert!(!api.rate_limited_within(Duration::from_secs(1800)));

    api.on_rate_limited(None);
    assert!(api.rate_limited_within(Duration::from_secs(1800)));

    // Wait is over, but the 429 is still recent
    api.rate_limit.lock().unwrap().until = Some(Instant::now());
    assert!(api.rate_limited_within(Duration::from_secs(1800)));
    assert!(!api.rate_limited_within(Duration::ZERO));
  }
}
