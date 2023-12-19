use log::*;

// From our libraries
use crate::model::board_mask::*;
use crate::model::piece::*;

/// Set of masks associated to each possible piece on a chess board.
/// These masks can be used to show what squares are occupied or controlled by
/// the pieces
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct PieceMasks {
  // Those 2 private fields are used to iterate over pieces.
  iter_type: Option<PieceType>,
  iter_mask: BoardMask,
  // List of king squares on the board
  pub king: BoardMask,
  // List of queen squares on the board
  pub queen: BoardMask,
  // List of rook squares on the board
  pub rook: BoardMask,
  // List of bishop squares on the board
  pub bishop: BoardMask,
  // List of knight squares on the board
  pub knight: BoardMask,
  // List of knight squares on the board
  pub pawn: BoardMask,
}

// -----------------------------------------------------------------------------
//  Constants
pub const WHITE_KING_START_SQUARE: usize = 4;
pub const BLACK_KING_START_SQUARE: usize = 60;

pub const WHITE_PIECES_DEFAULT_POSITIONS: PieceMasks = PieceMasks {
  iter_type: None,
  iter_mask: 0,
  king: 1 << WHITE_KING_START_SQUARE,
  queen: 1 << 3,
  rook: 1 | 1 << 7,
  bishop: 1 << 2 | 1 << 5,
  knight: 1 << 1 | 1 << 6,
  pawn: 1 << 8 | 1 << 9 | 1 << 10 | 1 << 11 | 1 << 12 | 1 << 13 | 1 << 14 | 1 << 15,
};

pub const BLACK_PIECES_DEFAULT_POSITIONS: PieceMasks = PieceMasks {
  iter_type: None,
  iter_mask: 0,
  king: 1 << BLACK_KING_START_SQUARE,
  queen: 1 << 59,
  rook: 1 << 56 | 1 << 63,
  bishop: 1 << 58 | 1 << 61,
  knight: 1 << 57 | 1 << 62,
  pawn: 1 << 48 | 1 << 49 | 1 << 50 | 1 << 51 | 1 << 52 | 1 << 53 | 1 << 54 | 1 << 55,
};

// -----------------------------------------------------------------------------
//  Implementations
impl PieceMasks {
  /// Initialize a PieceMasks with all zeroes
  fn new() -> Self {
    PieceMasks {
      iter_type: None,
      iter_mask: 0,
      king: 0,
      queen: 0,
      rook: 0,
      bishop: 0,
      knight: 0,
      pawn: 0,
    }
  }

  /// Initialize a PieceMasks with squares corresponding to the default start
  /// position for white pieces
  ///
  /// ### Return value
  ///
  /// A piece set with pieces set on the start position for White.
  ///
  #[inline]
  pub fn default_white_piece_set() -> Self {
    WHITE_PIECES_DEFAULT_POSITIONS
  }

  /// Initialize a PieceMasks with squares corresponding to the default start
  /// position for black pieces
  ///
  /// ### Return value
  ///
  /// A piece set with pieces set on the start position for Black.
  ///
  #[inline]
  pub fn default_black_piece_set() -> Self {
    BLACK_PIECES_DEFAULT_POSITIONS
  }

