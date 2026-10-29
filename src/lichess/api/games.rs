use crate::api::LichessApi;
use crate::error::LichessError;
use crate::helpers;
use crate::traits::GameStreamHandler;
use futures_util::StreamExt;
use log::*;
use serde_json::Value as JsonValue;

impl LichessApi {
  /// Unique ID used by the application to stream games
  const GAMES_STREAM_ID: &str = "schnecken_bot";

  /// Creates a stream of games, and a list of game
  /// IDs. The stream first outputs the games that already exists, then emits
  /// an event each time a game is started or finished.
  ///
  /// Games are streamed as ndjson. Maximum number of games: 500 for anonymous
  /// requests, or 1000 for OAuth2 authenticated requests.
  ///
  /// ### Arguments
  ///
  /// * `game_id` Game ID to stream
  /// * `callback` Function to invoke when data has been received on the stream
  ///
  /// ### Returns
  ///
  /// Result indicating if we had error receiving/parsing the event stream.
  pub async fn stream_games_by_id_with_callback<T>(&self,
                                                   game_id: &str,
                                                   handler: &T,
                                                   callback: fn(&T, JsonValue, String))
                                                   -> Result<(), LichessError> {
    info!("Requesting Lichess to stream games {game_id}");

    let response_result =
      self.post(&format!("stream/games/{}", Self::GAMES_STREAM_ID), game_id).await;

    if let Err(e) = response_result {
      warn!("Error issuing a POST request to Lichess: {e:?}");
      return Err(LichessError);
    }

    let mut nd_json_buffer = helpers::NdJsonBuffer::default();

    let mut stream = response_result.unwrap().bytes_stream();
    while let Some(chunk_response) = stream.next().await {
      match chunk_response {
        Err(e) => {
          warn!("Error receiving games stream for {game_id}: {e:?}");
          return Err(LichessError);
        },
        Ok(chunk) => {
          let json_entries = nd_json_buffer.push(&chunk);

          for json_entry in json_entries {
            debug!("Calling callback: {}", json_entry);
            debug!("callback: {:?}", callback);
            callback(handler, json_entry, String::from(game_id));
          }
          // Sending 1 byte is usually just the keep-alive message
          if chunk.len() == 1 {
            debug!("Received keep-alive message for Game State stream");
          }
        },
      }
    }

    info!("Finished to stream game events for game id {game_id}");
    Ok(())
  }

  pub async fn stream_games_by_id<T>(self, handler: &T, game_id: &str) -> Result<(), LichessError>
    where T: GameStreamHandler
  {
    info!("Requesting Lichess to stream games {game_id}");

    let response_result =
      self.post(&format!("stream/games/{}", Self::GAMES_STREAM_ID), game_id).await;

    if let Err(e) = response_result {
      warn!("Error issuing a POST request to Lichess: {e:?}");
      return Err(LichessError);
    }

    let mut nd_json_buffer = helpers::NdJsonBuffer::default();

    let mut stream = response_result.unwrap().bytes_stream();
    while let Some(chunk_response) = stream.next().await {
      match chunk_response {
        Err(e) => {
          warn!("Error receiving games stream for {game_id}: {e:?}");
          return Err(LichessError);
        },
        Ok(chunk) => {
          let json_entries = nd_json_buffer.push(&chunk);

          for json_entry in json_entries {
            handler.game_stream_handler(json_entry, String::from(game_id));
          }
          // Sending 1 byte is usually just the keep-alive message
          if chunk.len() == 1 {
            debug!("Received keep-alive message for Game State stream");
          }
        },
      }
    }

    info!("Finished to stream game events for game id {game_id}");
    Ok(())
  }
}
