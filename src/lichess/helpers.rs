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
///
pub fn parse_string_to_nd_json(string_value: &str) -> Vec<JsonValue> {
  let mut result: Vec<JsonValue> = Vec::new();

  // Decompose line by line:
  for line in string_value.lines() {
    if line.is_empty() {
      continue;
    }

    let json_value_result = serde_json::from_str(line);
    if let Err(e) = json_value_result {
      info!("Error Parsing string line into json? {}", e);
      continue;
    }

    let json_value = json_value_result.unwrap();
    result.push(json_value);
  }

  result
}

#[cfg(test)]
mod tests {
  use crate::helpers::parse_string_to_nd_json;

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
}