  /// Add a piece on a given square. Updates the piece if another piece was
  /// already present.
  ///
  /// ### Arguments
  ///
  /// * `self`    Mutable reference to the PieceMasks object being modified
  /// * `square`  Square on which no piece must be present.
  /// * `piece`   Piece to add on the square. E.g. PieceType::Queen
  ///
  #[inline]
  pub fn add(&mut self, square: u8, piece: PieceType) {
    match piece {
      PieceType::King => {
        set_square_in_mask!(square, self.king);
        unset_square_in_mask!(square, self.queen);
        unset_square_in_mask!(square, self.rook);
        unset_square_in_mask!(square, self.bishop);
        unset_square_in_mask!(square, self.knight);
        unset_square_in_mask!(square, self.pawn);
      },
      PieceType::Queen => {
        unset_square_in_mask!(square, self.king);
        set_square_in_mask!(square, self.queen);
        unset_square_in_mask!(square, self.rook);
        unset_square_in_mask!(square, self.bishop);
        unset_square_in_mask!(square, self.knight);
        unset_square_in_mask!(square, self.pawn);
      },
      PieceType::Rook => {
        unset_square_in_mask!(square, self.king);
        unset_square_in_mask!(square, self.queen);
        set_square_in_mask!(square, self.rook);
        unset_square_in_mask!(square, self.bishop);
        unset_square_in_mask!(square, self.knight);
        unset_square_in_mask!(square, self.pawn);
      },
      PieceType::Bishop => {
        unset_square_in_mask!(square, self.king);
        unset_square_in_mask!(square, self.queen);
        unset_square_in_mask!(square, self.rook);
        set_square_in_mask!(square, self.bishop);
        unset_square_in_mask!(square, self.knight);
        unset_square_in_mask!(square, self.pawn);
      },
      PieceType::Knight => {
        unset_square_in_mask!(square, self.king);
        unset_square_in_mask!(square, self.queen);
        unset_square_in_mask!(square, self.rook);
        unset_square_in_mask!(square, self.bishop);
        set_square_in_mask!(square, self.knight);
        unset_square_in_mask!(square, self.pawn);
      },
      PieceType::Pawn => {
        unset_square_in_mask!(square, self.king);
        unset_square_in_mask!(square, self.queen);
        unset_square_in_mask!(square, self.rook);
        unset_square_in_mask!(square, self.bishop);
        unset_square_in_mask!(square, self.knight);
        set_square_in_mask!(square, self.pawn);
      },
    }
  }

  /// Returns the square where the king is located
  ///
  /// Note that is several kings are present, the first one in the square
  /// indices will be returned. (i.e. if kings are on squares `a` and `b`
  /// and `a < b`, then `a` is returned)
  ///
  /// ### Arguments
  ///
  /// * `self`    Reference to the PieceMasks object where we're looking for a king
  ///
  /// ### Return value
  ///
  /// * `Some(square)`  Square on which the king is located.
  /// * `None`          No king is present on the board.
  ///
  #[inline]
  pub fn get_king(&self) -> Option<u8> {
    if self.king == 0 {
      None
    } else {
      Some(self.king.trailing_zeros() as u8)
    }
  }

  /// Returns true if we have a piece on the square
  ///
  /// ### Arguments
  ///
  /// * `self`    Reference to the PieceMasks object where we're looking for a king
  /// * `square`  Square on which we want to check if there is a piece
  ///
  /// ### Return value
  ///
  /// bool value indicating if we have a piece on the square.
  ///
  #[inline]
  pub fn has_piece(&self, square: u8) -> bool {
    square_in_mask!(square, self.all())
  }

  /// Returns a boardmask of all minor pieces.
  ///
  /// ### Return value
  ///
  /// BoardMask with bits set to 1 if a minor occupies a square.
  ///
  #[inline]
  pub fn minors(&self) -> BoardMask {
    self.bishop | self.knight
  }

  /// Returns a boardmask of all major pieces.
  ///
  /// ### Return value
  ///
  /// BoardMask with bits set to 1 if a major occupies a square.
  ///
  #[inline]
  pub fn majors(&self) -> BoardMask {
    self.rook | self.queen
  }

  /// Removes pieces from a square.
  ///
  /// ### Arguments
  ///
  /// * `self`    Mutable reference to the PieceMasks object being modified
  /// * `square`  Square on which no piece must be present.
  ///
  #[inline]
  pub fn remove(&mut self, square: u8) {
    unset_square_in_mask!(square, self.king);
    unset_square_in_mask!(square, self.queen);
    unset_square_in_mask!(square, self.rook);
    unset_square_in_mask!(square, self.bishop);
    unset_square_in_mask!(square, self.knight);
    unset_square_in_mask!(square, self.pawn);
  }

