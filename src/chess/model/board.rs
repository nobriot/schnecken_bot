use crate::chess::model::board_mask::*;
use crate::chess::model::castling_rights::*;
use crate::chess::model::moves::*;
use crate::chess::model::piece::*;
use crate::chess::model::tables::zobrist::*;

use log::*;
use std::hash::{Hash, Hasher};

// -----------------------------------------------------------------------------
//  Constants

/// Numerical value used to represent an invalid square
pub const INVALID_SQUARE: u8 = 255;

/// Default start position FEN
const START_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

// -----------------------------------------------------------------------------
//  Macros

/// Checks if a file/rank value is within bounds, else returns the rvalue.
macro_rules! fr_bounds_or_return {
  ($file:expr, $rvalue:expr) => {
    if !(1..=8).contains(&$file) {
      return $rvalue;
    }
  };
}
// Make this macro public
pub(crate) use fr_bounds_or_return;

// -----------------------------------------------------------------------------
//  Structs/Enums

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Board {
  pub squares: [u8; 64],
  pub side_to_play: Color,
  pub castling_rights: CastlingRights,
  pub en_passant_square: u8,
  pub hash: u64,
}

// -----------------------------------------------------------------------------
// Implementations

impl Board {
  /// Initialize a board with no piece, all zeroes
  fn new() -> Self {
    Board {
      squares: [0u8; 64],
      side_to_play: Color::White,
      castling_rights: CastlingRights::default(),
      en_passant_square: INVALID_SQUARE,
      hash: 0,
    }
  }

  /// Updates the board hash using the Zobrist tables from "scratch"
  fn compute_hash(&mut self) {
    self.hash = 0;

    // Add the hash from the pieces
    for i in 0..64 {
      if self.squares[i] != NO_PIECE {
        self.update_hash_piece(i);
      }
    }

    // Add the hash from the side to play
    if self.side_to_play == Color::White {
      self.update_hash_side_to_play();
    }

    // Add the hash from the castling rights
    self.update_hash_castling_rights();

    // Add the hash from the en-passant square:
    if self.en_passant_square != INVALID_SQUARE {
      self.update_hash_en_passant();
    }
  }

  fn update_hash_piece(&mut self, i: usize) {
    self.hash ^= ZOBRIST_TABLE[(self.squares[i] - 1) as usize][i];
  }

  fn update_hash_side_to_play(&mut self) {
    self.hash ^= ZOBRIST_WHITE_TO_MOVE;
  }

  fn update_hash_en_passant(&mut self) {
    self.hash ^= ZOBRIST_EN_PASSANT[self.en_passant_square as usize % 8];
  }

  fn update_hash_castling_rights(&mut self) {
    if self.castling_rights.K {
      self.hash ^= ZOBRIST_WHITE_KING_CASTLE;
    }
    if self.castling_rights.Q {
      self.hash ^= ZOBRIST_WHITE_QUEEN_CASTLE;
    }
    if self.castling_rights.k {
      self.hash ^= ZOBRIST_BLACK_KING_CASTLE;
    }
    if self.castling_rights.q {
      self.hash ^= ZOBRIST_BLACK_QUEEN_CASTLE;
    }
  }

  /// Converts Rank / File into a board index
  ///
  /// Returns an index in the range 0..63. Asserts in debug mode if
  /// the values passed are not valid.
  ///
  /// * `file`: [1..8]
  /// * `rank`: [1..8]
  ///
  pub fn fr_to_index(file: usize, rank: usize) -> usize {
    debug_assert!(file > 0);
    debug_assert!(file <= 8);
    debug_assert!(rank > 0);
    debug_assert!(rank <= 8);
    (file - 1) + (rank - 1) * 8
  }

  /// Converts a board index into Rank / File.
  ///
  /// Returns a file and rank in the range [1..8]. Asserts in debug mode if
  /// the values passed are not valid.
  ///
  /// * `index`: [0..63]
  ///
  pub fn index_to_fr(index: usize) -> (usize, usize) {
    debug_assert!(index < 64);
    (index % 8 + 1, index / 8 + 1)
  }

  /// Returns the piece currently set at the board file/rank a board index into Rank / File.
  ///
  /// * `file`: [1..8]
  /// * `rank`: [1..8]
  ///
  pub fn get_piece(&self, file: usize, rank: usize) -> u8 {
    self.squares[Board::fr_to_index(file, rank)]
  }

