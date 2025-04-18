use crate::model::board_geometry::diagonals::DIAGONALS;
use crate::model::board_geometry::lines::LINES;
use crate::model::board_geometry::rays::RAYS;
use crate::model::board_geometry::*;
use crate::model::board_mask::*;
use crate::model::castling_rights::*;
use crate::model::moves::*;
use crate::model::piece::*;
use crate::model::piece_moves::*;
use crate::model::piece_set::*;
use crate::model::tables::bishop_destinations::BISHOP_SPAN;
use crate::model::tables::pawn_destinations::*;
use crate::model::tables::rook_destinations::ROOK_SPAN;
use crate::model::tables::zobrist::*;
use log::*;
use rand::Rng;
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
  pub pieces:            PieceSet,
  pub side_to_play:      Color,
  pub en_passant_square: u8,
  pub castling_rights:   CastlingRights,
  /// Boardmask of pieces delivering check to the side to play.
  pub checkers:          BoardMask,
  /// Boardmask of pieces that are pinned
  pub pins:              BoardMask,
  pub hash:              BoardHash,
}

// -----------------------------------------------------------------------------
// Implementations

impl Board {
  /// Initialize a board with no piece, all zeroes
  fn new() -> Self {
    Board { pieces:            PieceSet::new(),
            side_to_play:      Color::White,
            castling_rights:   CastlingRights::default(),
            en_passant_square: INVALID_SQUARE,
            checkers:          0,
            pins:              0,
            hash:              0, }
  }

  /// Initialize a board with a random arrangement of pieces.
  ///
  /// **NOTE**: This may be an illegal configuration.
  pub fn new_random() -> Self {
    let mut board = Board::new();

    let color_rand = rand::random::<bool>();
    board.side_to_play = match color_rand {
      true => Color::White,
      false => Color::Black,
    };

    board.castling_rights = CastlingRights::none();

    let mut rng = rand::thread_rng();

    // Let's try to place pieces:
    // White King
    let square = rng.gen_range(0..64);
    set_square_in_mask!(square, board.pieces.white.king);

    // Place the black king, try until they do not touch:
    loop {
      let square = rng.gen_range(0..64);
      if board.pieces.get(square) != NO_PIECE {
        continue;
      }

      set_square_in_mask!(square, board.pieces.black.king);
      if (board.get_attackers(board.get_king(Color::White), Color::Black)) != 0 {
        board.pieces.black.king = 0;
        continue;
      } else {
        break;
      }
    }

    // Let's try to place 8 pawns of each color:
    for _ in 0..8 {
      let square = rng.gen_range(0..64);
      if board.pieces.get(square) != NO_PIECE {
        continue;
      }

      set_square_in_mask!(square, board.pieces.white.pawn);
      board.update_checkers();
      if board.get_attackers(board.get_king(Color::White), Color::Black)
         | board.get_attackers(board.get_king(Color::Black), Color::White)
         != 0
      {
        unset_square_in_mask!(square, board.pieces.white.pawn);
        continue;
      }
    }
    for _ in 0..8 {
      let square = rng.gen_range(0..64);
      if board.pieces.get(square) != NO_PIECE {
        continue;
      }

      set_square_in_mask!(square, board.pieces.black.pawn);
      if board.get_attackers(board.get_king(Color::White), Color::Black)
         | board.get_attackers(board.get_king(Color::Black), Color::White)
         != 0
      {
        unset_square_in_mask!(square, board.pieces.black.pawn);
        continue;
      }
    }

    // Clean up pawns on the 1st and 8th ranks before adding pieces that could
    // deliver checks
    board.pieces.white.pawn &= !(BOARD_UP_EDGE | BOARD_DOWN_EDGE);
    board.pieces.black.pawn &= !(BOARD_UP_EDGE | BOARD_DOWN_EDGE);

    // Try to add knights:
    for _ in 0..2 {
      let square = rng.gen_range(0..64);
      if board.pieces.get(square) != NO_PIECE {
        continue;
      }

      set_square_in_mask!(square, board.pieces.white.knight);
      if board.get_attackers(board.get_king(Color::White), Color::Black)
         | board.get_attackers(board.get_king(Color::Black), Color::White)
         != 0
      {
        unset_square_in_mask!(square, board.pieces.white.knight);
        continue;
      }
    }
    for _ in 0..2 {
      let square = rng.gen_range(0..64);
      if board.pieces.get(square) != NO_PIECE {
        continue;
      }

      set_square_in_mask!(square, board.pieces.black.knight);
      if board.get_attackers(board.get_king(Color::White), Color::Black)
         | board.get_attackers(board.get_king(Color::Black), Color::White)
         != 0
      {
        unset_square_in_mask!(square, board.pieces.black.knight);
        continue;
      }
    }
    // Try to add bishops:
    for _ in 0..2 {
      let square = rng.gen_range(0..64);
      if board.pieces.get(square) != NO_PIECE {
        continue;
      }

      set_square_in_mask!(square, board.pieces.white.bishop);
      if board.get_attackers(board.get_king(Color::White), Color::Black)
         | board.get_attackers(board.get_king(Color::Black), Color::White)
         != 0
      {
        unset_square_in_mask!(square, board.pieces.white.bishop);
        continue;
      }
    }
    for _ in 0..2 {
      let square = rng.gen_range(0..64);
      if board.pieces.get(square) != NO_PIECE {
        continue;
      }

      set_square_in_mask!(square, board.pieces.black.bishop);
      if board.get_attackers(board.get_king(Color::White), Color::Black)
         | board.get_attackers(board.get_king(Color::Black), Color::White)
         != 0
      {
        unset_square_in_mask!(square, board.pieces.black.bishop);
        continue;
      }
    }
    // Try to add rooks:
    for _ in 0..2 {
      let square = rng.gen_range(0..64);
      if board.pieces.get(square) != NO_PIECE {
        continue;
      }

      set_square_in_mask!(square, board.pieces.white.rook);
      if board.get_attackers(board.get_king(Color::White), Color::Black)
         | board.get_attackers(board.get_king(Color::Black), Color::White)
         != 0
      {
        unset_square_in_mask!(square, board.pieces.white.rook);
        continue;
      }
    }
    for _ in 0..2 {
      let square = rng.gen_range(0..64);
      if board.pieces.get(square) != NO_PIECE {
        continue;
      }

      set_square_in_mask!(square, board.pieces.black.rook);
      if board.get_attackers(board.get_king(Color::White), Color::Black)
         | board.get_attackers(board.get_king(Color::Black), Color::White)
         != 0
      {
        unset_square_in_mask!(square, board.pieces.black.rook);
        continue;
      }
    }
    // Try to add queens:
    for _ in 0..1 {
      let square = rng.gen_range(0..64);
      if board.pieces.get(square) != NO_PIECE {
        continue;
      }

      set_square_in_mask!(square, board.pieces.white.queen);
      if board.get_attackers(board.get_king(Color::White), Color::Black)
         | board.get_attackers(board.get_king(Color::Black), Color::White)
         != 0
      {
        unset_square_in_mask!(square, board.pieces.white.queen);
        continue;
      }
    }
    for _ in 0..1 {
      let square = rng.gen_range(0..64);
      if board.pieces.get(square) != NO_PIECE {
        continue;
      }

      set_square_in_mask!(square, board.pieces.black.queen);
      if board.get_attackers(board.get_king(Color::White), Color::Black)
         | board.get_attackers(board.get_king(Color::Black), Color::White)
         != 0
      {
        unset_square_in_mask!(square, board.pieces.black.queen);
        continue;
      }
    }

    board.compute_hash();
    board.update_checkers();
    board.update_pins();

    board
  }