  /// Removes pieces from a square.
  ///
  /// ### Arguments
  ///
  /// * `self`    Mutable reference to the PieceMasks object being modified
  /// * `square`  Square on which no piece must be present.
  ///
  #[inline]
  pub fn get(&self, square: u8) -> Option<PieceType> {
    if square_in_mask!(square, self.king) {
      return Some(PieceType::King);
    } else if square_in_mask!(square, self.queen) {
      return Some(PieceType::Queen);
    } else if square_in_mask!(square, self.rook) {
      return Some(PieceType::Rook);
    } else if square_in_mask!(square, self.bishop) {
      return Some(PieceType::Bishop);
    } else if square_in_mask!(square, self.knight) {
      return Some(PieceType::Knight);
    } else if square_in_mask!(square, self.pawn) {
      return Some(PieceType::Pawn);
    }

    None
  }

  /// Computes a boardmask of all pieces.
  ///
  /// ### Return value
  ///
  /// BoardMask with bits set to 1 if a piece occupies a square.
  ///
  #[inline]
  pub fn all(&self) -> BoardMask {
    self.king | self.queen | self.rook | self.bishop | self.knight | self.pawn
  }
}

// -----------------------------------------------------------------------------
//  Full  PieceSet Definition

/// List of masks describing black/white pieces.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct PieceSet {
  /// White pieces on the board
  pub white: PieceMasks,
  /// Black pieces on the board
  pub black: PieceMasks,
}

impl PieceSet {
  /// Initialize am empty piece set
  ///
  /// ### Return value
  ///
  /// PieceSet with no pieces
  ///
  pub fn new() -> Self {
    PieceSet {
      white: PieceMasks::new(),
      black: PieceMasks::new(),
    }
  }

