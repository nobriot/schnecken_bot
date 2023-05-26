// Submodules
pub mod account;
pub mod challenges;
pub mod game;
pub mod users;

// Other crates
use lazy_static::lazy_static;
use log::*;
use std::fs;

use futures::Future;
use futures_util::StreamExt;
use log::*;
use reqwest;
use serde_json::Value as JsonValue;
use std::time::Instant;
use tokio::time::*;
use urlencoding::encode;

use crate::chess::model::game_state::START_POSITION_FEN;
// From the same module:
use crate::lichess;
use crate::lichess::helpers;

// Constants
const API_BASE_URL: &'static str = "https://lichess.org/api/";
const API_TOKEN_FILE_NAME: &str = "/assets/lichess_api_token.txt";
const LICHESS_PLAYERS_FILE_NAME: &str = "/assets/players_we_like.txt";

// Type definitions
#[derive(Debug, Clone)]
pub struct LichessApi {
  pub client: reqwest::Client,
  pub token: String,
}

// Our one time init of the API data:
lazy_static! {
  static ref LICHESS_API_CONFIG: LichessApi = LichessApi {
    client: reqwest::Client::new(),
    token: fs::read_to_string(String::from(env!("CARGO_MANIFEST_DIR")) + API_TOKEN_FILE_NAME)
      .unwrap(),
  };
}

pub fn get_api() -> &'static LichessApi {
  &LICHESS_API_CONFIG
}

/// Checks if any of the players we like is online and sends a challenge.
pub async fn play() {
  let player_list =
    fs::read_to_string(String::from(env!("CARGO_MANIFEST_DIR")) + LICHESS_PLAYERS_FILE_NAME)
      .unwrap();
  //let parameters = serde_json::json!({ "rated": true, "clock" : {"limit":180,"increment":0}, "color":"random", "variant":"standard" });
  let players = player_list.lines();

  for username in players {
    if is_online(username).await == true {
      info!("{username} is online. Sending a challenge!");
      if let Err(()) = challenges::send_challenge(username).await {
        info!("Error sending a challenge to {username}");
        continue;
      }
      break;
    }
  }
}

////////////////////////////////////////////////////////////////////////////////
// Private functions
////////////////////////////////////////////////////////////////////////////////
async fn get(api_endpoint: &str) -> Result<reqwest::Response, reqwest::Error> {
  debug!("Lichess GET request at {}{}", API_BASE_URL, api_endpoint);
  get_api()
    .client
    .get(format!("{}{}", API_BASE_URL, api_endpoint))
    .header("Authorization", format!("Bearer {}", get_api().token))
    .header("Accept", "application/x-ndjson")
    .send()
    .await
}

async fn post(api_endpoint: &str, body: &str) -> Result<reqwest::Response, reqwest::Error> {
  debug!("Lichess POST request at {}{}", API_BASE_URL, api_endpoint);
  get_api()
    .client
    .post(format!("{}{}", API_BASE_URL, api_endpoint))
    .header("Authorization", format!("Bearer {}", get_api().token))
    .header("Accept", "application/x-ndjson")
    .header("Content-Type", "application/x-www-form-urlencoded")
    .body(format!("{}", body))
    .send()
    .await
}

////////////////////////////////////////////////////////////////////////////////
// Public functions
////////////////////////////////////////////////////////////////////////////////
pub async fn lichess_get(api_endpoint: &str) -> Result<JsonValue, ()> {
  debug!("Lichess get request at {}{}", API_BASE_URL, api_endpoint);
  let response_result = get(api_endpoint).await;

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
  let json_object;

  match json_value_result {
    Ok(object) => json_object = object,
    Err(error) => {
      warn!(
        "Error parsing JSON from the Lichess Response for API call {api_endpoint}. Error:{error}"
      );
      return Err(());
    },
  }

  debug!("Lichess get answer: {}", json_object);
  Ok(json_object)
}

#[allow(dead_code)]
pub async fn lichess_get_stream(
  api_endpoint: &str,
  stream_handler: &dyn Fn(JsonValue) -> Result<(), ()>,
) -> Result<(), ()> {
  let response_result = get(api_endpoint).await;

  if let Err(error) = response_result {
    warn!("Error issuing a Get Stream request to Lichess {}", error);
    return Err(());
  }

  let stream = response_result.unwrap().bytes_stream();
  stream
    .for_each(|chunk_response| {
      if let Err(e) = chunk_response {
        info!("Error receiving stream? {}", e);
        return futures::future::ready(());
      }

      let chunk = chunk_response.unwrap();
      let string_value: String = String::from_utf8_lossy(&chunk).to_string();
      let json_entries = helpers::parse_string_to_nd_json(&string_value);

      for json_entry in json_entries {
        if let Err(_) = stream_handler(json_entry) {
          error!("Error handling JSON value");
        }
      }

      info!("Received {} bytes", chunk.len());
      info!("Received data: {}", string_value);
      futures::future::ready(())
    })
    .await;

  // Set up event stream
  info!("We're done with Streaming : ");

  //while let Some(item) = stream.poll_next().await {
  //   info!("Chunk: {:?}", item?);
  // }

  Ok(())
}