  /// Checks if a move on the board is a capture
  ///
  /// ### Arguments
  ///
  /// * `self`: Board to look at
  /// * `m`:    Reference to a move to examine
  ///
  /// ### Return value
  ///
  /// True if the move is a capture, false otherwise
  ///
  pub fn is_move_a_capture(&self, m: &Move) -> bool {
    // If a piece is at the destination, it's a capture
    if self.squares[m.dest as usize] != NO_PIECE {
      return true;
    }

    // If a pawn moves to the en-passant square, it's a capture.
    if self.en_passant_square != INVALID_SQUARE
      && m.dest == self.en_passant_square
      && (self.squares[m.src as usize] == WHITE_PAWN || self.squares[m.src as usize] == BLACK_PAWN)
    {
      return true;
    }

    false
  }

  /// Applies a move on the board.
  ///
  /// Very few checks are done here, the caller has to check that the move is
  /// legal before applying it.
  pub fn apply_move(&mut self, chess_move: &Move) {
    let source = chess_move.src as usize;
    let destination = chess_move.dest as usize;

    // Check if we just castled, we need to move the rooks around!
    if self.squares[source] == WHITE_KING {
      if chess_move.src == 4 && chess_move.dest == 2 {
        self.update_hash_piece(0);
        self.squares[0] = NO_PIECE;
        self.squares[3] = WHITE_ROOK;
        self.update_hash_piece(3);
      } else if chess_move.src == 4 && chess_move.dest == 6 {
        self.update_hash_piece(7);
        self.squares[7] = NO_PIECE;
        self.squares[5] = WHITE_ROOK;
        self.update_hash_piece(5);
      }
    } else if self.squares[source] == BLACK_KING {
      if chess_move.src == 60 && chess_move.dest == 62 {
        self.update_hash_piece(63);
        self.squares[63] = NO_PIECE;
        self.squares[61] = BLACK_ROOK;
        self.update_hash_piece(61);
      } else if chess_move.src == 60 && chess_move.dest == 58 {
        self.update_hash_piece(56);
        self.squares[56] = NO_PIECE;
        self.squares[59] = BLACK_ROOK;
        self.update_hash_piece(59);
      }
    }

    // Update castling rights. (just look if something from the rook/king moved)
    self.update_hash_castling_rights();
    match chess_move.src {
      0 => self.castling_rights.Q = false,
      4 => {
        self.castling_rights.K = false;
        self.castling_rights.Q = false
      },
      7 => self.castling_rights.K = false,
      56 => self.castling_rights.q = false,
      60 => {
        self.castling_rights.k = false;
        self.castling_rights.q = false
      },
      63 => self.castling_rights.k = false,
      _ => {},
    }
    match chess_move.dest {
      0 => self.castling_rights.Q = false,
      4 => {
        self.castling_rights.K = false;
        self.castling_rights.Q = false
      },
      7 => self.castling_rights.K = false,
      56 => self.castling_rights.q = false,
      60 => {
        self.castling_rights.k = false;
        self.castling_rights.q = false
      },
      63 => self.castling_rights.k = false,
      _ => {},
    }
    self.update_hash_castling_rights();

    // Check if we have a en passant square
    if self.en_passant_square != INVALID_SQUARE {
      // Remove the previous en-passant square from the hash
      self.update_hash_en_passant();
    }

    // Check if we have a new en-passant location:
    self.en_passant_square = INVALID_SQUARE;
    if (self.squares[source] == WHITE_PAWN || self.squares[source] == BLACK_PAWN)
      && (chess_move.dest as isize - chess_move.src as isize).abs() == 16
    {
      let op_pawn = match self.squares[source] {
        WHITE_PAWN => BLACK_PAWN,
        _ => WHITE_PAWN,
      };
      let (rank, file) = Board::index_to_fr(destination);
      if file > 1 {
        let s = Board::fr_to_index(file - 1, rank);
        if self.squares[s] == op_pawn {
          self.en_passant_square = (chess_move.dest + chess_move.src) / 2;
        }
      }

      // Check on the right side:
      if file < 8 {
        let s = Board::fr_to_index(file + 1, rank);
        if self.squares[s] == op_pawn {
          self.en_passant_square = (chess_move.dest + chess_move.src) / 2;
        }
      }
    }

    // Add the new hash value, if a new square is valid:
    if self.en_passant_square != INVALID_SQUARE {
      self.update_hash_en_passant();
    }

    // Check if this is some en-passant action: PAWN is moving diagonally while the destination square is empty:
    // En passant needs to remove the captured pawn.
    if (self.squares[source] == WHITE_PAWN || self.squares[source] == BLACK_PAWN)
      && self.squares[destination] == NO_PIECE
    {
      let target_capture = match chess_move.dest as isize - chess_move.src as isize {
        7 => source - 1,
        9 => source + 1,
        -7 => source + 1,
        -9 => source - 1,
        _ => {
          // Not a en-passant move
          INVALID_SQUARE as usize
        },
      };

      if target_capture != (INVALID_SQUARE as usize) {
        self.update_hash_piece(target_capture);
        self.squares[target_capture] = NO_PIECE;
      }
    }

    // Now apply the initial move
    if self.squares[destination] != NO_PIECE {
      self.update_hash_piece(destination);
    }

    if chess_move.promotion != NO_PIECE {
      self.squares[destination] = chess_move.promotion;
    } else {
      self.squares[destination] = self.squares[source];
    }
    self.update_hash_piece(destination);
    self.update_hash_piece(source);
    self.squares[source] = NO_PIECE;

    // Update the side to play:
    if self.side_to_play == Color::White {
      self.side_to_play = Color::Black;
    } else {
      self.side_to_play = Color::White;
    }
    self.update_hash_side_to_play();
  }

