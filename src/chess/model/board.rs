use crate::chess::model::piece::*;
use log::*;

// -----------------------------------------------------------------------------
//  Constants

/// Numerical value used to represent an invalid square
pub const INVALID_SQUARE: u8 = 255;

// -----------------------------------------------------------------------------
//  Structs/Enums

#[derive(Debug, Clone, Copy)]
pub struct Board {
  pub squares: [u8; 64],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Move {
  // Source square of a move (value from 0 to 63)
  pub src: u8,
  // Destination square of a move (value from 0 to 63)
  pub dest: u8,
  // Piece to spawn in case of promotion. Encoded using piece constants (NO_PIECE, WHITE_QUEEN, etc.)
  pub promotion: u8,
}

// -----------------------------------------------------------------------------
// Generic functions

/// Converts the square number (from 0 to 63) to an algebraic notation
/// such as a1, h7, f4
///
/// We put zeroes ('00') when a value does not exist.
pub fn square_to_string(square: u8) -> String {
  let mut string = String::new();

  if square == INVALID_SQUARE {
    string = String::from("00");
    return string;
  }

  match square % 8 {
    0 => string.push('a'),
    1 => string.push('b'),
    2 => string.push('c'),
    3 => string.push('d'),
    4 => string.push('e'),
    5 => string.push('f'),
    6 => string.push('g'),
    7 => string.push('h'),
    _ => string.push('0'),
  }
  match square / 8 {
    0 => string.push('1'),
    1 => string.push('2'),
    2 => string.push('3'),
    3 => string.push('4'),
    4 => string.push('5'),
    5 => string.push('6'),
    6 => string.push('7'),
    7 => string.push('8'),
    _ => string.push('0'),
  }

  return string;
}

/// Converts the square algebraic notation to a number from 0 to 63.
pub fn string_to_square(string: &str) -> u8 {
  let mut square_value: u8 = 0;
  match string.chars().nth(0) {
    Some('a') => square_value += 0,
    Some('b') => square_value += 1,
    Some('c') => square_value += 2,
    Some('d') => square_value += 3,
    Some('e') => square_value += 4,
    Some('f') => square_value += 5,
    Some('g') => square_value += 6,
    Some('h') => square_value += 7,
    Some(_) | None => return INVALID_SQUARE,
  }
  match string.chars().nth(1) {
    // a does not add value to the square index
    Some('1') => {},
    Some('2') => square_value += 1 * 8,
    Some('3') => square_value += 2 * 8,
    Some('4') => square_value += 3 * 8,
    Some('5') => square_value += 4 * 8,
    Some('6') => square_value += 5 * 8,
    Some('7') => square_value += 6 * 8,
    Some('8') => square_value += 7 * 8,
    Some(_) | None => return INVALID_SQUARE,
  }

  square_value
}

pub fn board_mask_to_string(mask: u64) -> String {
  let mut string = String::new();
  for rank in (0..8 as u8).rev() {
    for file in 0..8 {
      let square_index = rank * 8 + file;
      if ((mask >> square_index) & 1) == 1 {
        string.push('x');
      } else {
        string.push('.');
      }
      string.push(' ');
    }
    string.push('\n');
  }
  string
}

// -----------------------------------------------------------------------------
// Implementations

impl Board {
  /// Initialize a board with no piece, all zeroes
  fn new() -> Self {
    Board { squares: [0u8; 64] }
  }

  /// Applies a move on the board.
  ///
  /// Very few checks are done here, the caller has to check that the move is
  /// legal before applying it.
  pub fn apply_move(&mut self, chess_move: Move) {
    // Check if we just castled, we need to move the rooks around!
    if self.squares[chess_move.src as usize] == WHITE_KING {
      if chess_move.src == 4 && chess_move.dest == 2 {
        let m = Move {
          src: 0,
          dest: 3,
          promotion: NO_PIECE,
        };
        self.apply_move(m);
      } else if chess_move.src == 4 && chess_move.dest == 6 {
        let m = Move {
          src: 7,
          dest: 5,
          promotion: NO_PIECE,
        };
        self.apply_move(m);
      }
    } else if self.squares[chess_move.src as usize] == BLACK_KING {
      if chess_move.src == 60 && chess_move.dest == 62 {
        let m = Move {
          src: 63,
          dest: 61,
          promotion: NO_PIECE,
        };
        self.apply_move(m);
      } else if chess_move.src == 60 && chess_move.dest == 58 {
        let m = Move {
          src: 56,
          dest: 59,
          promotion: NO_PIECE,
        };
        self.apply_move(m);
      }
    }

    // No apply the initial move
    if chess_move.promotion != NO_PIECE {
      self.squares[chess_move.dest as usize] = chess_move.promotion;
    } else {
      self.squares[chess_move.dest as usize] = self.squares[chess_move.src as usize];
    }

    self.squares[chess_move.src as usize] = NO_PIECE;
  }

