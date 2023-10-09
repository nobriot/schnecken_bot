// Internal crates
use crate::api::LichessApi;
use crate::types::*;

// External crates
use log::*;
use serde_json::Value as JsonValue;
use urlencoding::encode;

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
  ///
  pub async fn abort_game(&self, game_id: &str) -> Result<(), ()> {
    let api_endpoint: String = format!("bot/game/{game_id}/abort");
    let _json_response: JsonValue;
    if let Ok(json) = self.lichess_post(&api_endpoint, "").await {
      _json_response = json;
    } else {
      return Err(());
    }

    Ok(())
  }

  /// Resigns a game
  ///
  /// Note: we should never resign, always try to svindle something out of the situation :-D
  /// Perhaps just resign 1 move before being smothered mated
  ///
  /// ### Arguments
  ///
  /// * `game_id` Game ID to resign
  ///
  /// ### Returns
  ///
  /// Result indicating if we had error requesting a game abort
  ///
  pub async fn resign_game(&self, game_id: &str) -> Result<(), ()> {
    let api_endpoint: String = format!("bot/game/{game_id}/resign");
    let _json_response: JsonValue;
    if let Ok(json) = self.lichess_post(&api_endpoint, "").await {
      _json_response = json;
    } else {
      return Err(());
    }

    Ok(())
  }

  /// Writes in the game chat, using spectator room
  ///
  /// ### Arguments
  ///
  /// * `game_id` Game ID on which we should send a chat message
  /// * `message` Message to send
  ///
  pub async fn write_in_spectator_room(&self, game_id: &str, message: &str) -> () {
    info!("Sending message on Game ID {game_id} - {message}");
    let endpoint: String = format!("bot/game/{game_id}/chat");
    let body: String = format!("room=spectator&text={}", encode(message));

    let result = self.lichess_post(&endpoint, &body).await;

    if let Err(error) = result {
      warn!(
        "Error sending message to game id {} - Error: {:#?}",
        game_id, error
      );
    }
  }

  /// Writes in the game chat, using spectator room
  ///
  /// ### Arguments
  ///
  /// * `game_id` Game ID on which we should send a chat message
  /// * `message` Message to send
  ///
  pub async fn write_in_chat(&self, game_id: &str, message: &str) -> () {
    info!("Sending message on Game ID {game_id} - {message}");
    let endpoint: String = format!("bot/game/{game_id}/chat");
    let body: String = format!("room=player&text={}", encode(message));

    let result = self.lichess_post(&endpoint, &body).await;

    if let Err(error) = result {
      warn!(
        "Error sending message to game id {} - Error: {:#?}",
        game_id, error
      );
    }
  }

  /// Writes in the game chat on a specific room
  ///
  /// ### Arguments
  ///
  /// * `game_id` Game ID on which we should send a chat message
  /// * `room`    Room on which to send the message
  /// * `message` Message to send
  ///
  pub async fn write_in_chat_room(&self, game_id: &str, room: ChatRoom, message: &str) -> () {
    let room_str = match room {
      ChatRoom::Player => String::from("player"),
      ChatRoom::Spectator => String::from("spectator"),
    };
    info!("Sending message on Game ID {game_id} - {message} for room: {room_str}");
    let endpoint: String = format!("bot/game/{game_id}/chat");
    let body: String = format!("room={}&text={}", encode(&room_str), encode(message));

    let result = self.lichess_post(&endpoint, &body).await;

    if let Err(error) = result {
      warn!(
        "Error sending message to game id {} - Error: {:#?}",
        game_id, error
      );
    }
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
  ///
  pub async fn make_move(&self, game_id: &str, chess_move: &str, offer_draw: bool) -> bool {
    info!(
      "Trying chess move {} on game id {} - Draw offer: {}",
      chess_move, game_id, offer_draw
    );
    let api_endpoint: String =
      format!("bot/game/{game_id}/move/{chess_move}?offeringDraw={offer_draw}");

    let json_response: JsonValue;
    let mut retries = 0;

    loop {
      retries += 1;
      let move_result = self.lichess_post(&api_endpoint, "").await;

      if move_result.is_ok() {
        json_response = move_result.unwrap();
        break;
      }

      warn!("Move was not accepted by Lichess. {:?}", move_result);
      if retries > 10 {
        error!("Something is not working with making moves");
        return false;
      }
    }

    if json_response["ok"].as_bool().is_none() {
      error!(
        "Lichess refused our move! :'( - We're so bad - Error {:?}",
        json_response
      );
      self.write_in_chat(game_id, format!("Debug: I kind of got confused and tried to play move {}. I'll probably time out on this game. JSON response from Lichess: {}", chess_move,json_response).as_str()).await;
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
  ///
  pub async fn claim_victory(&self, game_id: &str) -> Result<(), ()> {
    info!("Attempting to claim victory for game id {}", game_id);
    let api_endpoint: String = format!("board/game/{game_id}/claim-victory");
    let body: String = format!("gameId={}", encode(game_id));
    let _json_response: JsonValue;
    if let Ok(json) = self.lichess_post(&api_endpoint, body.as_str()).await {
      _json_response = json;
    } else {
      return Err(());
    }

    Ok(())
  }

  /// Claims victory for a game where the opponent left after a timeout
  ///
  /// ### Arguments
  ///
  /// * `timeout` Number of seconds to wait before claiming victory
  /// * `game_id` Game ID on which we are claiming victory
  ///
  ///
  pub async fn claim_victory_after_timeout(&self, timeout: u64, game_id: &str) {
    tokio::time::sleep(tokio::time::Duration::from_secs(timeout + 1)).await;
    let _ = self.claim_victory(game_id).await;
  }
}
