use crate::chess::model::board::*;
use crate::chess::model::game_state::*;
use crate::chess::model::piece::*;

#[derive(PartialEq)]
enum PawnTrackingState {
  NoPawn,
  Pawn,
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
      if game_state.board.squares[rank + file * 8] == pawn_value {
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
pub fn is_protected(game_state: &GameState, index: usize) -> bool {
  // Same side pawn
  let ss_pawn;
  match game_state.board.squares[index] {
    WHITE_PAWN => ss_pawn = WHITE_PAWN,
    BLACK_PAWN => ss_pawn = BLACK_PAWN,
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
  if (rank > 8) || (rank < 1) {
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

  return false;
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
pub fn is_passed(game_state: &GameState, index: usize) -> bool {
  // Same side pawn
  let ss_pawn;
  let op_pawn;
  match game_state.board.squares[index] {
    WHITE_PAWN => {
      ss_pawn = WHITE_PAWN;
      op_pawn = BLACK_PAWN;
    },
    BLACK_PAWN => {
      ss_pawn = BLACK_PAWN;
      op_pawn = WHITE_PAWN;
    },
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
    if (rank > 8) || (rank < 1) {
      return true;
    }

    let s = Board::fr_to_index(file, rank);

    // Check straight ahead:
    if game_state.board.squares[s] == op_pawn {
      return false;
    }
    // Check on the left side:
    if file > 1 {
      let s = Board::fr_to_index(file - 1, rank);
      if game_state.board.squares[s] == op_pawn {
        return false;
      }
    }
    // Check on the right side:
    if file < 8 {
      let s = Board::fr_to_index(file + 1, rank);
      if game_state.board.squares[s] == op_pawn {
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
    if game_state.board.squares[i] == ss_pawn {
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
    if game_state.board.squares[i] == ss_pawn {
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
    if game_state.board.squares[i] == ss_pawn {
      if is_protected(game_state, i) {
        protected_pawns += 1;
      }
    }
  }

  protected_pawns
}

/// Determine the number of squares left for the pawn closest to promotion.
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine the closest pawn to promotion
///
/// # Return value
///
/// 8 if no pawn is present, else the number of squares that the closest pawn
/// has to travel to get promoted
pub fn get_distance_left_for_closest_pawn_to_promotion(
  game_state: &GameState,
  color: Color,
) -> usize {
  let mut best_distance: usize = 8;
  // Same side pawn
  let (ss_pawn, target_rank) = match color {
    Color::White => (WHITE_PAWN, 8),
    Color::Black => (BLACK_PAWN, 1),
  };

  for i in 0..64 {
    if game_state.board.squares[i] == ss_pawn {
      let (_, rank) = Board::index_to_fr(i);
      let distance = rank.abs_diff(target_rank);
      if distance < best_distance {
        best_distance = distance;
      }
    }
  }

  best_distance
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn get_islands_for_simple_pawn_structure() {
    let fen = "rnb1kbnr/pppp1ppp/5q2/4p3/4P3/5Q2/PPPP1PPP/RNB1KBNR w KQkq - 2 3";
    let game_state = GameState::from_string(fen);

    assert_eq!(1, get_number_of_pawn_islands(&game_state, Color::Black));
    assert_eq!(1, get_number_of_pawn_islands(&game_state, Color::White));
  }

  #[test]
  fn get_islands_with_no_pawn() {
    let fen = "8/7k/5n2/4Q3/q7/3N4/3K4/8 b - - 3 51";
    let game_state = GameState::from_string(fen);

    assert_eq!(0, get_number_of_pawn_islands(&game_state, Color::Black));
    assert_eq!(0, get_number_of_pawn_islands(&game_state, Color::White));
  }

  #[test]
  fn get_islands_endgame() {
    let fen = "8/5ppk/5n1p/4QP2/q3P3/p1PN4/2K5/8 w - - 2 51";
    let game_state = GameState::from_string(fen);

    assert_eq!(2, get_number_of_pawn_islands(&game_state, Color::Black));
    assert_eq!(2, get_number_of_pawn_islands(&game_state, Color::White));
  }

  #[test]
  fn get_pawn_data_endgame_1() {
    let fen = "6k1/R7/6p1/6P1/7P/8/p5K1/r7 w - - 14 55";
    let game_state = GameState::from_string(fen);

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
    let game_state = GameState::from_string(fen);

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
    let game_state = GameState::from_string(fen);

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
}
