pub mod book;
pub mod provocative_book;

// Dependencies
use regex::Regex;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::model::board::Board;
use crate::model::game_state::GameState;
use crate::model::moves::Move;

// -----------------------------------------------------------------------------
// Type definitions

// List of board configurations with an associated set of moves
pub type ChessBook = Mutex<HashMap<Board, Vec<Move>>>;

// -----------------------------------------------------------------------------
// Constants

/// Regex to parse PGN strings. We do not parse anotations here
pub const PGN_REGEX: &str = r#"(\d*\.{1,3}\s+)?(?P<mv>([BKQNR]?[abcdefgh]?[12345678]?x?[abcdefgh][12345678]=?[BQNRbqnr]?|O-O|O-O-O)[#\+]?)[\?!]*\s+"#;

// -----------------------------------------------------------------------------
// Functions

/// Initializes all our chess books
///
///
pub fn initialize_chess_books() {
  book::initialize_chess_book();
  provocative_book::initialize_chess_book();
}

/// Retrieves the book moves
///
/// ### Arguments
///
/// * `board`:        Board configuration to look up in the books
/// * `provocative`:  Set this to true to play provocative openings
///
pub fn get_book_moves(board: &Board, provocative: bool) -> Option<Vec<Move>> {
  if provocative {
    provocative_book::get_book_moves(board)
  } else {
    book::get_book_moves(board)
  }
}

/// Adds a line in the opening to the book
///
/// ### Arguments
///
/// * `line`: list of moves separated with spaces.
///
/// e.g. `e2e4 c7c5 g1f3 d7d6 c5d4 f3d4 g8f6 b1c3 a7a6`
///
pub fn add_line_to_book(chess_book: &ChessBook, line: &str) {
  let mut game_state = GameState::default();
  let moves: Vec<&str> = line.split(' ').collect();
  let mut book = chess_book.lock().unwrap();

  for chess_move in moves {
    if !book.contains_key(&game_state.board) {
      let _ = book.insert(game_state.board, Vec::new());
    }

    let move_list = book.get_mut(&game_state.board).unwrap();
    let m = Move::from_string(chess_move);
    if !move_list.contains(&m) {
      move_list.push(m);
    }

    game_state.apply_move_from_notation(chess_move);
  }
}

/// Adds a line in the opening to the book
///
/// ### Arguments
///
/// * `pgn`: PGN format str.
///
/// e.g. `1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3 O-O`
///
pub fn add_pgn_to_book(chess_book: &ChessBook, pgn: &str) {
  let mut game_state = GameState::default();
  let mut book = chess_book.lock().unwrap();

  let pgn_re = Regex::new(PGN_REGEX).unwrap();

  // Use regex to extract move notations
  let captures = pgn_re.captures_iter(&pgn);
  for value in captures {
    // Find the mv (e.g. 'Kf7') and the annotation (e.g. '{ [%eval 0.36] [%clk 0:10:00] }')
    let mv = value.name("mv");
    if mv.is_none() {
      return;
    }
    let mv = mv.unwrap().as_str();
    //println!("Move: {mv}");
    let m_result = game_state.board.find_move_from_pgn_notation(mv);

    if m_result.is_err() {
      println!("Could not parse move: {}", mv);
      return;
    }
    let m = m_result.unwrap();

    if !book.contains_key(&game_state.board) {
      let _ = book.insert(game_state.board, Vec::new());
    }

    let move_list = book.get_mut(&game_state.board).unwrap();
    if !move_list.contains(&m) {
      move_list.push(m);
    }

    game_state.apply_move(&m);
  } // for value in captures
}

/// Adds a line in the from a position
///
/// ### Arguments
///
/// * `chess_book`: Reference to the book in which the line will be added
/// * `fen`: Fen format str.
/// * `pgn`: PGN format str.
///
pub fn add_pgn_from_position(chess_book: &ChessBook, fen: &str, pgn: &str) {
  let mut game_state = GameState::from_fen(fen);
  let mut book = chess_book.lock().unwrap();

  let pgn_re = Regex::new(PGN_REGEX).unwrap();

  // Use regex to extract move notations
  let captures = pgn_re.captures_iter(&pgn);
  for value in captures {
    // Find the mv (e.g. 'Kf7') and the annotation (e.g. '{ [%eval 0.36] [%clk 0:10:00] }')
    let mv = value.name("mv");
    if mv.is_none() {
      return;
    }
    let mv = mv.unwrap().as_str();
    //println!("Move: {mv}");
    let m_result = game_state.board.find_move_from_pgn_notation(mv);

    if m_result.is_err() {
      println!("Could not parse move: {}", mv);
      return;
    }
    let m = m_result.unwrap();

    if !book.contains_key(&game_state.board) {
      let _ = book.insert(game_state.board, Vec::new());
    }

    let move_list = book.get_mut(&game_state.board).unwrap();
    if !move_list.contains(&m) {
      move_list.push(m);
    }

    game_state.apply_move(&m);
  } // for value in captures
}

/// Adds a position in the opening to the book
///
/// ### Arguments
///
/// * `fen`: Fen of the position to reach
/// * `mv`:  Notation of the move to play
///
pub fn add_single_move_to_book(chess_book: &ChessBook, fen: &str, mv: &str) {
  let game_state = GameState::from_fen(fen);
  let mut book = chess_book.lock().unwrap();
  let m = Move::from_string(mv);

  if !book.contains_key(&game_state.board) {
    let _ = book.insert(game_state.board, Vec::new());
  }

  let move_list = book.get_mut(&game_state.board).unwrap();
  if !move_list.contains(&m) {
    move_list.push(m);
  }
}
