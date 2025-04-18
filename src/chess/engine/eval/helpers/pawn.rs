use crate::model::board::*;
use crate::model::board_geometry::holes::*;
use crate::model::board_geometry::passed_pawns_areas::*;
use crate::model::board_geometry::*;
use crate::model::board_mask::*;
use crate::model::game_state::*;
use crate::model::piece::*;
use crate::model::tables::pawn_destinations::*;

// State to track pawn islands
#[derive(PartialEq)]
enum PawnTrackingState {
  NoPawn,
  Pawn,
}

/// Determine if a pawn on the board can be protected by another pawn
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play,
///   etc.
/// * `index` -      Index of the square on the board where the pawn is located
///
/// # Return value
///
/// true if the pawn is protected, false if there is no pawn on the square index
/// or it is not protected.
pub fn can_become_protected(game_state: &GameState, index: u8) -> bool {
  // Check if there is no other pawn behind:
  let pawn = game_state.board.pieces.get(index);
  let (file, mut rank) = Board::index_to_fr(index);

  // Check if it can be defended by another pawn behind or on the side:
  while (1..=8).contains(&rank) {
    // Check on the left side:
    if file > 1 {
      let s = Board::fr_to_index(file - 1, rank);
      if game_state.board.pieces.get(s) == pawn {
        return true;
      }
    }
    // Check on the right side:
    if file < 8 {
      let s = Board::fr_to_index(file + 1, rank);
      if game_state.board.pieces.get(s) == pawn {
        return true;
      }
    }

    // white pawns are behind in ranks, black pawns ahead
    if pawn == WHITE_PAWN {
      rank -= 1;
    } else {
      rank += 1;
    }
  }

  false
}

/// Returns a board mask with backwards pawns for a color
///
/// ### Arguments
///
/// * `game_state` - A GameState object representing a position, side to play,
///   etc.
/// * `color` -      The color for which we want to determine backwards pawns
///
/// ### Return value
///
/// BoardMask indicating which squares have a backwards pawn on it.
pub fn get_backwards_pawns(game_state: &GameState, color: Color) -> BoardMask {
  let mut mask: BoardMask = 0;

  let pawn = match color {
    Color::White => WHITE_PAWN,
    Color::Black => BLACK_PAWN,
  };
  let op_pawn = match color {
    Color::White => BLACK_PAWN,
    Color::Black => WHITE_PAWN,
  };

  for i in 0..64 {
    if game_state.board.pieces.get(i) != pawn {
      continue;
    }

    if can_become_protected(game_state, i) {
      continue;
    }

    // Check if a opposite pawn controls the square in front:
    let (file, rank) = Board::index_to_fr(i);
    let new_rank = match color {
      Color::White => rank + 2,
      Color::Black => rank - 2,
    };

    if (1..=8).contains(&new_rank) {
      continue;
    }

    // Check on the left side:
    if file > 1 {
      let s = Board::fr_to_index(file - 1, new_rank);
      if game_state.board.pieces.get(s) == op_pawn {
        mask |= 1 << i;
        continue;
      }
    }

    // Check on the right side:
    if file < 8 {
      let s = Board::fr_to_index(file + 1, new_rank);
      if game_state.board.pieces.get(s) == op_pawn {
        mask |= 1 << i;
        continue;
      }
    }
  }
  mask
}

/// Determine the number of pawn islands in a position for a given color.
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play,
///   etc.
/// * `color` -      The color for which we want to determine the number of pawn
///   islands
pub fn get_number_of_pawn_islands(game_state: &GameState, color: Color) -> usize {
  let pawns = match color {
    Color::White => game_state.board.pieces.white.pawn,
    Color::Black => game_state.board.pieces.black.pawn,
  };
  let mut pawn_islands: usize = 0;
  let mut pawn_state = PawnTrackingState::NoPawn;

  for file in &FILES {
    if file & pawns != 0 {
      if pawn_state == PawnTrackingState::NoPawn {
        pawn_islands += 1;
        pawn_state = PawnTrackingState::Pawn;
      }
    } else {
      pawn_state = PawnTrackingState::NoPawn;
    }
  }

  pawn_islands
}

