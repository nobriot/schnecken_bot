use crate::chess::model::board::*;
use crate::chess::model::game_state::*;
use crate::chess::model::piece::*;
use crate::chess::model::piece_moves::*;

/// Computes the values of the pieces that a bishop attacks.
///
/// ### Argument
/// * `game_state`: A GameState object representing a position, side to play, etc.
/// * `i`         : Index of the square on the board
///
/// ### Returns
///
/// Zero if there is no bishop on the square
/// The value of attacked enemy pieces if it attacks them.
///
pub fn bishop_attack(game_state: &GameState, i: usize) -> f32 {
  let mut value: f32 = 0.0;

  // If we have no bishop on the square, return immediately.
  let color = if game_state.board.squares[i] == WHITE_BISHOP {
    Color::White
  } else if game_state.board.squares[i] == BLACK_BISHOP {
    Color::Black
  } else {
    return value;
  };

  todo!("IMplement me");

  value
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_bishop_attack() {
    let fen = "r1b2r2/1p4bk/2pR1npn/p6p/2P1PP2/1PN4P/PB2N1B1/5RK1 b - - 0 19";
    let game_state = GameState::from_string(fen);
    assert_eq!(0.0, bishop_attack(&game_state, 0));
  }
}