  /// Verifies if the move is a castling move based on the board
  ///
  /// ### Arguments
  ///
  /// * `chess_move` - Chess move to look at
  ///
  /// ### Return value
  ///
  /// True if the move is a castling move, false otherwise
  ///
  pub fn is_castle(self, chess_move: &Move) -> bool {
    if self.squares[chess_move.src as usize] == WHITE_KING {
      if chess_move.src == 4 && (chess_move.dest == 2 || chess_move.dest == 6) {
        return true;
      }
    } else if self.squares[chess_move.src as usize] == BLACK_KING {
      if chess_move.src == 60 && (chess_move.dest == 62 || chess_move.dest == 58) {
        return true;
      }
    }
    false
  }

  /// Checks if there is a piece on a square
  ///
  /// ### Arguments
  ///
  /// * `square` - Square index in [0..63]
  ///
  /// ### Return value
  ///
  /// True if there is a piece on that square, false otherwise
  ///
  pub fn has_piece(&self, square: u8) -> bool {
    debug_assert!(square < 64, "has_piece called with too high square value");
    self.squares[square as usize] != NO_PIECE
  }

  /// Checks if there is a piece with a given color on a square
  ///
  /// ### Arguments
  ///
  /// * `square` - Square index in [0..63]
  /// * `color` -  Color to match the piece
  ///
  /// ### Return value
  ///
  /// True if there is a piece with the given color on that square, false otherwise
  ///
  pub fn has_piece_with_color(&self, square: u8, color: Color) -> bool {
    match self.squares[square as usize] {
      NO_PIECE => false,
      WHITE_KING => color == Color::White,
      WHITE_QUEEN => color == Color::White,
      WHITE_ROOK => color == Color::White,
      WHITE_BISHOP => color == Color::White,
      WHITE_KNIGHT => color == Color::White,
      WHITE_PAWN => color == Color::White,
      BLACK_KING => color == Color::Black,
      BLACK_QUEEN => color == Color::Black,
      BLACK_ROOK => color == Color::Black,
      BLACK_BISHOP => color == Color::Black,
      BLACK_KNIGHT => color == Color::Black,
      BLACK_PAWN => color == Color::Black,
      _ => false,
    }
  }

  /// Checks if a king is on the square
  ///
  /// ### Arguments
  ///
  /// * `square` - Square index in [0..63]
  ///
  /// ### Return value
  ///
  /// True if there is a king with the given square, false otherwise
  ///
  pub fn has_king(&self, square: usize) -> bool {
    match self.squares[square as usize] {
      WHITE_KING => true,
      BLACK_KING => true,
      _ => false,
    }
  }

