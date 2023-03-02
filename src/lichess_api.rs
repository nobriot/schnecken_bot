use json::JsonValue;
use log::*;
use reqwest;

// Constants.
const API_BASE_URL: &'static str = "https://lichess.org/api/";

// Type definitions
pub struct LichessApi {
    pub client: reqwest::Client,
    pub token: String,
}

pub async fn lichess_get(
    api: &LichessApi,
    api_endpoint: &str,
) -> Result<JsonValue, reqwest::Error> {
    let response = api
        .client
        .get(format!("{}{}", API_BASE_URL, api_endpoint))
        .header("Authorization", format!("Bearer {}", api.token))
        .header("Accept", "application/x-ndjson")
        .send()
        .await?
        .text()
        .await?;

    let json_parse_result = json::parse(&response);
    let json_object;
    match json_parse_result {
        Ok(object) => json_object = object,
        Err(error) => {
            warn!("Error parsing JSON from the Lichess Response for API call {api_endpoint}. Response: {response} - Error:{error}");
            return Ok(JsonValue::Null);
        }
    }

    Ok(json_object)
}

pub async fn lichess_post(
    api: &LichessApi,
    api_endpoint: &str,
    api_parameter: &str,
) -> Result<JsonValue, reqwest::Error> {
    let response = api
        .client
        .post(format!(
            "{}{}/{}",
            API_BASE_URL, api_endpoint, api_parameter
        ))
        .header("Authorization", format!("Bearer {}", api.token))
        .header("Accept", "application/x-ndjson")
        .send()
        .await?
        .text()
        .await?;

    let json_parse_result = json::parse(&response);
    let json_object;
    match json_parse_result {
        Ok(object) => json_object = object,
        Err(error) => {
            warn!("Error parsing JSON from the Lichess Response for API call {api_endpoint}. Response: {response} - Error:{error}");
            return Ok(JsonValue::Null);
        }
    }

    Ok(json_object)
}

pub async fn is_online(api: &LichessApi, user_id: &str) -> Result<bool, reqwest::Error> {
    let response = api
        .client
        .get(format!("{}users/status?ids={}", API_BASE_URL, user_id))
        .header("Authorization", format!("Bearer {}", api.token))
        .header("Accept", "application/x-ndjson")
        .send()
        .await?
        .text()
        .await?;

    let json_parse_result = json::parse(&response);
    let json_object;
    match json_parse_result {
        Ok(object) => json_object = object,
        Err(_) => {
            debug!("Error parsing JSON from the User Status.");
            return Ok(false);
        }
    }

    if json_object[0]["online"] == true {
        return Ok(true);
    }

    return Ok(false);
}
