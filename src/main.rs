// https://lichess.org/api

use log::*;
use reqwest;
use serde_json::Value as JsonValue;
use std::{fs, thread, time::Duration};

// Local modules
mod chess;
mod lichess;
mod user_commands;

const USER_NAME: &str = "schnecken_bot";
const API_TOKEN_FILE_NAME: &str = "/assets/lichess_api_token.txt";

// Main function
fn main() {
    env_logger::init();
    let mut rt = tokio::runtime::Runtime::new().unwrap();

    match rt.block_on(main_loop()) {
        Ok(_) => info!("Exiting successfully."),
        Err(_) => error!("An error ocurred"),
    };
}

async fn main_loop() -> Result<(), ()> {
    info!("Starting the Lichess bot... ");
    info!("Watch it at: https://lichess.org/@/{USER_NAME}");

    let api_token_file_path = String::from(env!("CARGO_MANIFEST_DIR")) + API_TOKEN_FILE_NAME;
    let lichess_api: lichess::api::LichessApi = lichess::api::LichessApi {
        client: reqwest::Client::new(),
        token: fs::read_to_string(api_token_file_path).unwrap(),
    };

    // Check that the Token is okay:
    if lichess_api.token.len() == 0 {
        error!("Error reading the API token. Make sure that you have added a token file.");
        return Err(());
    }
    info!("Lichess API token loaded successfully");

    // Check for our favorite player
    display_player_propaganda(&lichess_api).await;

    // Start checking what's our bot state
    let _ = display_account_info(&lichess_api).await;

    info!("Checking for incoming challenges and/or ongoing games");
    let mut playing_a_game = play_games(&lichess_api).await?;
    while playing_a_game == false {
        // Take it easy between each check
        thread::sleep(Duration::from_millis(4000));
        // Check for challenges, accept them, then try to play games.
        let _ = check_for_challenges(&lichess_api).await;
        playing_a_game = play_games(&lichess_api).await?;
    }

    // Read command line inputs for ever, until we have to exit
    let mut exit_requested: bool = false;
    loop {
        if let Err(_) = user_commands::read_user_commands(&mut exit_requested) {
            error!("Error reading user input");
        }
        if true == exit_requested {
            info!("Exiting the Lichess bot... ");
            break;
        }
        // Print a prompt
        print!(">");
        //io::stdout().flush().unwrap();
    }

    // End the main loop.
    Ok(())
}

async fn display_player_propaganda(lichess_api: &lichess::api::LichessApi) -> () {
    if lichess::api::is_online(&lichess_api, "SchnellSchnecke").await == true {
        info!("SchnellSchnecke is online. You should check him out playing at https://lichess.org/@/SchnellSchnecke");
    } else {
        info!("SchnellSchnecke is not online =(. Oh crappy day!");
    }
}

async fn display_account_info(lichess_api: &lichess::api::LichessApi) -> Result<(), ()> {
    info!("Checking Account information...");
    let account_json: JsonValue;
    if let Ok(json) = lichess::api::lichess_get(&lichess_api, "account").await {
        account_json = json;
    } else {
        return Err(());
    }

    debug!("JSON response: {account_json}");

    Ok(())
}

async fn check_for_challenges(lichess_api: &lichess::api::LichessApi) -> Result<(), ()> {
    let challenges_json: JsonValue = lichess::api::lichess_get(&lichess_api, "challenge").await?;

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
        let _ = accept_challenge(lichess_api, &challenge_id as &str).await?;
    }

    Ok(())
}

async fn accept_challenge(
    lichess_api: &lichess::api::LichessApi,
    challenge_id: &str,
) -> Result<(), ()> {
    let api_endpoint: String = String::from("challenge/") + challenge_id + "/accept";
    let json_response: JsonValue;
    if let Ok(json) = lichess::api::lichess_post(&lichess_api, &api_endpoint, "").await {
        json_response = json;
    } else {
        return Err(());
    }

    debug!("accept_challenge JSON response: {json_response}");
    Ok(())
}

async fn play_games(lichess_api: &lichess::api::LichessApi) -> Result<bool, ()> {
    let games_json = get_ongoing_games(lichess_api).await?;

    if games_json["nowPlaying"].as_array().is_none() {
        warn!("Cannot find the 'nowPlaying' array in ongoing games");
        return Ok(false);
    }

    let ongoing_games: Vec<JsonValue> = games_json["nowPlaying"].as_array().unwrap().to_owned();
    let mut playing_a_game: bool = false;

    for game in ongoing_games {
        if let JsonValue::String(game_id) = &game["gameId"] {
            info!("Picking up game {:?}", game_id);
            let _ = play_game(lichess_api, &game_id).await;
        }
        //error!("Should spawn a thread and play now");
        playing_a_game = true;
    }

    Ok(playing_a_game)
}

async fn get_ongoing_games(lichess_api: &lichess::api::LichessApi) -> Result<JsonValue, ()> {
    let json_response: JsonValue;
    if let Ok(json) = lichess::api::lichess_get(&lichess_api, "account/playing").await {
        json_response = json;
    } else {
        return Err(());
    }

    debug!("get_ongoing_games JSON response: {json_response}");
    Ok(json_response)
}

async fn play_game(api: &lichess::api::LichessApi, game_id: &str) -> Result<(), ()> {
    info!("Anouncing ourselves in the chat for game {:?}", game_id);
    lichess::api::write_in_chat(api, game_id, "I am ready! Gimme all you've got!").await;

    info!("Streaming game {:?}", game_id);

    while true == lichess::api::game_is_ongoing(api, game_id).await {
        // Wait for our turn
        while false == lichess::api::is_my_turn(api, game_id).await {
            thread::sleep(Duration::from_millis(4000));
        }
        info!("It's our turn for game {}", game_id);

        // Try to make a move
        let game_fen = lichess::api::get_game_fen(api, game_id).await;
        if let Ok(chess_move) = &chess::engine::play_move(&game_fen) {
            info!("Playing move {} for game id {}", chess_move, game_id);
            lichess::api::make_move(api, game_id, chess_move, false).await;
        } else {
            info!("Can't find a move... Let's offer draw");
            lichess::api::make_move(api, game_id, "", true).await;
        }
    }

    /*
    let api_clone = api.clone();
    let game_id_clone = String::from(game_id);

    let handler = thread::spawn(move || {
        lichess::api::play_game(&api_clone, &game_id_clone);
    });
    handler.join().expect("Game playing thread has panicked!");
    */

    info!("Nothing else to do?");
    Ok(())
}
