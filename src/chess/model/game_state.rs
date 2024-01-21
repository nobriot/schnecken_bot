use log::*;
use std::collections::VecDeque;

use crate::model::board::*;
use crate::model::board_mask::*;
use crate::model::moves::*;
use crate::model::piece::*;
use crate::model::tables::zobrist::BoardHash;

/// Start game state for a standard chess game.
pub const START_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
/// How many positions do we memorize in the GameState to check for 3-fold repetitions
pub const LAST_POSITIONS_SIZE: usize = 30;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GamePhase {
  Opening,
  Middlegame,
  Endgame,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum GameStatus {
  #[default]
  Ongoing,
  WhiteWon,
  BlackWon,
  Stalemate,
  ThreeFoldRepetition,
  Draw,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct GameState {
  pub board: Board,
  pub ply: u8,
  pub move_count: usize,
  // Vector of position representing the last x positions, from the start
  pub last_positions: VecDeque<BoardHash>,
  pub last_moves: Vec<Move>,
}

// -----------------------------------------------------------------------------
// Game state implementation

impl GameState {
  /// Takes a board and makes it into a GameState, assuming default values for the rest.
  ///
  /// ### Arguments
  ///
  /// * `board`: Board object to use for the Game State
  ///
  /// ### Return Value
  ///
  /// GameState object with the board passed in argument
  ///
  pub fn from_board(board: &Board) -> Self {
    GameState {
      board: board.clone(),
      ply: 0,
      move_count: 0,
      last_positions: VecDeque::with_capacity(LAST_POSITIONS_SIZE),
      last_moves: Vec::new(),
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
  ///
  pub fn from_fen(fen: &str) -> Self {
    let fen_parts: Vec<&str> = fen.split(' ').collect();
    if fen_parts.len() < 6 {
      error!("Fen too small to generate a game state");
      return GameState::default();
    }

    let board = Board::from_fen(fen);
    let ply: u8 = fen_parts[4].parse::<u8>().unwrap_or(0);
    let move_count: usize = fen_parts[5].parse::<usize>().unwrap_or(0);

    GameState {
      board: board,
      ply: ply,
      move_count: move_count,
      last_positions: VecDeque::with_capacity(LAST_POSITIONS_SIZE),
      last_moves: Vec::new(),
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
  ///
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
    fen += self.move_count.to_string().as_str();

    fen
  }

  /// Checks the previous board configuration and checks if we repeated the position
  ///
  /// ### Arguments
  ///
  /// * `self`: GameState reference
  ///
  /// ### Return value
  ///
  /// Number of repetitions that occurred for the current position.
  /// Current position is not counted.
  /// i.e. 0 the position just occured for the first time. 2 means a threefold repetition
  ///
  pub fn get_board_repetitions(&self) -> usize {
    self.last_positions.iter().filter(|x| **x == self.board.hash).count()
    // self.last_positions.iter().fold(0,|count, x| if *x == self.board.hash { count + 1 } else { count },)
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
  /// Move data. Will return a 0 value if the move is not found.
  ///
  pub fn get_move_from_notation(&self, move_notation: &str) -> Move {
    let candidates = self.board.get_moves();

    for c in candidates {
      if c.to_string() == move_notation {
        return c;
      }
    }

    warn!(
      "Could not identify move {} for board: {}",
      move_notation,
      self.to_fen()
    );
    Move { data: 0 }
  }

  /// Same as `apply_move`, except that it takes a move notation
  ///
  /// ### Arguments
  ///
  /// * `chess_move`: Reference to a move.
  ///
  pub fn apply_move_from_notation(&mut self, move_notation: &str) {
    let m = self.get_move_from_notation(move_notation.trim());
    self.apply_move(&m);
  }

  /// Applies a move for the game.
  ///
  /// ### Arguments
  ///
  /// * `chess_move`: Reference to a move.
  ///
  pub fn apply_move(&mut self, chess_move: &Move) -> () {
    /*
    println!("Applying move {} on game {}", chess_move, self.to_fen());
    let mut moves = String::new();
    for m in &self.last_moves {
      moves += m.to_string().as_str();
      moves.push(' ');
    }
    println!("Last moves: {}", moves);
     */
    debug_assert!(
      self.board.pieces.get(chess_move.src() as u8) != NO_PIECE,
      "Input moves with empty source square? {} - board:\n{}",
      chess_move,
      self.board
    );

    // Save the last position:
    let source_is_pawn: bool = square_in_mask!(chess_move.src(), self.board.pieces.pawns());
    if source_is_pawn {
      // Cannot really repeat a position after a pawn moves, assume anything forward is a novel position
      self.last_positions.clear();
    } else {
      if self.last_positions.len() >= LAST_POSITIONS_SIZE {
        self.last_positions.pop_back();
      }
    }
    self.last_positions.push_front(self.board.hash);

    // Update the ply-count
    let destination_piece = self.board.pieces.get(chess_move.dest() as u8);
    if destination_piece != NO_PIECE || source_is_pawn {
      self.ply = 0;
    } else if self.ply < 255 {
      self.ply += 1;
    }

    // Move count (keep in that order, as the side to play is updated in the board.apply_move())
    if self.board.side_to_play == Color::Black {
      self.move_count += 1;
    }
    // Move the pieces on the board
    self.board.apply_move(chess_move);

    // Save the move we applied.
    self.last_moves.push(chess_move.clone());
  }

  /// Applies all moves from a vector of moves
  ///
  /// ### Arguments
  ///
  /// * `move_list`: Vector of moves to apply on the position
  ///
  pub fn apply_moves(&mut self, move_list: &Vec<Move>) -> () {
    for chess_move in move_list {
      self.apply_move(chess_move);
    }
  }

  /// Applies all moves from a vector of a string of notations
  ///
  /// ### Arguments
  ///
  /// * `move_list`: String with move notations, e.g. "e2e4 e7e5"
  ///
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
  ///
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
    for i in 0..self.last_positions.len() {
      message += format!("- {}\n", self.last_positions[i]).as_str();
    }

    f.write_str(message.as_str())
  }
}

impl Default for GameState {
  fn default() -> Self {
    GameState {
      board: Board::from_fen(START_POSITION_FEN),
      ply: 0,
      move_count: 1,
      last_positions: VecDeque::with_capacity(LAST_POSITIONS_SIZE),
      last_moves: Vec::new(),
    }
  }
}
