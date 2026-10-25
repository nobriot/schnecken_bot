// Internal crates
use crate::api::LichessApi;
use crate::error::LichessError;
use crate::types::*;
// External crates
use log::*;
use serde_json::Value as JsonValue;
use std::time::Duration;
use urlencoding::encode;

/// How many times we try to send a move before giving up
const MAKE_MOVE_MAX_ATTEMPTS: u32 = 5;
/// Delay between attempts to send a move (multiplied by the attempt number)
const MAKE_MOVE_RETRY_DELAY: Duration = Duration::from_millis(100);

impl LichessApi {
  /// Attempts to abort a game.
  ///
  /// ### Arguments
  ///
  /// * `game_id` Game ID to abort
  ///
  /// ### Returns
  ///
  /// Result indicating if we had error requesting a game abort
  pub async fn abort_game(&self, game_id: &str) -> Result<(), LichessError> {
    let api_endpoint: String = format!("bot/game/{game_id}/abort");
    let _json_response: JsonValue;
    if let Ok(json) = self.lichess_post(&api_endpoint, "").await {
      _json_response = json;
    } else {
      return Err(LichessError);
    }

    Ok(())
  }

  /// Resigns a game
  ///
  /// Note: we should never resign, always try to svindle something out of the
  /// situation :-D Perhaps just resign 1 move before being smothered mated
  ///
  /// ### Arguments
  ///
  /// * `game_id` Game ID to resign
  ///
  /// ### Returns
  ///
  /// Result indicating if we had error requesting a game abort
  pub async fn resign_game(&self, game_id: &str) -> Result<(), LichessError> {
    let api_endpoint: String = format!("bot/game/{game_id}/resign");
    let _json_response: JsonValue;
    if let Ok(json) = self.lichess_post(&api_endpoint, "").await {
      _json_response = json;
    } else {
      return Err(LichessError);
    }

    Ok(())
  }

  /// Writes in the game chat, using spectator room
  ///
  /// ### Arguments
  ///
  /// * `game_id` Game ID on which we should send a chat message
  /// * `message` Message to send
  pub async fn write_in_spectator_room(&self, game_id: &str, message: &str) {
    info!("Sending message on Game ID {game_id} - {message}");
    let endpoint: String = format!("bot/game/{game_id}/chat");
    let body: String = format!("room=spectator&text={}", encode(message));

    let result = self.lichess_post(&endpoint, &body).await;

    if let Err(error) = result {
      warn!("Error sending message to game id {} - Error: {:#?}", game_id, error);
    }
  }

  /// Writes in the game chat, using spectator room
  ///
  /// ### Arguments
  ///
  /// * `game_id` Game ID on which we should send a chat message
  /// * `message` Message to send
  pub async fn write_in_chat(&self, game_id: &str, message: &str) {
    info!("Sending message on Game ID {game_id} - {message}");
    let endpoint: String = format!("bot/game/{game_id}/chat");
    let body: String = format!("room=player&text={}", encode(message));

    let result = self.lichess_post(&endpoint, &body).await;

    if let Err(error) = result {
      warn!("Error sending message to game id {} - Error: {:#?}", game_id, error);
    }
  }

  /// Writes in the game chat on a specific room
  ///
  /// ### Arguments
  ///
  /// * `game_id` Game ID on which we should send a chat message
  /// * `room`    Room on which to send the message
  /// * `message` Message to send
  pub async fn write_in_chat_room(&self, game_id: &str, room: ChatRoom, message: &str) {
    let room_str = match room {
      ChatRoom::Player => String::from("player"),
      ChatRoom::Spectator => String::from("spectator"),
    };
    info!("Sending message on Game ID {game_id} - {message} for room: {room_str}");
    let endpoint: String = format!("bot/game/{game_id}/chat");
    let body: String = format!("room={}&text={}", encode(&room_str), encode(message));

    let result = self.lichess_post(&endpoint, &body).await;

    if let Err(error) = result {
      warn!("Error sending message to game id {} - Error: {:#?}", game_id, error);
    }
  }

