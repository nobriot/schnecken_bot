use super::generic::*;

use crate::model::board::*;
use crate::model::board_mask::*;
use crate::model::game_state::*;
use crate::model::piece::*;

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
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `index` -      Index of the square on the board where the pawn is located
///
/// # Return value
///
/// true if the pawn is protected, false if there is no pawn on the square index
/// or it is not protected.
///
pub fn can_become_protected(game_state: &GameState, index: u8) -> bool {
  // Check if there is no other pawn behind:
  let pawn = game_state.board.pieces.get(index);
  let (file, mut rank) = Board::index_to_fr(index);

  // Check if it can be defended by another pawn behind or on the side:
  while (rank >= 1) && (rank <= 8) {
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
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine backwards pawns
///
/// ### Return value
///
/// BoardMask indicating which squares have a backwards pawn on it.
///
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
    if game_state.board.pieces.get(i as u8) != pawn {
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

    if (new_rank > 8) || (new_rank < 1) {
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
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine the number of pawn islands
pub fn get_number_of_pawn_islands(game_state: &GameState, color: Color) -> usize {
  let pawn_value = match color {
    Color::White => WHITE_PAWN,
    Color::Black => BLACK_PAWN,
  };
  let mut pawn_islands: usize = 0;
  let mut pawn_state = PawnTrackingState::NoPawn;
  for rank in 0..8 {
    for file in 0..8 {
      if game_state.board.pieces.get(rank + file * 8) == pawn_value {
        if pawn_state == PawnTrackingState::NoPawn {
          pawn_islands += 1;
          pawn_state = PawnTrackingState::Pawn;
        }
        break;
      }
      if file == 7 {
        pawn_state = PawnTrackingState::NoPawn;
      }
    }
  }

  pawn_islands
}

/// Determine if a pawn on the board is protected by another pawn
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `index` -      Index of the square on the board where the pawn is located
///
/// # Return value
///
/// true if the pawn is protected, false if there is no pawn on the square index
/// or it is not protected.
///
pub fn is_protected(game_state: &GameState, index: u8) -> bool {
  // Same side pawn
  let ss_pawn = match game_state.board.pieces.get(index) {
    WHITE_PAWN => WHITE_PAWN,
    BLACK_PAWN => BLACK_PAWN,
    _ => return false,
  };

  // Determine the rank / File:
  let (file, mut rank) = Board::index_to_fr(index);

  // Check if we are protected, back one rank and one file on the right and/or left.
  if ss_pawn == WHITE_PAWN {
    rank -= 1;
  } else {
    rank += 1;
  }
  // Would be strange here, (there should be no pawn on the edge of the board)
  if !(1..=8).contains(&rank) {
    debug_assert!(false, "There should be no pawn in such positions");
    return false;
  }

  // Check on the left side:
  if file > 1 {
    if game_state.board.get_piece(file - 1, rank) == ss_pawn {
      return true;
    }
  }
  // Check on the right side:
  if file < 8 {
    if game_state.board.get_piece(file + 1, rank) == ss_pawn {
      return true;
    }
  }

  false
}

/// Determine if a square on the board is protected by a pawn of a certain color
///
/// ### Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `index` -      Index of the square on the board
/// * `color` -      Color to check for pawns
///
/// ### Return value
///
/// true if the pawn is protected by a pawn of the indicated color, false otherwise
///
pub fn is_square_protected_by_pawn(game_state: &GameState, index: u8, color: Color) -> bool {
  // Find the pawn value
  let pawn = match color {
    Color::White => WHITE_PAWN,
    Color::Black => BLACK_PAWN,
  };

  // Determine the rank / File:
  let (file, mut rank) = Board::index_to_fr(index);

  // Check if we are protected, back one rank and one file on the right and/or left.
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
  if file > 1 {
    if game_state.board.get_piece(file - 1, rank) == pawn {
      return true;
    }
  }
  // Check on the right side:
  if file < 8 {
    if game_state.board.get_piece(file + 1, rank) == pawn {
      return true;
    }
  }

  false
}

/// Determine if a pawn on the board is passed.
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `index` -      Index of the square on the board where the pawn is located
///
/// # Return value
///
/// true if the pawn is passed, false if there is no pawn on the square index
/// or it is not passed.
///
pub fn is_passed(game_state: &GameState, index: u8) -> bool {
  // Same side pawn
  let (ss_pawn, op_pawn) = match game_state.board.pieces.get(index) {
    WHITE_PAWN => (WHITE_PAWN, BLACK_PAWN),
    BLACK_PAWN => (BLACK_PAWN, WHITE_PAWN),
    _ => return false,
  };

  // Determine the rank / File:
  let (file, mut rank) = Board::index_to_fr(index);
  // Swipe the board for pawns on the way:

  loop {
    // Note we decrement for black, as their pawn are going down the board
    if ss_pawn == WHITE_PAWN {
      rank += 1;
    } else {
      rank -= 1;
    }
    if !(1..=8).contains(&rank) {
      return true;
    }

    let s = Board::fr_to_index(file, rank);

    // Check straight ahead:
    if game_state.board.pieces.get(s) == op_pawn {
      return false;
    }
    // Check on the left side:
    if file > 1 {
      let s = Board::fr_to_index(file - 1, rank);
      if game_state.board.pieces.get(s) == op_pawn {
        return false;
      }
    }
    // Check on the right side:
    if file < 8 {
      let s = Board::fr_to_index(file + 1, rank);
      if game_state.board.pieces.get(s) == op_pawn {
        return false;
      }
    }
  }
}

/// Determine the number of passed pawns in a position for a given color.
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine the number of pawn islands
pub fn get_number_of_passers(game_state: &GameState, color: Color) -> usize {
  // Same side pawn
  let ss_pawn = match color {
    Color::White => WHITE_PAWN,
    Color::Black => BLACK_PAWN,
  };
  let mut passers: usize = 0;

  for i in 0..64 {
    if game_state.board.pieces.get(i as u8) == ss_pawn {
      if is_passed(game_state, i) {
        passers += 1;
      }
    }
  }

  passers
}

/// Determine the number of protected passed pawns in a position for a given color.
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine the number of pawn islands
pub fn get_number_of_protected_passers(game_state: &GameState, color: Color) -> usize {
  // Same side pawn
  let ss_pawn = match color {
    Color::White => WHITE_PAWN,
    Color::Black => BLACK_PAWN,
  };
  let mut connected_passers: usize = 0;

  for i in 0..64 {
    if game_state.board.pieces.get(i as u8) == ss_pawn {
      if is_passed(game_state, i) && is_protected(game_state, i) {
        connected_passers += 1;
      }
    }
  }

  connected_passers
}

/// Determine the number of protected pawns (by other pawns) in a position for a given color.
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine the number of pawn islands
pub fn get_number_of_protected_pawns(game_state: &GameState, color: Color) -> usize {
  // Same side pawn
  let ss_pawn = match color {
    Color::White => WHITE_PAWN,
    Color::Black => BLACK_PAWN,
  };
  let mut protected_pawns: usize = 0;

  for i in 0..64 {
    if game_state.board.pieces.get(i as u8) == ss_pawn {
      if is_protected(game_state, i) {
        protected_pawns += 1;
      }
    }
  }

  protected_pawns
}

/// Determine the number of squares left for the pawn closest to promotion.
///
/// ### Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine the closest pawn to promotion
///
/// ### Return value
///
/// 8 if no pawn is present, else the number of squares that the closest pawn
/// has to travel to get promoted
///
pub fn get_distance_left_for_closest_pawn_to_promotion(game_state: &GameState, color: Color) -> u8 {
  let mut best_distance: u8 = 8;
  // Same side pawn
  let (ss_pawn, target_rank) = match color {
    Color::White => (WHITE_PAWN, 8),
    Color::Black => (BLACK_PAWN, 1),
  };

  for i in 0..64 {
    if game_state.board.pieces.get(i as u8) == ss_pawn {
      let (_, rank) = Board::index_to_fr(i);
      let distance = rank.abs_diff(target_rank);
      if distance < best_distance {
        best_distance = distance;
      }
    }
  }

  best_distance
}

/// Determines holes in our position
/// (squares that cannot be defended by a pawn anymore)
///
/// ### Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine holes
///
/// ### Return value
///
/// Board mask with the holes for that 8 if no pawn is present, else the number of squares that the closest pawn
/// has to travel to get promoted
///
pub fn get_holes(game_state: &GameState, color: Color) -> BoardMask {
  let mut holes: BoardMask = 0;

  let pawn = match color {
    Color::White => WHITE_PAWN,
    Color::Black => BLACK_PAWN,
  };
  let stop_rank = match color {
    Color::White => 2,
    Color::Black => 7,
  };

  // ranks 1-2 and 7-8 are not counted here.
  for i in 16..48 {
    let (file, mut rank) = Board::index_to_fr(i);

    while rank != stop_rank {
      match color {
        Color::White => rank -= 1,
        Color::Black => rank += 1,
      };
      // Check on the left side:
      if file > 1 {
        let s = Board::fr_to_index(file - 1, rank);
        if game_state.board.pieces.get(s) == pawn {
          rank = 0;
          break;
        }
      }
      if file < 8 {
        let s = Board::fr_to_index(file + 1, rank);
        if game_state.board.pieces.get(s) == pawn {
          rank = 0;
          break;
        }
      }
    }

    if rank == stop_rank {
      holes |= 1 << i;
    }
  }

  holes
}

/// Computes the values of the pieces that a pawn attacks.
///
/// ### Argument
/// * `game_state`: A GameState object representing a position, side to play, etc.
/// * `i`         : Index of the square on the board
///
/// ### Returns
///
/// Zero if there is no pawn on the square
/// The value of attacked enemy pieces if it attacks them.
///
pub fn pawn_attack(game_state: &GameState, i: u8) -> f32 {
  let mut value: f32 = 0.0;

  // If we have no pawn on the square, return immediately.
  let color = if game_state.board.pieces.get(i as u8) == WHITE_PAWN {
    Color::White
  } else if game_state.board.pieces.get(i as u8) == BLACK_PAWN {
    Color::Black
  } else {
    return value;
  };

  // If the knight is attacked by the opponent and not defended, we do not even
  // consider anything here:
  if is_hanging(game_state, i) && is_attacked(game_state, i) {
    return value;
  }

  // Check if controlled by op_pawns:
  let (file, mut rank) = Board::index_to_fr(i);
  match color {
    Color::White => rank += 1,
    Color::Black => rank -= 1,
  }
  if rank > 8 || rank == 0 {
    return value;
  }

  // Check on the left side:
  if file > 1 {
    let s = Board::fr_to_index(file - 1, rank);
    if game_state
      .board
      .has_piece_with_color(s as u8, Color::opposite(color))
    {
      if !game_state.board.has_king(s) {
        value += Piece::material_value_from_u8(game_state.board.pieces.get(s));
      } else {
        value += 1.0 * Color::score_factor(Color::opposite(color));
      }
    }
  }

  // Check on the right side:
  if file < 8 {
    let s = Board::fr_to_index(file + 1, rank);
    if game_state
      .board
      .has_piece_with_color(s as u8, Color::opposite(color))
    {
      if !game_state.board.has_king(s) {
        value += Piece::material_value_from_u8(game_state.board.pieces.get(s));
      } else {
        value += 1.0 * Color::score_factor(Color::opposite(color));
      }
    }
  }

  value.abs()
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use super::*;
  use crate::model::board_mask::*;

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

    assert_eq!(
      0,
      get_number_of_protected_passers(&game_state, Color::Black)
    );
    assert_eq!(
      0,
      get_number_of_protected_passers(&game_state, Color::White)
    );
  }

  #[test]
  fn get_pawn_data_endgame_2() {
    let fen = "8/4kp2/1p6/3pK3/8/8/P1P1P1Pp/8 w - - 0 1";
    let game_state = GameState::from_fen(fen);

    assert_eq!(4, get_number_of_pawn_islands(&game_state, Color::White));
    assert_eq!(4, get_number_of_pawn_islands(&game_state, Color::Black));

    assert_eq!(1, get_number_of_passers(&game_state, Color::Black));
    assert_eq!(0, get_number_of_passers(&game_state, Color::White));

    assert_eq!(
      0,
      get_number_of_protected_passers(&game_state, Color::Black)
    );
    assert_eq!(
      0,
      get_number_of_protected_passers(&game_state, Color::White)
    );
    assert_eq!(
      6,
      get_distance_left_for_closest_pawn_to_promotion(&game_state, Color::White)
    );
    assert_eq!(
      1,
      get_distance_left_for_closest_pawn_to_promotion(&game_state, Color::Black)
    );
  }

  #[test]
  fn get_pawn_data_endgame_3() {
    let fen = "6k1/3p4/2p3pP/1p4P1/8/8/6K1/8 w - - 14 55";
    let game_state = GameState::from_fen(fen);

    assert_eq!(1, get_number_of_pawn_islands(&game_state, Color::White));
    assert_eq!(2, get_number_of_pawn_islands(&game_state, Color::Black));

    assert_eq!(3, get_number_of_passers(&game_state, Color::Black));
    assert_eq!(1, get_number_of_passers(&game_state, Color::White));

    assert_eq!(
      2,
      get_number_of_protected_passers(&game_state, Color::Black)
    );
    assert_eq!(
      1,
      get_number_of_protected_passers(&game_state, Color::White)
    );
    assert_eq!(
      2,
      get_distance_left_for_closest_pawn_to_promotion(&game_state, Color::White)
    );
    assert_eq!(
      4,
      get_distance_left_for_closest_pawn_to_promotion(&game_state, Color::Black)
    );
  }

  #[test]
  fn test_backwards_pawns() {
    // 2 backwards pawn in here: d7 and d4
    let fen = "rnbqkbnr/pp1p3p/2p3p1/2Pp2p1/PP1P4/8/6PP/RNBQKBNR w KQkq - 0 4";
    let game_state = GameState::from_fen(fen);
    let mask = get_backwards_pawns(&game_state, Color::White);
    print_board_mask(mask);
    assert_eq!(1, mask_sum(mask));
    assert_eq!(1 << 27, mask);

    let mask = get_backwards_pawns(&game_state, Color::Black);
    print_board_mask(mask);
    assert_eq!(1, mask_sum(mask));
    assert_eq!(1 << 51, mask)
  }

  #[test]
  fn test_get_holes() {
    let fen = "r1b2r2/1p4bk/2pR1npn/p6p/2P1PP2/1PN4P/PB2N1B1/5RK1 b - - 0 19";
    let game_state = GameState::from_fen(fen);
    let mask = get_holes(&game_state, Color::White);
    print_board_mask(mask);
    assert_eq!(13, mask_sum(mask));
    let mask = get_holes(&game_state, Color::Black);
    print_board_mask(mask);
    assert_eq!(10, mask_sum(mask));
  }

  #[test]
  fn test_pawn_attack() {
    // Black pawn attacking white pieces:
    let fen = "rnbq1rk1/ppp1bppp/3p1n2/8/3N4/2P5/PP2BPPP/RNBQ1RK1 b - - 7 9";
    let game_state = GameState::from_fen(fen);
    for i in 0..64 {
      assert_eq!(0.0, pawn_attack(&game_state, i));
    }

    let fen = "rnbq1rk1/pp2bppp/3p1n2/2p5/3N4/2P5/PP2BPPP/RNBQ1RK1 w - - 0 10";
    let game_state = GameState::from_fen(fen);
    for i in 0..64 {
      if i != 34 {
        assert_eq!(0.0, pawn_attack(&game_state, i));
      } else {
        assert_eq!(3.0, pawn_attack(&game_state, i));
      }
    }

    let fen = "rnbq1rk1/pp2bppp/3p1n2/2p5/1N1N4/2P5/PP2BPPP/R1BQ1RK1 w - - 0 10";
    let game_state = GameState::from_fen(fen);
    for i in 0..64 {
      if i != 34 {
        assert_eq!(0.0, pawn_attack(&game_state, i));
      } else {
        assert_eq!(6.0, pawn_attack(&game_state, i));
      }
    }

    // White pawn attacking black pieces:
    let fen = "rnbq1rk1/pp2bppp/n2p4/2p5/8/2P2N2/PP2BPPP/RNBQ1RK1 b - - 1 10";
    let game_state = GameState::from_fen(fen);
    for i in 0..64 {
      assert_eq!(0.0, pawn_attack(&game_state, i));
    }

    let fen = "rnbq1rk1/pp2bppp/3p4/2p5/1n6/2P2N2/PP2BPPP/RNBQ1RK1 b - - 1 10";
    let game_state = GameState::from_fen(fen);
    for i in 0..64 {
      if i != 18 {
        assert_eq!(0.0, pawn_attack(&game_state, i));
      } else {
        assert_eq!(3.0, pawn_attack(&game_state, i));
      }
    }

    let fen = "rn1q1rk1/pp2bppp/3p4/2p5/1n1b4/2P2N2/PP2BPPP/RNBQ1RK1 b - - 1 10";
    let game_state = GameState::from_fen(fen);
    for i in 0..64 {
      if i != 18 {
        assert_eq!(0.0, pawn_attack(&game_state, i));
      } else {
        assert_eq!(6.05, pawn_attack(&game_state, i));
      }
    }
  }
}
