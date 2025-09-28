use serde::{Deserialize, Serialize};

// -----------------------------------------------------------------------------
//  Constants

/// Numerical value used on the board to represent no piece.
pub const NO_PIECE: u8 = 0;
/// Numerical value used on the board to represent a White King
pub const WHITE_KING: u8 = 1;
/// Numerical value used on the board to represent a White Queen
pub const WHITE_QUEEN: u8 = 2;
/// Numerical value used on the board to represent a White Rook
pub const WHITE_ROOK: u8 = 3;
/// Numerical value used on the board to represent a White Bishop
pub const WHITE_BISHOP: u8 = 4;
/// Numerical value used on the board to represent a White Knight
pub const WHITE_KNIGHT: u8 = 5;
/// Numerical value used on the board to represent a White Pawn
pub const WHITE_PAWN: u8 = 6;
/// Numerical value used on the board to represent a Black King
pub const BLACK_KING: u8 = 7;
/// Numerical value used on the board to represent a Black Queen
pub const BLACK_QUEEN: u8 = 8;
/// Numerical value used on the board to represent a Black Rook
pub const BLACK_ROOK: u8 = 9;
/// Numerical value used on the board to represent a Black Bishop
pub const BLACK_BISHOP: u8 = 10;
/// Numerical value used on the board to represent a Black Knight
pub const BLACK_KNIGHT: u8 = 11;
/// Numerical value used on the board to represent a Black Pawn
pub const BLACK_PAWN: u8 = 12;

// Piece values
pub const KING_VALUE: f32 = 100.0;
pub const QUEEN_VALUE: f32 = 9.5;
pub const ROOK_VALUE: f32 = 5.0;
pub const BISHOP_VALUE: f32 = 3.05;
pub const KNIGHT_VALUE: f32 = 3.0;
pub const PAWN_VALUE: f32 = 1.0;

// -----------------------------------------------------------------------------
//  Strucs/Enums

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum Color {
  White,
  Black,
}

impl Color {
  pub fn opposite(color: Self) -> Self {
    match color {
      Color::Black => Color::White,
      Color::White => Color::Black,
    }
  }

  /// Returns -1.0 for black color and +1.0 for white, which can be used in
  /// evaluations
  pub fn score_factor(color: Self) -> f32 {
    match color {
      Color::Black => -1.0,
      Color::White => 1.0,
    }
  }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum PieceType {
  #[default]
  King,
  Queen,
  Rook,
  Bishop,
  Knight,
  Pawn,
}

impl PieceType {
  /// Looks at the u8 value of a piece and converts it into a piece type
  ///
  /// ### Arguments:
  ///
  /// * value: u8 value of a piece, e.g. `BLACK_KING`, `WHITE_PAWN`, etc.
  ///
  /// ### Return value
  ///
  /// PieceType matching the u8 value. /!\ Return PieceType King by default
  /// if the u8 value is invalid.
  ///
  pub fn from_u8(value: u8) -> PieceType {
    match value {
      WHITE_QUEEN | BLACK_QUEEN => PieceType::Queen,
      WHITE_ROOK | BLACK_ROOK => PieceType::Rook,
      WHITE_BISHOP | BLACK_BISHOP => PieceType::Bishop,
      WHITE_KNIGHT | BLACK_KNIGHT => PieceType::Knight,
      WHITE_PAWN | BLACK_PAWN => PieceType::Pawn,
      _ => PieceType::King,
    }
  }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Piece {
  pub p_type: PieceType,
  pub color: Color,
}

impl Piece {
  /// Indicates the material value of a piece.
  ///
  /// ### Arguments
  ///
  /// u8 value representing a piece, i.e. `WHITE_KING`, `WHITE_QUEEN`, etc.
  ///
  /// ### Return value
  ///
  /// f32 value assigned to the material value of the piece. 0 if the piece is invalid.
  /// Values are always positive, also for black pieces
  pub fn material_value(&self) -> f32 {
    match self.p_type {
      PieceType::King => KING_VALUE,
      PieceType::Queen => QUEEN_VALUE,
      PieceType::Rook => ROOK_VALUE,
      PieceType::Bishop => BISHOP_VALUE,
      PieceType::Knight => KNIGHT_VALUE,
      PieceType::Pawn => PAWN_VALUE,
    }
  }

  /// Indicates the material value (absolute value) of a piece.
  ///
  /// ### Arguments
  ///
  /// u8 value representing a piece, i.e. `WHITE_KING`, `WHITE_QUEEN`, etc.
  ///
  /// ### Return value
  ///
  /// f32 value assigned to the material value of the piece. 0 if the piece is invalid.
  /// Values are always positive, also for black pieces
  ///
  pub fn material_value_from_u8(piece: u8) -> f32 {
    match piece {
      WHITE_KING => KING_VALUE,
      WHITE_QUEEN => QUEEN_VALUE,
      WHITE_ROOK => ROOK_VALUE,
      WHITE_BISHOP => BISHOP_VALUE,
      WHITE_KNIGHT => KNIGHT_VALUE,
      WHITE_PAWN => PAWN_VALUE,
      BLACK_KING => KING_VALUE,
      BLACK_QUEEN => QUEEN_VALUE,
      BLACK_ROOK => ROOK_VALUE,
      BLACK_BISHOP => BISHOP_VALUE,
      BLACK_KNIGHT => KNIGHT_VALUE,
      BLACK_PAWN => PAWN_VALUE,
      _ => 0.0,
    }
  }

