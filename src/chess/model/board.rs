use crate::chess::model::piece::*;
#[derive(Debug, Clone, Copy)]
pub struct Board {
  squares: [u8; 64],
}

// -----------------------------------------------------------------------------
// Implementations

impl Board {
  /// Initialize a board with no piece, all zeroes
  fn new() -> Self {
    Board { squares: [0u8; 64] }
  }

  /// Converts the square number (from 0 to 63) to an algebraic notation
  /// such as a1, h7, f4
  ///
  /// We put zeroes ('00') when a value does not exist.
  fn square_to_string(square: u8) -> String {
    let mut string = String::new();

    match square / 8 {
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
    match square % 8 {
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
  fn string_to_square(string: &str) -> u8 {
    let mut square_value: u8 = 0;
    match string.chars().nth(0) {
      // a does not add value to the square index
      Some('b') => square_value = 1 * 8,
      Some('c') => square_value = 2 * 8,
      Some('d') => square_value = 3 * 8,
      Some('e') => square_value = 4 * 8,
      Some('f') => square_value = 5 * 8,
      Some('g') => square_value = 6 * 8,
      Some('h') => square_value = 7 * 8,
      Some(_) | None => {},
    }
    match string.chars().nth(1) {
      Some('2') => square_value += 1,
      Some('3') => square_value += 2,
      Some('4') => square_value += 3,
      Some('5') => square_value += 4,
      Some('6') => square_value += 5,
      Some('7') => square_value += 6,
      Some('8') => square_value += 7,
      Some(_) | None => {},
    }

    square_value
  }

  /// Converts first substring of a FEN (with the pieces) to a board
  fn string_to_board(string: &str) -> Self {
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
  fn to_string(&self) -> String {
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

// -----------------------------------------------------------------------------
// Display implementations for our board

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
  fn square_to_string() {
    assert_eq!("a1", Board::square_to_string(0));
    assert_eq!("a2", Board::square_to_string(1));
    assert_eq!("a3", Board::square_to_string(2));
    assert_eq!("a4", Board::square_to_string(3));
    assert_eq!("a5", Board::square_to_string(4));
    assert_eq!("a6", Board::square_to_string(5));
    assert_eq!("a7", Board::square_to_string(6));
    assert_eq!("a8", Board::square_to_string(7));
    assert_eq!("b1", Board::square_to_string(8));
    assert_eq!("b2", Board::square_to_string(9));
    assert_eq!("b3", Board::square_to_string(10));
    assert_eq!("b4", Board::square_to_string(11));
    assert_eq!("b5", Board::square_to_string(12));
    assert_eq!("b6", Board::square_to_string(13));
    assert_eq!("b7", Board::square_to_string(14));
    assert_eq!("b8", Board::square_to_string(15));
    assert_eq!("h1", Board::square_to_string(56));
    assert_eq!("h2", Board::square_to_string(57));
    assert_eq!("h3", Board::square_to_string(58));
    assert_eq!("h4", Board::square_to_string(59));
    assert_eq!("h5", Board::square_to_string(60));
    assert_eq!("h6", Board::square_to_string(61));
    assert_eq!("h7", Board::square_to_string(62));
    assert_eq!("h8", Board::square_to_string(63));
  }

  #[test]
  fn string_to_square() {
    assert_eq!(0, Board::string_to_square("a1"));
    assert_eq!(1, Board::string_to_square("a2"));
    assert_eq!(2, Board::string_to_square("a3"));
    assert_eq!(3, Board::string_to_square("a4"));
    assert_eq!(4, Board::string_to_square("a5"));
    assert_eq!(5, Board::string_to_square("a6"));
    assert_eq!(6, Board::string_to_square("a7"));
    assert_eq!(7, Board::string_to_square("a8"));
    assert_eq!(8, Board::string_to_square("b1"));
    assert_eq!(9, Board::string_to_square("b2"));
    assert_eq!(10, Board::string_to_square("b3"));
    assert_eq!(11, Board::string_to_square("b4"));
    assert_eq!(12, Board::string_to_square("b5"));
    assert_eq!(13, Board::string_to_square("b6"));
    assert_eq!(14, Board::string_to_square("b7"));
    assert_eq!(15, Board::string_to_square("b8"));
    assert_eq!(56, Board::string_to_square("h1"));
    assert_eq!(57, Board::string_to_square("h2"));
    assert_eq!(58, Board::string_to_square("h3"));
    assert_eq!(59, Board::string_to_square("h4"));
    assert_eq!(60, Board::string_to_square("h5"));
    assert_eq!(61, Board::string_to_square("h6"));
    assert_eq!(62, Board::string_to_square("h7"));
    assert_eq!(63, Board::string_to_square("h8"));
  }

  #[test]
  fn string_to_board() {
    let mut board = Board::string_to_board("rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR");
    println!("Board: {}", board);

    let test_fen = "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2";
    board = Board::string_to_board(test_fen);
    println!("Board: {}", board);

    assert_eq!(
      test_fen.split(' ').collect::<Vec<&str>>()[0],
      board.to_string()
    );

    let test_fen_2 = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    board = Board::string_to_board(test_fen_2);
    println!("Board: {}", board);

    assert_eq!(
      test_fen_2.split(' ').collect::<Vec<&str>>()[0],
      board.to_string()
    )
  }
}
