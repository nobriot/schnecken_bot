use super::message::GameMessage;
use lichess::traits::GameStreamHandler;
use log::*;
use std::sync::{Arc, mpsc};

type Handle = tokio::task::JoinHandle<()>;

#[derive(Debug, Clone)]
pub struct GameHandle {
  /// Channel to send messages to the game
  pub tx:     mpsc::Sender<GameMessage>,
  /// Handle to the game thread
  pub handle: Arc<Handle>,
  /// Lichess Game ID
  pub id:     String,
}

impl GameHandle {
  /// Resigns an ongoing game
  pub fn resign(&self) {
    let _ = self.tx.send(GameMessage::Resign);
  }

  /// Check if the game is over
  pub fn is_over(&self) -> bool {
    self.handle.is_finished()
  }
}

impl GameStreamHandler for GameHandle {
  fn game_stream_handler(&self, json_value: serde_json::Value, game_id: String) {
    debug!("GameStreamHandler called with game_id: {}, value : {}", game_id, json_value);

    debug!("Incoming stream event for Game ID {game_id}");

    if json_value["type"].as_str().is_none() {
      error!("No type for incoming stream event. JSON: {json_value}");

      if let Some(error) = json_value["error"].as_str() {
        if error.contains("token") {
          error!("Token error. Exiting the thread.");
          self.handle.abort();
        }
      }
      return;
    }

    debug!("Game Stream payload: \n{}", json_value);

    match json_value["type"].as_str().unwrap() {
      "gameFull" => {
        debug!("Full game state!");

        let game_full: Result<lichess::types::GameFull, serde_json::Error> =
          serde_json::from_value(json_value.clone());
        if let Err(error) = game_full {
          warn!("Error deserializing GameState data !! {:?}", error);
        } else {
          let game_full = game_full.unwrap();
          debug!("Parsed data: {:?}", game_full);

          let _ = self.tx.send(GameMessage::Update(game_full.state));
          // self.games.update_game_and_play(game_full.state, game_id.as_str());
        }
      },

      "gameState" => {
        // debug!("Game state update received: {}", json_value);
        let result: Result<lichess::types::GameState, serde_json::Error> =
          serde_json::from_value(json_value);
        if let Err(error) = result {
          warn!("Error deserializing GameState data !! {:?}", error);
        } else {
          let _ = self.tx.send(GameMessage::Update(result.unwrap()));
        }
      },

      "chatLine" => {
        let result: Result<lichess::types::ChatMessage, serde_json::Error> =
          serde_json::from_value(json_value);
        if let Err(error) = result {
          warn!("Error deserializing ChatLine data !! {:?}", error);
        } else {
          let message = result.unwrap();
          // Ignore our own messages
          // FIXME: remove harcoded username
          if message.username == "schnecken_bot" {
            return;
          }
          info!("Received a message on game ID {} - {:?}", game_id.as_str(), message);
          // self.on_incoming_message(game_id.as_str(), result.unwrap());
        }
      },

      "opponentGone" => {
        let gone = json_value["gone"].as_bool().unwrap_or(false);
        if gone {
          info!("Opponent gone! We'll just claim victory as soon as possible!");
          if let Some(timeout) = json_value["claimWinInSeconds"].as_u64() {
            let _ = self.tx.send(GameMessage::OpponentGone(Some(timeout)));
          }
        } else {
          let _ = self.tx.send(GameMessage::OpponentGone(None));
          info!("Opponent is back!");
        }
      },
      other => {
        // Ignore other events
        warn!("Received unknown streaming game state: {}", other);
        warn!("{}", json_value);
      },
    }
    // debug!("JSON: {}", json_value);
  }
}