  // ---------------------------------------------------------------------------
  // Hashing helpers

  /// Updates the board hash using the Zobrist tables from "scratch"
  fn compute_hash(&mut self) {
    self.hash = 0;

    // Add the hash from the pieces
    for i in 0..64_u8 {
      if self.pieces.get(i) != NO_PIECE {
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

  // Adds/Removes a piece in the board hash value.
  fn update_hash_piece(&mut self, i: u8) {
    self.hash ^= ZOBRIST_TABLE[(self.pieces.get(i) - 1) as usize][i as usize];
  }

  // Toggles the side to play in the board hash value
  fn update_hash_side_to_play(&mut self) {
    self.hash ^= ZOBRIST_WHITE_TO_MOVE;
  }

  // Adds/remove the en-passant square in the board hash value
  fn update_hash_en_passant(&mut self) {
    self.hash ^= ZOBRIST_EN_PASSANT[self.en_passant_square as usize % 8];
  }

  // Adds/Removes castling rights in the board hash value
  fn update_hash_castling_rights(&mut self) {
    if self.castling_rights.K() {
      self.hash ^= ZOBRIST_WHITE_KING_CASTLE;
    }
    if self.castling_rights.Q() {
      self.hash ^= ZOBRIST_WHITE_QUEEN_CASTLE;
    }
    if self.castling_rights.k() {
      self.hash ^= ZOBRIST_BLACK_KING_CASTLE;
    }
    if self.castling_rights.q() {
      self.hash ^= ZOBRIST_BLACK_QUEEN_CASTLE;
    }
  }

  // ---------------------------------------------------------------------------
  // Board masks and piece movement functions

  /// Computes if a square is under attack by the color for a given board
  /// position. X-Rays are ignored.
  ///
  /// ### Arguments
  ///
  /// * `self` -   A GameState object representing a position, side to play,
  ///   etc.
  /// * `color` -  The color for which we want to determine the bitmap
  ///
  /// ### Return value
  ///
  /// A bitmask indicating squares under control by the color for that game
  /// state.
  pub fn get_control_boardmask(&self, color: Color) -> BoardMask {
    let mut bitmap: BoardMask = 0;
    let mut pieces = self.get_color_mask(color);

    while pieces != 0 {
      bitmap |= self.get_piece_control_mask(pieces.trailing_zeros() as u8);
      pieces &= pieces - 1;
    }

    bitmap
  }

  /// Computes a boardmask of absolute pins rays for the side to play
  /// The pins shows the positions that the piece can go to without exposing
  /// their king to enemy attack.
  ///
  /// ### Arguments
  ///
  /// * `self` -   A Board object representing a position, side to play, etc.
  /// * `color` -  Color for which we want to find pinned pieces
  ///
  /// ### Return value
  ///
  /// A bitmask indicating squares that the pinned piece can move to.
  pub fn get_pins_rays(&self, color: Color) -> BoardMask {
    let king_position = self.get_king(color) as usize;
    debug_assert!(king_position < 64, "No king for board: {}", self.to_fen());

    let mut pins: BoardMask = 0;

    // Get the list of potential enemy pinning pieces
    let enemy_pieces = match color {
      Color::White => self.pieces.black,
      Color::Black => self.pieces.white,
    };

    let mut pinning_pieces = ROOK_SPAN[king_position] & (enemy_pieces.rook | enemy_pieces.queen);
    pinning_pieces |= BISHOP_SPAN[king_position] & (enemy_pieces.bishop | enemy_pieces.queen);

    // Now they are pinning only if we have 1 piece between them and our king.
    while pinning_pieces != 0 {
      let pinning_piece = pinning_pieces.trailing_zeros() as usize;

      let ray = RAYS[king_position][pinning_piece]; // King is excluded from the ray
      if (ray & (self.pieces.all())).count_ones() == 2
         && (ray & (enemy_pieces.all())).count_ones() == 1
      {
        // TBD: This will count enemy pieces as pinnned. See the unit test.
        // I don't think it is a problem as these pins restrict how our pieces move, not
        // the enemy pieces.
        pins |= ray;
      }

      pinning_pieces &= pinning_pieces - 1;
    }

    pins
  }

  /// Computes a boardmask of attackers of a square.
  ///
  /// ### Arguments
  ///
  /// * `self` -           A Board object representing a position, side to play,
  ///   etc.
  /// * `target_square` -  Square for which we want to know the attackers
  ///
  /// ### Return value
  ///
  /// A bitmask indicating squares of the pieces attacking the square
  pub fn get_attackers(&self, target_square: u8, color: Color) -> BoardMask {
    debug_assert!(target_square < 64,
                  "get_attackers for square {} - board: {}",
                  target_square,
                  self.to_fen());

    let (attacking_pieces, king_mask) = match color {
      Color::White => (self.pieces.white, self.pieces.black.king),
      Color::Black => (self.pieces.black, self.pieces.white.king),
    };

    // A bit like I do when I play as a human, here we start from the target piece
    // and go along diagonals, lines, knight jumps to find if there is an attacker.
    let mut attackers = 0;

    // note: Here it is inverted on purpose.
    attackers |= match color {
      Color::White => BLACK_PAWN_CONTROL[target_square as usize] & attacking_pieces.pawn,
      Color::Black => WHITE_PAWN_CONTROL[target_square as usize] & attacking_pieces.pawn,
    };

    attackers |= KING_MOVES[target_square as usize] & attacking_pieces.king;
    attackers |= KNIGHT_MOVES[target_square as usize] & attacking_pieces.knight;

    attackers |= get_rook_moves(0, self.pieces.all() & (!king_mask), target_square as usize)
                 & attacking_pieces.majors();
    attackers |= get_bishop_moves(0, self.pieces.all() & (!king_mask), target_square as usize)
                 & (attacking_pieces.bishop | attacking_pieces.queen);

    attackers
  }

  /// Computes a boardmask of attackers of a surface/boardmask.
  ///
  /// ### Arguments
  ///
  /// * `self` -     A Board object representing a position, side to play, etc.
  /// * `squares` -  Squares for which we want to know if they are attacked
  ///
  /// ### Return value
  ///
  /// A bitmask indicating squares of the pieces attacking the square
  pub fn get_attacked_squares(&self, squares: BoardMask, color: Color) -> BoardMask {
    let mut surface = squares;
    let mut attacked_squares: BoardMask = 0;
    while surface != 0 {
      let square = surface.trailing_zeros() as u8;
      if self.get_attackers(square, color) != 0 {
        set_square_in_mask!(square, attacked_squares);
      }
      surface &= surface - 1;
    }

    attacked_squares
  }

  /// Returns the number of checks on the board.
  ///
  /// ### Arguments
  ///
  /// * `self` -           A Board object representing a position, side to play,
  ///   etc.
  ///
  /// ### Return value
  ///
  /// Number of checks for the king whose side it is to play.
  #[inline]
  pub fn checks(&self) -> u32 {
    self.checkers.count_few_ones()
  }

  /// Computes the boardmask of the possible destinations for a piece on a
  /// square.
  ///
  /// ### Arguments
  ///
  /// * `self` -           A Board object representing a position, side to play,
  ///   etc.
  /// * `source_square` -  Square for which we want to know the destinations
  /// * `op` -             BoardSquare for which we want to know the
  ///   destinations
  ///
  /// ### Return value
  ///
  /// A bitmask indicating possible destinations for the piece present on the
  /// square.
  pub fn get_piece_destinations(&self,
                                source_square: usize,
                                op: BoardMask,
                                ssp: BoardMask)
                                -> (BoardMask, bool) {
    let mut promotion: bool = false;
    let destinations = match self.pieces.get_usize(source_square) {
      WHITE_KING => get_king_moves(ssp,
                                   self.get_attacked_squares(KING_MOVES[source_square],
                                                             Color::Black),
                                   source_square),
      BLACK_KING => get_king_moves(ssp,
                                   self.get_attacked_squares(KING_MOVES[source_square],
                                                             Color::White),
                                   source_square),
      WHITE_QUEEN | BLACK_QUEEN => get_queen_moves(ssp, op, source_square),
      WHITE_ROOK | BLACK_ROOK => get_rook_moves(ssp, op, source_square),
      WHITE_BISHOP | BLACK_BISHOP => get_bishop_moves(ssp, op, source_square),
      WHITE_KNIGHT | BLACK_KNIGHT => get_knight_moves(ssp, op, source_square),
      WHITE_PAWN => {
        let pawn_targets = if self.en_passant_square != INVALID_SQUARE {
          op | (1 << self.en_passant_square)
        } else {
          op
        };
        if (source_square / 8) == 6 {
          promotion = true;
        }
        get_white_pawn_moves(ssp, pawn_targets, source_square)
      },
      BLACK_PAWN => {
        let pawn_targets = if self.en_passant_square != INVALID_SQUARE {
          op | (1 << self.en_passant_square)
        } else {
          op
        };

        if (source_square / 8) == 1 {
          promotion = true;
        }
        get_black_pawn_moves(ssp, pawn_targets, source_square)
      },
      _ => 0,
    };

    (destinations, promotion)
  }

  /// Computes the boardmask of the squares controlled by a piece
  /// Sliding pieces will also control squares located behind the enemy king.
  ///
  ///
  /// ### Arguments
  ///
  /// * `self` -           A Board object representing a position, side to play,
  ///   etc.
  /// * `source_square` -  Square for which we want to know the controlled
  ///   squares
  ///
  /// ### Return value
  ///
  /// BoardMask value indicating all the squares controlled by the piece
  pub fn get_piece_control_mask(&self, source_square: u8) -> BoardMask {
    match self.pieces.get(source_square) {
      WHITE_KING | BLACK_KING => KING_MOVES[source_square as usize],
      WHITE_QUEEN => get_queen_moves(0,
                                     self.pieces.all() & (!self.pieces.black.king),
                                     source_square as usize),
      BLACK_QUEEN => get_queen_moves(0,
                                     self.pieces.all() & (!self.pieces.white.king),
                                     source_square as usize),
      WHITE_ROOK => get_rook_moves(0,
                                   self.pieces.all() & (!self.pieces.black.king),
                                   source_square as usize),
      BLACK_ROOK => get_rook_moves(0,
                                   self.pieces.all() & (!self.pieces.white.king),
                                   source_square as usize),
      WHITE_BISHOP => get_bishop_moves(0,
                                       self.pieces.all() & (!self.pieces.black.king),
                                       source_square as usize),
      BLACK_BISHOP => get_bishop_moves(0,
                                       self.pieces.all() & (!self.pieces.white.king),
                                       source_square as usize),
      WHITE_KNIGHT | BLACK_KNIGHT => KNIGHT_MOVES[source_square as usize],
      WHITE_PAWN => get_white_pawn_captures(source_square as usize),
      BLACK_PAWN => get_black_pawn_captures(source_square as usize),
      _ => 0,
    }
  }

  /// Converts Rank / File into a board index
  ///
  /// Returns an index in the range 0..63. Asserts in debug mode if
  /// the values passed are not valid.
  ///
  /// * `file`: [1..8]
  /// * `rank`: [1..8]
  #[inline]
  pub fn fr_to_index(file: u8, rank: u8) -> u8 {
    debug_assert!(file > 0);
    debug_assert!(file <= 8);
    debug_assert!(rank > 0);
    debug_assert!(rank <= 8);
    (file - 1) + (rank - 1) * 8
  }

  /// Converts a board index into Rank / File.
  ///
  /// Returns a file and rank in the range [1..=8]. Asserts in debug mode if
  /// the values passed are not valid.
  ///
  /// * `index`: [0..=63]
  ///
  #[inline]
  pub fn index_to_fr(index: u8) -> (u8, u8) {
    debug_assert!(index < 64);
    (index % 8 + 1, index / 8 + 1)
  }

  /// Returns the piece currently set at the board file/rank a board index into
  /// Rank / File.
  ///
  /// * `file`: [1..8]
  /// * `rank`: [1..8]
  pub fn get_piece(&self, file: u8, rank: u8) -> u8 {
    self.pieces.get(Board::fr_to_index(file, rank))
  }

  // ---------------------------------------------------------------------------
  // Move related functions

  /// Computes the list of legal moves on the board
  ///
  /// ### Arguments
  ///
  /// * `self`: Board reference to calculate legal moves from
  ///
  /// ### Return value
  ///
  /// A vector of Moves
  #[inline]
  pub fn get_moves(&self) -> Vec<Move> {
    match self.side_to_play {
      Color::White => self.get_white_moves(),
      Color::Black => self.get_black_moves(),
    }
  }

  /// Get all the possible moves for white in a position
  ///
  ///
  /// ### Return value
  ///
  /// Vector of Moves
  #[inline]
  pub fn get_white_moves(&self) -> Vec<Move> {
    let mut all_moves = Vec::with_capacity(MAXIMUM_LEGAL_MOVES);
    // This is used to down-prioritize some moves
    let mut other_moves = Vec::with_capacity(MAXIMUM_LEGAL_MOVES);

    let mut ssp = self.get_color_mask(Color::White);
    let op = self.get_color_mask(Color::Black);

    // Try castling first. This will have an influence on the engine if
    // interesting moves are placed first.
    if self.castling_rights.K()
       && self.checks() == 0
       && (self.pieces.all() & FREE_SQUARE_MASK_WHITE_KINGSIDE) == 0
       && self.get_attacked_squares(UNATTACKED_SQUARE_MASK_WHITE_KINGSIDE, Color::Black) == 0
    {
      other_moves.push(castle_mv!(4, 6));
    }
    if self.castling_rights.Q()
       && self.checks() == 0
       && (self.pieces.all() & FREE_SQUARE_MASK_WHITE_QUEENSIDE) == 0
       && self.get_attacked_squares(UNATTACKED_SQUARE_MASK_WHITE_QUEENSIDE, Color::Black) == 0
    {
      other_moves.push(castle_mv!(4, 2));
    }

    let mut checking_ray: BoardMask = u64::MAX;
    let king_position = self.get_king(Color::White) as usize;

    match self.checkers.count_few_ones() {
      0 => {},
      1 => {
        checking_ray = unsafe {
          RAYS.get_unchecked(king_position).get_unchecked(self.checkers.trailing_zeros() as usize)
          | self.checkers
        }
      },
      _ => ssp = self.pieces.white.king,
    }

    // Only generate moves if we have a piece on the square
    while ssp != 0 {
      let source_square = ssp.trailing_zeros() as u8;
      let (mut destinations, promotion) =
        self.get_piece_destinations(source_square as usize,
                                    op,
                                    self.get_color_mask(Color::White));

      // Restrict destinations not to move out of pins.
      // if there is a check, you can only move into checking rays with other pieces
      // than the king.
      if square_in_mask!(source_square, self.pins) {
        if LINES[king_position][source_square as usize] & self.pins != 0 {
          destinations &= self.pins & ROOK_SPAN[source_square as usize] & ROOK_SPAN[king_position];
        } else if DIAGONALS[king_position][source_square as usize] & self.pins != 0 {
          destinations &=
            self.pins & BISHOP_SPAN[source_square as usize] & BISHOP_SPAN[king_position];
        }
      }

      // If a pawn double jump delivers check, we should be able to en-passant it,
      // it removes the checking piece even though outside of the checking ray
      // If a pawn double jumps but no pawn is delivering check, it's a discovered
      // check.
      if square_in_mask!(source_square, self.pieces.white.pawn)
         && self.en_passant_square != INVALID_SQUARE
         && self.checkers.count_few_ones() == 1
         && (self.checkers & self.pieces.black.pawn) != 0
      {
        destinations &= checking_ray | (1 << self.en_passant_square);
      } else if source_square != king_position as u8 {
        destinations &= checking_ray;
      }

      while destinations != 0 {
        let destination_square = destinations.trailing_zeros() as u8;

        // Determine if this is a capture or en-passant
        let capture = PieceType::from_u8(self.pieces.get(destination_square as u8));
        let en_passant = square_in_mask!(source_square, self.pieces.white.pawn)
                         && destination_square == self.en_passant_square;

        if en_passant {
          all_moves.push(en_passant_mv!(source_square, destination_square));
        } else if promotion {
          all_moves.push(mv!(source_square,
                             destination_square,
                             Promotion::WhiteQueen,
                             capture));
          other_moves.push(mv!(source_square,
                               destination_square,
                               Promotion::WhiteRook,
                               capture));
          other_moves.push(mv!(source_square,
                               destination_square,
                               Promotion::WhiteKnight,
                               capture));
          other_moves.push(mv!(source_square,
                               destination_square,
                               Promotion::WhiteBishop,
                               capture));
        } else if capture != PieceType::King {
          // King means no capture.. not pretty I know.
          all_moves.push(mv!(source_square,
                             destination_square,
                             Promotion::NoPromotion,
                             capture));
        } else {
          other_moves.push(mv!(source_square,
                               destination_square,
                               Promotion::NoPromotion,
                               capture));
        }

        // Remove the last bit set to 1:
        destinations &= destinations - 1;
      }

      // Remove the last bit set to 1:
      ssp &= ssp - 1;
    }

    all_moves.append(&mut other_moves);
    all_moves
  }

  // Get all the possible moves for black in a position
  #[inline]
  pub fn get_black_moves(&self) -> Vec<Move> {
    let mut all_moves = Vec::with_capacity(MAXIMUM_LEGAL_MOVES);
    // This is used to down-prioritize some moves
    let mut other_moves = Vec::with_capacity(MAXIMUM_LEGAL_MOVES);

    let mut ssp = self.get_color_mask(Color::Black);
    let op = self.get_color_mask(Color::White);

    // Now check castling.
    if self.castling_rights.k()
       && self.checks() == 0
       && (self.pieces.all() & FREE_SQUARE_MASK_BLACK_KINGSIDE) == 0
       && self.get_attacked_squares(UNATTACKED_SQUARE_MASK_BLACK_KINGSIDE, Color::White) == 0
    {
      other_moves.push(castle_mv!(60, 62));
    }
    if self.castling_rights.q()
       && self.checks() == 0
       && (self.pieces.all() & FREE_SQUARE_MASK_BLACK_QUEENSIDE) == 0
       && self.get_attacked_squares(UNATTACKED_SQUARE_MASK_BLACK_QUEENSIDE, Color::White) == 0
    {
      other_moves.push(castle_mv!(60, 58));
    }

    let mut checking_ray: BoardMask = u64::MAX;
    let king_position = self.get_king(Color::Black) as usize;

    match self.checkers.count_ones() {
      0 => {},
      1 => {
        checking_ray = unsafe {
          RAYS.get_unchecked(king_position).get_unchecked(self.checkers.trailing_zeros() as usize)
          | self.checkers
        }
      },
      _ => ssp = self.pieces.black.king,
    }

    // Only generate moves if we have a piece on the square
    while ssp != 0 {
      let source_square = ssp.trailing_zeros() as u8;
      let (mut destinations, promotion) =
        self.get_piece_destinations(source_square as usize,
                                    op,
                                    self.get_color_mask(Color::Black));

      // Restrict destinations not to move out of pins.
      // if there is a check, you can only move into checking rays with other pieces
      // than the king.
      if square_in_mask!(source_square, self.pins) {
        if LINES[king_position][source_square as usize] & self.pins != 0 {
          destinations &= self.pins & ROOK_SPAN[source_square as usize] & ROOK_SPAN[king_position];
        } else if DIAGONALS[king_position][source_square as usize] & self.pins != 0 {
          destinations &=
            self.pins & BISHOP_SPAN[source_square as usize] & BISHOP_SPAN[king_position];
        }
      }

      // If a pawn double jump delivers check, we should be able to en-passant it,
      // it removes the checking piece even though outside of the checking ray
      // If a pawn double jumps but no pawn is delivering check, it's a discovered
      // check.
      if square_in_mask!(source_square, self.pieces.black.pawn)
         && self.en_passant_square != INVALID_SQUARE
         && self.checkers.count_few_ones() == 1
         && (self.checkers & self.pieces.white.pawn) != 0
      {
        destinations &= checking_ray | (1 << self.en_passant_square);
      } else if source_square != king_position as u8 {
        destinations &= checking_ray;
      }

      while destinations != 0 {
        let destination_square = destinations.trailing_zeros() as u8;

        // Determine if this is a capture or en-passant
        let capture = PieceType::from_u8(self.pieces.get(destination_square));
        let en_passant = square_in_mask!(source_square, self.pieces.black.pawn)
                         && destination_square == self.en_passant_square;

        if en_passant {
          all_moves.push(en_passant_mv!(source_square, destination_square));
        } else if promotion {
          all_moves.push(mv!(source_square,
                             destination_square,
                             Promotion::BlackQueen,
                             capture));
          other_moves.push(mv!(source_square,
                               destination_square,
                               Promotion::BlackRook,
                               capture));
          other_moves.push(mv!(source_square,
                               destination_square,
                               Promotion::BlackKnight,
                               capture));
          other_moves.push(mv!(source_square,
                               destination_square,
                               Promotion::BlackBishop,
                               capture));
        } else if capture != PieceType::King {
          // King means no capture.. not pretty I know.
          all_moves.push(mv!(source_square,
                             destination_square,
                             Promotion::NoPromotion,
                             capture));
        } else {
          other_moves.push(mv!(source_square,
                               destination_square,
                               Promotion::NoPromotion,
                               capture));
        }

        // Remove the last bit set to 1:
        destinations &= destinations - 1;
      }

      // Remove the last bit set to 1:
      ssp &= ssp - 1;
    }

    all_moves.append(&mut other_moves);
    all_moves
  }

  /// Applies a move on the board.
  ///
  /// Very few checks are done here, the caller has to check that the move is
  /// legal before applying it.
  pub fn apply_move(&mut self, chess_move: &Move) {
    let source = chess_move.src() as usize;
    let destination = chess_move.dest() as usize;

    // Check if we just castled, we need to move the rooks around!
    if square_in_mask!(source, self.pieces.white.king) {
      if chess_move.src() == 4 && chess_move.dest() == 2 {
        self.update_hash_piece(0);
        self.pieces.remove(0);
        self.pieces.update(WHITE_ROOK, 3);
        self.update_hash_piece(3);
      } else if chess_move.src() == 4 && chess_move.dest() == 6 {
        self.update_hash_piece(7);
        self.pieces.remove(7);
        self.pieces.update(WHITE_ROOK, 5);
        self.update_hash_piece(5);
      }
    } else if square_in_mask!(source, self.pieces.black.king) {
      if chess_move.src() == 60 && chess_move.dest() == 62 {
        self.update_hash_piece(63);
        self.pieces.remove(63);
        self.pieces.update(BLACK_ROOK, 61);
        self.update_hash_piece(61);
      } else if chess_move.src() == 60 && chess_move.dest() == 58 {
        self.update_hash_piece(56);
        self.pieces.remove(56);
        self.pieces.update(BLACK_ROOK, 59);
        self.update_hash_piece(59);
      }
    }

    // Update castling rights. (just look if something from the rook/king moved)
    self.update_hash_castling_rights();
    match chess_move.src() {
      0 => self.castling_rights.set_Q(false),
      4 => self.castling_rights.clear_white_rights(),
      7 => self.castling_rights.set_K(false),
      56 => self.castling_rights.set_q(false),
      60 => self.castling_rights.clear_black_rights(),
      63 => self.castling_rights.set_k(false),
      _ => {},
    }
    match chess_move.dest() {
      0 => self.castling_rights.set_Q(false),
      4 => self.castling_rights.clear_white_rights(),
      7 => self.castling_rights.set_K(false),
      56 => self.castling_rights.set_q(false),
      60 => self.castling_rights.clear_black_rights(),
      63 => self.castling_rights.set_k(false),
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
    if (square_in_mask!(source, self.pieces.white.pawn | self.pieces.black.pawn))
       && (chess_move.dest() as isize - chess_move.src() as isize).abs() == 16
    {
      let op_pawn = match self.pieces.get(source as u8) {
        WHITE_PAWN => BLACK_PAWN,
        _ => WHITE_PAWN,
      };
      let (file, rank) = Board::index_to_fr(chess_move.u8_dest());
      let en_passant_target = (chess_move.u8_dest() + chess_move.u8_src()) / 2;
      let pins = self.get_pins_rays(Color::opposite(self.side_to_play));
      if file > 1 {
        let s = Board::fr_to_index(file - 1, rank);
        if self.pieces.get(s) == op_pawn
           && (square_in_mask!(en_passant_target, pins) || !square_in_mask!(s, pins))
        {
          self.en_passant_square = en_passant_target;
        }
      }

      // Check on the right side:
      if file < 8 {
        let s = Board::fr_to_index(file + 1, rank);
        if self.pieces.get(s) == op_pawn
           && (square_in_mask!(en_passant_target, pins) || !square_in_mask!(s, pins))
        {
          self.en_passant_square = en_passant_target;
        }
      }
    }

    // Add the new hash value, if a new square is valid:
    if self.en_passant_square != INVALID_SQUARE {
      self.update_hash_en_passant();
    }

    // Check if this is some en-passant action: PAWN is moving diagonally while the
    // destination square is empty: En passant needs to remove the captured
    // pawn.
    if square_in_mask!(chess_move.src(), self.pieces.pawns())
       && !square_in_mask!(chess_move.dest(), self.pieces.all())
    {
      let target_capture = match chess_move.dest() as isize - chess_move.src() as isize {
        7 => chess_move.u8_src() - 1,
        9 => chess_move.u8_src() + 1,
        -7 => chess_move.u8_src() + 1,
        -9 => chess_move.u8_src() - 1,
        _ => {
          // Not a en-passant move
          INVALID_SQUARE
        },
      };

      if target_capture != INVALID_SQUARE {
        self.update_hash_piece(target_capture);
        self.pieces.remove(target_capture);
      }
    }

    // Now apply the initial move
    if self.pieces.get(chess_move.u8_dest()) != NO_PIECE {
      self.update_hash_piece(chess_move.u8_dest());
    }

    if chess_move.promotion() != Promotion::NoPromotion {
      self.pieces.update(chess_move.promotion().to_piece_const(),
                         chess_move.u8_dest());
    } else {
      self.pieces.update(self.pieces.get(chess_move.u8_src()), chess_move.u8_dest());
    }

    self.update_hash_piece(destination as u8);
    self.update_hash_piece(source as u8);
    self.pieces.remove(source as u8);

    // Update the side to play:
    self.flip();
    // Update all additional data
    self.update_hash_side_to_play();
    self.update_checkers();
    self.update_pins();
  }

  /// Flips the board, i.e. changes the side to play
  ///
  /// ### Arguments
  ///
  /// * `self`: Board reference to change the side to play
  #[inline]
  pub fn flip(&mut self) {
    self.side_to_play = Color::opposite(self.side_to_play);
  }

  /// Takes a move notation from a PGN, tries to find the corresponding move
  /// and apply it on our board.
  ///
  /// ### Arguments
  ///
  /// * `self`:           Board object to modify
  /// * `move_notation`   Move notation as specified in a PGN file
  ///
  /// ### Return Value
  ///
  /// Ok if the move was "identified" and applied.
  /// Err if the move was not identified or not applied.
  pub fn find_move_from_pgn_notation(&mut self, move_notation: &str) -> Result<Move, ()> {
    let mut notation = String::from(move_notation);
    let candidate_moves = self.get_moves();
    let mut mv = Move::default();

    // Remove the annotations at the end of the moves (!, ?, #, +)
    while notation.ends_with('?') || notation.ends_with('!') {
      notation.pop();
    }

    // Check if this is a capture:
    let capture = notation.contains('x');
    if capture {
      notation = notation.replace("x", "");
    }

    // Check if this is a check:
    let check = notation.contains('+') || notation.contains('#');
    if check {
      notation = notation.replace("+", "");
      notation = notation.replace("#", "");
    }

    // Check if this is a Promotion:
    let promotion = notation.contains('=');
    let mut promoted_piece = Promotion::NoPromotion;
    if promotion {
      // Last character should be the promoted piece:
      let p = notation.pop();
      match (p, self.side_to_play) {
        (Some('Q') | Some('q'), Color::White) => promoted_piece = Promotion::WhiteQueen,
        (Some('Q') | Some('q'), Color::Black) => promoted_piece = Promotion::BlackQueen,
        (Some('R') | Some('r'), Color::White) => promoted_piece = Promotion::WhiteRook,
        (Some('R') | Some('r'), Color::Black) => promoted_piece = Promotion::BlackRook,
        (Some('B') | Some('b'), Color::White) => promoted_piece = Promotion::WhiteBishop,
        (Some('B') | Some('b'), Color::Black) => promoted_piece = Promotion::BlackBishop,
        (Some('N') | Some('n'), Color::White) => promoted_piece = Promotion::WhiteKnight,
        (Some('N') | Some('n'), Color::Black) => promoted_piece = Promotion::BlackKnight,
        _ => {
          println!("Could not identify Promotion: {} for board {}",
                   move_notation,
                   self.to_fen());
          return Err(());
        },
      }

      // Also remove the '='
      notation = notation.replace("=", "");
    }

    // castles
    let kingside_castle = notation == "O-O";
    let queenside_castle = notation == "O-O-O";

    // println!("Stripped notation : {}", notation);

    // Now try to identify the corresponding move:
    if notation.len() == 2 {
      // This is a pawn move, just the destination square
      let destination_square = string_to_square(notation.as_str());
      for m in candidate_moves {
        if m.dest() == destination_square as move_t
           && square_in_mask!(m.src(), self.pieces.pawns())
           && m.promotion() == promoted_piece
           && m.is_capture() == capture
        {
          mv = m;
          break;
        }
      }
    } else if kingside_castle {
      for m in candidate_moves {
        if m.is_castle() && (m.dest() == 6 || m.dest() == 62) {
          mv = m;
          break;
        }
      }
    } else if queenside_castle {
      for m in candidate_moves {
        if m.is_castle() && (m.dest() == 2 || m.dest() == 58) {
          mv = m;
          break;
        }
      }
    } else {
      // Here we are in the case of Source piece, destination square:
      // Note that chars is inverted: Nbd7 -> 7,d,b,N
      let chars: Vec<char> = notation.chars().rev().collect();
      let mut dest = String::new();
      dest.push(chars[1]);
      dest.push(chars[0]);
      let destination_square = string_to_square(dest.as_str());

      // Put a limitation on the source square (with a BoardMask) if the initial piece
      // and/or file/rank is indicated
      let mut source_mask = u64::MAX;

      if !notation.starts_with('K')
         && !notation.starts_with('Q')
         && !notation.starts_with('R')
         && !notation.starts_with('B')
         && !notation.starts_with('N')
      {
        source_mask &= self.pieces.pawns();
      }

      for i in 0..(notation.chars().count() - 2) {
        source_mask &= match notation.chars().nth(i) {
          Some('a') => FILES[0],
          Some('b') => FILES[1],
          Some('c') => FILES[2],
          Some('d') => FILES[3],
          Some('e') => FILES[4],
          Some('f') => FILES[5],
          Some('g') => FILES[6],
          Some('h') => FILES[7],
          Some('1') => RANKS[0],
          Some('2') => RANKS[1],
          Some('3') => RANKS[2],
          Some('4') => RANKS[3],
          Some('5') => RANKS[4],
          Some('6') => RANKS[5],
          Some('7') => RANKS[6],
          Some('8') => RANKS[7],
          Some('K') => self.pieces.white.king | self.pieces.black.king,
          Some('Q') => self.pieces.queens(),
          Some('R') => self.pieces.rooks(),
          Some('B') => self.pieces.bishops(),
          Some('N') => self.pieces.knights(),
          _ => {
            println!("Could not identify source file/rank move: {} for board {}",
                     move_notation,
                     self.to_fen());
            return Err(());
          },
        }
      }

      for m in candidate_moves {
        if m.dest() == destination_square as move_t
           && m.is_capture() == capture
           && m.promotion() == promoted_piece
           && square_in_mask!(m.src(), source_mask)
        {
          mv = m;
          break;
        }
      }
    }

    // Did we find the move?
    if mv == Move::default() {
      println!("Could not identify matching move: {} for board {} - side to play {:#?}",
               move_notation,
               self.to_fen(),
               self.side_to_play);
      for m in self.get_moves() {
        println!("Candidate: {}", m);
      }
      println!("En-passant: {}", self.en_passant_square);

      return Err(());
    }

    Ok(mv)
  }

  /// Makes sure that the number of checks on the board is correct.
  ///
  /// ### Arguments
  ///
  /// * `self` - Board object to modify
  pub fn update_checkers(&mut self) {
    let king_position = self.get_king(self.side_to_play);
    debug_assert!(king_position < 64, "No king ?? fen: {}", self.to_fen());

    self.checkers = self.get_attackers(king_position, Color::opposite(self.side_to_play));
  }

  /// Makes sure that the pins on the board is correct.
  ///
  /// ### Arguments
  ///
  /// * `self` - Board object to modify
  pub fn update_pins(&mut self) {
    self.pins = self.get_pins_rays(self.side_to_play);
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
  #[inline]
  pub fn has_piece(&self, square: u8) -> bool {
    debug_assert!(square < 64, "has_piece called with too high square value");
    self.pieces.has_piece(square)
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
  /// True if there is a piece with the given color on that square, false
  /// otherwise
  #[inline]
  pub fn has_piece_with_color(&self, square: u8, color: Color) -> bool {
    self.pieces.has_piece_with_color(square, color)
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
  #[inline]
  pub fn has_king(&self, square: u8) -> bool {
    square_in_mask!(square, self.pieces.white.king | self.pieces.black.king)
  }

  /// Finds the square with a king of the desired color on it.
  ///
  /// ### Arguments
  ///
  /// * `color`: The color of the king to find on the board.
  ///
  /// ### Return value
  ///
  /// `square` - Square index in [0..63] where the black king is located.
  /// The lowest square value if there are several black kings.
  /// `64` if no black king is present on the board.
  #[inline]
  pub fn get_king(&self, color: Color) -> u8 {
    match color {
      Color::White => self.pieces.white.king.trailing_zeros() as u8,
      Color::Black => self.pieces.black.king.trailing_zeros() as u8,
    }
  }

  /// Finds the square with a black king on it.
  ///
  /// ### Return value
  ///
  /// `square` - Square index in [0..63] where the black king is located.
  /// The lowest square value if there are several black kings.
  /// `INVALID_SQUARE` if no black king is present on the board.
  #[inline]
  pub fn get_black_king_square(&self) -> u8 {
    if self.pieces.black.king == 0 {
      error!("No black king ?? ");
      return INVALID_SQUARE;
    }
    self.pieces.black.king.trailing_zeros() as u8
  }

  /// Finds the square with a white king on it.
  ///
  /// ### Return value
  ///
  /// `square` - Square index in [0..63] where the white king is located.
  /// The lowest square value if there are several white kings.
  /// `INVALID_SQUARE` if no white king is present on the board.
  #[inline]
  pub fn get_white_king_square(&self) -> u8 {
    if self.pieces.white.king == 0 {
      error!("No white king ?? ");
      return INVALID_SQUARE;
    }
    self.pieces.white.king.trailing_zeros() as u8
  }

  /// Return a board bismask with squares set to 1 when they
  /// have a piece with a certain color
  #[inline]
  pub fn get_color_mask(&self, color: Color) -> BoardMask {
    match color {
      Color::White => self.pieces.white.all(),
      Color::Black => self.pieces.black.all(),
    }
  }

  /// Returns a board bismask with squares set to 1 when they
  /// have a piece with a certain color, which is not a major piece (rook and
  /// queens excluded)
  #[inline]
  pub fn get_color_mask_without_major_pieces(&self, color: Color) -> BoardMask {
    match color {
      Color::White => {
        self.pieces.white.king
        | self.pieces.white.bishop
        | self.pieces.white.knight
        | self.pieces.white.pawn
      },
      Color::Black => {
        self.pieces.black.king
        | self.pieces.black.bishop
        | self.pieces.black.knight
        | self.pieces.black.pawn
      },
    }
  }

  /// Converts first substring of a FEN (with the pieces) to a board
  ///
  /// ### Arguments:
  ///
  /// * `fen` : fen to use to create a board object
  ///
  /// ### Return Value
  ///
  /// Board object matching the FEN
  pub fn from_fen(fen: &str) -> Self {
    let mut board = Board::new();

    let fen_parts: Vec<&str> = fen.split(' ').collect();
    if fen_parts.len() < 6 {
      error!("FEN string too small to generate a board");
      return board;
    }

    board.pieces = PieceSet::from_fen(fen);

    board.side_to_play = if fen_parts[1] == "w" { Color::White } else { Color::Black };

    board.castling_rights = CastlingRights::default();
    board.castling_rights.set_K(fen_parts[2].contains('K'));
    board.castling_rights.set_Q(fen_parts[2].contains('Q'));
    board.castling_rights.set_k(fen_parts[2].contains('k'));
    board.castling_rights.set_q(fen_parts[2].contains('q'));

    board.en_passant_square = if fen_parts[3] != "-" {
      string_to_square(fen_parts[3])
    } else {
      INVALID_SQUARE
    };

    board.compute_hash();
    board.update_checkers();
    board.update_pins();

    board
  }

  /// Converts a board to the first part of a FEN.
  ///
  /// ### Arguments:
  ///
  /// * `self` : Reference to a Board object
  ///
  /// ### Return Value
  ///
  /// String containing the FEN description of the board.
  pub fn to_fen(&self) -> String {
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

  /// Determines if a position is a game over due to insufficient material or
  /// not
  ///
  /// ### Arguments
  ///
  /// * `self` - A board object reference
  ///
  /// ### Returns
  ///
  /// True if is it a game over (draw) by insufficient material
  /// false otherwise
  pub fn is_game_over_by_insufficient_material(&self) -> bool {
    if (self.pieces.white.pawn
        | self.pieces.black.pawn
        | self.pieces.white.queen
        | self.pieces.black.queen
        | self.pieces.white.rook
        | self.pieces.black.rook)
       != 0
    {
      return false;
    }

    if (self.pieces.white.bishop
        | self.pieces.black.bishop
        | self.pieces.white.knight
        | self.pieces.black.knight)
                                   .count_ones()
       > 1
    {
      return false;
    }
    true
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