  /// Indicates the material value (absolute value) of a piece.
  ///
  /// ### Arguments
  ///
  /// * `piece_type`: PieceType value representing a piece, i.e. `WHITE_KING`, `WHITE_QUEEN`, etc.
  ///
  /// ### Return value
  ///
  /// f32 value assigned to the material value of the piece.
  ///
  pub fn material_value_from_type(piece_type: PieceType) -> f32 {
    match piece_type {
      PieceType::King => KING_VALUE,
      PieceType::Queen => QUEEN_VALUE,
      PieceType::Rook => ROOK_VALUE,
      PieceType::Bishop => BISHOP_VALUE,
      PieceType::Knight => KNIGHT_VALUE,
      PieceType::Pawn => PAWN_VALUE,
    }
  }

  /// Converts a piece into a u8, that can be used to represents on an array
  /// of squares for the chess board.
  ///
  pub fn as_u8(&self) -> u8 {
    match (self.p_type, self.color) {
      (PieceType::King, Color::White) => WHITE_KING,
      (PieceType::Queen, Color::White) => WHITE_QUEEN,
      (PieceType::Rook, Color::White) => WHITE_ROOK,
      (PieceType::Bishop, Color::White) => WHITE_BISHOP,
      (PieceType::Knight, Color::White) => WHITE_KNIGHT,
      (PieceType::Pawn, Color::White) => WHITE_PAWN,
      (PieceType::King, Color::Black) => BLACK_KING,
      (PieceType::Queen, Color::Black) => BLACK_QUEEN,
      (PieceType::Rook, Color::Black) => BLACK_ROOK,
      (PieceType::Bishop, Color::Black) => BLACK_BISHOP,
      (PieceType::Knight, Color::Black) => BLACK_KNIGHT,
      (PieceType::Pawn, Color::Black) => BLACK_PAWN,
    }
  }

  /// Converts a u8 into a piece, reverse of as_u8.
  pub fn from_u8(value: u8) -> Result<Self, ()> {
    let piece: Piece = match value {
      WHITE_KING => Piece {
        p_type: PieceType::King,
        color: Color::White,
      },
      WHITE_QUEEN => Piece {
        p_type: PieceType::Queen,
        color: Color::White,
      },
      WHITE_ROOK => Piece {
        p_type: PieceType::Rook,
        color: Color::White,
      },
      WHITE_BISHOP => Piece {
        p_type: PieceType::Bishop,
        color: Color::White,
      },
      WHITE_KNIGHT => Piece {
        p_type: PieceType::Knight,
        color: Color::White,
      },
      WHITE_PAWN => Piece {
        p_type: PieceType::Pawn,
        color: Color::White,
      },
      BLACK_KING => Piece {
        p_type: PieceType::King,
        color: Color::Black,
      },
      BLACK_QUEEN => Piece {
        p_type: PieceType::Queen,
        color: Color::Black,
      },
      BLACK_ROOK => Piece {
        p_type: PieceType::Rook,
        color: Color::Black,
      },
      BLACK_BISHOP => Piece {
        p_type: PieceType::Bishop,
        color: Color::Black,
      },
      BLACK_KNIGHT => Piece {
        p_type: PieceType::Knight,
        color: Color::Black,
      },
      BLACK_PAWN => Piece {
        p_type: PieceType::Pawn,
        color: Color::Black,
      },
      _ => return Err(()),
    };
    Ok(piece)
  }

  /// Converts a piece into a char, that can be used for FEN format.
  ///
  /// - white king:   K
  /// - white queen:  Q
  /// - white rook:   R
  /// - white bishop: B
  /// - white knight: N
  /// - white pawn:   P
  /// - black king:   k
  /// - black queen:  q
  /// - black rook:   r
  /// - black bishop: b
  /// - black knight: n
  /// - black pawn:   p
  pub fn as_char(&self) -> char {
    match (self.p_type, self.color) {
      (PieceType::King, Color::White) => 'K',
      (PieceType::Queen, Color::White) => 'Q',
      (PieceType::Rook, Color::White) => 'R',
      (PieceType::Bishop, Color::White) => 'B',
      (PieceType::Knight, Color::White) => 'N',
      (PieceType::Pawn, Color::White) => 'P',
      (PieceType::King, Color::Black) => 'k',
      (PieceType::Queen, Color::Black) => 'q',
      (PieceType::Rook, Color::Black) => 'r',
      (PieceType::Bishop, Color::Black) => 'b',
      (PieceType::Knight, Color::Black) => 'n',
      (PieceType::Pawn, Color::Black) => 'p',
    }
  }

