use log::*;
use reqwest;
use serde_json::json;
use serde_json::Value as JsonValue;
use std::env;
use urlencoding::encode;
use websocket::{ClientBuilder, OwnedMessage};

// Constants.
const API_BASE_URL: &'static str = "https://lichess.org/api/";

// Type definitions
#[derive(Debug, Clone)]
pub struct LichessApi {
    pub client: reqwest::Client,
    pub token: String,
}

pub async fn lichess_get(api: &LichessApi, api_endpoint: &str) -> Result<JsonValue, ()> {
    let response_result = api
        .client
        .get(format!("{}{}", API_BASE_URL, api_endpoint))
        .header("Authorization", format!("Bearer {}", api.token))
        .header("Accept", "application/x-ndjson")
        .send()
        .await;

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
            warn!("Error parsing JSON from the Lichess Response for API call {api_endpoint}. Error:{error}");
            return Err(());
        }
    }

    debug!("Lichess get answer: {}", json_object);
    Ok(json_object)
}

pub async fn lichess_post(
    api: &LichessApi,
    api_endpoint: &str,
    body: &str,
) -> Result<JsonValue, ()> {
    let response_result = api
        .client
        .post(format!("{}{}", API_BASE_URL, api_endpoint))
        .header("Authorization", format!("Bearer {}", api.token))
        .header("Accept", "application/x-ndjson")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(format!("{}", body))
        .send()
        .await;

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
            warn!("Error parsing JSON from the Lichess Response for API call {api_endpoint}. Error:{error}");
            return Ok(JsonValue::Null);
        }
    }

    debug!("Lichess post answer: {}", json_object);
    Ok(json_object)
}

pub async fn is_online(api: &LichessApi, user_id: &str) -> bool {
    let endpoint: String = String::from(format!("users/status?ids={}", user_id));
    let result = lichess_get(api, &endpoint).await;

    if let Err(error) = result {
        warn!("Error parsing the result for is_online API");
        return false;
    }

    let json_object: JsonValue = result.unwrap();
    return json_object[0]["online"].as_bool().unwrap_or(false);
}

pub async fn write_in_chat(api: &LichessApi, game_id: &str, message: &str) -> () {
    let endpoint: String = String::from(format!("bot/game/{game_id}/chat"));
    let body: String = String::from(format!("room=player&text={}", encode(message)));
    debug!("Body : {}", body);

    let result = lichess_post(api, &endpoint, &body).await;

    if let Err(error) = result {
        warn!("Error sending message to game id {}", game_id);
    }

    return;
}

/// https://lichess.org/api/bot/game/stream/%7BgameId%7D
pub fn stream_game(api: &LichessApi, game_id: &str) -> () {
    let url: String = String::from(format!("{}bot/game/stream/{}", API_BASE_URL, game_id));

    let client = reqwest::blocking::Client::builder()
        .user_agent(reqwest::header::HeaderValue::from_static(
            "schnecken_bot/1.0",
        ))
        .build()
        .unwrap();

    let response_result = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api.token))
        .header("Accept", "application/x-ndjson")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send();

    if let Err(error) = response_result {
        warn!("Error streaming game id {} - {:?}", game_id, error);
        return;
    }

    let mut response = response_result.unwrap();
    for line in response.text().unwrap_or_default().lines() {
        info!("Read line: {}", line);
    }
}

pub fn play_game(api: &LichessApi, game_id: &str) {
    let mut client = ClientBuilder::new("wss://lichess.org/api/stream/event")
        .unwrap()
        .add_protocol("irc")
        .connect_insecure()
        .unwrap();

    let mut message = OwnedMessage::Text(json!({"t": "pong"}).to_string());
    client.send_message(&message).unwrap();

    message = OwnedMessage::Text(json!({"t": "hello", "d": {"token": api.token}}).to_string());
    client.send_message(&message).unwrap();

    for message in client.incoming_messages() {
        let message = message.unwrap();
        match message {
            OwnedMessage::Text(message) => {
                let event: serde_json::Value = serde_json::from_str(&message).unwrap();
                if let Some(event_type) = event.get("type").and_then(|t| t.as_str()) {
                    match event_type {
                        "gameFull" => {
                            // Game started or game information updated
                            let game_id = event["id"].as_str().unwrap();
                            let white = event["white"]["name"].as_str().unwrap();
                            let black = event["black"]["name"].as_str().unwrap();
                            info!("Game {}: {} (white) vs {} (black)", game_id, white, black);
                        }
                        "gameState" => {
                            // Move played
                            let game_id = event["id"].as_str().unwrap();
                            let moves = event["moves"].as_str().unwrap();
                            info!("Move played in game {}: {}", game_id, moves);
                        }
                        "chatLine" => {
                            // Chat message sent in game
                            let username = event["username"].as_str().unwrap();
                            let text = event["text"].as_str().unwrap();
                            info!("{}: {}", username, text);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

pub async fn make_move(
    api: &LichessApi,
    game_id: &str,
    chess_move: &str,
    offer_draw: bool,
) -> bool {
    info!(
        "Trying chess move {} on game id {} - Draw offer: {}",
        chess_move, game_id, offer_draw
    );
    let api_endpoint: String = String::from(format!(
        "bot/game/{game_id}/move/{chess_move}?offeringDraw={offer_draw}"
    ));

    let json_response: JsonValue;
    if let Ok(json) = lichess_post(&api, &api_endpoint, "").await {
        json_response = json;
    } else {
        return false;
    }

    if json_response["ok"].as_bool().is_none() {
        return false;
    }

    return json_response["ok"].as_bool().unwrap();
}

pub async fn is_my_turn(api: &LichessApi, game_id: &str) -> bool {
    //https://lichess.org/api/account/playing

    let json_response: JsonValue;
    if let Ok(json) = lichess_get(&api, "account/playing").await {
        json_response = json;
    } else {
        return false;
    }

    if json_response["nowPlaying"].as_array().is_none() {
        warn!("Cannot find the 'nowPlaying' array in ongoing games");
        return false;
    }

    let json_game_array = json_response["nowPlaying"].as_array().unwrap();

    for json_game in json_game_array {
        if let JsonValue::String(json_game_id) = &json_game["gameId"] {
            if json_game_id.eq(game_id) {
                return json_game["isMyTurn"].as_bool().unwrap_or(false);
            }
        }
    }

    return false;
}

pub async fn game_is_ongoing(api: &LichessApi, game_id: &str) -> bool {
    //https://lichess.org/api/account/playing

    let json_response: JsonValue;
    if let Ok(json) = lichess_get(&api, "account/playing").await {
        json_response = json;
    } else {
        return false;
    }

    if json_response["nowPlaying"].as_array().is_none() {
        warn!("Cannot find the 'nowPlaying' array in ongoing games");
        return false;
    }

    let json_game_array = json_response["nowPlaying"].as_array().unwrap();

    for json_game in json_game_array {
        if let JsonValue::String(json_game_id) = &json_game["gameId"] {
            if json_game_id.eq(game_id) {
                return true;
            }
        }
    }

    return false;
}
