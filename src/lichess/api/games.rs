use crate::api::LichessApi;
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
                                                   -> Result<(), ()> {
    info!("Requesting Lichess to stream games {game_id}");

    let response_result =
      self.post(&format!("stream/games/{}", Self::GAMES_STREAM_ID), game_id).await;

    if let Err(e) = response_result {
      warn!("Error issuing a Post request to Lichess {}", e);
      return Err(());
    }

    let stream = response_result.unwrap().bytes_stream();
    stream.for_each(|chunk_response| async {
            if let Err(e) = chunk_response {
              info!("Error receiving stream? {}", e);
              return;
            }

            let chunk = chunk_response.unwrap();
            let string_value: String = String::from_utf8_lossy(&chunk).to_string();
            let json_entries = helpers::parse_string_to_nd_json(&string_value);

            for json_entry in json_entries {
              debug!("Calling callback: {}", json_entry);
              debug!("callback: {:?}", callback);
              callback(handler, json_entry, String::from(game_id));
            }
            // Sending 1 byte is usually just the keep-alive message
            if chunk.len() == 1 {
              debug!("Received keep-alive message for Game State stream");
            }

            ()
          })
          .await;

    info!("Finished to stream game events for game id {game_id}");
    Ok(())
  }

  pub async fn stream_games_by_id<T>(self, handler: &T, game_id: &str) -> Result<(), ()>
    where T: GameStreamHandler
  {
    info!("Requesting Lichess to stream games {game_id}");

    let response_result =
      self.post(&format!("stream/games/{}", Self::GAMES_STREAM_ID), game_id).await;

    if let Err(e) = response_result {
      warn!("Error issuing a Post request to Lichess {}", e);
      return Err(());
    }

    let stream = response_result.unwrap().bytes_stream();
    stream.for_each(|chunk_response| async {
            if let Err(e) = chunk_response {
              info!("Error receiving stream? {}", e);
              return;
            }

            let chunk = chunk_response.unwrap();
            let string_value: String = String::from_utf8_lossy(&chunk).to_string();
            let json_entries = helpers::parse_string_to_nd_json(&string_value);

            for json_entry in json_entries {
              handler.game_stream_handler(json_entry, String::from(game_id));
            }
            // Sending 1 byte is usually just the keep-alive message
            if chunk.len() == 1 {
              debug!("Received keep-alive message for Game State stream");
            }

            ()
          })
          .await;

    info!("Finished to stream game events for game id {game_id}");
    Ok(())
  }
}