  pub fn from_char(c: char) -> Result<Self, ()> {
    let piece: Piece = match c {
      'K' => Piece {
        p_type: PieceType::King,
        color: Color::White,
      },
      'Q' => Piece {
        p_type: PieceType::Queen,
        color: Color::White,
      },
      'R' => Piece {
        p_type: PieceType::Rook,
        color: Color::White,
      },
      'B' => Piece {
        p_type: PieceType::Bishop,
        color: Color::White,
      },
      'N' => Piece {
        p_type: PieceType::Knight,
        color: Color::White,
      },
      'P' => Piece {
        p_type: PieceType::Pawn,
        color: Color::White,
      },
      'k' => Piece {
        p_type: PieceType::King,
        color: Color::Black,
      },
      'q' => Piece {
        p_type: PieceType::Queen,
        color: Color::Black,
      },
      'r' => Piece {
        p_type: PieceType::Rook,
        color: Color::Black,
      },
      'b' => Piece {
        p_type: PieceType::Bishop,
        color: Color::Black,
      },
      'n' => Piece {
        p_type: PieceType::Knight,
        color: Color::Black,
      },
      'p' => Piece {
        p_type: PieceType::Pawn,
        color: Color::Black,
      },
      _ => return Err(()),
    };
    Ok(piece)
  }

  /// Convenience function that takes a u8 and returns a char, without converting
  /// to the intermediate Piece Struct
  ///
  /// This function also tolerates the absence of pieces,
  /// i.e. unknown u8 value convers to .
  pub fn u8_to_char(u: u8) -> Result<char, u8> {
    match u {
      WHITE_KING => Ok('K'),
      WHITE_QUEEN => Ok('Q'),
      WHITE_ROOK => Ok('R'),
      WHITE_BISHOP => Ok('B'),
      WHITE_KNIGHT => Ok('N'),
      WHITE_PAWN => Ok('P'),
      BLACK_KING => Ok('k'),
      BLACK_QUEEN => Ok('q'),
      BLACK_ROOK => Ok('r'),
      BLACK_BISHOP => Ok('b'),
      BLACK_KNIGHT => Ok('n'),
      BLACK_PAWN => Ok('p'),
      NO_PIECE => Ok('.'),
      _ => {
        println!("Offending: {u}");
        Err(u)
      },
    }
  }

  /// Convenience function that takes a u8 and returns color, without converting
  /// to the intermediate Piece Struct
  ///
  /// This function also tolerates the absence of pieces
  pub fn color(u: u8) -> Option<Color> {
    match u {
      WHITE_KING | WHITE_QUEEN | WHITE_ROOK | WHITE_BISHOP | WHITE_KNIGHT | WHITE_PAWN => {
        Some(Color::White)
      },
      BLACK_KING | BLACK_QUEEN | BLACK_ROOK | BLACK_BISHOP | BLACK_KNIGHT | BLACK_PAWN => {
        Some(Color::Black)
      },
      _ => None,
    }
  }

  /// Convenience function that determines if a char is a piece (Rook, Bishop, Knight or Queen)
  ///
  /// /// # Arguments
  ///
  /// * `u` - Value of the square on the board
  ///
  /// # Return value
  ///
  /// True if is it a piece, false in any other case
  ///
  pub fn is_piece(u: u8) -> bool {
    matches!(
      u,
      WHITE_QUEEN
        | WHITE_ROOK
        | WHITE_BISHOP
        | WHITE_KNIGHT
        | BLACK_QUEEN
        | BLACK_ROOK
        | BLACK_BISHOP
        | BLACK_KNIGHT
    )
  }

  /// Convenience function that takes a u8 and returns a char, without converting
  /// to the intermediate Piece Struct
  ///
  /// This function also tolerates the absence of pieces,
  /// i.e. unknown char value convers to 0
  pub fn char_to_u8(c: char) -> Result<u8, char> {
    match c {
      '.' => Ok(NO_PIECE),
      'K' => Ok(WHITE_KING),
      'Q' => Ok(WHITE_QUEEN),
      'R' => Ok(WHITE_ROOK),
      'B' => Ok(WHITE_BISHOP),
      'N' => Ok(WHITE_KNIGHT),
      'P' => Ok(WHITE_PAWN),
      'k' => Ok(BLACK_KING),
      'q' => Ok(BLACK_QUEEN),
      'r' => Ok(BLACK_ROOK),
      'b' => Ok(BLACK_BISHOP),
      'n' => Ok(BLACK_KNIGHT),
      'p' => Ok(BLACK_PAWN),
      _ => Err(c),
    }
  }
}

// -----------------------------------------------------------------------------
// Display implementations for our types

impl std::fmt::Display for Color {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match *self {
      Color::Black => f.write_str("Black"),
      Color::White => f.write_str("White"),
    }
  }
}

impl std::fmt::Display for PieceType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match *self {
      PieceType::King => f.write_str("King"),
      PieceType::Queen => f.write_str("Queen"),
      PieceType::Rook => f.write_str("Rook"),
      PieceType::Bishop => f.write_str("Bishop"),
      PieceType::Knight => f.write_str("Knight"),
      PieceType::Pawn => f.write_str("Pawn"),
    }
  }
}

impl std::fmt::Display for Piece {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.as_char().to_string().as_str())
  }
}
