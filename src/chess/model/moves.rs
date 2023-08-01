use crate::chess::model::board::INVALID_SQUARE;
use crate::chess::model::piece::*;

use log::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Move {
  /// Source square of a move (value from 0 to 63)
  pub src: u8,
  /// Destination square of a move (value from 0 to 63)
  pub dest: u8,
  /// Piece to spawn in case of promotion. Encoded using piece constants (NO_PIECE, WHITE_QUEEN, etc.)
  pub promotion: u8,
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

// -----------------------------------------------------------------------------
// Generic functions

/// Converts the square number (from 0 to 63) to an algebraic notation
/// such as a1, h7, f4
///
/// We put zeroes ('00') when a value does not exist.
pub fn square_to_string(square: u8) -> String {
  let mut string = String::new();

  if square == INVALID_SQUARE {
    error!("Cannot convert invalid square to string");
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
    Some('2') => square_value += 8,
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

impl Move {
  /// Converts a move to the algebraic notation, e.g. a3f3
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
    let mut promotion;
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

    // By default the notation has small letter, so it will produce black pieces.
    // Change here the black piece into white piece if the destination rank is the 8th
    if (dest / 8) == 7 && promotion != NO_PIECE {
      match promotion {
        BLACK_QUEEN => promotion = WHITE_QUEEN,
        BLACK_ROOK => promotion = WHITE_ROOK,
        BLACK_BISHOP => promotion = WHITE_BISHOP,
        BLACK_KNIGHT => promotion = WHITE_KNIGHT,
        _ => {},
      }
    }

    Move {
      src,
      dest,
      promotion,
    }
  }

  pub fn vec_to_string(move_list: &Vec<Move>) -> String {
    let mut string: String = String::new();
    for &chess_move in move_list {
      string += &chess_move.to_string();
      string.push(' ');
    }
    string.pop();
    string
  }

  pub fn string_to_vec(string: &str) -> Vec<Move> {
    let move_list: Vec<&str> = string.split(' ').collect();
    let mut chess_moves: Vec<Move> = Vec::new();
    for move_string in move_list {
      if !move_string.is_empty() {
        chess_moves.push(Move::from_string(move_string));
      }
    }
    chess_moves
  }
}

// -----------------------------------------------------------------------------
// Tests

#[cfg(test)]
mod tests {
  use super::*;

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

  #[test]
  fn vec_to_string() {
    let mut vec = Vec::new();
    vec.push(Move {
      src: 0,
      dest: 1,
      promotion: WHITE_BISHOP,
    });
    vec.push(Move {
      src: 63,
      dest: 0,
      promotion: NO_PIECE,
    });

    assert_eq!("a1b1B h8a1", Move::vec_to_string(&vec));
  }

  #[test]
  fn string_to_vec() {
    let moves = "a1b1B h8a1";
    let vec = Move::string_to_vec(moves);
    let m0 = Move {
      src: 0,
      dest: 1,
      promotion: WHITE_BISHOP,
    };
    let m1 = Move {
      src: 63,
      dest: 0,
      promotion: NO_PIECE,
    };
    assert_eq!(vec[0], m0);
    assert_eq!(vec[1], m1);
  }


}