  /// Finds the square with a black king on it.
  ///
  /// ### Return value
  ///
  /// `square` - Square index in [0..63] where the black king is located.
  /// The lowest square value if there are several black kings.
  /// `INVALID_SQUARE` if no black king is present on the board.
  ///
  pub fn get_black_king_square(&self) -> u8 {
    for i in 0..64 {
      if self.squares[i as usize] == BLACK_KING {
        return i;
      }
    }
    error!("No black king ?? ");
    println!("Board: {}", self);

    INVALID_SQUARE
  }

  /// Finds the square with a white king on it.
  ///
  /// ### Return value
  ///
  /// `square` - Square index in [0..63] where the white king is located.
  /// The lowest square value if there are several white kings.
  /// `INVALID_SQUARE` if no white king is present on the board.
  ///
  pub fn get_white_king_square(&self) -> u8 {
    for i in 0..64 {
      if self.squares[i as usize] == WHITE_KING {
        return i;
      }
    }
    error!("No white king ?? ");
    INVALID_SQUARE
  }

  /// Return a board bismask with squares set to 1 when they
  /// have a piece with a certain color
  pub fn get_color_mask(&self, color: Color) -> BoardMask {
    let mut board_mask = 0;

    for i in 0..64 {
      match self.squares[i as usize] {
        NO_PIECE => {},
        WHITE_KING | WHITE_QUEEN | WHITE_ROOK | WHITE_BISHOP | WHITE_KNIGHT | WHITE_PAWN => {
          if color == Color::White {
            board_mask |= 1 << i;
          }
        },
        BLACK_KING | BLACK_QUEEN | BLACK_ROOK | BLACK_BISHOP | BLACK_KNIGHT | BLACK_PAWN => {
          if color == Color::Black {
            board_mask |= 1 << i;
          }
        },
        _ => {},
      }
    }
    board_mask
  }

  /// Return a board bismask with squares set to 1 when they
  /// have a piece with a certain color, which is not a major piece (rook and queens excluded)
  pub fn get_color_mask_without_major_pieces(&self, color: Color) -> BoardMask {
    let mut board_mask = 0;

    for i in 0..64 {
      match self.squares[i as usize] {
        NO_PIECE => {},
        WHITE_KING | WHITE_BISHOP | WHITE_KNIGHT | WHITE_PAWN => {
          if color == Color::White {
            board_mask |= 1 << i;
          }
        },
        BLACK_KING | BLACK_BISHOP | BLACK_KNIGHT | BLACK_PAWN => {
          if color == Color::Black {
            board_mask |= 1 << i;
          }
        },
        _ => {},
      }
    }
    board_mask
  }

  /// Converts first substring of a FEN (with the pieces) to a board
  pub fn from_fen(fen: &str) -> Self {
    let mut board = Board::new();
    let mut rank = 7;
    let mut file = 0;

    let fen_parts: Vec<&str> = fen.split(' ').collect();
    if fen_parts.len() < 6 {
      error!("FEN string too small to generate a board");
      return board;
    }

    // First set of chars is the board squares.
    for c in fen_parts[0].chars() {
      match c {
        'K' | 'Q' | 'R' | 'B' | 'N' | 'P' | 'k' | 'q' | 'r' | 'b' | 'n' | 'p' => {
          board.squares[(rank * 8 + file) as usize] = Piece::char_to_u8(c).unwrap();
          file += 1;
        },
        '1' => file += 1,
        '2' => file += 2,
        '3' => file += 3,
        '4' => file += 4,
        '5' => file += 5,
        '6' => file += 6,
        '7' => file += 7,
        '/' => {
          rank -= 1;
          file = 0
        },
        ' ' => {
          // If we find a space, abort, as we are getting somewhere else in the FEN
          break;
        },
        _ => {},
      }
    }

    board.side_to_play = if fen_parts[1] == "w" {
      Color::White
    } else {
      Color::Black
    };

    board.castling_rights = CastlingRights {
      K: fen_parts[2].contains('K'),
      Q: fen_parts[2].contains('Q'),
      k: fen_parts[2].contains('k'),
      q: fen_parts[2].contains('q'),
    };

    board.en_passant_square = if fen_parts[3] != "-" {
      string_to_square(fen_parts[3])
    } else {
      INVALID_SQUARE
    };

    board.compute_hash();

    board
  }

