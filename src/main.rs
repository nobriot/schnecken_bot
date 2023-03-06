// https://lichess.org/api

use log::*;
use serde_json::Value as JsonValue;
use std::{thread, time::Duration};

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

        // info!("Checking for incoming challenges and/or ongoing games");
        // let mut playing_a_game = play_games().await?;
        // while playing_a_game == false {
        //     // Take it easy between each check
        //     thread::sleep(Duration::from_millis(4000));
        //     // Check for challenges, accept them, then try to play games.
        //     let _ = check_for_challenges().await;
        //     playing_a_game = play_games().await?;
        // }

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
        }
        "gameFinish" => {
            info!("Game finished! ");
        }
        "challenge" => {
            info!("Incoming challenge!");
        }
        "challengeCanceled" => {
            info!("Challenge cancelled ");
        }
        "challengeDeclined" => {
            info!("Challenge declined");
        }
        other => {
            // Ignore other events
            warn!("Received unknown streaming event: {}", other);
        }
    }
    Ok(())
}

async fn stream_game_state_handler(json_value: JsonValue) -> Result<(), ()> {
    if json_value["type"].as_str().is_none() {
        error!("No type for incoming stream event.");
        return Err(());
    }

    match json_value["type"].as_str().unwrap() {
        "gameFull" => {
            info!("Full game state!");
        }
        "gameState" => {
            info!("Game finished! ");
        }
        "chatLine" => {
            info!("Incoming challenge!");
        }
        "opponentGone" => {
            info!("Challenge cancelled ");
        }
        other => {
            // Ignore other events
            warn!("Received unknown streaming game state: {}", other);
        }
    }
    debug!("JSON: {}", json_value);

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

async fn display_player_propaganda(username: &str) -> () {
    if lichess::api::is_online(username).await == true {
        info!("{username} is online. You should check him out playing at https://lichess.org/@/{username}");
    } else {
        info!("{username} is not online =(. Oh crappy day!");
    }
}

async fn display_account_info() -> Result<(), ()> {
    info!("Checking Account information...");
    let account_json: JsonValue;
    if let Ok(json) = lichess::api::lichess_get("account").await {
        account_json = json;
    } else {
        return Err(());
    }

    debug!("JSON response: {account_json}");
    Ok(())
}

async fn check_for_challenges() -> Result<(), ()> {
    let challenges_json: JsonValue = lichess::api::lichess_get("challenge").await?;

    debug!("JSON response: {challenges_json}");

    if challenges_json["in"].as_array().is_none() {
        warn!("Cannot find the 'in' object in challenges");
        return Ok(());
    }

    let challenges: Vec<JsonValue> = challenges_json["in"].as_array().unwrap().to_owned();

    if challenges.len() == 0 {
        debug!("No new challenger. We are so lonely :'(");
        return Ok(());
    }

    info!(
        "Yay! We have a challenger! Accepting challenge ID {}",
        challenges[0]["id"]
    );

    if let JsonValue::String(challenge_id) = &challenges[0]["id"] {
        let _ = accept_challenge(&challenge_id as &str).await?;
    }

    Ok(())
}

async fn accept_challenge(challenge_id: &str) -> Result<(), ()> {
    let api_endpoint: String = String::from("challenge/") + challenge_id + "/accept";
    let json_response: JsonValue;
    if let Ok(json) = lichess::api::lichess_post(&api_endpoint, "").await {
        json_response = json;
    } else {
        return Err(());
    }

    debug!("accept_challenge JSON response: {json_response}");
    Ok(())
}

async fn play_games() -> Result<bool, ()> {
    let games_json = get_ongoing_games().await?;

    if games_json["nowPlaying"].as_array().is_none() {
        warn!("Cannot find the 'nowPlaying' array in ongoing games");
        return Ok(false);
    }

    let ongoing_games: Vec<JsonValue> = games_json["nowPlaying"].as_array().unwrap().to_owned();
    let mut playing_a_game: bool = false;

    for game in ongoing_games {
        if let JsonValue::String(game_id) = &game["gameId"] {
            info!("Picking up game {:?}", game_id);
            let _ = play_game(&game_id).await;
        }
        //error!("Should spawn a thread and play now");
        playing_a_game = true;
    }

    Ok(playing_a_game)
}

async fn get_ongoing_games() -> Result<JsonValue, ()> {
    let json_response: JsonValue;
    if let Ok(json) = lichess::api::lichess_get("account/playing").await {
        json_response = json;
    } else {
        return Err(());
    }

    debug!("get_ongoing_games JSON response: {json_response}");
    Ok(json_response)
}

async fn play_game(game_id: &str) -> Result<(), ()> {
    info!("Anouncing ourselves in the chat for game {:?}", game_id);
    lichess::api::write_in_chat(game_id, "I am ready! Gimme all you've got!").await;

    info!("Streaming game {:?}", game_id);
    lichess::api::stream_game(game_id).await;
    loop {
        // Here we let the streaming do the work, react on updates from the server.
    }

    while true == lichess::api::game_is_ongoing(game_id).await {
        // Wait for our turn
        while false == lichess::api::is_my_turn(game_id).await {
            thread::sleep(Duration::from_millis(4000));
        }
        info!("It's our turn for game {}", game_id);

        // Try to make a move
        let game_fen = lichess::api::get_game_fen(game_id).await;
        if let Ok(chess_move) = &chess::engine::play_move(&game_fen) {
            info!("Playing move {} for game id {}", chess_move, game_id);
            lichess::api::make_move(game_id, chess_move, false).await;
        } else {
            info!("Can't find a move... Let's offer draw");
            lichess::api::make_move(game_id, "", true).await;
        }
    }

    info!("Nothing else to do?");
    Ok(())
}
