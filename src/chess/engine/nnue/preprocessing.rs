use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::BufWriter;

use crate::model::game_state::*;

/// Probably not fail-proof for PGN parsing, but it seems to work with our data.
/// Won't match if we do not have anotated eval in the PGN.
const PGN_REGEX: &str = r#"\d*\.{1,3}\s+(?<mv>([BKQNR]?[abcdefgh]?[12345678]?x?[abcdefgh][12345678]=?[BQNRbqnr]?|O-O|O-O-O)[#\+]?)[\?!]*\s+(?<annotation>\{.*?\})?"#;

/// Finds the Eval value from the anotated curly brace notation
const EVAL_REGEX: &str = r#"\[%eval (-?\d*\.\d*|#-?\d+)]"#;

/// Print helpers:
const WARNING: &str = "\x1B[1m\x1B[93mWarning\x1B[0m";

/// Takes a file with PGN data and converts it into a training set
/// for the chess NNUE
///
/// ### Arguments
///
/// * `input_file`: &str indicating the path where we want to get the data from
/// * `output_file`: &str indicating the path where we want to export the training set to.
///
pub fn create_training_data_from_pgn_file(
  input_file: &str,
  output_file: &str,
) -> std::io::Result<()> {
  let file = File::open(input_file)?;
  let mut reader = BufReader::new(file);
  let mut line = String::new();

  // Save to the output file
  let output_file = File::create(output_file)?;
  let mut writer = BufWriter::new(output_file);

  let mut i: usize = 0;
  let mut game_state: GameState = GameState::default();

  let pgn_re = Regex::new(PGN_REGEX).unwrap();
  let eval_re = Regex::new(EVAL_REGEX).unwrap();

  // Parse each line until we are EOF:
  let mut read_bytes = 1;
  while read_bytes != 0 {
    line.clear();
    read_bytes = reader.read_line(&mut line)?;

    // Just skip lines with game metadata for now
    // Assume it leads us to a new game.
    if line.starts_with("[") {
      game_state = GameState::default();
      continue;
    }

    // Use regex to extract move notations
    let captures = pgn_re.captures_iter(&line);
    'move_loop: for value in captures {
      // Find the mv (e.g. 'Kf7') and the annotation (e.g. '{ [%eval 0.36] [%clk 0:10:00] }')
      let mv = value.name("mv");
      let annotation = value.name("annotation");
      if mv.is_none() || annotation.is_none() {
        break 'move_loop;
      }
      let mv = mv.unwrap().as_str();
      let annotation = annotation.unwrap().as_str();
      //println!("Move : {:#?}", mv);
      //println!("Annotation : {:#?}", annotation);

      // Now find if we have an eval in the annotations
      let eval_capture = eval_re.captures(annotation);
      if eval_capture.is_none() {
        break 'move_loop;
      }
      let eval_capture = eval_capture.unwrap();
      let mut eval_string = String::from(eval_capture.get(1).map_or("", |m| m.as_str()));
      if eval_string.is_empty() {
        break 'move_loop;
      }

      let mate_sequence = eval_string.starts_with("#");
      if mate_sequence {
        let _ = eval_string.remove(0);
      }

      let eval_result = eval_string.parse::<f32>();
      if eval_result.is_err() {
        println!("Eval Result Error: {:?}", eval_result.err());
        break 'move_loop;
      }

      // Convert mating sequences into 200 - 'number of half-moves'
      let mut eval = eval_result.unwrap();
      if mate_sequence {
        if eval < 0.0 {
          eval = -201.0 - eval;
        } else {
          eval = 201.0 - eval;
        }
      }

      // Print the eval
      //println!("Move : {:#?} - Eval : {:#?} ", mv, eval);
      let move_result = game_state.apply_pgn_move(mv);
      if move_result.is_err() {
        println!("Error processing PGN: {}", line);
        break 'move_loop;
      }

      // Save our data:
      //cache.insert(board, eval);
      writer.write_fmt(format_args!("{};{}\n", eval, game_state.to_fen()))?;
      //println!("Board after move {} : {}", mv, board.to_fen());
    } // 'move_loop

    i += 1;
    if i % 100_000 == 0 {
      println!("Game {} processed", i);
    }
  }

  Ok(())
}

/// Takes a file with PGN data and converts it into a training set
/// for the chess NNUE
///
/// ### Arguments
///
/// * `input_file`: &str indicating the path where we have a CSV formatted training set
///
pub fn load_training_set_in_cache(input_file: &str) -> std::io::Result<Vec<(GameState, f32)>> {
  // Save the results in here:
  let mut cache: Vec<(GameState, f32)> = Vec::new();

  let file = File::open(input_file)?;

  let mut reader = BufReader::new(file);
  let mut line = String::new();

  loop {
    line.clear();
    let read_bytes = reader.read_line(&mut line)?;
    if read_bytes == 0 {
      break;
    }

    let csv_elements: Vec<&str> = line.split(';').collect();

    if csv_elements.len() < 2 {
      println!(
        "{WARNING}: Could not read elemeents from CSV line: {}",
        line
      );
      continue;
    }

    let game_state = GameState::from_fen(csv_elements[1]);
    let eval = csv_elements[0].parse::<f32>().unwrap_or(f32::NAN);
    if eval.is_nan() {
      println!(
        "{WARNING}: Could not parse string into f32 value: {}",
        csv_elements[0]
      );
      continue;
    }

    //println!("inserting: {} / {}", game_state.to_fen(), eval);
    cache.push((game_state, eval));
  }

  println!("Loaded {} samples in the cache", cache.len());

  Ok(cache)
}
