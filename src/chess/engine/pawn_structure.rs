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

/// Determine the number of passed pawns in a position for a given color.
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine the number of pawn islands
pub fn get_number_of_passers(game_state: &GameState, color: Color) -> usize {
  // Same side pawn
  let (ss_pawn, direction) = match color {
    Color::White => (WHITE_PAWN, 1),
    Color::Black => (BLACK_PAWN, -1),
  };
  // Opposite side pawn
  let op_pawn = match color {
    Color::White => BLACK_PAWN,
    Color::Black => WHITE_PAWN,
  };
  let mut passers: usize = 0;

  for i in 0..64 {
    if game_state.board.squares[i] == ss_pawn {
      let mut rank = (i / 8) as isize;
      let file = (i % 8) as isize;
      loop {
        rank += direction;

        let s = (file + rank * 8) as usize;
        if (rank > 7) || (rank < 0) {
          passers += 1;
          break;
        }

        if game_state.board.squares[s] == op_pawn {
          break;
        }
        if (s > 0 && file > 0) && game_state.board.squares[s - 1] == op_pawn {
          break;
        }
        if (s < 63 && file < 7) && game_state.board.squares[s + 1] == op_pawn {
          break;
        }
      }
    }
  }

  passers
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
  }

  #[test]
  fn get_pawn_data_endgame_2() {
    let fen = "8/4kp2/1p6/3pK3/8/8/P1P1P1Pp/8 w - - 0 1";
    let game_state = GameState::from_string(fen);

    assert_eq!(4, get_number_of_pawn_islands(&game_state, Color::White));
    assert_eq!(4, get_number_of_pawn_islands(&game_state, Color::Black));

    assert_eq!(1, get_number_of_passers(&game_state, Color::Black));
    assert_eq!(0, get_number_of_passers(&game_state, Color::White));
  }
}
