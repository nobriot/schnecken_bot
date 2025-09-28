use std::fmt::Display;

use crate::model::board::*;
use crate::model::board_mask::*;
use crate::model::containers::position_list::*;
use crate::model::moves::*;
use crate::model::piece::*;
use log::*;

/// Start game state for a standard chess game.
pub const START_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
/// How many positions do we memorize in the GameState to check for 3-fold
/// repetitions
pub const LAST_POSITIONS_SIZE: usize = 30;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GamePhase {
  Opening,
  Middlegame,
  Endgame,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, PartialOrd)]
pub enum GameStatus {
  #[default]
  Ongoing,
  WhiteWon,
  BlackWon,
  Stalemate,
  ThreeFoldRepetition,
  Draw,
}

/// Captures all the data required in a Chess Game
/// to identify Stalemates, repetitions, etc.
#[derive(Clone)]
pub struct GameState {
  pub board: Board,
  pub ply: u8,
  // Number of half-moves in the game
  pub move_count: u16,
  // Vector of position representing the last x positions, from the start
  pub last_positions: PositionList,
}

// -----------------------------------------------------------------------------
// Game state implementation

impl GameState {
  /// Takes a board and makes it into a GameState, assuming default values for
  /// the rest.
  ///
  /// ### Arguments
  ///
  /// * `board`: Board object to use for the Game State
  ///
  /// ### Return Value
  ///
  /// GameState object with the board passed in argument
  pub fn from_board(board: &Board) -> Self {
    GameState {
      board: board.clone(),
      ply: 0,
      move_count: 0,
      last_positions: PositionList::new(),
    }
  }

  /// Takes a full FEN notation and converts it into a Game State
  ///
  /// ### Arguments
  ///
  /// * `fen`: String to use to create a GameState object
  ///
  /// ### Return Value
  ///
  /// GameState object
  pub fn from_fen(fen: &str) -> Self {
    let fen_parts: Vec<&str> = fen.split(' ').collect();
    if fen_parts.len() < 6 {
      error!("Fen too small to generate a game state");
      return GameState::default();
    }

    let board = Board::from_fen(fen);
    let ply: u8 = fen_parts[4].parse::<u8>().unwrap_or(0);
    let mut move_count: u16 = fen_parts[5].parse::<u16>().unwrap_or(1);
    move_count = move_count.saturating_sub(1);
    move_count *= 2;
    if board.side_to_play == Color::Black {
      move_count += 1;
    }

    GameState {
      board,
      ply,
      move_count,
      last_positions: PositionList::new(),
    }
  }

  /// Exports the game state to a FEN notation
  ///
  /// ### Arguments
  ///
  /// * `self`: Reference to a GameState object
  ///
  /// ### Return Value
  ///
  /// String representing the game state.
  pub fn to_fen(&self) -> String {
    let mut fen = String::new();

    fen += self.board.to_fen().as_str();
    fen.push(' ');
    match self.board.side_to_play {
      Color::White => fen.push('w'),
      Color::Black => fen.push('b'),
    }
    fen.push(' ');

    fen += self.board.castling_rights.to_fen().as_str();
    fen.push(' ');

    if self.board.en_passant_square != INVALID_SQUARE {
      fen += square_to_string(self.board.en_passant_square).as_str();
    } else {
      fen.push('-');
    }
    fen.push(' ');
    fen += self.ply.to_string().as_str();
    fen.push(' ');
    fen += (self.move_count / 2 + 1).to_string().as_str();

    fen
  }

  /// Checks the previous board configuration and checks if we repeated the
  /// position
  ///
  /// ### Arguments
  ///
  /// * `self`: GameState reference
  ///
  /// ### Return value
  ///
  /// Number of repetitions that occurred for the current position.
  /// Current position is not counted.
  /// i.e. 0 the position just occurred for the first time. 2 means a threefold
  /// repetition
  pub fn get_board_repetitions(&self) -> usize {
    self.last_positions.count(self.board.hash)
    // self.last_positions.iter().fold(0,|count, x| if *x == self.board.hash {
    // count + 1 } else { count },)
  }

  /// Get all the possible moves in a position, for the side to play.
  #[inline]
  pub fn get_moves(&self) -> Vec<Move> {
    self.board.get_moves()
  }

  pub fn get_king_square(&self) -> u8 {
    match self.board.side_to_play {
      Color::White => self.board.get_white_king_square(),
      Color::Black => self.board.get_black_king_square(),
    }
  }

  /// Looks at the board and finds the move (with all move data associated)
  /// based on the move notation
  ///
  /// ### Arguments
  ///
  /// * `move_notation`: Move notation to find on the board, e.g. "e2e4"
  ///
  /// ### Return value
  ///
  /// Move data. Sanity from the current board is not checked
  ///
  pub fn get_move_from_notation(&self, move_notation: &str) -> Move {
    let mv = Move::from_string(move_notation);
    debug_assert!(!mv.is_null(), "Got null move from {move_notation}");

    // Check if the king is flying (going to more than 1 square away, and in this case
    // mark it as a castling move
    let king_position = match self.board.side_to_play {
      Color::White => self.board.get_white_king_square(),
      Color::Black => self.board.get_black_king_square(),
    };

    if mv.u8_src() != king_position {
      return mv;
    }

    // Adjust the move if it is a castle
    if move_notation == "e8h8" || move_notation == "e8g8" {
      return castle_mv!(60, 62);
    } else if move_notation == "e8c8" || move_notation == "e8a8" {
      return castle_mv!(60, 58);
    } else if move_notation == "e1h1" || move_notation == "e1g1" {
      return castle_mv!(4, 6);
    } else if move_notation == "e1c1" || move_notation == "e1a1" {
      return castle_mv!(4, 2);
    }

    mv
  }