  /// Converts the first part of the FEN string into a Piece Set.
  ///
  /// ### Return value
  ///
  /// PieceSet containing the pieces indicated in the FEN string
  ///
  pub fn from_fen(fen: &str) -> Self {
    let mut piece_set = PieceSet::new();

    let fen_parts: Vec<&str> = fen.split(' ').collect();
    if fen_parts.is_empty() {
      error!("FEN string too small to generate a piece set. Returning empty set.");
      return piece_set;
    }

    let mut rank = 7;
    let mut file = 0;
    for c in fen_parts[0].chars() {
      match c {
        'K' => {
          piece_set.white.add(rank * 8 + file, PieceType::King);
          file += 1;
        },
        'Q' => {
          piece_set.white.add(rank * 8 + file, PieceType::Queen);
          file += 1;
        },
        'R' => {
          piece_set.white.add(rank * 8 + file, PieceType::Rook);
          file += 1;
        },
        'B' => {
          piece_set.white.add(rank * 8 + file, PieceType::Bishop);
          file += 1;
        },
        'N' => {
          piece_set.white.add(rank * 8 + file, PieceType::Knight);
          file += 1;
        },
        'P' => {
          piece_set.white.add(rank * 8 + file, PieceType::Pawn);
          file += 1;
        },
        'k' => {
          piece_set.black.add(rank * 8 + file, PieceType::King);
          file += 1;
        },
        'q' => {
          piece_set.black.add(rank * 8 + file, PieceType::Queen);
          file += 1;
        },
        'r' => {
          piece_set.black.add(rank * 8 + file, PieceType::Rook);
          file += 1;
        },
        'b' => {
          piece_set.black.add(rank * 8 + file, PieceType::Bishop);
          file += 1;
        },
        'n' => {
          piece_set.black.add(rank * 8 + file, PieceType::Knight);
          file += 1;
        },
        'p' => {
          piece_set.black.add(rank * 8 + file, PieceType::Pawn);
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

    piece_set
  }

  /// Returns the piece on a square.
  ///
  /// ### Arguments
  ///
  /// * `self`    Reference to the PieceSet object for which we are reading a square
  /// * `square`  Square on which we would like to know what piece is on
  ///
  /// ### Return value
  ///
  /// u8 representation of the piece, e.g. `WHITE_KING`, `BLACK_PAWN`, `NO_PIECE`, etc.
  ///
  #[inline]
  pub fn get(&self, square: u8) -> u8 {
    if square_in_mask!(square, self.white.king) {
      WHITE_KING
    } else if square_in_mask!(square, self.white.queen) {
      WHITE_QUEEN
    } else if square_in_mask!(square, self.white.rook) {
      WHITE_ROOK
    } else if square_in_mask!(square, self.white.bishop) {
      WHITE_BISHOP
    } else if square_in_mask!(square, self.white.knight) {
      WHITE_KNIGHT
    } else if square_in_mask!(square, self.white.pawn) {
      WHITE_PAWN
    } else if square_in_mask!(square, self.black.king) {
      BLACK_KING
    } else if square_in_mask!(square, self.black.queen) {
      BLACK_QUEEN
    } else if square_in_mask!(square, self.black.rook) {
      BLACK_ROOK
    } else if square_in_mask!(square, self.black.bishop) {
      BLACK_BISHOP
    } else if square_in_mask!(square, self.black.knight) {
      BLACK_KNIGHT
    } else if square_in_mask!(square, self.black.pawn) {
      BLACK_PAWN
    } else {
      NO_PIECE
    }
  }

  /// Sets a piece on a square.
  ///
  /// Note: We won't check that another piece is marked as present on the same square.
  /// call `remove(square)` if you are not sure.
  ///
  /// ### Arguments
  ///
  /// * `self`    Reference to the PieceSet object for which we are modifying a square
  /// * `square`  Square on which we would like to configure which piece is on
  /// * `piece`   u8 representation of the piece to put on the square, e.g. `WHITE_KING`, `BLACK_PAWN`, `NO_PIECE`, etc.
  ///
  #[inline]
  pub fn set(&mut self, square: u8, piece: u8) {
    self.remove(square);
    match piece {
      WHITE_KING => set_square_in_mask!(square, self.white.king),
      WHITE_QUEEN => set_square_in_mask!(square, self.white.queen),
      WHITE_ROOK => set_square_in_mask!(square, self.white.rook),
      WHITE_BISHOP => set_square_in_mask!(square, self.white.bishop),
      WHITE_KNIGHT => set_square_in_mask!(square, self.white.knight),
      WHITE_PAWN => set_square_in_mask!(square, self.white.pawn),
      BLACK_KING => set_square_in_mask!(square, self.black.king),
      BLACK_QUEEN => set_square_in_mask!(square, self.black.queen),
      BLACK_ROOK => set_square_in_mask!(square, self.black.rook),
      BLACK_BISHOP => set_square_in_mask!(square, self.black.bishop),
      BLACK_KNIGHT => set_square_in_mask!(square, self.black.knight),
      BLACK_PAWN => set_square_in_mask!(square, self.black.pawn),
      _ => {},
    }
  }

  /// Makes sure that there is no piece on a square
  ///
  /// ### Arguments
  ///
  /// * `self`    Reference to the PieceSet object for which we are modifying a square
  /// * `square`  Square on which no piece must be present.
  ///
  #[inline]
  pub fn remove(&mut self, square: u8) {
    self.white.remove(square);
    self.black.remove(square);
  }

  /// Returns a boardmask of all pieces.
  ///
  /// ### Return value
  ///
  /// BoardMask with bits set to 1 if a piece occupies a square.
  ///
  #[inline]
  pub fn all(&self) -> BoardMask {
    self.white.all() | self.black.all()
  }

  /// Returns a boardmask of all queen pieces.
  ///
  /// ### Return value
  ///
  /// BoardMask with bits set to 1 if a queen occupies a square.
  ///
  #[inline]
  pub fn queens(&self) -> BoardMask {
    self.white.queen | self.black.queen
  }

  /// Returns a boardmask of all rook pieces.
  ///
  /// ### Return value
  ///
  /// BoardMask with bits set to 1 if a rook occupies a square.
  ///
  #[inline]
  pub fn rooks(&self) -> BoardMask {
    self.white.rook | self.black.rook
  }

  /// Returns a boardmask of all bishop pieces.
  ///
  /// ### Return value
  ///
  /// BoardMask with bits set to 1 if a bishop occupies a square.
  ///
  #[inline]
  pub fn bishops(&self) -> BoardMask {
    self.white.bishop | self.black.bishop
  }

  /// Returns a boardmask of all knight pieces.
  ///
  /// ### Return value
  ///
  /// BoardMask with bits set to 1 if a knight occupies a square.
  ///
  #[inline]
  pub fn knights(&self) -> BoardMask {
    self.white.knight | self.black.knight
  }

  /// Returns a boardmask of all minor pieces.
  ///
  /// ### Return value
  ///
  /// BoardMask with bits set to 1 if a minor occupies a square.
  ///
  #[inline]
  pub fn minors(&self) -> BoardMask {
    self.bishops() | self.knights()
  }

  /// Returns a boardmask of all major pieces.
  ///
  /// ### Return value
  ///
  /// BoardMask with bits set to 1 if a major occupies a square.
  ///
  #[inline]
  pub fn majors(&self) -> BoardMask {
    self.rooks() | self.queens()
  }

  /// Returns a boardmask of all pawns.
  ///
  /// ### Return value
  ///
  /// BoardMask with bits set to 1 if a pawns occupies a square.
  ///
  #[inline]
  pub fn pawns(&self) -> BoardMask {
    self.white.pawn | self.black.pawn
  }

  /// Returns a BoardMask of all pieces of a certain color
  ///
  /// ### Arguments
  ///
  /// * `self`    Reference to the PieceSet object where we are looking for pieces
  /// * `color`   Color of the piece set we are looking for
  ///
  /// ### Return value
  ///
  /// BoardMask with all the pieces of a given color
  ///
  #[inline]
  pub fn all_with_color(&self, color: Color) -> BoardMask {
    match color {
      Color::White => self.white.all(),
      Color::Black => self.black.all(),
    }
  }

  /// Returns true if we have a piece on the square
  ///
  /// ### Arguments
  ///
  /// * `self`    Reference to the PieceSet object where we're looking for a king
  /// * `square`  Square on which we want to check if there is a piece
  ///
  /// ### Return value
  ///
  /// bool value indicating if we have a piece on the square.
  ///
  #[inline]
  pub fn has_piece(&self, square: u8) -> bool {
    square_in_mask!(square, self.all())
  }

  /// Returns true if we have a piece of a certain color on the square
  ///
  /// ### Arguments
  ///
  /// * `self`    Reference to the PieceSet object where we're looking for a king
  /// * `square`  Square on which we want to check if there is a piece
  /// * `color`   Color of the piece set we are looking for
  ///
  /// ### Return value
  ///
  /// bool value indicating if we have a piece of a given color on the square.
  ///
  #[inline]
  pub fn has_piece_with_color(&self, square: u8, color: Color) -> bool {
    match color {
      Color::White => square_in_mask!(square, self.white.all()),
      Color::Black => square_in_mask!(square, self.black.all()),
    }
  }
}

impl Default for PieceSet {
  fn default() -> Self {
    PieceSet {
      white: PieceMasks::default_white_piece_set(),
      black: PieceMasks::default_black_piece_set(),
    }
  }
}

// -----------------------------------------------------------------------------
//  Iterator implementation
impl Iterator for PieceMasks {
  // We can refer to this type using Self::Item
  type Item = (u8, PieceType);

  // Returns the next piece on the piece mask
  fn next(&mut self) -> Option<Self::Item> {
    while self.iter_mask == 0 {
      match self.iter_type {
        None => {
          self.iter_mask = self.king;
          self.iter_type = Some(PieceType::King);
        },
        Some(PieceType::King) => {
          self.iter_mask = self.queen;
          self.iter_type = Some(PieceType::Queen);
        },
        Some(PieceType::Queen) => {
          self.iter_mask = self.rook;
          self.iter_type = Some(PieceType::Rook);
        },
        Some(PieceType::Rook) => {
          self.iter_mask = self.bishop;
          self.iter_type = Some(PieceType::Bishop);
        },
        Some(PieceType::Bishop) => {
          self.iter_mask = self.knight;
          self.iter_type = Some(PieceType::Knight);
        },
        Some(PieceType::Knight) => {
          self.iter_mask = self.pawn;
          self.iter_type = Some(PieceType::Pawn);
        },
        Some(PieceType::Pawn) => {
          self.iter_mask = 0;
          self.iter_type = None;
          return None;
        },
      }
    }

    let square = self.iter_mask.trailing_zeros() as u8;
    self.iter_mask &= self.iter_mask - 1;

    Some((square, self.iter_type.unwrap()))
  }
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use crate::model::game_state::START_POSITION_FEN;

  use super::*;

  #[test]
  fn test_piece_masks() {
    let mut pieces: PieceMasks = PieceMasks::new();

    for i in 0..64 {
      assert_eq!(None, pieces.get(i));
    }
    assert_eq!(0, pieces.all());
    assert_eq!(None, pieces.get_king());

    pieces.add(0, PieceType::Rook);
    pieces.add(1, PieceType::King);

    assert_eq!(Some(PieceType::Rook), pieces.get(0));
    assert_eq!(Some(PieceType::King), pieces.get(1));
    assert_eq!(Some(1), pieces.get_king());

    // Now the king becomes a rook:
    pieces.add(1, PieceType::Rook);
    assert_eq!(Some(PieceType::Rook), pieces.get(0));
    assert_eq!(Some(PieceType::Rook), pieces.get(1));

    // Now the king becomes a rook:
    pieces.add(1, PieceType::Rook);
    assert_eq!(Some(PieceType::Rook), pieces.get(0));
    assert_eq!(Some(PieceType::Rook), pieces.get(1));
    assert_eq!(None, pieces.get_king());

    assert_eq!(0b11, pieces.all());
  }

  #[test]
  fn test_piece_mask_iterator() {
    let mut pieces: PieceMasks = PieceMasks::default_white_piece_set();

    assert_eq!(Some((4, PieceType::King)), pieces.next());
    assert_eq!(Some((3, PieceType::Queen)), pieces.next());
    assert_eq!(Some((0, PieceType::Rook)), pieces.next());
    assert_eq!(Some((7, PieceType::Rook)), pieces.next());
    assert_eq!(Some((2, PieceType::Bishop)), pieces.next());
    assert_eq!(Some((5, PieceType::Bishop)), pieces.next());
    assert_eq!(Some((1, PieceType::Knight)), pieces.next());
    assert_eq!(Some((6, PieceType::Knight)), pieces.next());
    assert_eq!(Some((8, PieceType::Pawn)), pieces.next());
    assert_eq!(Some((9, PieceType::Pawn)), pieces.next());
    assert_eq!(Some((10, PieceType::Pawn)), pieces.next());
    assert_eq!(Some((11, PieceType::Pawn)), pieces.next());
    assert_eq!(Some((12, PieceType::Pawn)), pieces.next());
    assert_eq!(Some((13, PieceType::Pawn)), pieces.next());
    assert_eq!(Some((14, PieceType::Pawn)), pieces.next());
    assert_eq!(Some((15, PieceType::Pawn)), pieces.next());
    assert_eq!(None, pieces.next());

    let mut pieces: PieceMasks = PieceMasks::new();

    assert_eq!(None, pieces.next());
    assert_eq!(None, pieces.next());
    assert_eq!(None, pieces.next());

    let mut pieces: PieceMasks = PieceMasks::default_black_piece_set();
    assert_eq!(Some((60, PieceType::King)), pieces.next());
    assert_eq!(Some((59, PieceType::Queen)), pieces.next());
    assert_eq!(Some((56, PieceType::Rook)), pieces.next());
    assert_eq!(Some((63, PieceType::Rook)), pieces.next());
  }

  #[test]
  fn test_piece_set() {
    let piece_set: PieceSet = PieceSet::default();
    let piece_set_from_fen: PieceSet = PieceSet::from_fen(START_POSITION_FEN);

    assert_eq!(piece_set, piece_set_from_fen);
  }
}
