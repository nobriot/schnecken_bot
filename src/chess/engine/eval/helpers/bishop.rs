use super::generic::*;

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
/// The value of the top 2 attacked enemy pieces if it attacks them.
/// A +1.0 point bonus if the enemy king is under attack
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

  // If the bishop is attacked by the opponent and not defended, we do not even
  // consider anything here:
  if is_hanging(game_state, i) && is_attacked(game_state, i) {
    return value;
  }

  let ssp = game_state.board.get_color_mask(color);
  let op = game_state.board.get_color_mask(Color::opposite(color));
  let destinations = get_bishop_moves(ssp, op, i);
  let mut piece_value_1: f32 = 0.0;
  let mut piece_value_2: f32 = 0.0;

  for s in 0..64 {
    if destinations & (1 << s) == 0 {
      continue;
    }
    if game_state
      .board
      .has_piece_with_color(s as u8, Color::opposite(color))
    {
      if !game_state.board.has_king(s) {
        let value_to_update = if piece_value_1.abs() < piece_value_2.abs() {
          &mut piece_value_1
        } else {
          &mut piece_value_2
        };

        if value_to_update.abs() < Piece::material_value_from_u8(game_state.board.squares[s]).abs()
        {
          *value_to_update = Piece::material_value_from_u8(game_state.board.squares[s]);
        }
      } else {
        // 1 pt bonus for attacking the king
        value += 1.0 * Color::score_factor(Color::opposite(color));
      }
    }
  }
  value += piece_value_1;
  value += piece_value_2;

  value.abs()
}

/// Computes the values of the pieces that a bishop attacks, including major pieces
/// pins
///
/// ### Argument
/// * `game_state`: A GameState object representing a position, side to play, etc.
/// * `i`         : Index of the square on the board
///
/// ### Returns
///
/// Zero if there is no bishop on the square
/// The value of the top 2 attacked enemy pieces if it attacks them
/// A +1.0 point bonus if the enemy king is under attack.
///
pub fn bishop_attack_with_pins(game_state: &GameState, i: usize) -> f32 {
  let mut value: f32 = 0.0;

  // If we have no bishop on the square, return immediately.
  let color = if game_state.board.squares[i] == WHITE_BISHOP {
    Color::White
  } else if game_state.board.squares[i] == BLACK_BISHOP {
    Color::Black
  } else {
    return value;
  };

  // If the bishop is attacked by the opponent and not defended, we do not even
  // consider anything here:
  if is_hanging(game_state, i) && is_attacked(game_state, i) {
    return value;
  }

  let ssp = game_state.board.get_color_mask(color);
  let op = game_state
    .board
    .get_color_mask_without_major_pieces(Color::opposite(color));
  let destinations = get_bishop_moves(ssp, op, i);
  let mut piece_value_1: f32 = 0.0;
  let mut piece_value_2: f32 = 0.0;

  for s in 0..64 {
    if destinations & (1 << s) == 0 {
      continue;
    }
    if game_state
      .board
      .has_piece_with_color(s as u8, Color::opposite(color))
    {
      if !game_state.board.has_king(s) {
        let value_to_update = if piece_value_1.abs() < piece_value_2.abs() {
          &mut piece_value_1
        } else {
          &mut piece_value_2
        };

        if value_to_update.abs() < Piece::material_value_from_u8(game_state.board.squares[s]).abs()
        {
          *value_to_update = Piece::material_value_from_u8(game_state.board.squares[s]);
        }
      } else {
        // 1 pt bonus for attacking the king
        value += 1.0 * Color::score_factor(Color::opposite(color));
      }
    }
  }

  value += piece_value_1;
  value += piece_value_2;

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
        assert_eq!(15.0, bishop_attack(&game_state, i));
      }
    }

    let fen = "8/2Q3R1/8/4b3/8/2K3N1/8/1k2r3 w - - 0 1";
    let game_state = GameState::from_string(fen);
    for i in 0..64 {
      if i != 36 {
        assert_eq!(0.0, bishop_attack(&game_state, i));
      } else {
        assert_eq!(15.0, bishop_attack(&game_state, i));
      }
    }
  }

  #[test]
  fn test_bishop_attack_with_pins() {
    let fen = "4r3/6R1/8/4b3/3Q4/2K3N1/8/1k6 w - - 0 1";
    let game_state = GameState::from_string(fen);
    for i in 0..64 {
      if i != 36 {
        assert_eq!(0.0, bishop_attack_with_pins(&game_state, i));
      } else {
        assert_eq!(15.0, bishop_attack_with_pins(&game_state, i));
      }
    }

    let fen = "8/6R1/5Q2/3K4/8/6N1/1b6/1k6 w - - 0 1";
    let game_state = GameState::from_string(fen);
    for i in 0..64 {
      if i != 9 {
        assert_eq!(0.0, bishop_attack_with_pins(&game_state, i));
      } else {
        assert_eq!(14.0, bishop_attack_with_pins(&game_state, i));
      }
    }
  }
}
