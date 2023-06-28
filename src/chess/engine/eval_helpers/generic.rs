use crate::chess::model::board::*;
use crate::chess::model::game_state::*;
use crate::chess::model::piece::*;

/// Makes the sum of a board mask
///
/// # Arguments
///
/// * `mask` - u64 bitmask representing a board with 0 and 1s.
///
/// # Return value
///
/// the sum of all bits set to 1.
pub fn mask_sum(mask: u64) -> usize {
  let mut sum: usize = 0;
  for i in 0..64 {
    if mask >> i & 1 == 1 {
      sum += 1;
    }
  }
  sum
}

/// Computes the material score of a side
///
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine the material score
///
/// # Return value
///
/// Score for material
pub fn get_material_score(game_state: &GameState, color: Color) -> f32 {
  // Basic material count
  let mut score: f32 = 0.0;
  for i in 0..64 {
    if color == Color::White {
      match game_state.board.squares[i] {
        WHITE_QUEEN => score += 9.5,
        WHITE_ROOK => score += 5.0,
        WHITE_BISHOP => score += 3.05,
        WHITE_KNIGHT => score += 3.0,
        WHITE_PAWN => score += 1.0,
        _ => {},
      }
    } else {
      match game_state.board.squares[i] {
        BLACK_QUEEN => score += 9.5,
        BLACK_ROOK => score += 5.0,
        BLACK_BISHOP => score += 3.05,
        BLACK_KNIGHT => score += 3.0,
        BLACK_PAWN => score += 1.0,
        _ => {},
      }
    }
  }
  score
}
