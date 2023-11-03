use crate::model::board::INVALID_SQUARE;
use crate::model::piece::*;

use log::*;
use std::ops::Shl;

// -----------------------------------------------------------------------------
//  Type definition
#[allow(non_camel_case_types)]
pub type move_t = u32;

// -----------------------------------------------------------------------------
//  Constants

/// Bitmask used to parse square data (0..63)
pub const SQUARE_MASK: move_t = 0b111111;
/// Bitmask used to parse promotion data (0..8), see `Promotion`
pub const PROMOTION_MASK: move_t = 0b01111;
/// Left bitshift to apply to the Move Data to get the promotion
pub const PROMOTION_SHIFT: move_t = 12;
/// Right bitshift to apply to the Move Data to get the Destination
pub const DESTINATION_SHIFT: move_t = 6;

/// Bit shift to apply to verify if the move is a capture
pub const CAPTURE_SHIFT: move_t = 16;
/// Bit shift to apply to verify if the move delivers check
pub const CHECK_SHIFT: move_t = 17;
/// Mask to apply to the number of checks
const CHECK_MASK: move_t = 0b11;

/// Bit shift to apply to verify if the move is marked as a castling move
pub const CASTLE_SHIFT: move_t = 19;
/// Bit shift to apply to verify if the move is marked as a en-passant move
pub const EN_PASSANT_SHIFT: move_t = 19;

// -----------------------------------------------------------------------------
//  Macros

/// Helper macro that creates a Move
///
/// Use like this for all parameters: `mv!(source, destination, promotion, is_capture, gives_check)`
///
/// Only 2 madatory parameters are source and destinations: `mv!(source, destination)`
///
/// ### Arguments
///
/// * `source`          Source square for the move : 0..63;
/// * `destination`     Destination square for the move : 0..63;
/// * `promotion`       Whether the move yields a promotion
/// * `is_capture`      bool value to indicate if this is a capture on the board it is applied.
/// * `gives_check`     bool value to indicate if this moves gives check on the board it is applied.
///
/// ### Returns
///
/// Move struct with the indicated data packed inside.
///
#[macro_export]
macro_rules! mv {
  // All parameters
  ($src:expr, $dest:expr, $prom:expr, $capture:expr, $check:expr) => {
    Move {
      data: $src as move_t
        | (($dest as move_t & SQUARE_MASK) << DESTINATION_SHIFT)
        | (($prom as move_t & PROMOTION_MASK) << PROMOTION_SHIFT)
        | (($capture as move_t & 1) << CAPTURE_SHIFT)
        | (($check as move_t & CHECK_MASK) << CHECK_SHIFT),
    }
  };
  // 4 parameters
  ($src:expr, $dest:expr, $prom:expr, $capture:expr) => {
    Move {
      data: $src as move_t
        | (($dest as move_t & SQUARE_MASK) << DESTINATION_SHIFT)
        | (($prom as move_t & PROMOTION_MASK) << PROMOTION_SHIFT)
        | (($capture as move_t & 1) << CAPTURE_SHIFT),
    }
  };

  // 3 parameters
  ($src:expr, $dest:expr, $prom:expr) => {
    Move {
      data: $src as move_t
        | (($dest as move_t & SQUARE_MASK) << DESTINATION_SHIFT)
        | (($prom as move_t & PROMOTION_MASK) << PROMOTION_SHIFT),
    }
  };
  // 2 parameters, just source and destination.
  ($src:expr, $dest:expr) => {
    Move {
      data: $src as move_t | (($dest as move_t & SQUARE_MASK) << DESTINATION_SHIFT),
    }
  };
}

/// Helper macro that creates a castle Move
///
/// Use like this parameters: `mv!(source, destination)`
///
/// ### Arguments
///
/// * `source`          Source square for the move : 0..63;
/// * `destination`     Destination square for the move : 0..63;
///
/// ### Returns
///
/// Move struct with the indicated data packed inside.
///
#[macro_export]
macro_rules! castle_mv {
  // All parameters
  ($src:expr, $dest:expr) => {
    Move {
      data: $src | (($dest as move_t & SQUARE_MASK) << DESTINATION_SHIFT) | (1 << CASTLE_SHIFT),
    }
  };
}

/// Helper macro that creates a en-passant Move
///
/// Use like this parameters: `mv!(source, destination)`
///
/// ### Arguments
///
/// * `source`          Source square for the move : 0..63;
/// * `destination`     Destination square for the move : 0..63;
///
/// ### Returns
///
/// Move struct with the indicated data packed inside.
///
#[macro_export]
macro_rules! en_passant_mv {
  // All parameters
  ($src:expr, $dest:expr) => {
    Move {
      data: $src as move_t
        | (($dest as move_t & SQUARE_MASK) << DESTINATION_SHIFT)
        | (1 << CAPTURE_SHIFT)
        | (1 << EN_PASSANT_SHIFT),
    }
  };
}

