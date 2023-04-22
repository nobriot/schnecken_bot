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

// -----------------------------------------------------------------------------
//  Strucs/Enums

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum PieceType {
  King,
  Queen,
  Rook,
  Bishop,
  Knight,
  Pawn,
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Piece {
  p_type: PieceType,
  color: Color,
}

impl Piece {
  /// Indicates the material value of a piece.
  pub fn material_value(&self) -> f32 {
    match self.p_type {
      PieceType::King => 100.0,
      PieceType::Queen => 9.5,
      PieceType::Rook => 5.0,
      PieceType::Bishop => 3.0,
      PieceType::Knight => 3.0,
      PieceType::Pawn => 1.0,
    }
  }

  /// Indicates the material value of a piece.
  pub fn material_value_from_u8(piece: u8) -> f32 {
    match piece {
      WHITE_KING => 100.0,
      WHITE_QUEEN => 9.0,
      WHITE_ROOK => 5.0,
      WHITE_BISHOP => 3.05,
      WHITE_KNIGHT => 3.0,
      WHITE_PAWN => 1.0,
      BLACK_KING => -100.0,
      BLACK_QUEEN => -9.0,
      BLACK_ROOK => -5.0,
      BLACK_BISHOP => -3.05,
      BLACK_KNIGHT => -3.0,
      BLACK_PAWN => -1.0,
      _ => 0.0,
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
      (_, _) => NO_PIECE,
    }
  }

  /// Converts a u8 into a piece, reverse of as_u8.
  pub fn from_u8(value: u8) -> Result<Self, ()> {
    let piece: Piece;
    match value {
      WHITE_KING => {
        piece = Piece {
          p_type: PieceType::King,
          color: Color::White,
        };
      },
      WHITE_QUEEN => {
        piece = Piece {
          p_type: PieceType::Queen,
          color: Color::White,
        };
      },
      WHITE_ROOK => {
        piece = Piece {
          p_type: PieceType::Rook,
          color: Color::White,
        };
      },
      WHITE_BISHOP => {
        piece = Piece {
          p_type: PieceType::Bishop,
          color: Color::White,
        };
      },
      WHITE_KNIGHT => {
        piece = Piece {
          p_type: PieceType::Knight,
          color: Color::White,
        };
      },
      WHITE_PAWN => {
        piece = Piece {
          p_type: PieceType::Pawn,
          color: Color::White,
        };
      },
      BLACK_KING => {
        piece = Piece {
          p_type: PieceType::King,
          color: Color::Black,
        };
      },
      BLACK_QUEEN => {
        piece = Piece {
          p_type: PieceType::Queen,
          color: Color::Black,
        };
      },
      BLACK_ROOK => {
        piece = Piece {
          p_type: PieceType::Rook,
          color: Color::Black,
        };
      },
      BLACK_BISHOP => {
        piece = Piece {
          p_type: PieceType::Bishop,
          color: Color::Black,
        };
      },
      BLACK_KNIGHT => {
        piece = Piece {
          p_type: PieceType::Knight,
          color: Color::Black,
        };
      },
      BLACK_PAWN => {
        piece = Piece {
          p_type: PieceType::Pawn,
          color: Color::Black,
        };
      },
      _ => return Err(()),
    }
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
    let piece: Piece;
    match c {
      'K' => {
        piece = Piece {
          p_type: PieceType::King,
          color: Color::White,
        };
      },
      'Q' => {
        piece = Piece {
          p_type: PieceType::Queen,
          color: Color::White,
        };
      },
      'R' => {
        piece = Piece {
          p_type: PieceType::Rook,
          color: Color::White,
        };
      },
      'B' => {
        piece = Piece {
          p_type: PieceType::Bishop,
          color: Color::White,
        };
      },
      'N' => {
        piece = Piece {
          p_type: PieceType::Knight,
          color: Color::White,
        };
      },
      'P' => {
        piece = Piece {
          p_type: PieceType::Pawn,
          color: Color::White,
        };
      },
      'k' => {
        piece = Piece {
          p_type: PieceType::King,
          color: Color::Black,
        };
      },
      'q' => {
        piece = Piece {
          p_type: PieceType::Queen,
          color: Color::Black,
        };
      },
      'r' => {
        piece = Piece {
          p_type: PieceType::Rook,
          color: Color::Black,
        };
      },
      'b' => {
        piece = Piece {
          p_type: PieceType::Bishop,
          color: Color::Black,
        };
      },
      'n' => {
        piece = Piece {
          p_type: PieceType::Knight,
          color: Color::Black,
        };
      },
      'p' => {
        piece = Piece {
          p_type: PieceType::Pawn,
          color: Color::Black,
        };
      },
      _ => return Err(()),
    }
    Ok(piece)
  }

  /// Convenience function that takes a char and returns a u8, without converting
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

  /// Convenience function that takes a char and returns a u8, without converting
  /// to the intermediate Piece Struct
  ///
  /// This function also tolerates the absence of pieces,
  /// i.e. unknown u8 value convers to .
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

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn color_display_test() {
    let mut color = Color::Black;
    assert_eq!("Black", format!("{color}"));
    color = Color::White;
    assert_eq!("White", format!("{color}"));
  }
  #[test]
  fn piece_type_display_test() {
    let mut piece_type = PieceType::King;
    assert_eq!("King", format!("{piece_type}"));
    piece_type = PieceType::Queen;
    assert_eq!("Queen", format!("{piece_type}"));
    piece_type = PieceType::Rook;
    assert_eq!("Rook", format!("{piece_type}"));
    piece_type = PieceType::Bishop;
    assert_eq!("Bishop", format!("{piece_type}"));
    piece_type = PieceType::Knight;
    assert_eq!("Knight", format!("{piece_type}"));
    piece_type = PieceType::Pawn;
    assert_eq!("Pawn", format!("{piece_type}"));
  }

  #[test]
  fn piece_display_test() {
    let mut piece = Piece {
      p_type: PieceType::King,
      color: Color::White,
    };
    assert_eq!("K", format!("{piece}"));
    piece.p_type = PieceType::Queen;
    assert_eq!("Q", format!("{piece}"));
    piece.p_type = PieceType::Rook;
    assert_eq!("R", format!("{piece}"));
    piece.p_type = PieceType::Bishop;
    assert_eq!("B", format!("{piece}"));
    piece.p_type = PieceType::Knight;
    assert_eq!("N", format!("{piece}"));
    piece.p_type = PieceType::Pawn;
    assert_eq!("P", format!("{piece}"));

    piece.color = Color::Black;
    piece.p_type = PieceType::King;
    assert_eq!("k", format!("{piece}"));
    piece.p_type = PieceType::Queen;
    assert_eq!("q", format!("{piece}"));
    piece.p_type = PieceType::Rook;
    assert_eq!("r", format!("{piece}"));
    piece.p_type = PieceType::Bishop;
    assert_eq!("b", format!("{piece}"));
    piece.p_type = PieceType::Knight;
    assert_eq!("n", format!("{piece}"));
    piece.p_type = PieceType::Pawn;
    assert_eq!("p", format!("{piece}"));
  }

  #[test]
  fn piece_type_material_value() {
    let mut piece = Piece {
      p_type: PieceType::King,
      color: Color::White,
    };
    assert_eq!(100.0, piece.material_value());
    piece.p_type = PieceType::Queen;
    assert_eq!(9.5, piece.material_value());
    piece.p_type = PieceType::Rook;
    assert_eq!(5.0, piece.material_value());
    piece.p_type = PieceType::Bishop;
    assert_eq!(3.0, piece.material_value());
    piece.p_type = PieceType::Knight;
    assert_eq!(3.0, piece.material_value());
    piece.p_type = PieceType::Pawn;
    assert_eq!(1.0, piece.material_value());
  }

  #[test]
  fn piece_as_char() {
    let mut piece = Piece {
      p_type: PieceType::King,
      color: Color::White,
    };

    assert_eq!('K', piece.as_char());
    piece.p_type = PieceType::Queen;
    assert_eq!('Q', piece.as_char());
    piece.p_type = PieceType::Rook;
    assert_eq!('R', piece.as_char());
    piece.p_type = PieceType::Bishop;
    assert_eq!('B', piece.as_char());
    piece.p_type = PieceType::Knight;
    assert_eq!('N', piece.as_char());
    piece.p_type = PieceType::Pawn;
    assert_eq!('P', piece.as_char());

    piece.color = Color::Black;
    piece.p_type = PieceType::King;

    assert_eq!('k', piece.as_char());
    piece.p_type = PieceType::Queen;
    assert_eq!('q', piece.as_char());
    piece.p_type = PieceType::Rook;
    assert_eq!('r', piece.as_char());
    piece.p_type = PieceType::Bishop;
    assert_eq!('b', piece.as_char());
    piece.p_type = PieceType::Knight;
    assert_eq!('n', piece.as_char());
    piece.p_type = PieceType::Pawn;
    assert_eq!('p', piece.as_char());
  }

  #[test]
  fn piece_as_u8() {
    let mut piece = Piece {
      p_type: PieceType::King,
      color: Color::White,
    };

    assert_eq!(WHITE_KING, piece.as_u8());
    piece.p_type = PieceType::Queen;
    assert_eq!(WHITE_QUEEN, piece.as_u8());
    piece.p_type = PieceType::Rook;
    assert_eq!(WHITE_ROOK, piece.as_u8());
    piece.p_type = PieceType::Bishop;
    assert_eq!(WHITE_BISHOP, piece.as_u8());
    piece.p_type = PieceType::Knight;
    assert_eq!(WHITE_KNIGHT, piece.as_u8());
    piece.p_type = PieceType::Pawn;
    assert_eq!(WHITE_PAWN, piece.as_u8());

    piece.color = Color::Black;
    piece.p_type = PieceType::King;

    assert_eq!(BLACK_KING, piece.as_u8());
    piece.p_type = PieceType::Queen;
    assert_eq!(BLACK_QUEEN, piece.as_u8());
    piece.p_type = PieceType::Rook;
    assert_eq!(BLACK_ROOK, piece.as_u8());
    piece.p_type = PieceType::Bishop;
    assert_eq!(BLACK_BISHOP, piece.as_u8());
    piece.p_type = PieceType::Knight;
    assert_eq!(BLACK_KNIGHT, piece.as_u8());
    piece.p_type = PieceType::Pawn;
    assert_eq!(BLACK_PAWN, piece.as_u8());
  }

  #[test]
  fn full_loop_u8() {
    let mut initial_piece = Piece {
      p_type: PieceType::King,
      color: Color::White,
    };

    assert_eq!(
      initial_piece,
      Piece::from_u8(initial_piece.as_u8()).unwrap()
    );

    initial_piece.p_type = PieceType::Queen;
    assert_eq!(
      initial_piece,
      Piece::from_u8(initial_piece.as_u8()).unwrap()
    );
    initial_piece.p_type = PieceType::Rook;
    assert_eq!(
      initial_piece,
      Piece::from_u8(initial_piece.as_u8()).unwrap()
    );
    initial_piece.p_type = PieceType::Bishop;
    assert_eq!(
      initial_piece,
      Piece::from_u8(initial_piece.as_u8()).unwrap()
    );
    initial_piece.p_type = PieceType::Knight;
    assert_eq!(
      initial_piece,
      Piece::from_u8(initial_piece.as_u8()).unwrap()
    );
    initial_piece.p_type = PieceType::Pawn;
    assert_eq!(
      initial_piece,
      Piece::from_u8(initial_piece.as_u8()).unwrap()
    );

    initial_piece.p_type = PieceType::King;
    initial_piece.color = Color::Black;
    assert_eq!(
      initial_piece,
      Piece::from_u8(initial_piece.as_u8()).unwrap()
    );
    initial_piece.p_type = PieceType::Queen;
    assert_eq!(
      initial_piece,
      Piece::from_u8(initial_piece.as_u8()).unwrap()
    );
    initial_piece.p_type = PieceType::Rook;
    assert_eq!(
      initial_piece,
      Piece::from_u8(initial_piece.as_u8()).unwrap()
    );
    initial_piece.p_type = PieceType::Bishop;
    assert_eq!(
      initial_piece,
      Piece::from_u8(initial_piece.as_u8()).unwrap()
    );
    initial_piece.p_type = PieceType::Knight;
    assert_eq!(
      initial_piece,
      Piece::from_u8(initial_piece.as_u8()).unwrap()
    );
    initial_piece.p_type = PieceType::Pawn;
    assert_eq!(
      initial_piece,
      Piece::from_u8(initial_piece.as_u8()).unwrap()
    );
  }

  #[test]
  fn full_loop_char() {
    let mut initial_piece = Piece {
      p_type: PieceType::King,
      color: Color::White,
    };

    assert_eq!(
      initial_piece,
      Piece::from_char(initial_piece.as_char()).unwrap()
    );

    initial_piece.p_type = PieceType::Queen;
    assert_eq!(
      initial_piece,
      Piece::from_char(initial_piece.as_char()).unwrap()
    );
    initial_piece.p_type = PieceType::Rook;
    assert_eq!(
      initial_piece,
      Piece::from_char(initial_piece.as_char()).unwrap()
    );
    initial_piece.p_type = PieceType::Bishop;
    assert_eq!(
      initial_piece,
      Piece::from_char(initial_piece.as_char()).unwrap()
    );
    initial_piece.p_type = PieceType::Knight;
    assert_eq!(
      initial_piece,
      Piece::from_char(initial_piece.as_char()).unwrap()
    );
    initial_piece.p_type = PieceType::Pawn;
    assert_eq!(
      initial_piece,
      Piece::from_char(initial_piece.as_char()).unwrap()
    );

    initial_piece.p_type = PieceType::King;
    initial_piece.color = Color::Black;
    assert_eq!(
      initial_piece,
      Piece::from_char(initial_piece.as_char()).unwrap()
    );
    initial_piece.p_type = PieceType::Queen;
    assert_eq!(
      initial_piece,
      Piece::from_char(initial_piece.as_char()).unwrap()
    );
    initial_piece.p_type = PieceType::Rook;
    assert_eq!(
      initial_piece,
      Piece::from_char(initial_piece.as_char()).unwrap()
    );
    initial_piece.p_type = PieceType::Bishop;
    assert_eq!(
      initial_piece,
      Piece::from_char(initial_piece.as_char()).unwrap()
    );
    initial_piece.p_type = PieceType::Knight;
    assert_eq!(
      initial_piece,
      Piece::from_char(initial_piece.as_char()).unwrap()
    );
    initial_piece.p_type = PieceType::Pawn;
    assert_eq!(
      initial_piece,
      Piece::from_char(initial_piece.as_char()).unwrap()
    );
  }
}
