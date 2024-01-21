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
  /// List of square -> piece. Use this to find which piece from a square.
  pub squares: [u8; 64],
  /// White pieces on the board.
  /// Use masks to iterate through the pieces
  pub white: PieceMasks,
  /// Black pieces on the board.
  /// Use masks to iterate through the pieces
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
      squares: [0; 64],
      white: PieceMasks::new(),
      black: PieceMasks::new(),
    }
  }

  /// Initializes an array of squares with the pieces default positions.
  ///
  /// ### Return value
  ///
  /// Square slice indicating where which pieces are present by default.
  ///
  #[inline]
  pub fn square_default_pieces() -> [u8; 64] {
    let mut squares: [u8; 64] = [NO_PIECE; 64];

    squares[0] = WHITE_ROOK;
    squares[1] = WHITE_KNIGHT;
    squares[2] = WHITE_BISHOP;
    squares[3] = WHITE_QUEEN;
    squares[4] = WHITE_KING;
    squares[5] = WHITE_BISHOP;
    squares[6] = WHITE_KNIGHT;
    squares[7] = WHITE_ROOK;
    squares[8] = WHITE_PAWN;
    squares[9] = WHITE_PAWN;
    squares[10] = WHITE_PAWN;
    squares[11] = WHITE_PAWN;
    squares[12] = WHITE_PAWN;
    squares[13] = WHITE_PAWN;
    squares[14] = WHITE_PAWN;
    squares[15] = WHITE_PAWN;
    squares[48] = BLACK_PAWN;
    squares[49] = BLACK_PAWN;
    squares[50] = BLACK_PAWN;
    squares[51] = BLACK_PAWN;
    squares[52] = BLACK_PAWN;
    squares[53] = BLACK_PAWN;
    squares[54] = BLACK_PAWN;
    squares[55] = BLACK_PAWN;
    squares[56] = BLACK_ROOK;
    squares[57] = BLACK_KNIGHT;
    squares[58] = BLACK_BISHOP;
    squares[59] = BLACK_QUEEN;
    squares[60] = BLACK_KING;
    squares[61] = BLACK_BISHOP;
    squares[62] = BLACK_KNIGHT;
    squares[63] = BLACK_ROOK;
    squares
  }

  /// Adds a piece on the board, assuming the square is empty.
  /// Will update both the square table and the masks
  ///
  /// ### Arguments
  ///
  /// * `self`: Object to modify
  /// * `piece`: u8 value for the piece to add on the square
  /// * `square`: u8 square value to update.
  ///
  #[inline]
  pub fn add(&mut self, piece: u8, square: u8) {
    debug_assert!(square < 64);
    let color = Piece::color(piece);
    debug_assert!(!color.is_none());
    let color = color.unwrap();
    let piece_type = PieceType::from_u8(piece);

    let mask_to_update = match color {
      Color::White => &mut self.white,
      Color::Black => &mut self.black,
    };

    // Update the squares:
    self.squares[square as usize] = piece;
    match piece_type {
      PieceType::King => set_square_in_mask!(square, mask_to_update.king),
      PieceType::Queen => set_square_in_mask!(square, mask_to_update.queen),
      PieceType::Rook => set_square_in_mask!(square, mask_to_update.rook),
      PieceType::Bishop => set_square_in_mask!(square, mask_to_update.bishop),
      PieceType::Knight => set_square_in_mask!(square, mask_to_update.knight),
      PieceType::Pawn => set_square_in_mask!(square, mask_to_update.pawn),
    }
  }

  /// Removes a piece from the board.
  ///
  /// ### Arguments
  ///
  /// * `self`: Object to modify
  /// * `piece`: u8 value for the piece to add on the square
  /// * `square`: u8 square value to update.
  ///
  #[inline]
  pub fn remove(&mut self, square: u8) {
    debug_assert!(square < 64);

    if self.squares[square as usize] == NO_PIECE {
      return;
    }

    match self.squares[square as usize] {
      WHITE_KING => unset_square_in_mask!(square, self.white.king),
      WHITE_QUEEN => unset_square_in_mask!(square, self.white.queen),
      WHITE_ROOK => unset_square_in_mask!(square, self.white.rook),
      WHITE_BISHOP => unset_square_in_mask!(square, self.white.bishop),
      WHITE_KNIGHT => unset_square_in_mask!(square, self.white.knight),
      WHITE_PAWN => unset_square_in_mask!(square, self.white.pawn),
      BLACK_KING => unset_square_in_mask!(square, self.black.king),
      BLACK_QUEEN => unset_square_in_mask!(square, self.black.queen),
      BLACK_ROOK => unset_square_in_mask!(square, self.black.rook),
      BLACK_BISHOP => unset_square_in_mask!(square, self.black.bishop),
      BLACK_KNIGHT => unset_square_in_mask!(square, self.black.knight),
      BLACK_PAWN => unset_square_in_mask!(square, self.black.pawn),
      _ => {},
    }

    self.squares[square as usize] = NO_PIECE;
  }

  /// Update a piece on the board
  ///
  /// ### Arguments
  ///
  /// * `self`: Object to modify
  /// * `piece`: u8 value for the piece to add on the square
  /// * `square`: u8 square value to update.
  ///
  pub fn update(&mut self, piece: u8, square: u8) {
    // TODO: Is there something more optimal than remove and add ? ... Probably

    self.remove(square);
    self.add(piece, square);
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
          piece_set.add(WHITE_KING, rank * 8 + file);
          file += 1;
        },
        'Q' => {
          piece_set.add(WHITE_QUEEN, rank * 8 + file);
          file += 1;
        },
        'R' => {
          piece_set.add(WHITE_ROOK, rank * 8 + file);
          file += 1;
        },
        'B' => {
          piece_set.add(WHITE_BISHOP, rank * 8 + file);
          file += 1;
        },
        'N' => {
          piece_set.add(WHITE_KNIGHT, rank * 8 + file);
          file += 1;
        },
        'P' => {
          piece_set.add(WHITE_PAWN, rank * 8 + file);
          file += 1;
        },
        'k' => {
          piece_set.add(BLACK_KING, rank * 8 + file);
          file += 1;
        },
        'q' => {
          piece_set.add(BLACK_QUEEN, rank * 8 + file);
          file += 1;
        },
        'r' => {
          piece_set.add(BLACK_ROOK, rank * 8 + file);
          file += 1;
        },
        'b' => {
          piece_set.add(BLACK_BISHOP, rank * 8 + file);
          file += 1;
        },
        'n' => {
          piece_set.add(BLACK_KNIGHT, rank * 8 + file);
          file += 1;
        },
        'p' => {
          piece_set.add(BLACK_PAWN, rank * 8 + file);
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
    self.squares[square as usize]
    // This is actually slower...
    //unsafe { *self.squares.get_unchecked(square as usize) }
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
  pub fn get_usize(&self, square: usize) -> u8 {
    self.squares[square]
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
      squares: PieceSet::square_default_pieces(),
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
