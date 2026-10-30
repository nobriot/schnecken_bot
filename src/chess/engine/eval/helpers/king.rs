use crate::engine::eval::helpers::generic::*;
use crate::engine::eval::position::Score;
use crate::model::board_geometry::*;
use crate::model::board_mask::BoardMask;
use crate::model::game_state::*;
use crate::model::piece::*;
use crate::model::piece_moves::KING_MOVES;
use crate::square_in_mask;
use log::*;

// -----------------------------------------------------------------------------
// Constants

/// White King shelter pawns
pub const WHITE_KING_SHELTER_PAWNS: [u64; 64] = [0x0303030303030300,
                                                 0x0707070707070700,
                                                 0x0E0E0E0E0E0E0E00,
                                                 0x1C1C1C1C1C1C1C00,
                                                 0x3838383838383800,
                                                 0x7070707070707000,
                                                 0xE0E0E0E0E0E0E000,
                                                 0xC0C0C0C0C0C0C000,
                                                 0x0303030303030000,
                                                 0x0707070707070000,
                                                 0x0E0E0E0E0E0E0000,
                                                 0x1C1C1C1C1C1C0000,
                                                 0x3838383838380000,
                                                 0x7070707070700000,
                                                 0xE0E0E0E0E0E00000,
                                                 0xC0C0C0C0C0C00000,
                                                 0x0303030303000000,
                                                 0x0707070707000000,
                                                 0x0E0E0E0E0E000000,
                                                 0x1C1C1C1C1C000000,
                                                 0x3838383838000000,
                                                 0x7070707070000000,
                                                 0xE0E0E0E0E0000000,
                                                 0xC0C0C0C0C0000000,
                                                 0x0303030300000000,
                                                 0x0707070700000000,
                                                 0x0E0E0E0E00000000,
                                                 0x1C1C1C1C00000000,
                                                 0x3838383800000000,
                                                 0x7070707000000000,
                                                 0xE0E0E0E000000000,
                                                 0xC0C0C0C000000000,
                                                 0x0303030000000000,
                                                 0x0707070000000000,
                                                 0x0E0E0E0000000000,
                                                 0x1C1C1C0000000000,
                                                 0x3838380000000000,
                                                 0x7070700000000000,
                                                 0xE0E0E00000000000,
                                                 0xC0C0C00000000000,
                                                 0x0303000000000000,
                                                 0x0707000000000000,
                                                 0x0E0E000000000000,
                                                 0x1C1C000000000000,
                                                 0x3838000000000000,
                                                 0x7070000000000000,
                                                 0xE0E0000000000000,
                                                 0xC0C0000000000000,
                                                 0x0300000000000000,
                                                 0x0700000000000000,
                                                 0x0E00000000000000,
                                                 0x1C00000000000000,
                                                 0x3800000000000000,
                                                 0x7000000000000000,
                                                 0xE000000000000000,
                                                 0xC000000000000000,
                                                 0x0000000000000000,
                                                 0x0000000000000000,
                                                 0x0000000000000000,
                                                 0x0000000000000000,
                                                 0x0000000000000000,
                                                 0x0000000000000000,
                                                 0x0000000000000000,
                                                 0x0000000000000000];

/// White King shelter pawns
pub const BLACK_KING_SHELTER_PAWNS: [u64; 64] = [0x0000000000000000,
                                                 0x0000000000000000,
                                                 0x0000000000000000,
                                                 0x0000000000000000,
                                                 0x0000000000000000,
                                                 0x0000000000000000,
                                                 0x0000000000000000,
                                                 0x0000000000000000,
                                                 0x0000000000000003,
                                                 0x0000000000000007,
                                                 0x000000000000000E,
                                                 0x000000000000001C,
                                                 0x0000000000000038,
                                                 0x0000000000000070,
                                                 0x00000000000000E0,
                                                 0x00000000000000C0,
                                                 0x0000000000000303,
                                                 0x0000000000000707,
                                                 0x0000000000000E0E,
                                                 0x0000000000001C1C,
                                                 0x0000000000003838,
                                                 0x0000000000007070,
                                                 0x000000000000E0E0,
                                                 0x000000000000C0C0,
                                                 0x0000000000030303,
                                                 0x0000000000070707,
                                                 0x00000000000E0E0E,
                                                 0x00000000001C1C1C,
                                                 0x0000000000383838,
                                                 0x0000000000707070,
                                                 0x0000000000E0E0E0,
                                                 0x0000000000C0C0C0,
                                                 0x0000000003030303,
                                                 0x0000000007070707,
                                                 0x000000000E0E0E0E,
                                                 0x000000001C1C1C1C,
                                                 0x0000000038383838,
                                                 0x0000000070707070,
                                                 0x00000000E0E0E0E0,
                                                 0x00000000C0C0C0C0,
                                                 0x0000000303030303,
                                                 0x0000000707070707,
                                                 0x0000000E0E0E0E0E,
                                                 0x0000001C1C1C1C1C,
                                                 0x0000003838383838,
                                                 0x0000007070707070,
                                                 0x000000E0E0E0E0E0,
                                                 0x000000C0C0C0C0C0,
                                                 0x0000030303030303,
                                                 0x0000070707070707,
                                                 0x00000E0E0E0E0E0E,
                                                 0x00001C1C1C1C1C1C,
                                                 0x0000383838383838,
                                                 0x0000707070707070,
                                                 0x0000E0E0E0E0E0E0,
                                                 0x0000C0C0C0C0C0C0,
                                                 0x0003030303030303,
                                                 0x0007070707070707,
                                                 0x000E0E0E0E0E0E0E,
                                                 0x001C1C1C1C1C1C1C,
                                                 0x0038383838383838,
                                                 0x0070707070707070,
                                                 0x00E0E0E0E0E0E0E0,
                                                 0x00C0C0C0C0C0C0C0];