  /// Offers a draw, or accepts the one offered by the opponent.
  /// When a threefold repetition (or the 50 moves rule) is on the board, this
  /// claims the draw.
  ///
  /// ### Arguments
  ///
  /// * `game_id` Game ID in which we offer the draw
  ///
  /// ### Returns
  ///
  /// Result indicating if Lichess accepted the request
  pub async fn offer_draw(&self, game_id: &str) -> Result<(), LichessError> {
    info!("Offering/claiming a draw on game id {game_id}");
    let json = self.lichess_post(&format!("bot/game/{game_id}/draw/yes"), "").await?;
    if json["ok"].as_bool() == Some(true) {
      Ok(())
    } else {
      warn!("Lichess refused our draw offer/claim on game {game_id}: {json}");
      Err(LichessError)
    }
  }

  /// Fetches the messages posted in the player chat of a game
  ///
  /// ### Arguments
  ///
  /// * `game_id` Game ID for which we want the chat
  ///
  /// ### Returns
  ///
  /// The chat lines, oldest first
  pub async fn get_game_chat(&self, game_id: &str) -> Result<Vec<ChatLine>, LichessError> {
    let json = self.lichess_get(&format!("bot/game/{game_id}/chat")).await?;
    serde_json::from_value::<Vec<ChatLine>>(json).map_err(|error| {
      warn!("Could not parse chat for game {game_id}: {error}");
      LichessError
    })
  }

  /// Makes a move on a Game
  ///
  /// Will make a few retries if the move is not accepted at the first attempt.
  ///
  /// ### Arguments
  ///
  /// * `game_id` Game ID on which we should send a chat message
  /// * `chess_move` Notation of the move to make
  /// * `offer_draw` Set this to true to make a draw offer.
  ///
  /// ### Returns
  ///
  /// True if the move was sent and accepted by the Lichess server
  /// False otherwise
  pub async fn make_move(&self, game_id: &str, chess_move: &str, offer_draw: bool) -> bool {
    info!("Trying chess move {} on game id {} - Draw offer: {}",
          chess_move, game_id, offer_draw);
    let api_endpoint: String =
      format!("bot/game/{game_id}/move/{chess_move}?offeringDraw={offer_draw}");

    let json_response: JsonValue;
    let mut retries = 0;

    loop {
      retries += 1;
      let move_result = self.lichess_post(&api_endpoint, "").await;

      if let Ok(result) = move_result {
        json_response = result;
        break;
      }

      warn!("Move was not accepted by Lichess. {:?}", move_result);
      if retries >= MAKE_MOVE_MAX_ATTEMPTS {
        error!("Something is not working with making moves");
        return false;
      }
      // Give the network a short break before trying again
      tokio::time::sleep(MAKE_MOVE_RETRY_DELAY * retries).await;
    }

    if json_response["ok"].as_bool().is_none() {
      error!("Lichess refused our move! :'( - We're so bad - Error {:?}", json_response);
      // self .write_in_spectator_room( game_id, format!( "Debug: Just tried to
      // play move {} but it was refused: {}", chess_move, json_response)
      // .as_str(),) .await;
      return false;
    }

    json_response["ok"].as_bool().unwrap()
  }

  /// Claims victory for a game where the opponent left
  ///
  /// ### Arguments
  ///
  /// * `game_id` Game ID on which we are claiming victory
  ///
  /// ### Returns
  ///
  /// True if the move was sent and accepted by the Lichess server
  /// False otherwise
  pub async fn claim_victory(&self, game_id: &str) -> Result<(), LichessError> {
    info!("Attempting to claim victory for game id {}", game_id);
    let api_endpoint: String = format!("board/game/{game_id}/claim-victory");
    let body: String = format!("gameId={}", encode(game_id));
    if let Ok(json) = self.lichess_post(&api_endpoint, body.as_str()).await {
      if json.get("error").is_some() {
        error!("POST refused from lichess:{}", json);
        return Err(LichessError);
      }
    } else {
      return Err(LichessError);
    }

    Ok(())
  }

  /// Claims victory for a game where the opponent left after a timeout
  ///
  /// ### Arguments
  ///
  /// * `timeout` Number of seconds to wait before claiming victory
  /// * `game_id` Game ID on which we are claiming victory
  pub async fn claim_victory_after_timeout(&self, timeout: u64, game_id: &str) {
    tokio::time::sleep(tokio::time::Duration::from_secs(timeout + 1)).await;
    let _ = self.claim_victory(game_id).await;
  }
}
