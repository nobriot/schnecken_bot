use log::*;
use serde_json::Value as JsonValue;

/// Parses a string to a Vector of JSON values.
///
/// ### Arguments
///
/// * `string_value` The ND-JSON string
///
/// ### Returns
///
/// A vector of serde_json::Value. Can be empty.
pub fn parse_string_to_nd_json(string_value: &str) -> Vec<JsonValue> {
  let mut result: Vec<JsonValue> = Vec::new();

  // Decompose line by line:
  for line in string_value.lines() {
    if line.is_empty() {
      continue;
    }

    let json_value_result = serde_json::from_str(line);
    if let Err(e) = json_value_result {
      warn!("Error Parsing string line into json? {}", e);
      continue;
    }

    let json_value = json_value_result.unwrap();
    result.push(json_value);
  }

  result
}

/// Accumulates the bytes received on a ND-JSON stream, so that lines split
/// across several chunks are only parsed once they are complete.
#[derive(Debug, Default)]
pub struct NdJsonBuffer {
  pending: Vec<u8>,
}

impl NdJsonBuffer {
  /// Appends a chunk received on the stream and parses all the lines that are
  /// now complete. An incomplete trailing line is kept for the next chunk.
  ///
  /// ### Arguments
  ///
  /// * `chunk` Bytes received on the stream
  ///
  /// ### Returns
  ///
  /// A vector of serde_json::Value. Can be empty.
  pub fn push(&mut self, chunk: &[u8]) -> Vec<JsonValue> {
    self.pending.extend_from_slice(chunk);

    let Some(last_newline) = self.pending.iter().rposition(|&b| b == b'\n') else {
      return Vec::new();
    };

    let complete_lines: Vec<u8> = self.pending.drain(..=last_newline).collect();
    parse_string_to_nd_json(&String::from_utf8_lossy(&complete_lines))
  }
}

#[cfg(test)]
mod tests {
  use crate::helpers::{NdJsonBuffer, parse_string_to_nd_json};

  #[test]
  fn parse_nd_json_empty_lines() {
    let test_string = "\n\r\n\r\n\r\n\r\n\r\n\r\n\r
        
        \n\r
        
        \n\r
        ";
    let result = parse_string_to_nd_json(test_string);
    assert_eq!(0, result.len());
  }

  #[test]
  fn parse_nd_json_3_valid_lines() {
    let test_string = r#"{"value": 1}
                {"value": 2}
                {"value": 3}
                "#;
    let result = parse_string_to_nd_json(test_string);
    assert_eq!(3, result.len());
  }

  #[test]
  fn parse_nd_json_1_valid_line() {
    let test_string = r#"{"value: 1}
        {"value": 2
        {"value": 3}
        "#;
    let result = parse_string_to_nd_json(test_string);
    assert_eq!(1, result.len());
  }

  #[test]
  fn nd_json_buffer_complete_lines() {
    let mut buffer = NdJsonBuffer::default();
    let result = buffer.push(b"{\"value\": 1}\n{\"value\": 2}\n");
    assert_eq!(2, result.len());
    assert_eq!(1, result[0]["value"]);
    assert_eq!(2, result[1]["value"]);
  }

  #[test]
  fn nd_json_buffer_keep_alive() {
    let mut buffer = NdJsonBuffer::default();
    assert!(buffer.push(b"\n").is_empty());
    assert!(buffer.push(b"\n").is_empty());
    assert_eq!(1, buffer.push(b"{\"value\": 1}\n").len());
  }

  #[test]
  fn nd_json_buffer_line_split_across_chunks() {
    // This is how a challenge event got lost: split at column 27
    let line = br#"{"type":"challenge","challenge":{"id":"abcd1234","status":"created"}}"#;
    let mut buffer = NdJsonBuffer::default();
    assert!(buffer.push(&line[..27]).is_empty());
    let mut second_chunk = line[27..].to_vec();
    second_chunk.push(b'\n');
    let result = buffer.push(&second_chunk);
    assert_eq!(1, result.len());
    assert_eq!("challenge", result[0]["type"]);
    assert_eq!("abcd1234", result[0]["challenge"]["id"]);
  }

  #[test]
  fn nd_json_buffer_line_split_byte_by_byte() {
    let input = b"{\"value\": 1}\n\n{\"value\": 2}\n";
    let mut buffer = NdJsonBuffer::default();
    let mut result = Vec::new();
    for byte in input {
      result.extend(buffer.push(&[*byte]));
    }
    assert_eq!(2, result.len());
    assert_eq!(2, result[1]["value"]);
  }

  #[test]
  fn nd_json_buffer_complete_and_partial_line_in_one_chunk() {
    let mut buffer = NdJsonBuffer::default();
    let result = buffer.push(b"{\"value\": 1}\n{\"val");
    assert_eq!(1, result.len());
    assert_eq!(1, result[0]["value"]);
    let result = buffer.push(b"ue\": 2}\n");
    assert_eq!(1, result.len());
    assert_eq!(2, result[0]["value"]);
  }

  #[test]
  fn nd_json_buffer_utf8_split_across_chunks() {
    let line = "{\"text\": \"Schnecke 🐌\"}\n".as_bytes();
    // Split in the middle of the 4-byte emoji
    let split = line.len() - 5;
    let mut buffer = NdJsonBuffer::default();
    assert!(buffer.push(&line[..split]).is_empty());
    let result = buffer.push(&line[split..]);
    assert_eq!(1, result.len());
    assert_eq!("Schnecke 🐌", result[0]["text"]);
  }
}