pub async fn lichess_post(api_endpoint: &str, body: &str) -> Result<JsonValue, ()> {
  let response_result = post(api_endpoint, body).await;
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
  let json_object;

  match json_value_result {
    Ok(object) => json_object = object,
    Err(error) => {
      warn!(
        "Error parsing JSON from the Lichess Response for API call {api_endpoint}. Error:{error}"
      );
      return Ok(JsonValue::Null);
    },
  }

  debug!("Lichess post answer: {}", json_object);
  Ok(json_object)
}

// Starts listenings to incoming events and sends the JSON data to the incoming
// event handler
// See https://lichess.org/api/stream/event
pub async fn stream_incoming_events<Func, Fut>(stream_handler: Func) -> Result<(), ()>
where
  Func: Fn(serde_json::Value) -> Fut,
  Fut: Future<Output = Result<(), ()>>,
{
  let response_result = get("stream/event").await;

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
        if let Err(_) = stream_handler(json_entry).await {
          error!("Error handling JSON value");
        }
      }
      // Sending 1 byte is usually just the keep-alive message
      if chunk.len() == 1 {
        info!("Received keep-alive message for event stream");
      }

      //futures::future::ready(())
      ()
    })
    .await;

  Ok(())
}

// Starts listenings to incoming game state updates and sends the JSON data
// to the incoming event handler
// See https://lichess.org/api/bot/game/stream/{gameId}
pub async fn stream_game_state<Func, Fut>(game_id: &str, stream_handler: Func) -> Result<(), ()>
where
  Func: Fn(serde_json::Value, String) -> Fut,
  Fut: Future<Output = Result<(), ()>>,
{
  info!("Requesting Lichess to stream Game ID {game_id}");
  let response_result = get(&format!("bot/game/stream/{game_id}")).await;

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
        if let Err(_) = stream_handler(json_entry, String::from(game_id)).await {
          error!("Error handling JSON value");
        }
      }
      // Sending 1 byte is usually just the keep-alive message
      if chunk.len() == 1 {
        info!("Received keep-alive message for Game State stream");
      }

      //futures::future::ready(())
      ()
    })
    .await;

  info!("Finished to stream game events for game id {game_id}");
  Ok(())
}

#[allow(dead_code)]
pub async fn abort_game(game_id: &str) -> Result<(), ()> {
  let api_endpoint: String = String::from(format!("bot/game/{game_id}/abort"));
  let _json_response: JsonValue;
  if let Ok(json) = lichess_post(&api_endpoint, "").await {
    _json_response = json;
  } else {
    return Err(());
  }

  Ok(())
}

#[allow(dead_code)]
pub async fn resign_game(game_id: &str) -> Result<(), ()> {
  let api_endpoint: String = String::from(format!("bot/game/{game_id}/resign"));
  let _json_response: JsonValue;
  if let Ok(json) = lichess_post(&api_endpoint, "").await {
    _json_response = json;
  } else {
    return Err(());
  }

  Ok(())
}

pub async fn is_online(user_id: &str) -> bool {
  let endpoint: String = String::from(format!("users/status?ids={}", user_id));
  let result = lichess_get(&endpoint).await;

  if let Err(error) = result {
    warn!("Error parsing the result for is_online API. {:#?}", error);
    return false;
  }

  let json_object: JsonValue = result.unwrap();
  return json_object[0]["online"].as_bool().unwrap_or(false);
}

#[allow(dead_code)]
pub async fn write_in_chat(game_id: &str, message: &str) -> () {
  let endpoint: String = String::from(format!("bot/game/{game_id}/chat"));
  let body: String = String::from(format!("room=player&text={}", encode(message)));

  let result = lichess_post(&endpoint, &body).await;

  if let Err(error) = result {
    warn!(
      "Error sending message to game id {} - Error: {:#?}",
      game_id, error
    );
  }

  return;
}

