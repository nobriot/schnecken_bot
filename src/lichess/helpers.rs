use log::*;
use serde_json::Value as JsonValue;

pub fn parse_string_to_nd_json(string_value: &str) -> Vec<JsonValue> {
    let mut result: Vec<JsonValue> = Vec::new();

    // Decompose line by line:
    for line in string_value.lines() {
        if line.len() == 0 {
            continue;
        }

        let json_value_result = serde_json::from_str(&line);
        if let Err(e) = json_value_result {
            info!("Error Parsing string line into json? {}", e);
            continue;
        }

        let json_value = json_value_result.unwrap();
        result.push(json_value);
    }

    return result;
}

mod tests {
    #[test]
    fn parse_nd_json_empty_lines() {
        use crate::lichess::helpers::*;
        let test_string = "\n\r\n\r\n\r\n\r\n\r\n\r\n\r
        
        \n\r
        
        \n\r
        ";
        // Some eval values, from strongest to weakest (from white perspective)
        let result = parse_string_to_nd_json(test_string);
        assert_eq!(0, result.len());
    }

    #[test]
    fn parse_nd_json_3_valid_lines() {
        use crate::lichess::helpers::*;

        let test_string = r#"{"value": 1}
                {"value": 2}
                {"value": 3}
                "#;
        // Some eval values, from strongest to weakest (from white perspective)
        let result = parse_string_to_nd_json(test_string);
        assert_eq!(3, result.len());
    }

    #[test]
    fn parse_nd_json_1_valid_line() {
        use crate::lichess::helpers::*;

        let test_string = r#"{"value: 1}
        {"value": 2
        {"value": 3}
        "#;
        // Some eval values, from strongest to weakest (from white perspective)
        let result = parse_string_to_nd_json(test_string);
        assert_eq!(1, result.len());
    }
}
