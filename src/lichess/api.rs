use futures_util::StreamExt;
use log::*;
use reqwest;
use serde_json::Value as JsonValue;
use urlencoding::encode;

// From the same module:
use crate::lichess;
use crate::lichess::helpers;

// Constants.
const API_BASE_URL: &'static str = "https://lichess.org/api/";



pub async fn lichess_get(api_endpoint: &str) -> Result<JsonValue, ()> {
    let response_result = lichess::get_api()
        .client
        .get(format!("{}{}", API_BASE_URL, api_endpoint))
        .header(
            "Authorization",
            format!("Bearer {}", lichess::get_api().token),
        )
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

pub async fn lichess_get_stream(
    api_endpoint: &str,
    stream_handler: &dyn Fn(JsonValue) -> Result<(), ()>,
) -> Result<(), ()> {
    let response_result = lichess::get_api()
        .client
        .get(format!("{}{}", API_BASE_URL, api_endpoint))
        .header(
            "Authorization",
            format!("Bearer {}", lichess::get_api().token),
        )
        .header("Accept", "application/x-ndjson")
        .send()
        .await;

    if let Err(error) = response_result {
        warn!("Error issuing a Get request to Lichess {}", error);
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
    let response_result = lichess::get_api()
        .client
        .post(format!("{}{}", API_BASE_URL, api_endpoint))
        .header(
            "Authorization",
            format!("Bearer {}", lichess::get_api().token),
        )
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

pub async fn is_online(user_id: &str) -> bool {
    let endpoint: String = String::from(format!("users/status?ids={}", user_id));
    let result = lichess_get(&endpoint).await;

    if let Err(error) = result {
        warn!("Error parsing the result for is_online API");
        return false;
    }

    let json_object: JsonValue = result.unwrap();
    return json_object[0]["online"].as_bool().unwrap_or(false);
}

pub async fn write_in_chat(game_id: &str, message: &str) -> () {
    let endpoint: String = String::from(format!("bot/game/{game_id}/chat"));
    let body: String = String::from(format!("room=player&text={}", encode(message)));
    debug!("Body : {}", body);

    let result = lichess_post(&endpoint, &body).await;

    if let Err(error) = result {
        warn!("Error sending message to game id {}", game_id);
    }

    return;
}

/// https://lichess.org/api/bot/game/stream/%7BgameId%7D
pub async fn stream_game(game_id: &str) -> () {
    let api_endpoint: String = String::from(format!("bot/game/stream/{}", game_id));
    let _ = lichess_get_stream(&api_endpoint, &on_game_state_changed).await;
}

pub fn on_game_state_changed(json_value: JsonValue) -> Result<(), ()> {
    info!("Game state changed called!");
    info!("Json: {}", json_value);

    Ok(())
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
    if let Ok(json) = lichess_post(&api_endpoint, "").await {
        json_response = json;
    } else {
        return false;
    }

    if json_response["ok"].as_bool().is_none() {
        return false;
    }

    return json_response["ok"].as_bool().unwrap();
}

pub async fn is_my_turn(game_id: &str) -> bool {
    //https://lichess.org/api/account/playing

    let json_response: JsonValue;
    if let Ok(json) = lichess_get("account/playing").await {
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

pub async fn get_game_fen(game_id: &str) -> String {
    //https://lichess.org/api/account/playing

    let mut game_fen: String = String::from("");
    let json_response: JsonValue;
    if let Ok(json) = lichess_get("account/playing").await {
        json_response = json;
    } else {
        return game_fen;
    }

    if json_response["nowPlaying"].as_array().is_none() {
        warn!("Cannot find the 'nowPlaying' array in ongoing games");
        return game_fen;
    }

    let json_game_array = json_response["nowPlaying"].as_array().unwrap();

    for json_game in json_game_array {
        if let JsonValue::String(json_game_id) = &json_game["gameId"] {
            if json_game_id.eq(game_id) {
                game_fen = String::from(json_game["fen"].as_str().unwrap_or(""));
                return game_fen;
            }
        }
    }

    return game_fen;
}

pub async fn game_is_ongoing(game_id: &str) -> bool {
    //https://lichess.org/api/account/playing

    let json_response: JsonValue;
    if let Ok(json) = lichess_get("account/playing").await {
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
