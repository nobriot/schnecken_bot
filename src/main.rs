// https://lichess.org/api

use log::*;
use serde_json::Value as JsonValue;

// Local modules
mod chess;
mod lichess;
mod user_commands;

const USER_NAME: &str = "schnecken_bot";

// Main function
fn main() {
  env_logger::init();
  let rt = tokio::runtime::Runtime::new().unwrap();

  match rt.block_on(main_loop()) {
    Ok(_) => info!("Exiting successfully."),
    Err(_) => error!("An error ocurred"),
  };
}

async fn main_loop() -> Result<(), ()> {
  info!("Starting the Lichess bot... ");
  info!("Watch it at: https://lichess.org/@/{USER_NAME}");

  // Check that the Token is okay:
  if lichess::get_api().token.len() == 0 {
    error!("Error reading the API token. Make sure that you have added a token file.");
    return Err(());
  }
  info!("Lichess API token loaded successfully");

  // Check for our favorite player
  display_player_propaganda("SchnellSchnecke").await;

  // Start checking what's our bot state
  let _ = display_account_info().await;

  loop {
    tokio::spawn(async { lichess::api::stream_incoming_events(&stream_event_handler).await });

    // Read command line inputs for ever, until we have to exit
    let mut exit_requested: bool = false;
    if let Err(_) = user_commands::read_user_commands(&mut exit_requested) {
      error!("Error reading user input");
    }
    if true == exit_requested {
      info!("Exiting the Lichess bot... ");
      break;
    }
  }

  // End the main loop.
  Ok(())
}

async fn stream_event_handler(json_value: JsonValue) -> Result<(), ()> {
  if json_value["type"].as_str().is_none() {
    error!("No type for incoming stream event.");
    return Err(());
  }

  match json_value["type"].as_str().unwrap() {
    "gameStart" => {
      info!("New game Started!");
      tokio::spawn(async move { on_new_game_started(json_value["game"].clone()).await });
      return Ok(());
    },
    "gameFinish" => {
      info!("Game finished! ");
    },
    "challenge" => {
      info!("Incoming challenge!");
      tokio::spawn(async move { on_incoming_challenge(json_value["challenge"].clone()).await });
    },
    "challengeCanceled" => {
      info!("Challenge cancelled ");
    },
    "challengeDeclined" => {
      info!("Challenge declined");
    },
    other => {
      // Ignore other events
      warn!("Received unknown streaming event: {}", other);
    },
  }
  Ok(())
}

async fn stream_game_state_handler(json_value: JsonValue, game_id: String) -> Result<(), ()> {
  info!("Incoming stream event for Game ID {game_id}");
  if json_value["type"].as_str().is_none() {
    error!("No type for incoming stream event.");
    return Err(());
  }

  match json_value["type"].as_str().unwrap() {
    "gameFull" => {
      info!("Full game state!");
      tokio::spawn(
        async move { play_on_game(&game_id.clone(), json_value["state"].clone()).await },
      );
    },
    "gameState" => {
      info!("Game state update received.");
      tokio::spawn(async move { play_on_game(&game_id.clone(), json_value.clone()).await });
    },
    "chatLine" => {
      info!("Incoming challenge!");
    },
    "opponentGone" => {
      info!("Opponent gone! We'll just claim victory now, you chicken!");
    },
    other => {
      // Ignore other events
      warn!("Received unknown streaming game state: {}", other);
    },
  }
  //debug!("JSON: {}", json_value);

  Ok(())
}

async fn on_new_game_started(json_value: JsonValue) {
  if json_value["gameId"].as_str().is_none() {
    return;
  }

  // Let's stream the game!
  tokio::spawn(async move {
    lichess::api::stream_game_state(
      json_value["gameId"].as_str().unwrap(),
      &stream_game_state_handler,
    )
    .await
  });
}

async fn on_incoming_challenge(json_value: JsonValue) {
  debug!("Incoming challenge JSON: {}", json_value);
  let challenger = json_value["challenger"]["name"]
    .as_str()
    .unwrap_or("Unknown challenger");
  let challenger_rating = json_value["challenger"]["rating"]
    .as_str()
    .unwrap_or("unknown rating");
  let variant = json_value["variant"]["key"]
    .as_str()
    .unwrap_or("Unknown variant");
  let challenge_id = json_value["id"].as_str().unwrap_or("UnknownID").to_owned();

  info!("{challenger} would like to play with us! Challenge {challenge_id}");
  info!("{} is rated {} ", challenger, challenger_rating);

  if variant != "standard" && variant != "chess960" {
    info!(
      "Ignoring challenge for variant {}. We play only standard and chess 960.",
      variant
    );

    // Declining gracefully
    tokio::spawn(async move {
      lichess::api::decline_challenge(&challenge_id, lichess::types::DECLINE_VARIANT).await
    });
    return;
  }

  // Else we just accept.
  tokio::spawn(async move { lichess::api::accept_challenge(&challenge_id).await });
}

async fn display_player_propaganda(username: &str) -> () {
  if lichess::api::is_online(username).await == true {
    info!(
      "{username} is online. You should check him out playing at https://lichess.org/@/{username}"
    );
  } else {
    info!("{username} is not online =(. Oh crappy day!");
  }
}

async fn display_account_info() -> Result<(), ()> {
  info!("Checking Account information...");
  let _account_json: JsonValue;
  if let Ok(json) = lichess::api::lichess_get("account").await {
    _account_json = json;
  } else {
    return Err(());
  }

  Ok(())
}

#[allow(dead_code)]
async fn get_ongoing_games() -> Result<JsonValue, ()> {
  let json_response: JsonValue;
  if let Ok(json) = lichess::api::lichess_get("account/playing").await {
    json_response = json;
  } else {
    return Err(());
  }

  Ok(json_response)
}

async fn play_on_game(game_id: &str, game_state: JsonValue) -> Result<(), ()> {
  // Double check that the game is still alive and it's our turn
  let (game_is_ongoing, is_my_turn, time_remaining) = lichess::api::game_is_ongoing(game_id).await;
  if false == game_is_ongoing {
    return Ok(());
  }
  if false == is_my_turn {
    info!("Not our turn. Now relying on the stream to tell us when to play for game {game_id}");
    return Ok(());
  }

  info!("Trying to find a move for game id {game_id}");

  let moves = game_state["moves"].as_str().unwrap_or("Unknown move list");
  let increment_ms = game_state["winc"].as_f64().unwrap_or(0.0);
  let mut game_state = chess::model::game_state::GameState::default();
  game_state.apply_move_list(moves);

  let suggested_time_ms = (time_remaining as f64 / 90.0) * 1000.0 + increment_ms;

  if let Ok(chess_move) = &chess::engine::core::play_move(&game_state, suggested_time_ms as u64) {
    info!("Playing move {} for game id {}", chess_move, game_id);
    lichess::api::make_move(game_id, chess_move, false).await;
  } else {
    warn!("Can't find a move... Let's offer draw");
    lichess::api::make_move(game_id, "", true).await;
  }

  Ok(())
}