/// Determine if a pawn on the board is protected by another pawn
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play,
///   etc.
/// * `index` -      Index of the square on the board where the pawn is located
///
/// # Return value
///
/// true if the pawn is protected, false if there is no pawn on the square index
/// or it is not protected.
pub fn is_protected(game_state: &GameState, index: u8) -> bool {
  // Same side pawn
  let ss_pawn = match game_state.board.pieces.get(index) {
    WHITE_PAWN => WHITE_PAWN,
    BLACK_PAWN => BLACK_PAWN,
    _ => return false,
  };

  // Determine the rank / File:
  let (file, mut rank) = Board::index_to_fr(index);

  // Check if we are protected, back one rank and one file on the right and/or
  // left.
  if ss_pawn == WHITE_PAWN {
    rank -= 1;
  } else {
    rank += 1;
  }
  // Would be strange here, (there should be no pawn on the edge of the board,
  // unless the position is kinda random)
  if !(1..=8).contains(&rank) {
    return false;
  }

  // Check on the left side:
  if file > 1 && game_state.board.get_piece(file - 1, rank) == ss_pawn {
    return true;
  }
  // Check on the right side:
  if file < 8 && game_state.board.get_piece(file + 1, rank) == ss_pawn {
    return true;
  }

  false
}

/// Determine if a square on the board is protected by a pawn of a certain color
///
/// ### Arguments
///
/// * `game_state` - A GameState object representing a position, side to play,
///   etc.
/// * `index` -      Index of the square on the board
/// * `color` -      Color to check for pawns
///
/// ### Return value
///
/// true if the pawn is protected by a pawn of the indicated color, false
/// otherwise
pub fn is_square_protected_by_pawn(game_state: &GameState, index: u8, color: Color) -> bool {
  // Find the pawn value
  let pawn = match color {
    Color::White => WHITE_PAWN,
    Color::Black => BLACK_PAWN,
  };

  // Determine the rank / File:
  let (file, mut rank) = Board::index_to_fr(index);

  // Check if we are protected, back one rank and one file on the right and/or
  // left.
  if pawn == WHITE_PAWN {
    rank -= 1;
  } else {
    rank += 1;
  }
  // 1st rank can be defended by white pawns, and same for 8th rank/black pawns
  if !(1..=8).contains(&rank) {
    return false;
  }

  // Check on the left side:
  if file > 1 && game_state.board.get_piece(file - 1, rank) == pawn {
    return true;
  }
  // Check on the right side:
  if file < 8 && game_state.board.get_piece(file + 1, rank) == pawn {
    return true;
  }

  false
}

/// Determine if a pawn on the board is passed.
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play,
///   etc.
/// * `index` -      Index of the square on the board where the pawn is located
///
/// # Return value
///
/// true if the pawn is passed, false if there is no pawn on the square index
/// or it is not passed.
pub fn is_passed(game_state: &GameState, index: u8) -> bool {
  if square_in_mask!(index, game_state.board.pieces.white.pawn) {
    return WHITE_PASSED_PAWN_AREA[index as usize] & game_state.board.pieces.black.pawn == 0;
  } else if square_in_mask!(index, game_state.board.pieces.black.pawn) {
    return BLACK_PASSED_PAWN_AREA[index as usize] & game_state.board.pieces.white.pawn == 0;
  }

  false
}

/// Determine the number of passed pawns in a position for a given color.
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play,
///   etc.
/// * `color` -      The color for which we want to determine the number of pawn
///   islands
pub fn get_number_of_passers(game_state: &GameState, color: Color) -> usize {
  // Same side pawn
  let mut pawns = match color {
    Color::White => game_state.board.pieces.white.pawn,
    Color::Black => game_state.board.pieces.black.pawn,
  };
  let mut passers: usize = 0;

  while pawns != 0 {
    if is_passed(game_state, pawns.trailing_zeros() as u8) {
      passers += 1;
    }

    pawns &= pawns - 1;
  }

  passers
}

/// Determine the number of passed pawns in a position for a given color.
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play,
///   etc.
/// * `color` -      The color for which we want to determine the number of pawn
///   islands
pub fn get_passed_pawns(game_state: &GameState, color: Color) -> BoardMask {
  // Same side pawn
  let mut pawns = match color {
    Color::White => game_state.board.pieces.white.pawn,
    Color::Black => game_state.board.pieces.black.pawn,
  };
  let mut passers: BoardMask = 0;

  while pawns != 0 {
    if is_passed(game_state, pawns.trailing_zeros() as u8) {
      passers += 1;
    }

    pawns &= pawns - 1;
  }

  passers
}