  /// Previous version of get_move_from_notation, had a crash with castling
  /// Looks at the board and finds the move (with all move data associated)
  /// based on the move notation
  ///
  /// ### Arguments
  ///
  /// * `move_notation`: Move notation to find on the board, e.g. "e2e4"
  ///
  /// ### Return value
  ///
  /// Move data. Will return a Null Move if the move is not found.
  pub fn get_move_from_notation_old(&self, move_notation: &str) -> Move {
    let candidates = self.board.get_moves();

    for c in candidates {
      if c.to_string() == move_notation {
        return c;
      }
    }

    error!(
      "Could not identify move {} for board: {}",
      move_notation,
      self.to_fen()
    );
    error!("Here are the candiate moves: {:?}", self.board.get_moves());

    Move::null()
  }
  /// Same as `apply_move`, except that it takes a move notation
  ///
  /// ### Arguments
  ///
  /// * `chess_move`: Reference to a move.
  pub fn apply_move_from_notation(&mut self, move_notation: &str) {
    let m = self.get_move_from_notation(move_notation.trim());
    self.apply_move(&m);
  }

  /// Applies a move for the game.
  ///
  /// ### Arguments
  ///
  /// * `chess_move`: Reference to a move.
  pub fn apply_move(&mut self, chess_move: &Move) -> () {
    // println!("Applying move {} on game {}", chess_move, self.to_fen());
    // let mut moves = String::new();
    debug_assert!(!chess_move.is_null(), "Null move passed for applying.");
    debug_assert!(
      self.board.pieces.get(chess_move.src() as u8) != NO_PIECE,
      "Input moves with empty source square? {} - board:{}\n{:#?}",
      chess_move,
      self.board,
      self
    );

    // TODO: Remove this check
    if self.board.pieces.get(chess_move.src() as u8) == NO_PIECE {
      error!(
        "Input moves with empty source square? {} - board:{}\n{:#?}",
        chess_move, self.board, self
      );
    }

    // Save the last position:
    let source_is_pawn: bool = square_in_mask!(chess_move.src(), self.board.pieces.pawns());
    if source_is_pawn || chess_move.is_capture() {
      // Cannot really repeat a position after a pawn moves or a capture
      // assume anything forward is a novel position
      self.last_positions.clear();
    } else {
      self.last_positions.add(self.board.hash);
    }

    // Update the ply-count
    let destination_piece = self.board.pieces.get(chess_move.dest() as u8);
    if destination_piece != NO_PIECE || source_is_pawn {
      self.ply = 0;
    } else {
      self.ply = self.ply.saturating_add(1);
    }

    // Half Move count
    self.move_count += 1;

    // Move the pieces on the board
    self.board.apply_move(chess_move);
  }

  /// Applies all moves from a vector of moves
  ///
  /// ### Arguments
  ///
  /// * `move_list`: Vector of moves to apply on the position
  pub fn apply_moves(&mut self, move_list: &[Move]) -> () {
    for chess_move in move_list {
      self.apply_move(chess_move);
    }
  }

  /// Applies all moves from a vector of a string of notations
  ///
  /// ### Arguments
  ///
  /// * `move_list`: String with move notations, e.g. "e2e4 e7e5"
  pub fn apply_move_list(&mut self, move_list: &str) -> () {
    if move_list.is_empty() {
      return;
    }

    let moves: Vec<&str> = move_list.split(' ').collect();
    for chess_move in moves {
      self.apply_move_from_notation(chess_move);
    }
  }

  /// Attempts to apply a move on the board based on its PGN notation
  ///
  /// ### Arguments
  ///
  /// * `self`: Position on which we would like to apply a move.
  /// * `move_notation`: PGN notation of the move, e.g e4 or Bxf7
  ///
  /// ### Return value
  ///
  /// Result, indicating if the move was identified and applied or not.
  pub fn apply_pgn_move(&mut self, move_notation: &str) -> Result<(), ()> {
    let board_result = self.board.find_move_from_pgn_notation(move_notation);
    if let Ok(mv) = board_result {
      self.apply_move(&mv);
      Ok(())
    } else {
      Err(())
    }
  }
}

// -----------------------------------------------------------------------------
// Display/Default implementations for our game state
impl std::fmt::Debug for GameState {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut message = String::from("\n");
    message += format!(
      "Move: {}, Ply: {} Side to play {} - Checks {}\n",
      self.move_count,
      self.ply,
      self.board.side_to_play,
      self.board.checks()
    )
    .as_str();
    message += format!(
      "En passant: {} - Castling rights: {}\n",
      square_to_string(self.board.en_passant_square),
      self.board.castling_rights.to_fen(),
    )
    .as_str();

    message += format!("Board: {}\n", self.board.to_fen()).as_str();

    message += "last positions:\n";
    message += format!("- {}\n", self.last_positions).as_str();

    f.write_str(message.as_str())
  }
}

impl Default for GameState {
  fn default() -> Self {
    GameState {
      board: Board::from_fen(START_POSITION_FEN),
      ply: 0,
      move_count: 0,
      last_positions: PositionList::new(),
    }
  }
}