// -----------------------------------------------------------------------------
// Functions

/// Determine the number of attacked squares surrounding the king
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine the number of pawn islands
///
/// # Returns
///
/// The percentage (0 to 100) of the squares surrounding the king attacked by
/// enemy pieces.
pub fn get_king_danger_score(game_state: &GameState, color: Color) -> Score {
  // debug_assert!(game_state.board.pieces.white.king != 0 &&
  // game_state.board.pieces.black.king != 0);
  if game_state.board.pieces.white.king == 0 || game_state.board.pieces.black.king == 0 {
    debug!("King disappeared {}", game_state.to_fen());
    return 0;
  }

  let surrounding_squares = match color {
    Color::White => KING_MOVES[game_state.board.pieces.white.king.trailing_zeros() as usize],
    Color::Black => KING_MOVES[game_state.board.pieces.black.king.trailing_zeros() as usize],
  };

  let total_squares = surrounding_squares.count_ones() as Score;
  let attacked_squares =
    game_state.board
              .get_attacked_squares(surrounding_squares, Color::opposite(color))
              .count_ones() as Score;

  attacked_squares * 100 / total_squares
}

/// Counts "danger units" around the king of a color: the higher, the more
/// likely the king is to get mated. Only counts when the opponent still has a
/// queen, as attacks without queen are rarely deadly.
///
/// Units:
/// * 1 per square around the king attacked by the opponent
/// * 2 per file next to the king (or the king file) without our pawns, plus 2 if an opponent rook
///   or queen is sitting on that file
/// * 2 per opponent pawn close to the king (pawn storm, or a pawn wedged next to the king like a
///   pawn on f6 against a king on g8)
///
/// ### Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color of the king for which we evaluate the danger
///
/// ### Returns
///
/// Number of danger units for the king.
pub fn get_king_attack_units(game_state: &GameState, color: Color) -> u32 {
  let (ssp, op) = match color {
    Color::White => (&game_state.board.pieces.white, &game_state.board.pieces.black),
    Color::Black => (&game_state.board.pieces.black, &game_state.board.pieces.white),
  };
  if op.queen == 0 || ssp.king == 0 {
    return 0;
  }

  let king = ssp.king.trailing_zeros() as u8;
  let king_zone = KING_MOVES[king as usize];
  let mut units =
    game_state.board.get_attacked_squares(king_zone, Color::opposite(color)).count_ones();

  let king_file = (king % 8) as usize;
  let first_file = king_file.saturating_sub(1);
  let last_file = (king_file + 1).min(7);
  for file in &FILES[first_file..=last_file] {
    if file & ssp.pawn == 0 {
      units += 2;
      if file & (op.rook | op.queen) != 0 {
        units += 2;
      }
    }
  }

  let mut opponent_pawns = op.pawn;
  while opponent_pawns != 0 {
    let pawn = opponent_pawns.trailing_zeros() as u8;
    if get_king_distance(king, pawn) <= 2 {
      units += 2;
    }
    opponent_pawns &= opponent_pawns - 1;
  }

  units
}

