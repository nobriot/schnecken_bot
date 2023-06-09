use crate::chess::engine::eval_helpers::mobility::*;
use crate::chess::engine::position_evaluation::default_position_evaluation;
use crate::chess::model::game_state::GameState;
use crate::chess::model::piece::Color;

const PIECE_MOBILITY_FACTOR: f32 = 0.1;

/// Gives a score based on the position in the middlegame
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
pub fn get_middlegame_position_evaluation(game_state: &GameState) -> f32 {
  let mut score: f32 = 0.0;

  score += PIECE_MOBILITY_FACTOR * (get_piece_mobility(game_state, Color::White) as f32)
    - (get_piece_mobility(game_state, Color::Black) as f32);

  // TOOD: Update this
  return score + default_position_evaluation(game_state);
}
