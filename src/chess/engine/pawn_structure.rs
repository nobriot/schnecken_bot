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

pub fn get_number_of_passers(game_state: &GameState, color: Color) -> usize {
  let passers: usize = 0;

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
}