/// Determine the number of protected passed pawns in a position for a given
/// color.
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play,
///   etc.
/// * `color` -      The color for which we want to determine the number of pawn
///   islands
pub fn get_number_of_protected_passers(game_state: &GameState, color: Color) -> usize {
  // Same side pawn
  let mut pawns = match color {
    Color::White => game_state.board.pieces.white.pawn,
    Color::Black => game_state.board.pieces.black.pawn,
  };
  let mut connected_passers: usize = 0;

  while pawns != 0 {
    let i = pawns.trailing_zeros() as u8;
    if is_passed(game_state, i) && is_protected(game_state, i) {
      connected_passers += 1;
    }

    pawns &= pawns - 1;
  }

  connected_passers
}

/// Determine the number of protected pawns (by other pawns) in a position for a
/// given color.
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play,
///   etc.
/// * `color` -      The color for which we want to determine the number of pawn
///   islands
pub fn get_number_of_protected_pawns(game_state: &GameState, color: Color) -> usize {
  // Same side pawn
  let mut pawns = match color {
    Color::White => game_state.board.pieces.white.pawn,
    Color::Black => game_state.board.pieces.black.pawn,
  };
  let mut protected_pawns: usize = 0;

  while pawns != 0 {
    let i = pawns.trailing_zeros() as u8;
    if is_protected(game_state, i) {
      protected_pawns += 1;
    }

    pawns &= pawns - 1;
  }

  protected_pawns
}

/// Determine the number of squares left for the pawn closest to promotion.
///
/// ### Arguments
///
/// * `game_state` - A GameState object representing a position, side to play,
///   etc.
/// * `color` -      The color for which we want to determine the closest pawn
///   to promotion
///
/// ### Return value
///
/// 8 if no pawn is present, else the number of squares that the closest pawn
/// has to travel to get promoted
pub fn get_distance_left_for_closest_pawn_to_promotion(game_state: &GameState, color: Color) -> u8 {
  let mut best_distance: u8 = 8;

  // Same side pawn
  let (mut pawns, target_rank) = match color {
    Color::White => (game_state.board.pieces.white.pawn, 8),
    Color::Black => (game_state.board.pieces.black.pawn, 1),
  };

  while pawns != 0 {
    let square = pawns.trailing_zeros();
    let (_, rank) = Board::index_to_fr(square as u8);
    let distance = rank.abs_diff(target_rank);
    if distance < best_distance {
      best_distance = distance;
    }

    pawns &= pawns - 1;
  }

  best_distance
}

/// Determines holes in our position
/// (squares that cannot be defended by a pawn anymore)
///
/// ### Arguments
///
/// * `game_state` - A GameState object representing a position, side to play,
///   etc.
/// * `color` -      The color for which we want to determine holes
///
/// ### Return value
///
/// Board mask with the holes for that 8 if no pawn is present, else the number
/// of squares that the closest pawn has to travel to get promoted
pub fn get_holes(game_state: &GameState, color: Color) -> BoardMask {
  let mut holes: BoardMask = 0;

  let mut area = HOLES_BOARD_AREA;

  while area != 0 {
    let i = area.trailing_zeros() as usize;

    match color {
      Color::White => {
        if game_state.board.pieces.white.pawn & HOLES_WHITE_PAWN_PLACEMENT[i] == 0 {
          holes |= 1 << i;
        }
      },
      Color::Black => {
        if game_state.board.pieces.black.pawn & HOLES_BLACK_PAWN_PLACEMENT[i] == 0 {
          holes |= 1 << i;
        }
      },
    };

    area &= area - 1;
  }

  holes
}

