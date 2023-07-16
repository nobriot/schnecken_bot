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

  let ssp = game_state.board.get_color_mask(color);
  let op = game_state.board.get_color_mask(Color::opposite(color));
  let destinations = get_bishop_moves(ssp, op, i);

  for s in 0..64 {
    if destinations & (1 << s) == 0 {
      continue;
    }
    if game_state
      .board
      .has_piece_with_color(s as u8, Color::opposite(color))
    {
      if !game_state.board.has_king(s) {
        value += Piece::material_value_from_u8(game_state.board.squares[s]);
      } else {
        value += 1.0 * Color::score_factor(Color::opposite(color));
      }
    }
  }

  value.abs()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_bishop_attack() {
    let fen = "8/2q3r1/8/4B3/8/2k3n1/8/1K2R3 w - - 0 1";
    let game_state = GameState::from_string(fen);
    for i in 0..64 {
      if i != 36 {
        assert_eq!(0.0, bishop_attack(&game_state, i));
      } else {
        assert_eq!(18.0, bishop_attack(&game_state, i));
      }
    }

    let fen = "8/2Q3R1/8/4b3/8/2K3N1/8/1k6 w - - 0 1";
    let game_state = GameState::from_string(fen);
    for i in 0..64 {
      if i != 36 {
        assert_eq!(0.0, bishop_attack(&game_state, i));
      } else {
        assert_eq!(18.0, bishop_attack(&game_state, i));
      }
    }
  }
}