  /// Checks if there is a piece on a square
  pub fn has_piece(&self, square: u8) -> bool {
    self.squares[square as usize] != NO_PIECE
  }

  /// Checks if there is a piece on a square
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

  /// Finds the square with a white king on it.
  pub fn get_black_white_square(&self) -> u8 {
    for i in 0..64 {
      if self.squares[i as usize] == WHITE_KING {
        return i;
      }
    }
    return INVALID_SQUARE;
  }

  /// Finds the square with a black king on it.
  pub fn get_black_king_square(&self) -> u8 {
    for i in 0..64 {
      if self.squares[i as usize] == BLACK_KING {
        return i;
      }
    }
    error!("No black king ?? ");
    return INVALID_SQUARE;
  }

  /// Finds the square with a white king on it.
  pub fn get_white_king_square(&self) -> u8 {
    for i in 0..64 {
      if self.squares[i as usize] == WHITE_KING {
        return i;
      }
    }
    error!("No white king ?? ");
    return INVALID_SQUARE;
  }

  /// Return a board bismask with squares set to 1 when they
  /// have a piece with a certain color
  pub fn get_color_mask(&self, color: Color) -> u64 {
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

  /// Converts first substring of a FEN (with the pieces) to a board
  pub fn from_string(string: &str) -> Self {
    let mut board = Board::new();
    let mut rank = 7;
    let mut file = 0;

    for c in string.chars() {
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

    board
  }

  /// Converts a board to the first part of a FEN.
  pub fn to_string(&self) -> String {
    let mut fen = String::new();
    let mut empty_squares = 0;
    for rank in (0..8 as u8).rev() {
      for file in 0..8 {
        match self.squares[(rank * 8 + file) as usize] {
          NO_PIECE => empty_squares += 1,
          WHITE_KING..=BLACK_PAWN => {
            if empty_squares > 0 {
              fen.push(char::from_digit(empty_squares, 10).unwrap());
              empty_squares = 0;
            }
            fen.push(Piece::u8_to_char(self.squares[(rank * 8 + file) as usize]).unwrap());
          },
          _ => {},
        }
      }
      if empty_squares > 0 {
        fen.push(char::from_digit(empty_squares, 10).unwrap());
        empty_squares = 0;
      }
      if rank != 0 {
        fen.push('/');
      }
    }

    fen
  }
}

impl Move {
  /// Initialize a board with no piece, all zeroes
  pub fn new() -> Self {
    Move {
      src: 0,
      dest: 0,
      promotion: NO_PIECE,
    }
  }

  /// Converts a move to the algebraic notation, e.g.
  pub fn to_string(&self) -> String {
    if self.promotion != NO_PIECE {
      let mut move_string = square_to_string(self.src) + &square_to_string(self.dest);
      move_string.push(
        Piece::u8_to_char(self.promotion)
          .expect("Should be a valid piece!")
          .to_ascii_uppercase(),
      );
      move_string
    } else {
      square_to_string(self.src) + &square_to_string(self.dest)
    }
  }

  /// Converts a move to the algebraic notation, e.g.
  pub fn from_string(move_notation: &str) -> Self {
    let src = string_to_square(&move_notation[0..2]);
    let dest = string_to_square(&move_notation[2..4]);
    let promotion;
    if move_notation.len() == 5 {
      promotion = Piece::char_to_u8(
        move_notation
          .chars()
          .nth(4)
          .expect("Invalid promoted piece ??"),
      )
      .expect("unexpected piece");
    } else {
      promotion = NO_PIECE;
    }

    Move {
      src,
      dest,
      promotion,
    }
  }
}

// -----------------------------------------------------------------------------
// Display implementations for our board/move

impl std::fmt::Display for Board {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut board = String::from("\n");
    for rank in (0..8 as u8).rev() {
      for file in 0..8 {
        board.push(Piece::u8_to_char(self.squares[(rank * 8 + file) as usize]).unwrap());
        board.push(' ');
      }
      board.push('\n');
    }
    f.write_str(board.as_str())
  }
}

impl std::fmt::Display for Move {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.to_string().as_str())
  }
}