/// Checks if the king is way too adventurous
///
/// Noticed that the engine likes to walk
/// the king up the board, but it should not do that unless opponent has no more
/// major pieces
///
/// ### Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine the number of pawn islands
///
/// ### Returns
///
/// True if the king has left its home rank and major enemy pieces are still
/// here.
pub fn is_king_too_adventurous(game_state: &GameState, color: Color) -> bool {
  if color == Color::White {
    (game_state.board.pieces.white.king & BOARD_DOWN_EDGE == 0)
    && game_state.board.pieces.black.majors().count_ones() > 0
  } else {
    (game_state.board.pieces.black.king & BOARD_UP_EDGE == 0)
    && game_state.board.pieces.white.majors().count_ones() > 0
  }
}

/// Returns a BoardMasks of the pawns serving as a shelter for the king.
///
///
/// ### Arguments
///
/// * `game_state` - Reference to a GameState
/// * `color` -      The color for which we want to determine the number of pawn islands
///
/// ### Returns
///
/// integer in the range [-10;+10], -10 being a king living on the edge and +10
/// is a very cosy king.
pub fn get_king_pawns(game_state: &GameState, color: Color) -> BoardMask {
  let king = game_state.board.get_king(color) as usize;

  match color {
    Color::White => WHITE_KING_SHELTER_PAWNS[king] & game_state.board.pieces.white.pawn,
    Color::Black => BLACK_KING_SHELTER_PAWNS[king] & game_state.board.pieces.black.pawn,
  }
}

/// Tries to assess the king safety based on how many pawns it has on its side
/// of the board (if it moved from the start square)
///
/// ### Arguments
///
/// * `game_state` - Reference to a GameState
/// * `color` -      The color for which we want to determine the number of pawn islands
///
/// ### Returns
///
/// integer in the range [-10;+10], -10 being a king living on the edge and +10
/// is a very cosy king.
pub fn king_shelter_value(game_state: &GameState, color: Color) -> isize {
  let king = game_state.board.get_king(color) as usize;

  let board_side = if square_in_mask!(king, QUEEN_SIDE_MASK) {
    QUEEN_SIDE_MASK
  } else {
    KING_SIDE_MASK
  };

  let mut open_files = 0;
  let mut half_open_files = 0;
  for f in 1..=8 {
    if FILES[f - 1] & board_side == 0 {
      continue;
    }

    match get_file_state(game_state, f as u8) {
      FileState::Open => open_files += 1,
      FileState::Closed => continue,
      FileState::HalfOpen => half_open_files += 1,
    }
  }

  let (ssp, op) = match color {
    Color::White => (game_state.board.pieces.white, game_state.board.pieces.black),
    Color::Black => (game_state.board.pieces.black, game_state.board.pieces.white),
  };

  let piece_imbalance = (board_side & (ssp.majors() | ssp.minors())).count_ones() as isize
                        - (board_side & (op.majors() | op.minors())).count_ones() as isize;

  let score = piece_imbalance * (open_files + half_open_files / 2);
  score.clamp(-10, 10)
}