  /// Converts a board to the first part of a FEN.
  pub fn to_string(&self) -> String {
    let mut fen = String::new();
    let mut empty_squares = 0;
    for rank in (1..=8).rev() {
      for file in 1..=8 {
        match self.get_piece(file, rank) {
          NO_PIECE => empty_squares += 1,
          WHITE_KING..=BLACK_PAWN => {
            if empty_squares > 0 {
              fen.push(char::from_digit(empty_squares, 10).unwrap());
              empty_squares = 0;
            }
            fen.push(Piece::u8_to_char(self.get_piece(file, rank)).unwrap());
          },
          _ => {},
        }
      }
      if empty_squares > 0 {
        fen.push(char::from_digit(empty_squares, 10).unwrap());
        empty_squares = 0;
      }
      if rank != 1 {
        fen.push('/');
      }
    }

    fen
  }
}

// -----------------------------------------------------------------------------
// Display implementations for our board

impl std::fmt::Display for Board {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut board = String::from("\n");
    for rank in (1..=8).rev() {
      for file in 1..=8 {
        board.push(Piece::u8_to_char(self.get_piece(file, rank)).unwrap());
        board.push(' ');
      }
      board.push('\n');
    }
    f.write_str(board.as_str())
  }
}

// -----------------------------------------------------------------------------
// Default implementations for our board

impl Default for Board {
  fn default() -> Self {
    Board::from_fen(START_POSITION_FEN)
  }
}

// -----------------------------------------------------------------------------
// Hash implementations for our board

impl Hash for Board {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.hash.hash(state);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn display_board() {
    let mut board = Board {
      squares: [0; 64],
      side_to_play: Color::White,
      castling_rights: CastlingRights::default(),
      en_passant_square: INVALID_SQUARE,
      hash: 0,
    };
    board.squares[0] = WHITE_ROOK;
    board.squares[1] = WHITE_KNIGHT;
    board.squares[2] = WHITE_BISHOP;
    board.squares[3] = WHITE_QUEEN;
    board.squares[4] = WHITE_KING;
    board.squares[5] = WHITE_BISHOP;
    board.squares[6] = WHITE_KNIGHT;
    board.squares[7] = WHITE_ROOK;
    board.squares[8] = WHITE_PAWN;
    board.squares[9] = WHITE_PAWN;
    board.squares[10] = WHITE_PAWN;
    board.squares[11] = WHITE_PAWN;
    board.squares[12] = WHITE_PAWN;
    board.squares[13] = WHITE_PAWN;
    board.squares[14] = WHITE_PAWN;
    board.squares[15] = WHITE_PAWN;

    board.squares[48] = BLACK_PAWN;
    board.squares[49] = BLACK_PAWN;
    board.squares[50] = BLACK_PAWN;
    board.squares[51] = BLACK_PAWN;
    board.squares[52] = BLACK_PAWN;
    board.squares[53] = BLACK_PAWN;
    board.squares[54] = BLACK_PAWN;
    board.squares[55] = BLACK_PAWN;
    board.squares[56] = BLACK_ROOK;
    board.squares[57] = BLACK_KNIGHT;
    board.squares[58] = BLACK_BISHOP;
    board.squares[59] = BLACK_QUEEN;
    board.squares[60] = BLACK_KING;
    board.squares[61] = BLACK_BISHOP;
    board.squares[62] = BLACK_KNIGHT;
    board.squares[63] = BLACK_ROOK;

    println!("Board: {}", board);
  }

  #[test]
  fn from_string() {
    let mut board = Board::from_fen("rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR");
    println!("Board: {}", board);

    let test_fen = "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2";
    board = Board::from_fen(test_fen);
    println!("Board: {}", board);

    assert_eq!(
      test_fen.split(' ').collect::<Vec<_>>()[0],
      board.to_string()
    );

    let test_fen_2 = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    board = Board::from_fen(test_fen_2);
    println!("Board: {}", board);

    assert_eq!(
      test_fen_2.split(' ').collect::<Vec<_>>()[0],
      board.to_string()
    )
  }

