// Other crates
use serde_json::Value as JsonValue;

/// Trait for an object that can be invoked using the Event Streams
pub trait EventStreamHandler {
  /// Receives an incoming event on a Bot Account
  ///
  /// See https://lichess.org/api#tag/Bot/operation/apiStreamEvent
  /// for the JSON payload
  ///
  /// ### Arguments
  ///
  /// * `self`       Reference to the object streaming the events
  /// * `json_value` JSON object with the event details.
  fn event_stream_handler(&self, json_value: JsonValue);
}

/// Trait for an object that can be invoked using the Game Streams
pub trait GameStreamHandler {
  /// Receives an incoming game event on a Bot Account
  ///
  /// https://lichess.org/api#tag/Bot/operation/botGameStream
  /// for the JSON payload
  ///
  /// ### Arguments
  ///
  /// * `self`       Reference to the object streaming the game events
  /// * `json_value` JSON object with the event details.
  /// * `game_id`    Game ID
  fn game_stream_handler(&self, json_value: JsonValue, game_id: String);
}
