use crate::chess::engine::eval_helpers::generic::get_material_score;
use crate::chess::engine::eval_helpers::king::*;
use crate::chess::engine::eval_helpers::mobility::*;
use crate::chess::engine::position_evaluation::default_position_evaluation;
use crate::chess::model::game_state::GameState;
use crate::chess::model::piece::Color;

const PIECE_MOBILITY_FACTOR: f32 = 0.05;
const KING_DANGER_FACTOR: f32 = 0.3;
const MATERIAL_COUNT_FACTOR: f32 = 1.0;
const KING_TOO_ADVENTUROUS_PENALTY: f32 = 1.5;

/// Gives a score based on the position in the middlegame
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
pub fn get_middlegame_position_evaluation(game_state: &GameState) -> f32 {
  let mut score: f32 = 0.0;

  score += PIECE_MOBILITY_FACTOR
    * ((get_piece_mobility(game_state, Color::White) as f32)
      - (get_piece_mobility(game_state, Color::Black) as f32));

  score += KING_DANGER_FACTOR
    * (get_king_danger_score(game_state, Color::Black)
      - get_king_danger_score(game_state, Color::White));
  if is_king_too_adventurous(game_state, Color::White) {
    score -= KING_TOO_ADVENTUROUS_PENALTY;
  }
  if is_king_too_adventurous(game_state, Color::Black) {
    score += KING_TOO_ADVENTUROUS_PENALTY;
  }

  let white_material = get_material_score(game_state, Color::White);
  let black_material = get_material_score(game_state, Color::Black);
  score += MATERIAL_COUNT_FACTOR * (white_material - black_material);

  // TOOD: Update this
  return score + default_position_evaluation(game_state);
}