/// Computes the number of pieces attacked by defended pawns
#[inline]
pub fn get_pawn_victims(game_state: &GameState, color: Color) -> u32 {
  let mut victims: u32 = 0;
  // Look for pawns attacking pieces, or forking
  let mut pawns = match color {
    Color::White => game_state.board.pieces.white.pawn,
    Color::Black => game_state.board.pieces.black.pawn,
  };
  while pawns != 0 {
    let pawn = pawns.trailing_zeros() as u8;
    let defenders = game_state.board.get_attackers(pawn, color);
    let attackers = game_state.board.get_attackers(pawn, Color::opposite(color));

    // Check that the pawn cannot be taken out too easily before assigning a bonus
    // for the pawn attack.
    if attackers.count_ones() <= defenders.count_ones() {
      let attacked_pieces = match color {
        Color::White => (WHITE_PAWN_CONTROL[pawn as usize]
                         & (game_state.board.pieces.black.majors()
                            | game_state.board.pieces.black.minors()
                            | game_state.board.pieces.black.king))
                                                                  .count_few_ones(),
        Color::Black => (BLACK_PAWN_CONTROL[pawn as usize]
                         & (game_state.board.pieces.white.majors()
                            | game_state.board.pieces.white.minors()
                            | game_state.board.pieces.white.king))
                                                                  .count_few_ones(),
      };

      victims += attacked_pieces;
    }

    pawns &= pawns - 1;
  }

  victims
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn get_islands_for_simple_pawn_structure() {
    let fen = "rnb1kbnr/pppp1ppp/5q2/4p3/4P3/5Q2/PPPP1PPP/RNB1KBNR w KQkq - 2 3";
    let game_state = GameState::from_fen(fen);

    assert_eq!(1, get_number_of_pawn_islands(&game_state, Color::Black));
    assert_eq!(1, get_number_of_pawn_islands(&game_state, Color::White));
  }

  #[test]
  fn get_islands_with_no_pawn() {
    let fen = "8/7k/5n2/4Q3/q7/3N4/3K4/8 b - - 3 51";
    let game_state = GameState::from_fen(fen);

    assert_eq!(0, get_number_of_pawn_islands(&game_state, Color::Black));
    assert_eq!(0, get_number_of_pawn_islands(&game_state, Color::White));
  }

  #[test]
  fn get_islands_endgame() {
    let fen = "8/5ppk/5n1p/4QP2/q3P3/p1PN4/2K5/8 w - - 2 51";
    let game_state = GameState::from_fen(fen);

    assert_eq!(2, get_number_of_pawn_islands(&game_state, Color::Black));
    assert_eq!(2, get_number_of_pawn_islands(&game_state, Color::White));
  }

  #[test]
  fn get_pawn_data_endgame_1() {
    let fen = "6k1/R7/6p1/6P1/7P/8/p5K1/r7 w - - 14 55";
    let game_state = GameState::from_fen(fen);

    assert_eq!(2, get_number_of_pawn_islands(&game_state, Color::Black));
    assert_eq!(1, get_number_of_pawn_islands(&game_state, Color::White));

    assert_eq!(1, get_number_of_passers(&game_state, Color::Black));
    assert_eq!(0, get_number_of_passers(&game_state, Color::White));

    assert_eq!(0,
               get_number_of_protected_passers(&game_state, Color::Black));
    assert_eq!(0,
               get_number_of_protected_passers(&game_state, Color::White));
  }

  #[test]
  fn get_pawn_data_endgame_2() {
    let fen = "8/4kp2/1p6/3pK3/8/8/P1P1P1Pp/8 w - - 0 1";
    let game_state = GameState::from_fen(fen);

    assert_eq!(4, get_number_of_pawn_islands(&game_state, Color::White));
    assert_eq!(4, get_number_of_pawn_islands(&game_state, Color::Black));

    assert_eq!(1, get_number_of_passers(&game_state, Color::Black));
    assert_eq!(0, get_number_of_passers(&game_state, Color::White));

    assert_eq!(0,
               get_number_of_protected_passers(&game_state, Color::Black));
    assert_eq!(0,
               get_number_of_protected_passers(&game_state, Color::White));
    assert_eq!(6,
               get_distance_left_for_closest_pawn_to_promotion(&game_state, Color::White));
    assert_eq!(1,
               get_distance_left_for_closest_pawn_to_promotion(&game_state, Color::Black));
  }

  #[test]
  fn get_pawn_data_endgame_3() {
    let fen = "6k1/3p4/2p3pP/1p4P1/8/8/6K1/8 w - - 14 55";
    let game_state = GameState::from_fen(fen);

    assert_eq!(1, get_number_of_pawn_islands(&game_state, Color::White));
    assert_eq!(2, get_number_of_pawn_islands(&game_state, Color::Black));

    assert_eq!(3, get_number_of_passers(&game_state, Color::Black));
    assert_eq!(1, get_number_of_passers(&game_state, Color::White));

    assert_eq!(2,
               get_number_of_protected_passers(&game_state, Color::Black));
    assert_eq!(1,
               get_number_of_protected_passers(&game_state, Color::White));
    assert_eq!(2,
               get_distance_left_for_closest_pawn_to_promotion(&game_state, Color::White));
    assert_eq!(4,
               get_distance_left_for_closest_pawn_to_promotion(&game_state, Color::Black));
  }

  #[test]
  fn test_backwards_pawns() {
    // 2 backwards pawn in here: d7 and d4
    let fen = "rnbqkbnr/pp1p3p/2p3p1/2Pp2p1/PP1P4/8/6PP/RNBQKBNR w KQkq - 0 4";
    let game_state = GameState::from_fen(fen);
    let mask = get_backwards_pawns(&game_state, Color::White);
    print_board_mask(mask);
    assert_eq!(1, mask.count_ones());
    assert_eq!(1 << 27, mask);

    let mask = get_backwards_pawns(&game_state, Color::Black);
    print_board_mask(mask);
    assert_eq!(1, mask.count_ones());
    assert_eq!(1 << 51, mask)
  }

  #[test]
  fn test_get_holes() {
    let fen = "r1b2r2/1p4bk/2pR1npn/p6p/2P1PP2/1PN4P/PB2N1B1/5RK1 b - - 0 19";
    let game_state = GameState::from_fen(fen);
    let mask = get_holes(&game_state, Color::White);
    print_board_mask(mask);
    assert_eq!(13, mask.count_ones());
    let mask = get_holes(&game_state, Color::Black);
    print_board_mask(mask);
    assert_eq!(10, mask.count_ones());
  }

  #[test]
  fn test_pawn_attack() {
    // Black pawn attacking white pieces:
    let fen = "rnbq1rk1/ppp1bppp/3p1n2/8/3N4/2P5/PP2BPPP/RNBQ1RK1 b - - 7 9";
    let game_state = GameState::from_fen(fen);
    assert_eq!(0, get_pawn_victims(&game_state, Color::White));
    assert_eq!(0, get_pawn_victims(&game_state, Color::Black));

    let fen = "rnbq1rk1/pp2bppp/3p1n2/2p5/3N4/2P5/PP2BPPP/RNBQ1RK1 w - - 0 10";
    let game_state = GameState::from_fen(fen);
    assert_eq!(0, get_pawn_victims(&game_state, Color::White));
    assert_eq!(1, get_pawn_victims(&game_state, Color::Black));

    let fen = "rnbq1rk1/pp2bppp/3p1n2/2p5/1N1N4/2P5/PP2BPPP/R1BQ1RK1 w - - 0 10";
    let game_state = GameState::from_fen(fen);
    assert_eq!(0, get_pawn_victims(&game_state, Color::White));
    assert_eq!(2, get_pawn_victims(&game_state, Color::Black));

    // White pawn attacking black pieces:
    let fen = "rnbq1rk1/pp2bppp/n2p4/2p5/8/2P2N2/PP2BPPP/RNBQ1RK1 b - - 1 10";
    let game_state = GameState::from_fen(fen);
    assert_eq!(0, get_pawn_victims(&game_state, Color::White));
    assert_eq!(0, get_pawn_victims(&game_state, Color::Black));

    let fen = "rnbq1rk1/pp2bppp/3p4/2p5/1n6/2P2N2/PP2BPPP/RNBQ1RK1 b - - 1 10";
    let game_state = GameState::from_fen(fen);
    assert_eq!(1, get_pawn_victims(&game_state, Color::White));
    assert_eq!(0, get_pawn_victims(&game_state, Color::Black));

    let fen = "rn1q1rk1/pp2bppp/3p4/2p5/1n1b4/2P2N2/PP2BPPP/RNBQ1RK1 b - - 1 10";
    let game_state = GameState::from_fen(fen);
    assert_eq!(2, get_pawn_victims(&game_state, Color::White));
    assert_eq!(0, get_pawn_victims(&game_state, Color::Black));
  }

  #[test]
  fn test_detect_passed_pawns() {
    let fen = "r1bqkb1r/pp3ppp/3p4/2pPp3/3nP3/3B1N2/PP1P1PPP/R1BQK2R w KQkq - 1 8";
    let game_state = GameState::from_fen(fen);
    for i in 0..64 {
      assert_eq!(false, is_passed(&game_state, i));
    }

    let fen = "6k1/r4pp1/8/7p/Pp5P/8/3Q1PP1/6K1 w - - 0 31";
    let game_state = GameState::from_fen(fen);
    for i in 0..64 {
      if i == 24 || i == 25 {
        assert_eq!(true, is_passed(&game_state, i));
      } else {
        assert_eq!(false, is_passed(&game_state, i));
      }
    }

    let fen = "nnnnn1k1/rnnnn1p1/nnnnn3/7p/Pp5P/1Pp5/2Q3P1/1QQ3K1 w - - 0 31";
    let game_state = GameState::from_fen(fen);
    for i in 0..64 {
      if i == 24 || i == 18 {
        assert_eq!(true, is_passed(&game_state, i));
      } else {
        assert_eq!(false, is_passed(&game_state, i));
      }
    }
  }
}
