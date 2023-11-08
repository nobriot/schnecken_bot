// Imports / dependencies
use chess::engine::*;
use chess::model::game_state::START_POSITION_FEN;
use regex::Regex;
use std::io::prelude::*;
use std::io::BufReader;
use std::process::ExitCode;

// -----------------------------------------------------------------------------
// Constants
const POSITION_CMD_FEN_REGEX: &str = r#"^position\s*fen\s*(?P<fen>[1-8kqrbnpKQRBNP/]*\s[bw]?\s?[kqKQ-]*\s?[abcdefgh12345678-]{0,2}\s?\d*\s?\d*)"#;
const POSITION_CMD_MOVE_REGEX: &str =
  r#"^position\s*[\s0-9a-zA-Z\/-]*\smoves\s(?P<moves>[\s0-9a-zA-Z\/-]*)"#;
const SET_OPTION_NAME_VALUE_REGEX: &str =
  r#"^setoption\s+name\s+(?P<name>.+)\s+value\s+(?P<value>.+)"#;

/// This module prodives a UCI interface to the engine
///
/// Refer to the specifications here: https://www.wbec-ridderkerk.nl/html/UCIProtocol.html
///
///

const HELP_MESSAGE: &str = "
DESCRIPTION
  schnecken_engine is a UCI chess engine.
  I used this specification: https://backscattering.de/chess/uci/,
  though I am not sure which one is the official one, it's kind of hard to find.

  options:
  
    setoption name use_nnue value <bool>
      Decides if the engine should use the NNUE. The NNUE is currently very slow
      and not incredible at prediction positions.

    setoption name ponder value <bool>
      Decides if we should ponder. Same as running \"go ponder\"

    setoption name play_style type combo default Normal var Conservative var Normal var Aggressive var Provocative
      Decides how the engine should play. Normal is the default.
      Use Conservative to try to draw stronger opponents.
      Use Aggressive to play aggressively.
      Use Provocative to play weaker opponents.
";

// Main function
fn main() -> ExitCode {
  let stdin = std::io::stdin();
  let mut reader = BufReader::new(stdin);

  // Get an engine instance running:
  let mut engine = Engine::new();

  // Regex for parsing those commands
  let position_fen_re = Regex::new(POSITION_CMD_FEN_REGEX).unwrap();
  let position_moves_re = Regex::new(POSITION_CMD_MOVE_REGEX).unwrap();
  let options_re = Regex::new(SET_OPTION_NAME_VALUE_REGEX).unwrap();

  // Parse each line until we are EOF:
  let mut read_bytes = 1;
  let mut line = String::new();

  // parsing loop
  while read_bytes != 0 {
    // Read the input
    line.clear();
    read_bytes = reader.read_line(&mut line).unwrap_or(0);

    // Parse the command with parameters
    // The command is parameters[0]
    let parameters: Vec<&str> = line.trim().split(" ").collect();

    match parameters[0] {
      // Generic UCI commands
      "uci" => {
        println!("id name schnecken_engine {}", env!("CARGO_PKG_VERSION"));
        println!("id author Nicolas W");
        println!("");
        println!("option name use_nnue type check default false");
        println!("option name ponder type check default false");
        println!("option name play_style type combo default Normal var Conservative var Normal var Aggressive var Provocative");
        println!("uciok");
      },
      "isready" => {
        println!("readyok");
      },

      "debug" => {
        if line.contains("on") {
          engine.set_debug(true);
        } else {
          engine.set_debug(false);
        }
      },

      // Engine options
      "setoption" => {
        let option_capture = options_re.captures(&line);
        if option_capture.is_none() {
          continue;
        }
        let option_capture = option_capture.unwrap();
        let name = option_capture.name("name");
        let value = option_capture.name("value");
        if name.is_none() || value.is_none() {
          continue;
        }
        let name = name.unwrap().as_str();
        let value = value.unwrap().as_str();

        match name {
          "use_nnue" => {
            let value = value.parse::<bool>().unwrap_or(false);
            engine.set_use_nnue(value);
          },
          "ponder" => {
            let value = value.parse::<bool>().unwrap_or(false);
            engine.set_ponder(value);
          },
          "play_style" => {
            let value = value.parse::<PlayStyle>().unwrap_or_default();
            engine.set_play_style(value);
          },
          _ => {},
        }
      },

      "clear" => {
        let _ = std::io::stdout().flush();
      },

      // Game play:
      "position" => {
        if parameters.len() < 2 {
          println!("Sorry, we need at least 1 argument for position. Example : position startpos");
          continue;
        }
        if parameters[1] == "startpos" {
          engine.set_position(START_POSITION_FEN);
        } else if parameters[1] == "fen" {
          // Use the regex to retrieve the FEN.
          let fen_capture = position_fen_re.captures(&line);
          if fen_capture.is_none() {
            continue;
          }
          let fen = fen_capture.unwrap().name("fen");
          if fen.is_none() {
            continue;
          }
          let fen = fen.unwrap().as_str();
          engine.set_position(fen);
        }

        // If we got a movelist, then apply them:
        if line.contains("moves") {
          let move_list_capture = position_moves_re.captures(&line);
          if move_list_capture.is_none() {
            continue;
          }
          let move_list = move_list_capture.unwrap().name("moves");
          if move_list.is_none() {
            continue;
          }
          let move_list = move_list.unwrap().as_str();
          engine.position.apply_move_list(move_list);
        }
      },
      "ucinewgame" => {
        stop_engine_blocking(&engine);
        engine.reset();
      },

      "flip" => {
        if engine.is_active() {
          continue;
        }
        engine.position.board.flip();
      },

      "go" => {
        // Check some of the options passed:
        if line.contains("infinite") {
          engine.set_maximum_depth(0);
        }
        if line.contains("ponder") {
          engine.set_ponder(true);
        }
        // Get started searching:
        let engine_clone = engine.clone();
        let _ = std::thread::spawn(move || engine_clone.go());
        // TODO: Find out why the cache is empty when we stop here.
      },
      "stop" => {
        stop_engine_blocking(&engine);
        engine.print_evaluations();
      },

      // TODO: Use a debug option instead
      "show_state" => {
        println!("Position: {}", engine.position.to_fen());
        println!("searching: {}", engine.is_active());
      },

      // Program commands
      "quit" | "exit" | "q" => {
        println!("bye bye! 🙂");
        break;
      },
      "help" | "h" => {
        println!("{}", HELP_MESSAGE);
      },
      _ => {
        println!("Sorry, not implemented yet 🙂");
      },
    }
  }

  return ExitCode::SUCCESS;
}

// -----------------------------------------------------------------------------
// Helper functions

/// Synchronously request the engine to stop searching and blocks while the
/// engine is active, returns as soon as the engine has stopped.
///
pub fn stop_engine_blocking(engine: &Engine) {
  while engine.is_active() {
    engine.stop();
    std::thread::sleep(std::time::Duration::from_millis(10));
  }
}