/// Helper macro that creates a Capture Move
///
/// Use like this parameters: `mv!(source, destination)`
///
/// ### Arguments
///
/// * `source`          Source square for the move : 0..63;
/// * `destination`     Destination square for the move : 0..63;
///
/// ### Returns
///
/// Move struct with the indicated data packed inside.
///
#[macro_export]
macro_rules! capture_mv {
  // All parameters
  ($src:expr, $dest:expr) => {
    Move {
      data: $src | (($dest as move_t & SQUARE_MASK) << DESTINATION_SHIFT) | (1 << CAPTURE_SHIFT),
    }
  };
}

pub use capture_mv;
pub use castle_mv;
pub use en_passant_mv;
pub use mv;

/// List of possible promotions in a chess game
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Promotion {
  #[default]
  NoPromotion = 0,
  WhiteQueen = 1,
  WhiteRook = 2,
  WhiteBishop = 3,
  WhiteKnight = 4,
  BlackQueen = 5,
  BlackRook = 6,
  BlackBishop = 7,
  BlackKnight = 8,
}

impl Promotion {
  /// Converts a promotion value to an optional character that can be
  /// used in move notations
  ///
  pub fn to_char(&self) -> Option<char> {
    match self {
      Promotion::NoPromotion => None,
      Promotion::WhiteQueen => Some('Q'),
      Promotion::WhiteRook => Some('R'),
      Promotion::WhiteBishop => Some('B'),
      Promotion::WhiteKnight => Some('N'),
      Promotion::BlackQueen => Some('q'),
      Promotion::BlackRook => Some('r'),
      Promotion::BlackBishop => Some('b'),
      Promotion::BlackKnight => Some('n'),
    }
  }

  /// Converts a character used in  value to an optional character that can be
  /// used in move notations
  ///
  pub fn from_char(promotion: char) -> Self {
    match promotion {
      'Q' => Promotion::WhiteQueen,
      'R' => Promotion::WhiteRook,
      'B' => Promotion::WhiteBishop,
      'N' => Promotion::WhiteKnight,
      'q' => Promotion::BlackQueen,
      'r' => Promotion::BlackRook,
      'b' => Promotion::BlackBishop,
      'n' => Promotion::BlackKnight,
      _ => Promotion::NoPromotion,
    }
  }

  pub fn to_piece_const(&self) -> u8 {
    match self {
      Promotion::NoPromotion => NO_PIECE,
      Promotion::WhiteQueen => WHITE_QUEEN,
      Promotion::WhiteRook => WHITE_ROOK,
      Promotion::WhiteBishop => WHITE_BISHOP,
      Promotion::WhiteKnight => WHITE_KNIGHT,
      Promotion::BlackQueen => BLACK_QUEEN,
      Promotion::BlackRook => BLACK_ROOK,
      Promotion::BlackBishop => BLACK_BISHOP,
      Promotion::BlackKnight => BLACK_KNIGHT,
    }
  }
}

// We want to be able to do a left shift directly on the Promotion enum,
// so that we can integrate it to the data without conversions.
// Data target is move_t, so we return a move_t already.
impl Shl<move_t> for Promotion {
  type Output = move_t;

  fn shl(self, rhs: move_t) -> Self::Output {
    return (unsafe { std::mem::transmute::<Promotion, u8>(self) } as move_t) << rhs;
  }
}

/// Represents a move on the board.
///
///
/// ### Fields
///
/// * `data`: Contains the source, destination and promotion
///
///
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct Move {
  /// Move data, representing source -> destination and an optional promotion.
  ///
  /// source mask         : 0b 0000 0000 0000 0000 0000 0000 0011 1111
  /// destination mask    : 0b 0000 0000 0000 0000 0000 1111 1100 0000
  /// promotion mask      : 0b 0000 0000 0000 0000 1111 0000 0000 0000
  /// is_capture mask     : 0b 0000 0000 0000 0001 0000 0000 0000 0000
  /// checks mask         : 0b 0000 0000 0000 0110 0000 0000 0000 0000
  /// is_casle mask       : 0b 0000 0000 0000 1000 0000 0000 0000 0000
  /// en_passant mask     : 0b 0000 0000 0001 0000 0000 0000 0000 0000
  ///
  /// Note that capture/gives_check depends on the board configuration and
  /// does not need to be exact in all use-cases.
  ///
  /// Use the helper functions rather than access this field directly.
  pub data: move_t,
}

impl Move {
  /// Returns the source square of a move, will be in the range 0..63
  #[inline]
  pub fn src(&self) -> move_t {
    self.data & SQUARE_MASK
  }

