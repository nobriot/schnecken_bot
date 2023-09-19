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
pub const WHITE_PIECES_DEFAULT_POSITIONS: PieceMasks = PieceMasks {
  iter_type: None,
  iter_mask: 0,
  king: 1 << 4,
  queen: 1 << 3,
  rook: 1 | 1 << 7,
  bishop: 1 << 2 | 1 << 5,
  knight: 1 << 1 | 1 << 6,
  pawn: 1 << 8 | 1 << 9 | 1 << 10 | 1 << 11 | 1 << 12 | 1 << 13 | 1 << 14 | 1 << 15,
};

pub const BLACK_PIECES_DEFAULT_POSITIONS: PieceMasks = PieceMasks {
  iter_type: None,
  iter_mask: 0,
  king: 1 << 60,
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
  pub fn add_piece(&mut self, square: u8, piece: PieceType) {
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
  pub fn get_king(&self) -> Option<u8> {
    if self.king == 0 {
      None
    } else {
      Some(self.king.trailing_zeros() as u8)
    }
  }

  /// Removes pieces from a square.
  ///
  /// ### Arguments
  ///
  /// * `self`    Mutable reference to the PieceMasks object being modified
  /// * `square`  Square on which no piece must be present.
  ///
  pub fn remove_piece(&mut self, square: u8) {
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
  pub fn get_piece(&self, square: u8) -> Option<PieceType> {
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
  pub fn all_pieces(&self) -> BoardMask {
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
          piece_set.white.add_piece(rank * 8 + file, PieceType::King);
          file += 1;
        },
        'Q' => {
          piece_set.white.add_piece(rank * 8 + file, PieceType::Queen);
          file += 1;
        },
        'R' => {
          piece_set.white.add_piece(rank * 8 + file, PieceType::Rook);
          file += 1;
        },
        'B' => {
          piece_set
            .white
            .add_piece(rank * 8 + file, PieceType::Bishop);
          file += 1;
        },
        'N' => {
          piece_set
            .white
            .add_piece(rank * 8 + file, PieceType::Knight);
          file += 1;
        },
        'P' => {
          piece_set.white.add_piece(rank * 8 + file, PieceType::Pawn);
          file += 1;
        },
        'k' => {
          piece_set.black.add_piece(rank * 8 + file, PieceType::King);
          file += 1;
        },
        'q' => {
          piece_set.black.add_piece(rank * 8 + file, PieceType::Queen);
          file += 1;
        },
        'r' => {
          piece_set.black.add_piece(rank * 8 + file, PieceType::Rook);
          file += 1;
        },
        'b' => {
          piece_set
            .black
            .add_piece(rank * 8 + file, PieceType::Bishop);
          file += 1;
        },
        'n' => {
          piece_set
            .black
            .add_piece(rank * 8 + file, PieceType::Knight);
          file += 1;
        },
        'p' => {
          piece_set.black.add_piece(rank * 8 + file, PieceType::Pawn);
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
  /// * `square`  Square on which no piece must be present.
  ///
  pub fn get_piece(&self, square: u8) -> u8 {
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
      assert_eq!(None, pieces.get_piece(i));
    }
    assert_eq!(0, pieces.all_pieces());
    assert_eq!(None, pieces.get_king());

    pieces.add_piece(0, PieceType::Rook);
    pieces.add_piece(1, PieceType::King);

    assert_eq!(Some(PieceType::Rook), pieces.get_piece(0));
    assert_eq!(Some(PieceType::King), pieces.get_piece(1));
    assert_eq!(Some(1), pieces.get_king());

    // Now the king becomes a rook:
    pieces.add_piece(1, PieceType::Rook);
    assert_eq!(Some(PieceType::Rook), pieces.get_piece(0));
    assert_eq!(Some(PieceType::Rook), pieces.get_piece(1));

    // Now the king becomes a rook:
    pieces.add_piece(1, PieceType::Rook);
    assert_eq!(Some(PieceType::Rook), pieces.get_piece(0));
    assert_eq!(Some(PieceType::Rook), pieces.get_piece(1));
    assert_eq!(None, pieces.get_king());

    assert_eq!(0b11, pieces.all_pieces());
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
    let mut piece_set: PieceSet = PieceSet::default();
    let mut piece_set_from_fen: PieceSet = PieceSet::from_fen(START_POSITION_FEN);

    assert_eq!(piece_set, piece_set_from_fen);
  }
}