  #[test]
  fn apply_move() {
    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    let mut board = Board::from_fen(fen);
    println!("Board: {}", board);

    // Try and capture a piece
    board.apply_move(&Move {
      src: string_to_square("b3"),
      dest: string_to_square("g3"),
      promotion: NO_PIECE,
    });
    println!("Board: {}", board);

    // Try and promote a piece (super jump from h2 to h8)
    board.apply_move(&Move {
      src: string_to_square("h2"),
      dest: string_to_square("h8"),
      promotion: WHITE_KNIGHT,
    });
    println!("Board: {}", board);
  }

  #[test]
  fn test_fr_to_index() {
    assert_eq!(0, Board::fr_to_index(1, 1));
    assert_eq!(1, Board::fr_to_index(2, 1));
    assert_eq!(3, Board::fr_to_index(4, 1));
    assert_eq!(6, Board::fr_to_index(7, 1));
    assert_eq!(7, Board::fr_to_index(8, 1));
    assert_eq!(8, Board::fr_to_index(1, 2));
    assert_eq!(9, Board::fr_to_index(2, 2));
    assert_eq!(62, Board::fr_to_index(7, 8));
    assert_eq!(63, Board::fr_to_index(8, 8));
  }

  #[test]
  fn test_index_to_fr() {
    assert_eq!((1, 1), Board::index_to_fr(0));
    assert_eq!((2, 1), Board::index_to_fr(1));
    assert_eq!((4, 1), Board::index_to_fr(3));
    assert_eq!((7, 1), Board::index_to_fr(6));
    assert_eq!((8, 1), Board::index_to_fr(7));
    assert_eq!((1, 2), Board::index_to_fr(8));
    assert_eq!((2, 2), Board::index_to_fr(9));
    assert_eq!((7, 8), Board::index_to_fr(62));
    assert_eq!((8, 8), Board::index_to_fr(63));
  }

  #[test]
  fn test_get_piece() {
    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    let board = Board::from_fen(fen);
    assert_eq!(BLACK_ROOK, board.get_piece(2, 3));
    assert_eq!(WHITE_KING, board.get_piece(6, 4));
    assert_eq!(BLACK_KING, board.get_piece(7, 7));
  }

