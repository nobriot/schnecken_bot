use super::tables::rook_destinations::ROOK_SPAN;
use crate::model::board_geometry::diagonals::DIAGONALS;
use crate::model::board_geometry::lines::LINES;
use crate::model::board_mask::*;
use crate::model::castling_rights::*;
use crate::model::moves::*;
use crate::model::piece::*;
use crate::model::piece_moves::*;
use crate::model::piece_set::*;
use crate::model::tables::pawn_destinations::*;
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
  pub pieces: PieceSet,
  pub side_to_play: Color,
  pub en_passant_square: u8,
  pub castling_rights: CastlingRights,
  /// Boardmask of pieces delivering check to the side to play.
  pub checkers: BoardMask,
  pub hash: u64,
}

// -----------------------------------------------------------------------------
// Implementations

impl Board {
  /// Initialize a board with no piece, all zeroes
  fn new() -> Self {
    Board {
      pieces: PieceSet::new(),
      side_to_play: Color::White,
      castling_rights: CastlingRights::default(),
      en_passant_square: INVALID_SQUARE,
      checkers: 0,
      hash: 0,
    }
  }

  /// Initialize a board with a random arrangement of pieces.
  ///
  /// **NOTE**: This may be an illegal configuration.
  ///
  pub fn new_random() -> Self {
    let mut board = Board::new();

    let color_rand = rand::random::<bool>();
    board.side_to_play = match color_rand {
      true => Color::White,
      false => Color::Black,
    };

    board.castling_rights = CastlingRights::none();

    let mut rng = rand::thread_rng();
    let black_king_position = rng.gen_range(0..64);
    board.pieces.set(black_king_position, BLACK_KING);
    let white_king_position = rng.gen_range(0..64);
    board.pieces.set(white_king_position, WHITE_KING);

    for _ in 0..14 {
      board
        .pieces
        .set(rng.gen_range(0..64), rng.gen_range(NO_PIECE..=BLACK_PAWN));
    }

    board.compute_hash();
    board.update_checkers();

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

  /// Computes if a square is under attack by the color for a given board position.
  /// X-Rays are ignored.
  ///
  /// ### Arguments
  ///
  /// * `self` -   A GameState object representing a position, side to play, etc.
  /// * `color` -  The color for which we want to determine the bitmap
  ///
  /// ### Return value
  ///
  /// A bitmask indicating squares under control by the color for that game state.
  ///
  pub fn get_control_boardmask(&self, color: Color) -> BoardMask {
    let mut bitmap: BoardMask = 0;
    let mut pieces = self.get_piece_color_mask(color);

    while pieces != 0 {
      bitmap |= self.get_piece_control_mask(pieces.trailing_zeros() as u8);
      pieces &= pieces - 1;
    }

    bitmap
  }

  /// Computes a boardmask of pins if a piece happens to be in an absolute pin.
  /// The pins shows the positions that the piece can go to without exposing
  /// their king to enemy attack.
  ///
  /// ### Arguments
  ///
  /// * `self` -   A Board object representing a position, side to play, etc.
  /// * `source_square` -  Square for which we want to know the destinations
  ///
  /// ### Return value
  ///
  /// A bitmask indicating squares that the pinned piece can move to.
  ///
  pub fn get_pins(&self, source_square: u8) -> BoardMask {
    if !self.has_piece(source_square) {
      return u64::MAX;
    }

    let color = match self.has_piece_with_color(source_square, Color::White) {
      true => Color::White,
      false => Color::Black,
    };

    let king_position = match color {
      Color::White => self.get_white_king_square(),
      Color::Black => self.get_black_king_square(),
    } as usize;

    let ssp = match color {
      Color::White => self.pieces.white.all(),
      Color::Black => self.pieces.black.all(),
    };

    let mut pins: BoardMask = u64::MAX;

    // Check line pins
    if square_in_mask!(source_square, ROOK_SPAN[king_position]) {
      let enemy_pieces = match color {
        Color::White => self.pieces.black.rook | self.pieces.black.queen,
        Color::Black => self.pieces.white.rook | self.pieces.white.queen,
      };

      //print_board_mask(ROOK_SPAN[king_position]);
      //print_board_mask(enemy_pieces);

      let mut pinning_pieces = ROOK_SPAN[king_position] & enemy_pieces;
      //println!("Pinning pieces in lines:");
      //print_board_mask(pinning_pieces);

      while pinning_pieces != 0 {
        let ray = LINES[pinning_pieces.trailing_zeros() as usize][king_position];
        if (ray & ssp).count_ones() == 2 && square_in_mask!(source_square, ray) {
          // The king and the source square are the only ones in the ray, meaning that the piece is pinned.
          pins &= LINES[king_position][pinning_pieces.trailing_zeros() as usize];
        }
        pinning_pieces &= pinning_pieces - 1;
      }
    }

    // Check diagonal pins
    if square_in_mask!(source_square, BISHOP_SPAN[king_position]) {
      let enemy_pieces = match color {
        Color::White => self.pieces.black.bishop | self.pieces.black.queen,
        Color::Black => self.pieces.white.bishop | self.pieces.white.queen,
      };
      let mut pinning_pieces = BISHOP_SPAN[king_position] & enemy_pieces;

      while pinning_pieces != 0 {
        let ray = DIAGONALS[pinning_pieces.trailing_zeros() as usize][king_position];
        if (ray & ssp).count_ones() == 2 && square_in_mask!(source_square, ray) {
          // The king and the source square are the only ones in the ray, meaning that the piece is pinned.
          pins &= DIAGONALS[king_position][pinning_pieces.trailing_zeros() as usize];
        }
        pinning_pieces &= pinning_pieces - 1;
      }
    }

    pins
  }

  /// Computes a boardmask of attackers of a square.
  ///
  /// ### Arguments
  ///
  /// * `self` -           A Board object representing a position, side to play, etc.
  /// * `target_square` -  Square for which we want to know the attackers
  ///
  /// ### Return value
  ///
  /// A bitmask indicating squares of the pieces attacking the square
  ///
  pub fn get_attackers(&self, target_square: u8, color: Color) -> BoardMask {
    if target_square > 63 {
      warn!("Received get_attackers on square: {}", target_square);
      return 0;
    }

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
  ///
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
  /// * `self` -           A Board object representing a position, side to play, etc.
  ///
  /// ### Return value
  ///
  /// Number of checks for the king whose side it is to play.
  ///
  #[inline]
  pub fn checks(&self) -> u32 {
    return self.checkers.count_ones();
  }

  /// Computes the boardmask of the possible destinations for a piece on a square.
  ///
  /// ### Arguments
  ///
  /// * `self` -           A Board object representing a position, side to play, etc.
  /// * `source_square` -  Square for which we want to know the destinations
  /// * `op` -             BoardSquare for which we want to know the destinations
  ///
  /// ### Return value
  ///
  /// A bitmask indicating possible destinations for the piece present on the square.
  ///
  pub fn get_piece_destinations(
    &self,
    source_square: usize,
    op: BoardMask,
    ssp: BoardMask,
  ) -> (BoardMask, bool) {
    let mut promotion: bool = false;
    let destinations = match self.pieces.get(source_square as u8) {
      WHITE_KING => get_king_moves(
        ssp,
        self.get_attacked_squares(KING_MOVES[source_square], Color::Black),
        source_square,
      ),
      BLACK_KING => get_king_moves(
        ssp,
        self.get_attacked_squares(KING_MOVES[source_square], Color::White),
        source_square,
      ),
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
  /// * `self` -           A Board object representing a position, side to play, etc.
  /// * `source_square` -  Square for which we want to know the controlled squares
  ///
  /// ### Return value
  ///
  /// BoardMask value indicating all the squares controlled by the piece
  ///
  pub fn get_piece_control_mask(&self, source_square: u8) -> BoardMask {
    match self.pieces.get(source_square) {
      WHITE_KING | BLACK_KING => KING_MOVES[source_square as usize],
      WHITE_QUEEN => get_queen_moves(
        0,
        self.pieces.all() & (!self.pieces.black.king),
        source_square as usize,
      ),
      BLACK_QUEEN => get_queen_moves(
        0,
        self.pieces.all() & (!self.pieces.white.king),
        source_square as usize,
      ),
      WHITE_ROOK => get_rook_moves(
        0,
        self.pieces.all() & (!self.pieces.black.king),
        source_square as usize,
      ),
      BLACK_ROOK => get_rook_moves(
        0,
        self.pieces.all() & (!self.pieces.white.king),
        source_square as usize,
      ),
      WHITE_BISHOP => get_bishop_moves(
        0,
        self.pieces.all() & (!self.pieces.black.king),
        source_square as usize,
      ),
      BLACK_BISHOP => get_bishop_moves(
        0,
        self.pieces.all() & (!self.pieces.white.king),
        source_square as usize,
      ),
      WHITE_KNIGHT | BLACK_KNIGHT => KNIGHT_MOVES[source_square as usize],
      WHITE_PAWN => return get_white_pawn_captures(source_square as usize),
      BLACK_PAWN => return get_black_pawn_captures(source_square as usize),
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
  ///
  pub fn fr_to_index(file: u8, rank: u8) -> u8 {
    debug_assert!(file > 0);
    debug_assert!(file <= 8);
    debug_assert!(rank > 0);
    debug_assert!(rank <= 8);
    ((file - 1) + (rank - 1) * 8) as u8
  }

  /// Converts a board index into Rank / File.
  ///
  /// Returns a file and rank in the range [1..8]. Asserts in debug mode if
  /// the values passed are not valid.
  ///
  /// * `index`: [0..63]
  ///
  pub fn index_to_fr(index: u8) -> (u8, u8) {
    debug_assert!(index < 64);
    (index % 8 + 1, index / 8 + 1)
  }

  /// Returns the piece currently set at the board file/rank a board index into Rank / File.
  ///
  /// * `file`: [1..8]
  /// * `rank`: [1..8]
  ///
  pub fn get_piece(&self, file: u8, rank: u8) -> u8 {
    self.pieces.get(Board::fr_to_index(file, rank) as u8)
  }

  // ---------------------------------------------------------------------------
  // Move related functions

  /// Checks if a move on the board is a capture
  ///
  /// ### Arguments
  ///
  /// * `self`: Board to look at
  /// * `m`:    Reference to a move to examine
  ///
  /// ### Return value
  ///
  /// True if the move is a capture, false otherwise
  ///
  pub fn is_move_a_capture(&self, m: &Move) -> bool {
    // If a piece is at the destination, it's a capture
    if square_in_mask!(m.dest, self.pieces.all()) {
      return true;
    }

    // If a pawn moves to the en-passant square, it's a capture.
    if self.en_passant_square != INVALID_SQUARE
      && m.dest == self.en_passant_square
      && (square_in_mask!(m.src, self.pieces.white.pawn | self.pieces.black.pawn))
    {
      return true;
    }

    false
  }

  /// Applies a move on the board.
  ///
  /// Very few checks are done here, the caller has to check that the move is
  /// legal before applying it.
  pub fn apply_move(&mut self, chess_move: &Move) {
    let source = chess_move.src as usize;
    let destination = chess_move.dest as usize;

    // Check if we just castled, we need to move the rooks around!
    if square_in_mask!(source, self.pieces.white.king) {
      if chess_move.src == 4 && chess_move.dest == 2 {
        self.update_hash_piece(0);
        self.pieces.white.remove(0);
        self.pieces.white.add(3, PieceType::Rook);
        self.update_hash_piece(3);
      } else if chess_move.src == 4 && chess_move.dest == 6 {
        self.update_hash_piece(7);
        self.pieces.white.remove(7);
        self.pieces.white.add(5, PieceType::Rook);
        self.update_hash_piece(5);
      }
    } else if square_in_mask!(source, self.pieces.black.king) {
      if chess_move.src == 60 && chess_move.dest == 62 {
        self.update_hash_piece(63);
        self.pieces.black.remove(63);
        self.pieces.black.add(61, PieceType::Rook);
        self.update_hash_piece(61);
      } else if chess_move.src == 60 && chess_move.dest == 58 {
        self.update_hash_piece(56);
        self.pieces.black.remove(56);
        self.pieces.black.add(59, PieceType::Rook);
        self.update_hash_piece(59);
      }
    }

    // Update castling rights. (just look if something from the rook/king moved)
    self.update_hash_castling_rights();
    match chess_move.src {
      0 => self.castling_rights.set_Q(false),
      4 => self.castling_rights.clear_white_rights(),
      7 => self.castling_rights.set_K(false),
      56 => self.castling_rights.set_q(false),
      60 => self.castling_rights.clear_black_rights(),
      63 => self.castling_rights.set_k(false),
      _ => {},
    }
    match chess_move.dest {
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
      && (chess_move.dest as isize - chess_move.src as isize).abs() == 16
    {
      let op_pawn = match self.pieces.get(source as u8) {
        WHITE_PAWN => BLACK_PAWN,
        _ => WHITE_PAWN,
      };
      let (rank, file) = Board::index_to_fr(chess_move.dest);
      if file > 1 {
        let s = Board::fr_to_index(file - 1, rank) as u8;
        if self.pieces.get(s) == op_pawn {
          self.en_passant_square = (chess_move.dest + chess_move.src) / 2;
        }
      }

      // Check on the right side:
      if file < 8 {
        let s = Board::fr_to_index(file + 1, rank) as u8;
        if self.pieces.get(s) == op_pawn {
          self.en_passant_square = (chess_move.dest + chess_move.src) / 2;
        }
      }
    }

    // Add the new hash value, if a new square is valid:
    if self.en_passant_square != INVALID_SQUARE {
      self.update_hash_en_passant();
    }

    // Check if this is some en-passant action: PAWN is moving diagonally while the destination square is empty:
    // En passant needs to remove the captured pawn.
    if square_in_mask!(chess_move.src, self.pieces.pawns())
      && !square_in_mask!(chess_move.dest, self.pieces.all())
    {
      let target_capture = match chess_move.dest as isize - chess_move.src as isize {
        7 => chess_move.src - 1,
        9 => chess_move.src + 1,
        -7 => chess_move.src + 1,
        -9 => chess_move.src - 1,
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
    if self.pieces.get(chess_move.dest) != NO_PIECE {
      self.update_hash_piece(chess_move.dest);
    }

    if chess_move.promotion != NO_PIECE {
      self.pieces.set(chess_move.dest, chess_move.promotion);
    } else {
      self
        .pieces
        .set(chess_move.dest, self.pieces.get(chess_move.src));
    }

    self.update_hash_piece(destination as u8);
    self.update_hash_piece(source as u8);
    self.pieces.remove(source as u8);

    // Update the side to play:
    if self.side_to_play == Color::White {
      self.side_to_play = Color::Black;
    } else {
      self.side_to_play = Color::White;
    }
    self.update_hash_side_to_play();
    self.update_checkers();
  }

  /// Makes sure that the number of checks on the board is correct.
  ///
  /// ### Arguments
  ///
  /// * `self` - Board object to modify
  ///
  pub fn update_checkers(&mut self) {
    let king_position = match self.side_to_play {
      Color::White => self.get_white_king_square(),
      Color::Black => self.get_black_king_square(),
    };

    self.checkers = self.get_attackers(king_position, Color::opposite(self.side_to_play));
  }

  /// Verifies if the move is a castling move based on the board
  ///
  /// ### Arguments
  ///
  /// * `chess_move` - Chess move to look at
  ///
  /// ### Return value
  ///
  /// True if the move is a castling move, false otherwise
  ///
  pub fn is_castle(self, chess_move: &Move) -> bool {
    if self.pieces.white.get_king().unwrap_or(INVALID_SQUARE) == chess_move.src {
      if chess_move.src == 4 && (chess_move.dest == 2 || chess_move.dest == 6) {
        return true;
      }
    } else if self.pieces.black.get_king().unwrap_or(INVALID_SQUARE) == chess_move.src {
      if chess_move.src == 60 && (chess_move.dest == 62 || chess_move.dest == 58) {
        return true;
      }
    }
    false
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
  ///
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
  /// True if there is a piece with the given color on that square, false otherwise
  ///
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
  ///
  #[inline]
  pub fn has_king(&self, square: u8) -> bool {
    square_in_mask!(square, self.pieces.white.king | self.pieces.black.king)
  }

  /// Finds the square with a black king on it.
  ///
  /// ### Return value
  ///
  /// `square` - Square index in [0..63] where the black king is located.
  /// The lowest square value if there are several black kings.
  /// `INVALID_SQUARE` if no black king is present on the board.
  ///
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
  ///
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
  pub fn get_piece_color_mask(&self, color: Color) -> BoardMask {
    match color {
      Color::White => self.pieces.white.all(),
      Color::Black => self.pieces.black.all(),
    }
  }

  /// Return a board bismask with squares set to 1 when they
  /// have a piece with a certain color, which is not a major piece (rook and queens excluded)
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

    board
  }

  /// Converts a board to the first part of a FEN.
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

  /// Determines if a position is a game over due to insufficient material or not
  ///
  /// ### Arguments
  ///
  /// * `self` - A board object reference
  ///
  /// ### Returns
  ///
  /// True if is it a game over (draw) by insufficient material
  /// false otherwise
  ///
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

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn display_board() {
    let mut board = Board {
      pieces: PieceSet::new(),
      side_to_play: Color::White,
      castling_rights: CastlingRights::default(),
      en_passant_square: INVALID_SQUARE,
      checkers: 0,
      hash: 0,
    };
    board.pieces.set(0, WHITE_ROOK);
    board.pieces.set(1, WHITE_KNIGHT);
    board.pieces.set(2, WHITE_BISHOP);
    board.pieces.set(3, WHITE_QUEEN);
    board.pieces.set(4, WHITE_KING);
    board.pieces.set(5, WHITE_BISHOP);
    board.pieces.set(6, WHITE_KNIGHT);
    board.pieces.set(7, WHITE_ROOK);
    board.pieces.set(8, WHITE_PAWN);
    board.pieces.set(9, WHITE_PAWN);
    board.pieces.set(1, WHITE_PAWN);
    board.pieces.set(1, WHITE_PAWN);
    board.pieces.set(1, WHITE_PAWN);
    board.pieces.set(1, WHITE_PAWN);
    board.pieces.set(1, WHITE_PAWN);
    board.pieces.set(1, WHITE_PAWN);

    board.pieces.set(48, BLACK_PAWN);
    board.pieces.set(49, BLACK_PAWN);
    board.pieces.set(50, BLACK_PAWN);
    board.pieces.set(51, BLACK_PAWN);
    board.pieces.set(52, BLACK_PAWN);
    board.pieces.set(53, BLACK_PAWN);
    board.pieces.set(54, BLACK_PAWN);
    board.pieces.set(55, BLACK_PAWN);
    board.pieces.set(56, BLACK_ROOK);
    board.pieces.set(57, BLACK_KNIGHT);
    board.pieces.set(58, BLACK_BISHOP);
    board.pieces.set(59, BLACK_QUEEN);
    board.pieces.set(60, BLACK_KING);
    board.pieces.set(61, BLACK_BISHOP);
    board.pieces.set(62, BLACK_KNIGHT);
    board.pieces.set(63, BLACK_ROOK);

    println!("Board: {}", board);
  }

  #[test]
  fn from_string() {
    let mut board = Board::from_fen("rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR");
    println!("Board: {}", board);

    let test_fen = "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2";
    board = Board::from_fen(test_fen);
    println!("Board: {}", board);

    assert_eq!(test_fen.split(' ').collect::<Vec<_>>()[0], board.to_fen());

    let test_fen_2 = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    board = Board::from_fen(test_fen_2);
    println!("Board: {}", board);

    assert_eq!(test_fen_2.split(' ').collect::<Vec<_>>()[0], board.to_fen())
  }

  #[test]
  fn apply_move() {
    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    let mut board = Board::from_fen(fen);
    println!("Board: {}", board.to_fen());

    // Try and capture a piece
    board.apply_move(&Move {
      src: string_to_square("b3"),
      dest: string_to_square("g3"),
      promotion: NO_PIECE,
    });
    println!("Board: {}", board.to_fen());
    assert_eq!(board.to_fen(), "8/5pk1/5p1p/2R5/5K2/6r1/7P/8");

    // Try and promote a piece (super jump from h2 to h8)
    board.apply_move(&Move {
      src: string_to_square("h2"),
      dest: string_to_square("h8"),
      promotion: WHITE_KNIGHT,
    });
    println!("Board: {}", board.to_fen());
    assert_eq!(board.to_fen(), "7N/5pk1/5p1p/2R5/5K2/6r1/8/8");

    // Same for black: promote to a black queen:
    board.apply_move(&Move {
      src: string_to_square("f6"),
      dest: string_to_square("f1"),
      promotion: BLACK_QUEEN,
    });
    println!("Board: {}", board.to_fen());
    assert_eq!(board.to_fen(), "7N/5pk1/7p/2R5/5K2/6r1/8/5q2");
  }

  #[test]
  fn test_fr_to_index() {
    assert_eq!(0, Board::fr_to_index(1, 1));
    assert_eq!(1, Board::fr_to_index(2, 1));
    assert_eq!(3, Board::fr_to_index(4, 1));
    assert_eq!(6, Board::fr_to_index(7, 1));
    assert_eq!(7, Board::fr_to_index(8, 1));
    assert_eq!(8, Board::fr_to_index(1, 2));
    assert_eq!(9, Board::fr_to_index(2, 2));
    assert_eq!(62, Board::fr_to_index(7, 8));
    assert_eq!(63, Board::fr_to_index(8, 8));
  }

  #[test]
  fn test_index_to_fr() {
    assert_eq!((1, 1), Board::index_to_fr(0));
    assert_eq!((2, 1), Board::index_to_fr(1));
    assert_eq!((4, 1), Board::index_to_fr(3));
    assert_eq!((7, 1), Board::index_to_fr(6));
    assert_eq!((8, 1), Board::index_to_fr(7));
    assert_eq!((1, 2), Board::index_to_fr(8));
    assert_eq!((2, 2), Board::index_to_fr(9));
    assert_eq!((7, 8), Board::index_to_fr(62));
    assert_eq!((8, 8), Board::index_to_fr(63));
  }

  #[test]
  fn test_get_piece() {
    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    let board = Board::from_fen(fen);
    assert_eq!(BLACK_ROOK, board.get_piece(2, 3));
    assert_eq!(WHITE_KING, board.get_piece(6, 4));
    assert_eq!(BLACK_KING, board.get_piece(7, 7));
  }

  #[test]
  fn apply_board_en_passant_move() {
    let fen = "r1b2rk1/p1Q2p1p/4p1p1/1p6/2pPP3/5N2/PPP2PPP/2KR1B1R b - d3 0 1";
    let mut board = Board::from_fen(fen);

    board.apply_move(&Move::from_string("c4d3"));

    let expected_board =
      Board::from_fen("r1b2rk1/p1Q2p1p/4p1p1/1p6/4P3/3p1N2/PPP2PPP/2KR1B1R w - - 0 2");

    println!("{}", board);
    print_board_mask(board.pieces.pawns());
    print_board_mask(board.pieces.all());
    assert_eq!(board, expected_board);
  }

  #[test]
  fn test_hash_values() {
    // Position 1 - regular move
    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    let mut board = Board::from_fen(fen);
    board.apply_move(&Move {
      src: string_to_square("b3"),
      dest: string_to_square("b4"),
      promotion: NO_PIECE,
    });
    let after_move = board.hash;

    let fen = "8/5pk1/5p1p/2R5/1r3K2/6P1/7P/8 w - - 9 44";
    let board = Board::from_fen(fen);
    println!("Board after move hash: {}", after_move);
    println!("Board computed hash: {}", board.hash);
    assert_eq!(board.hash, after_move);

    // Position 2 - start position
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let mut board = Board::from_fen(fen);
    board.apply_move(&Move {
      src: string_to_square("e2"),
      dest: string_to_square("e4"),
      promotion: NO_PIECE,
    });
    let after_move = board.hash;

    let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1";
    let board = Board::from_fen(fen);
    println!("Board after move hash: {}", after_move);
    println!("Board computed hash: {}", board.hash);
    assert_eq!(board.hash, after_move);

    // Position 3 - King side white castle
    let fen = "rn1qkb1r/pbpp2pp/1p2pn2/8/4p3/2NP2P1/PPP1NPBP/R1BQK2R w KQkq - 0 7";
    let mut board = Board::from_fen(fen);
    board.apply_move(&Move {
      src: string_to_square("e1"),
      dest: string_to_square("g1"),
      promotion: NO_PIECE,
    });
    let after_move = board.hash;

    let fen = "rn1qkb1r/pbpp2pp/1p2pn2/8/4p3/2NP2P1/PPP1NPBP/R1BQ1RK1 b kq - 1 7";
    let board = Board::from_fen(fen);
    println!("Board after move hash: {}", after_move);
    println!("Board computed hash: {}", board.hash);
    assert_eq!(board.hash, after_move);

    // Position 3 - King side black castle
    let fen = "r2qk2r/pbpp2pp/1pn1pn2/2b5/4PB2/2N3P1/PPP1NPBP/R2Q1RK1 b kq - 2 9";
    let mut board = Board::from_fen(fen);
    board.apply_move(&Move {
      src: string_to_square("e8"),
      dest: string_to_square("g8"),
      promotion: NO_PIECE,
    });
    let after_move = board.hash;

    let fen = "r2q1rk1/pbpp2pp/1pn1pn2/2b5/4PB2/2N3P1/PPP1NPBP/R2Q1RK1 w - - 3 10";
    let board = Board::from_fen(fen);
    println!("Board after move hash: {}", after_move);
    println!("Board computed hash: {}", board.hash);
    assert_eq!(board.hash, after_move);

    // Position - Regular capture
    println!("Checking Regular capture");
    let fen = "r2q1rk1/pbpp2p1/1pn2n1p/2b1p1B1/4P3/2N3P1/PPP2PBP/R1NQ1RK1 w - - 0 12";
    let mut board = Board::from_fen(fen);
    board.apply_move(&Move {
      src: string_to_square("g5"),
      dest: string_to_square("f6"),
      promotion: NO_PIECE,
    });
    let after_move = board.hash;

    let fen = "r2q1rk1/pbpp2p1/1pn2B1p/2b1p3/4P3/2N3P1/PPP2PBP/R1NQ1RK1 b - - 0 12";
    let board = Board::from_fen(fen);
    println!("Board after move hash: {}", after_move);
    println!("Board computed hash: {}", board.hash);
    assert_eq!(board.hash, after_move);

    // Position - Promotion
    println!("Checking promotion");
    let fen = "8/7k/6pb/7p/4P3/3n2P1/B1p1N1KP/8 b - - 0 52";
    let mut board = Board::from_fen(fen);
    board.apply_move(&Move {
      src: string_to_square("c2"),
      dest: string_to_square("c1"),
      promotion: BLACK_QUEEN,
    });
    let after_move = board.hash;

    let fen = "8/7k/6pb/7p/4P3/3n2P1/B3N1KP/2q5 w - - 0 53";
    let board = Board::from_fen(fen);
    println!("Board after move hash: {}", after_move);
    println!("Board computed hash: {}", board.hash);
    assert_eq!(board.hash, after_move);

    // Position - White Queen side castle
    println!("Checking promotions queen side");
    let fen = "r3kbnr/ppp2ppp/2npbq2/4p3/4P3/2NPBQ2/PPP2PPP/R3KBNR w KQkq - 4 6";
    let mut board = Board::from_fen(fen);
    board.apply_move(&Move {
      src: string_to_square("e1"),
      dest: string_to_square("c1"),
      promotion: NO_PIECE,
    });
    let after_move = board.hash;

    let fen = "r3kbnr/ppp2ppp/2npbq2/4p3/4P3/2NPBQ2/PPP2PPP/2KR1BNR b kq - 5 6";
    let board = Board::from_fen(fen);
    println!("Board after move hash: {}", after_move);
    println!("Board computed hash: {}", board.hash);
    assert_eq!(board.hash, after_move);

    // Position - Black Queen side castle
    let fen = "r3kbnr/ppp2ppp/2npbq2/4p3/4P3/2NPBQ2/PPP2PPP/2KR1BNR b kq - 5 6";
    let mut board = Board::from_fen(fen);
    board.apply_move(&Move {
      src: string_to_square("e8"),
      dest: string_to_square("c8"),
      promotion: NO_PIECE,
    });
    let after_move = board.hash;

    let fen = "2kr1bnr/ppp2ppp/2npbq2/4p3/4P3/2NPBQ2/PPP2PPP/2KR1BNR w - - 6 7";
    let board = Board::from_fen(fen);
    println!("Board after move hash: {}", after_move);
    println!("Board computed hash: {}", board.hash);
    assert_eq!(board.hash, after_move);

    // Position - Losing Black king side castle
    let fen = "rnb1kbnr/ppp1pNpp/4q3/3p4/8/8/PPPPPPPP/RNBQKB1R w KQkq - 1 4";
    let mut board = Board::from_fen(fen);
    board.apply_move(&Move {
      src: string_to_square("f7"),
      dest: string_to_square("h8"),
      promotion: NO_PIECE,
    });
    let after_move = board.hash;

    let fen = "rnb1kbnN/ppp1p1pp/4q3/3p4/8/8/PPPPPPPP/RNBQKB1R b KQq - 0 4";
    let board = Board::from_fen(fen);
    println!("Board after move hash: {}", after_move);
    println!("Board computed hash: {}", board.hash);
    assert_eq!(board.hash, after_move);
  }

  #[test]
  fn test_game_over_insufficient_material() {
    let fen = "8/4nk2/8/8/8/2K5/8/8 w - - 0 1";
    let board = Board::from_fen(fen);
    assert_eq!(true, board.is_game_over_by_insufficient_material());

    let fen = "8/5k2/8/8/8/2KB4/8/8 w - - 0 1";
    let board = Board::from_fen(fen);
    assert_eq!(true, board.is_game_over_by_insufficient_material());

    let fen = "8/4nk2/8/8/8/2KB4/8/8 w - - 0 1";
    let board = Board::from_fen(fen);
    assert_eq!(false, board.is_game_over_by_insufficient_material());

    let fen = "8/4nk2/8/8/8/2KP4/8/8 w - - 0 1";
    let board = Board::from_fen(fen);
    assert_eq!(false, board.is_game_over_by_insufficient_material());
  }

  #[ignore]
  #[test]
  fn generate_ranks_files() {
    let mut ranks: [u64; 8] = [0; 8];
    let mut files: [u64; 8] = [0; 8];

    for i in 0..64 {
      let (file, rank) = Board::index_to_fr(i);
      set_square_in_mask!(i, ranks[(rank - 1) as usize]);
      set_square_in_mask!(i, files[(file - 1) as usize]);
    }
    println!("pub const RANKS:[u64; 8] = {:#018X?};", ranks);
    println!("pub const FILES:[u64; 8] = {:#018X?};", files);
  }

  #[test]
  fn test_pin_mask_calculations() {
    // Here we have a queen pinning a pawn
    let board = Board::from_fen("rnbqkbnr/pppp1ppp/8/4p2Q/4P3/8/PPPP1PPP/RNB1KBNR b KQkq - 1 2");

    for i in 0..63 {
      //print_board_mask(1 << i);
      if i != 53 {
        assert_eq!(u64::MAX, board.get_pins(i));
      } else {
        assert_eq!(9078117754732544, board.get_pins(i));
      }
    }

    // Here basically all white pieces are pinned.
    let board = Board::from_fen("4r3/3k4/4r2b/8/4RP2/q2PKN1q/8/8 w - - 0 1");
    println!("Board: {}", board);

    // Pawn is pinned to the rook direction
    print_board_mask(board.get_pins(19));
    assert_eq!(983040, board.get_pins(19));

    // knight is pinned to the queen direction
    print_board_mask(board.get_pins(21));
    assert_eq!(14680064, board.get_pins(21));

    // Rook is pinned to the rooks direction
    print_board_mask(board.get_pins(28));
    assert_eq!(17661173956608, board.get_pins(28));

    // Pawn is pinned to the bishop direction
    print_board_mask(board.get_pins(29));
    assert_eq!(141012903133184, board.get_pins(29));
  }

  #[test]
  fn test_get_attackers() {
    let board = Board::from_fen("4r3/2k5/4r2b/6P1/3RRP2/q2PKN1q/8/3B4 w - - 0 1");
    println!("Board: {}", board);

    // A3 attackers:
    assert_eq!(0, board.get_attackers(string_to_square("a3"), Color::White));
    assert_eq!(0, board.get_attackers(string_to_square("a3"), Color::Black));

    // D3 attackers:
    let e = (1 << string_to_square("d4")) | (1 << string_to_square("e3"));
    let a = board.get_attackers(string_to_square("d3"), Color::White);
    assert_eq!(e, a);

    let e = 1 << string_to_square("a3");
    let a = board.get_attackers(string_to_square("d3"), Color::Black);
    assert_eq!(e, a);

    // F3 attackers:
    let e = (1 << string_to_square("d1")) | (1 << string_to_square("e3"));
    let a = board.get_attackers(string_to_square("f3"), Color::White);
    assert_eq!(e, a);

    let e = 1 << string_to_square("h3");
    let a = board.get_attackers(string_to_square("f3"), Color::Black);
    assert_eq!(e, a);

    // G5 attackers:
    let e = (1 << string_to_square("f3")) | (1 << string_to_square("f4"));
    let a = board.get_attackers(string_to_square("g5"), Color::White);
    assert_eq!(e, a);

    let e = 1 << string_to_square("h6");
    let a = board.get_attackers(string_to_square("g5"), Color::Black);
    assert_eq!(e, a);

    // F6 attackers:
    let e = 1 << string_to_square("g5");
    let a = board.get_attackers(string_to_square("f6"), Color::White);
    assert_eq!(e, a);

    let e = 1 << string_to_square("e6");
    let a = board.get_attackers(string_to_square("f6"), Color::Black);
    assert_eq!(e, a);

    // Test with board edges/pawns
    let board = Board::from_fen("6k1/5P2/4P3/8/4K3/8/8/8 w - - 0 1  ");
    println!("---------------------------------------------");
    println!("Board: {}", board);
    // G8 attackers:
    let e = 1 << string_to_square("f7");
    let a = board.get_attackers(string_to_square("g8"), Color::White);
    assert_eq!(e, a);

    let e = 0;
    let a = board.get_attackers(string_to_square("g8"), Color::Black);
    assert_eq!(e, a);

    let board = Board::from_fen("8/8/8/3k4/8/5p2/4p3/3K4 w - - 0 1");
    println!("---------------------------------------------");
    println!("Board: {}", board);
    // D1 attackers:
    let e = 0;
    let a = board.get_attackers(string_to_square("d1"), Color::White);
    assert_eq!(e, a);

    let e = 1 << string_to_square("e2");
    let a = board.get_attackers(string_to_square("d1"), Color::Black);
    assert_eq!(e, a);
  }
}