impl Default for Move {
  fn default() -> Self {
    Move {
      src: 0,
      dest: 0,
      promotion: NO_PIECE,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn display_board() {
    let mut board = Board { squares: [0; 64] };
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
  fn square_to_string_test() {
    assert_eq!("a1", square_to_string(0));
    assert_eq!("b1", square_to_string(1));
    assert_eq!("c1", square_to_string(2));
    assert_eq!("d1", square_to_string(3));
    assert_eq!("e1", square_to_string(4));
    assert_eq!("f1", square_to_string(5));
    assert_eq!("g1", square_to_string(6));
    assert_eq!("h1", square_to_string(7));
    assert_eq!("a2", square_to_string(8));
    assert_eq!("b2", square_to_string(9));
    assert_eq!("c2", square_to_string(10));
    assert_eq!("d2", square_to_string(11));
    assert_eq!("e2", square_to_string(12));
    assert_eq!("f2", square_to_string(13));
    assert_eq!("g2", square_to_string(14));
    assert_eq!("h2", square_to_string(15));
    assert_eq!("a8", square_to_string(56));
    assert_eq!("b8", square_to_string(57));
    assert_eq!("c8", square_to_string(58));
    assert_eq!("d8", square_to_string(59));
    assert_eq!("e8", square_to_string(60));
    assert_eq!("f8", square_to_string(61));
    assert_eq!("g8", square_to_string(62));
    assert_eq!("h8", square_to_string(63));
  }

  #[test]
  fn string_to_square_test() {
    assert_eq!(0, string_to_square("a1"));
    assert_eq!(1, string_to_square("b1"));
    assert_eq!(2, string_to_square("c1"));
    assert_eq!(3, string_to_square("d1"));
    assert_eq!(4, string_to_square("e1"));
    assert_eq!(5, string_to_square("f1"));
    assert_eq!(6, string_to_square("g1"));
    assert_eq!(7, string_to_square("h1"));
    assert_eq!(8, string_to_square("a2"));
    assert_eq!(9, string_to_square("b2"));
    assert_eq!(10, string_to_square("c2"));
    assert_eq!(11, string_to_square("d2"));
    assert_eq!(12, string_to_square("e2"));
    assert_eq!(13, string_to_square("f2"));
    assert_eq!(14, string_to_square("g2"));
    assert_eq!(15, string_to_square("h2"));
    assert_eq!(56, string_to_square("a8"));
    assert_eq!(57, string_to_square("b8"));
    assert_eq!(58, string_to_square("c8"));
    assert_eq!(59, string_to_square("d8"));
    assert_eq!(60, string_to_square("e8"));
    assert_eq!(61, string_to_square("f8"));
    assert_eq!(62, string_to_square("g8"));
    assert_eq!(63, string_to_square("h8"));
  }

  #[test]
  fn from_string() {
    let mut board = Board::from_string("rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR");
    println!("Board: {}", board);

    let test_fen = "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2";
    board = Board::from_string(test_fen);
    println!("Board: {}", board);

    assert_eq!(
      test_fen.split(' ').collect::<Vec<_>>()[0],
      board.to_string()
    );

    let test_fen_2 = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    board = Board::from_string(test_fen_2);
    println!("Board: {}", board);

    assert_eq!(
      test_fen_2.split(' ').collect::<Vec<_>>()[0],
      board.to_string()
    )
  }

  #[test]
  fn apply_move() {
    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    let mut board = Board::from_string(fen);
    println!("Board: {}", board);

    // Try and capture a piece
    board.apply_move(Move {
      src: string_to_square("b3"),
      dest: string_to_square("g3"),
      promotion: NO_PIECE,
    });
    println!("Board: {}", board);

    // Try and promote a piece (super jump from h2 to h8)
    board.apply_move(Move {
      src: string_to_square("h2"),
      dest: string_to_square("h8"),
      promotion: WHITE_KNIGHT,
    });
    println!("Board: {}", board);
  }

  #[test]
  fn move_to_string() {
    let m = Move {
      src: 0,
      dest: 1,
      promotion: WHITE_BISHOP,
    };
    assert_eq!("a1b1B", m.to_string());

    let m = Move {
      src: 63,
      dest: 1,
      promotion: NO_PIECE,
    };
    assert_eq!("h8b1", m.to_string());
  }
}