pub async fn make_move(game_id: &str, chess_move: &str, offer_draw: bool) -> bool {
  info!(
    "Trying chess move {} on game id {} - Draw offer: {}",
    chess_move, game_id, offer_draw
  );
  let api_endpoint: String = String::from(format!(
    "bot/game/{game_id}/move/{chess_move}?offeringDraw={offer_draw}"
  ));

  let json_response: JsonValue;
  let mut retries = 0;

  loop {
    retries += 1;
    let move_result = lichess_post(&api_endpoint, "").await;

    if move_result.is_ok() {
      json_response = move_result.unwrap();
      break;
    }

    if retries > 10 {
      error!("Something is not working with making moves");
      return false;
    }
  }

  if json_response["ok"].as_bool().is_none() {
    warn!("Lichess refused our move! :'( - We're so bad");
    /*let _ = write_in_chat(
      game_id,
      format!(
        "I just tried to play an invalid move! ({}) Embarassing for my developer! :'(",
        chess_move
      )
      .as_str(),
    )
    .await;*/
    return false;
  }

  return json_response["ok"].as_bool().unwrap();
}

#[allow(dead_code)]
pub async fn claim_victory(game_id: &str) -> Result<(), ()> {
  let api_endpoint: String = String::from(format!("board/game/{game_id}/claim-victory"));
  let _json_response: JsonValue;
  if let Ok(json) = lichess_post(&api_endpoint, "").await {
    _json_response = json;
  } else {
    return Err(());
  }

  Ok(())
}

// Returns if a game is ongoing and if it is our turn, if it is our turn, how many seconds we have left.
pub async fn game_is_ongoing(game_id: &str) -> (bool, bool, u64) {
  //https://lichess.org/api/account/playing

  let json_response: JsonValue;
  if let Ok(json) = lichess_get("account/playing").await {
    json_response = json;
  } else {
    warn!("Error in the response");
    return (false, false, 0);
  }

  if json_response["nowPlaying"].as_array().is_none() {
    warn!("Cannot find the 'nowPlaying' array in ongoing games");
    return (false, false, 0);
  }

  let json_game_array = json_response["nowPlaying"].as_array().unwrap();

  for json_game in json_game_array {
    let current_game_id = json_game["gameId"].as_str().unwrap();
    if current_game_id == game_id {
      let is_my_turn = json_game["isMyTurn"].as_bool().unwrap_or(true);
      let seconds_left = json_game["secondsLeft"].as_u64().unwrap_or(20);
      return (true, is_my_turn, seconds_left);
    }
  }

  return (false, false, 0);
}

pub async fn get_game_fen(game_id: &str) -> String {
  //https://lichess.org/api/account/playing

  let json_response: JsonValue;
  if let Ok(json) = lichess_get("account/playing").await {
    json_response = json;
  } else {
    warn!("Error in the response");
    return String::from(START_POSITION_FEN);
  }

  if json_response["nowPlaying"].as_array().is_none() {
    warn!("Cannot find the 'nowPlaying' array in ongoing games");
    return String::from(START_POSITION_FEN);
  }

  let json_game_array = json_response["nowPlaying"].as_array().unwrap();

  for json_game in json_game_array {
    let current_game_id = json_game["gameId"].as_str().unwrap();
    if current_game_id == game_id {
      let fen = json_game["fen"]
        .as_str()
        .unwrap_or(START_POSITION_FEN)
        .to_owned();
      return fen;
    }
  }

  return String::from(START_POSITION_FEN);
}

/// Checks when was last time we played a game, and if more than the indicated
/// number of seconds, we go and challenge somebody.
///
/// # Arguments
///
/// * `timeout` - The longest time of seconds to wait without playing before throwing challenges.
pub async fn send_challenges_with_interval(timeout: u64) {
  let mut last_play = Instant::now();

  loop {
    let json_response: JsonValue;
    if let Ok(json) = lichess_get("account/playing").await {
      json_response = json;
    } else {
      continue;
    }

    if json_response["nowPlaying"].as_array().is_none() {
      debug!("Cannot find the 'nowPlaying' array in ongoing games, considering that we are not playing now.");
    } else {
      if json_response["nowPlaying"].as_array().unwrap().len() > 0 {
        last_play = Instant::now();
      }
    }

    // Throw a challenge somewhere
    if Instant::now() > last_play + Duration::from_secs(timeout) {
      info!("We did not play in a while. Throwing a challenge.");
      tokio::spawn(async { lichess::api::play().await });
    }

    tokio::time::sleep(Duration::from_millis(timeout * 1000)).await;
  }
}