  #[test]
  fn test_hash_values() {
    // Position 1 - regular move
    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    let mut board = Board::from_fen(fen);
    board.apply_move(&Move {
      src: string_to_square("b3"),
      dest: string_to_square("b4"),
      promotion: NO_PIECE,
    });
    let after_move = board.hash;

    let fen = "8/5pk1/5p1p/2R5/1r3K2/6P1/7P/8 w - - 9 44";
    let board = Board::from_fen(fen);
    println!("Board after move hash: {}", after_move);
    println!("Board computed hash: {}", board.hash);
    assert_eq!(board.hash, after_move);

    // Position 2 - start position
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let mut board = Board::from_fen(fen);
    board.apply_move(&Move {
      src: string_to_square("e2"),
      dest: string_to_square("e4"),
      promotion: NO_PIECE,
    });
    let after_move = board.hash;

    let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1";
    let board = Board::from_fen(fen);
    println!("Board after move hash: {}", after_move);
    println!("Board computed hash: {}", board.hash);
    assert_eq!(board.hash, after_move);

    // Position 3 - King side white castle
    let fen = "rn1qkb1r/pbpp2pp/1p2pn2/8/4p3/2NP2P1/PPP1NPBP/R1BQK2R w KQkq - 0 7";
    let mut board = Board::from_fen(fen);
    board.apply_move(&Move {
      src: string_to_square("e1"),
      dest: string_to_square("g1"),
      promotion: NO_PIECE,
    });
    let after_move = board.hash;

    let fen = "rn1qkb1r/pbpp2pp/1p2pn2/8/4p3/2NP2P1/PPP1NPBP/R1BQ1RK1 b kq - 1 7";
    let board = Board::from_fen(fen);
    println!("Board after move hash: {}", after_move);
    println!("Board computed hash: {}", board.hash);
    assert_eq!(board.hash, after_move);

    // Position 3 - King side black castle
    let fen = "r2qk2r/pbpp2pp/1pn1pn2/2b5/4PB2/2N3P1/PPP1NPBP/R2Q1RK1 b kq - 2 9";
    let mut board = Board::from_fen(fen);
    board.apply_move(&Move {
      src: string_to_square("e8"),
      dest: string_to_square("g8"),
      promotion: NO_PIECE,
    });
    let after_move = board.hash;

    let fen = "r2q1rk1/pbpp2pp/1pn1pn2/2b5/4PB2/2N3P1/PPP1NPBP/R2Q1RK1 w - - 3 10";
    let board = Board::from_fen(fen);
    println!("Board after move hash: {}", after_move);
    println!("Board computed hash: {}", board.hash);
    assert_eq!(board.hash, after_move);

    // Position - Regular capture
    println!("Checking Regular capture");
    let fen = "r2q1rk1/pbpp2p1/1pn2n1p/2b1p1B1/4P3/2N3P1/PPP2PBP/R1NQ1RK1 w - - 0 12";
    let mut board = Board::from_fen(fen);
    board.apply_move(&Move {
      src: string_to_square("g5"),
      dest: string_to_square("f6"),
      promotion: NO_PIECE,
    });
    let after_move = board.hash;

    let fen = "r2q1rk1/pbpp2p1/1pn2B1p/2b1p3/4P3/2N3P1/PPP2PBP/R1NQ1RK1 b - - 0 12";
    let board = Board::from_fen(fen);
    println!("Board after move hash: {}", after_move);
    println!("Board computed hash: {}", board.hash);
    assert_eq!(board.hash, after_move);

    // Position - Promotion
    println!("Checking promotion");
    let fen = "8/7k/6pb/7p/4P3/3n2P1/B1p1N1KP/8 b - - 0 52";
    let mut board = Board::from_fen(fen);
    board.apply_move(&Move {
      src: string_to_square("c2"),
      dest: string_to_square("c1"),
      promotion: BLACK_QUEEN,
    });
    let after_move = board.hash;

    let fen = "8/7k/6pb/7p/4P3/3n2P1/B3N1KP/2q5 w - - 0 53";
    let board = Board::from_fen(fen);
    println!("Board after move hash: {}", after_move);
    println!("Board computed hash: {}", board.hash);
    assert_eq!(board.hash, after_move);

    // Position - White Queen side castle
    println!("Checking promotions queen side");
    let fen = "r3kbnr/ppp2ppp/2npbq2/4p3/4P3/2NPBQ2/PPP2PPP/R3KBNR w KQkq - 4 6";
    let mut board = Board::from_fen(fen);
    board.apply_move(&Move {
      src: string_to_square("e1"),
      dest: string_to_square("c1"),
      promotion: NO_PIECE,
    });
    let after_move = board.hash;

    let fen = "r3kbnr/ppp2ppp/2npbq2/4p3/4P3/2NPBQ2/PPP2PPP/2KR1BNR b kq - 5 6";
    let board = Board::from_fen(fen);
    println!("Board after move hash: {}", after_move);
    println!("Board computed hash: {}", board.hash);
    assert_eq!(board.hash, after_move);

    // Position - Black Queen side castle
    let fen = "r3kbnr/ppp2ppp/2npbq2/4p3/4P3/2NPBQ2/PPP2PPP/2KR1BNR b kq - 5 6";
    let mut board = Board::from_fen(fen);
    board.apply_move(&Move {
      src: string_to_square("e8"),
      dest: string_to_square("c8"),
      promotion: NO_PIECE,
    });
    let after_move = board.hash;

    let fen = "2kr1bnr/ppp2ppp/2npbq2/4p3/4P3/2NPBQ2/PPP2PPP/2KR1BNR w - - 6 7";
    let board = Board::from_fen(fen);
    println!("Board after move hash: {}", after_move);
    println!("Board computed hash: {}", board.hash);
    assert_eq!(board.hash, after_move);

    // Position - Losing Black king side castle
    let fen = "rnb1kbnr/ppp1pNpp/4q3/3p4/8/8/PPPPPPPP/RNBQKB1R w KQkq - 1 4";
    let mut board = Board::from_fen(fen);
    board.apply_move(&Move {
      src: string_to_square("f7"),
      dest: string_to_square("h8"),
      promotion: NO_PIECE,
    });
    let after_move = board.hash;

    let fen = "rnb1kbnN/ppp1p1pp/4q3/3p4/8/8/PPPPPPPP/RNBQKB1R b KQq - 0 4";
    let board = Board::from_fen(fen);
    println!("Board after move hash: {}", after_move);
    println!("Board computed hash: {}", board.hash);
    assert_eq!(board.hash, after_move);
  }
}