/// Returns true if the king does not seem castled and has lost his castling
/// rights
///
/// ### Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine the number of pawn islands
///
/// ### Returns
///
/// True if the king is not on a castling destination square and has no castling
/// rights
pub fn are_casling_rights_lost(game_state: &GameState, color: Color) -> bool {
  match color {
    Color::White => {
      if game_state.board.castling_rights.K() || game_state.board.castling_rights.Q() {
        return false;
      }
    },
    Color::Black => {
      if game_state.board.castling_rights.k() || game_state.board.castling_rights.q() {
        return false;
      }
    },
  }

  let king_square = match color {
    Color::White => game_state.board.get_white_king_square(),
    Color::Black => game_state.board.get_black_king_square(),
  };

  (color == Color::White && (king_square != 2 && king_square != 6))
  || (color == Color::Black && (king_square != 62 && king_square != 58))
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_king_attack_units() {
    // Starting position: no danger
    let game_state = GameState::from_fen(START_POSITION_FEN);
    assert_eq!(0, get_king_attack_units(&game_state, Color::White));
    assert_eq!(0, get_king_attack_units(&game_state, Color::Black));

    // No opponent queen, no danger counted
    let game_state = GameState::from_fen("rnb1kbnr/pppppppp/8/8/8/8/8/RNBQKBNR w KQkq - 0 1");
    assert_eq!(0, get_king_attack_units(&game_state, Color::White));

    // https://lichess.org/lrChWe6R after 18. Nf6+ gxf6 19. gxf6
    // Black king is busted: g and h files open, rook on h1, pawn on f6.
    let game_state =
      GameState::from_fen("r1b1r1k1/p1nq1p2/2p1pP2/1p6/2pP4/P7/1P1Q1PP1/2KR1B1R b - - 0 19");
    let black_units = get_king_attack_units(&game_state, Color::Black);
    let white_units = get_king_attack_units(&game_state, Color::White);
    println!("King attack units: White {white_units} - Black {black_units}");
    assert!(black_units >= 10);
    assert!(black_units > 2 * white_units);
  }

  #[test]
  fn test_get_king_danger_score() {
    let fen = "rnb1kbnr/pppp1ppp/5q2/4p3/4P3/5Q2/PPPP1PPP/RNB1KBNR w KQkq - 2 3";
    let game_state = GameState::from_fen(fen);
    assert_eq!(0, get_king_danger_score(&game_state, Color::Black));
    assert_eq!(0, get_king_danger_score(&game_state, Color::White));

    let fen = "1r1qk1nr/p2bppbp/6p1/1p2N3/3p1P2/1Q4P1/PP1PP1BP/R1B1K2R b KQk - 0 12";
    let game_state = GameState::from_fen(fen);
    assert_eq!(200 / 5, get_king_danger_score(&game_state, Color::Black));
    assert_eq!(0, get_king_danger_score(&game_state, Color::White));

    let fen = "6k1/4pp1p/2n3p1/P7/8/6P1/3P1QKP/2q5 b - - 1 33";
    let game_state = GameState::from_fen(fen);
    assert_eq!(100 / 5, get_king_danger_score(&game_state, Color::Black));
    assert_eq!(300 / 8, get_king_danger_score(&game_state, Color::White));

    let fen = "8/4ppkp/2n3p1/P7/8/6P1/3P1QKP/2q5 w - - 2 34";
    let game_state = GameState::from_fen(fen);

    assert_eq!(200 / 8, get_king_danger_score(&game_state, Color::Black));
    assert_eq!(300 / 8, get_king_danger_score(&game_state, Color::White));
  }

  #[test]
  fn test_get_king_shelter_value() {
    let fen = "rnb1kbnr/pppp1ppp/5q2/4p3/4P3/5Q2/PPPP1PPP/RNB1KBNR w KQkq - 2 3";
    let game_state = GameState::from_fen(fen);
    assert_eq!(0, king_shelter_value(&game_state, Color::Black));
    assert_eq!(0, king_shelter_value(&game_state, Color::White));

    let fen = "1r1qk1nr/p2bppbp/6p1/1p2N3/3p1P2/1Q4P1/PP1PP1BP/R1B1K2R b KQk - 0 12";
    let game_state = GameState::from_fen(fen);
    assert_eq!(0, king_shelter_value(&game_state, Color::Black));
    assert_eq!(0, king_shelter_value(&game_state, Color::White));

    let fen = "6k1/4pp1p/2n3p1/P7/8/6P1/3P1QKP/2q5 b - - 1 33";
    let game_state = GameState::from_fen(fen);
    assert_eq!(-1, king_shelter_value(&game_state, Color::Black));
    assert_eq!(1, king_shelter_value(&game_state, Color::White));

    let fen = "8/4ppkp/2n3p1/P7/8/6P1/3P1QKP/2q5 w - - 2 34";
    let game_state = GameState::from_fen(fen);

    assert_eq!(-1, king_shelter_value(&game_state, Color::Black));
    assert_eq!(1, king_shelter_value(&game_state, Color::White));

    let fen = "r1bq1r2/ppp1bpk1/4p2p/2npP1pQ/7P/2P1P1B1/PPBN1PP1/R3K2R b KQ - 2 13";
    let game_state = GameState::from_fen(fen);

    assert_eq!(0, king_shelter_value(&game_state, Color::Black));
    assert_eq!(0, king_shelter_value(&game_state, Color::White));

    let fen = "r1bq1r2/ppp1bpk1/4p2p/2npP2Q/7p/2P1P1B1/PPBN1PP1/R3K2R w KQ - 0 14";
    let game_state = GameState::from_fen(fen);

    assert_eq!(-1, king_shelter_value(&game_state, Color::Black));
    assert_eq!(1, king_shelter_value(&game_state, Color::White));

    let fen = "r1bq1r2/ppp5/4pkQp/2np2b1/5B1R/2P1P3/PPBN1PP1/R3K3 b Q - 1 17";
    let game_state = GameState::from_fen(fen);

    assert_eq!(-1, king_shelter_value(&game_state, Color::Black));
    assert_eq!(1, king_shelter_value(&game_state, Color::White));
  }
}
