// Other crates
use lazy_static::lazy_static;
use std::fs;

// Modules within this module
pub mod api;
pub mod helpers;
pub mod types;

// Constants
const API_TOKEN_FILE_NAME: &str = "/assets/lichess_api_token.txt";

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
