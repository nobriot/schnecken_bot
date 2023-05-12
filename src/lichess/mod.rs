// Other crates
use lazy_static::lazy_static;
use log::*;
//use rand::Rng;
use std::fs;

use crate::lichess::types::Player;

// Modules within this module
pub mod api;
pub mod helpers;
pub mod types;

// Constants
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
    if api::is_online(username).await == true {
      info!("{username} is online. Sending a challenge!");
      if let Err(()) = api::send_challenge(username).await {
        info!("Error sending a challenge to {username}");
        continue;
      }
      break;
    }
  }
}
