use crate::chess::engine::development::get_development_score;
use crate::chess::engine::eval_helpers::king::get_king_danger_score;
use crate::chess::engine::eval_helpers::mobility::*;
use crate::chess::engine::position_evaluation::default_position_evaluation;
use crate::chess::model::game_state::GameState;
use crate::chess::model::piece::Color;

// Constants
const DEVELOPMENT_FACTOR: f32 = 0.20;
const PIECE_MOBILITY_FACTOR: f32 = 0.05;
const KING_DANGER_FACTOR: f32 = 0.1;

/// Gives a score based on the position in the opening
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
pub fn get_opening_position_evaluation(game_state: &GameState) -> f32 {
  let mut score: f32 = 0.0;

  score += DEVELOPMENT_FACTOR
    * (get_development_score(game_state, Color::White) as f32
      - get_development_score(game_state, Color::Black) as f32);

  score += PIECE_MOBILITY_FACTOR
    * (get_piece_mobility(game_state, Color::White) as f32
      - get_piece_mobility(game_state, Color::Black) as f32);

  score += KING_DANGER_FACTOR
    * (get_king_danger_score(game_state, Color::Black)
      - get_king_danger_score(game_state, Color::White));

  // TOOD: Update this
  return score + default_position_evaluation(game_state);
}