  /// Returns the source square of a move, will be in the range 0..63
  #[inline]
  pub fn u8_src(&self) -> u8 {
    (self.data & SQUARE_MASK) as u8
  }

  /// Returns the destination square of a move, will be in the range 0..63
  #[inline]
  pub fn dest(&self) -> move_t {
    (self.data >> DESTINATION_SHIFT) & SQUARE_MASK
  }

  /// Returns the destination square of a move, will be in the range 0..63
  #[inline]
  pub fn u8_dest(&self) -> u8 {
    ((self.data >> DESTINATION_SHIFT) & SQUARE_MASK) as u8
  }

  /// Returns the promotion value for a move, it's to say the piece that will spawn
  /// on the destination square
  ///
  #[inline]
  pub fn promotion(&self) -> Promotion {
    unsafe { std::mem::transmute(((self.data >> PROMOTION_SHIFT) & PROMOTION_MASK) as u8) }
  }

  /// Returns true if the move has been marked to be a capture.
  /// This depends on the board, and moves generated e.g. from a notation
  /// may not have accurate information here.
  ///
  #[inline]
  pub fn is_capture(&self) -> bool {
    (self.data >> CAPTURE_SHIFT) != 0
  }

  /// Returns the number of checks if the move has been marked to give checks
  /// This depends on the board, and moves generated e.g. from a notation
  /// may not have accurate information here.
  ///
  #[inline]
  pub fn gives_check(&self) -> move_t {
    (self.data >> CHECK_SHIFT) & CHECK_MASK
  }

  /// Returns whether the move is a castling move or not
  /// This depends on the board, and moves generated e.g. from a notation
  /// may not have accurate information here.
  ///
  #[inline]
  pub fn is_castle(&self) -> bool {
    (self.data >> CASTLE_SHIFT) != 0
  }

  /// Returns whether the move is a en-passant move or not
  /// This depends on the board, and moves generated e.g. from a notation
  /// may not have accurate information here.
  ///
  #[inline]
  pub fn is_en_passant(&self) -> bool {
    (self.data >> EN_PASSANT_SHIFT) != 0
  }
}

impl std::fmt::Display for Move {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.to_string().as_str())
  }
}

impl Default for Move {
  fn default() -> Self {
    Move { data: 0 }
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
    let promotion = self.promotion();

    match promotion {
      Promotion::NoPromotion => {
        square_to_string(self.src() as u8) + &square_to_string(self.dest() as u8)
      },
      _ => {
        let mut move_string =
          square_to_string(self.src() as u8) + &square_to_string(self.dest() as u8);
        move_string.push(Promotion::to_char(&promotion).expect("Should be a valid piece!"));
        move_string
      },
    }
  }

  /// Converts a move to the algebraic notation, e.g.
  pub fn from_string(move_notation: &str) -> Self {
    let dest: move_t = string_to_square(&move_notation[2..4]) as move_t;

    let mut promotion = if move_notation.len() == 5 {
      Promotion::from_char(
        move_notation
          .chars()
          .nth(4)
          .expect("Invalid promoted piece ??"),
      )
    } else {
      Promotion::NoPromotion
    };

    // By default the notation has small letter, so it will produce black pieces.
    // Change here the black piece into white piece if the destination rank is the 8th
    if (dest / 8) == 7 && promotion != Promotion::NoPromotion {
      match promotion {
        Promotion::BlackQueen => promotion = Promotion::WhiteQueen,
        Promotion::BlackRook => promotion = Promotion::WhiteRook,
        Promotion::BlackBishop => promotion = Promotion::WhiteBishop,
        Promotion::BlackKnight => promotion = Promotion::WhiteKnight,
        _ => {},
      }
    }

    mv!(string_to_square(&move_notation[0..2]), dest, promotion)
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
    let m = mv!(0, 1, Promotion::WhiteBishop);
    assert_eq!("a1b1B", m.to_string());

    let m = mv!(63, 1);
    assert_eq!("h8b1", m.to_string());

    let m = mv!(9, 1, Promotion::BlackQueen);
    assert_eq!("b2b1q", m.to_string());
    assert_eq!(m, Move::from_string(m.to_string().as_str()));
  }

  #[test]
  fn vec_to_string() {
    let mut vec = Vec::new();
    vec.push(mv!(0, 1, Promotion::WhiteBishop));
    vec.push(mv!(63, 0));

    assert_eq!("a1b1B h8a1", Move::vec_to_string(&vec));
  }

  #[test]
  fn string_to_vec() {
    let moves = "a1b1B h8a1";
    let vec = Move::string_to_vec(moves);
    let m0 = mv!(0, 1, Promotion::WhiteBishop);
    let m1 = mv!(63, 0);
    assert_eq!(vec[0], m0);
    assert_eq!(vec[1], m1);
  }
}
