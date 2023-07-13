use crate::chess::engine::eval_helpers::generic::*;
use crate::chess::engine::eval_helpers::king::*;
use crate::chess::engine::eval_helpers::mobility::*;
use crate::chess::engine::position_evaluation::default_position_evaluation;
use crate::chess::engine::square_affinity::*;
use crate::chess::model::game_state::GameState;
use crate::chess::model::piece::*;

const PIECE_MOBILITY_FACTOR: f32 = 0.1;
const KING_DANGER_FACTOR: f32 = 0.3;
const MATERIAL_COUNT_FACTOR: f32 = 1.0;
const KING_TOO_ADVENTUROUS_PENALTY: f32 = 1.5;

const KING_XRAY_FACTOR: f32 = 0.05;
const HANGING_PENALTY: f32 = 0.1;
const HANGING_FACTOR: f32 = 0.5;
const REACHABLE_OUTPOST_BONUS: f32 = 0.2;
const OUTPOST_BONUS: f32 = 0.9;
const SQUARE_TABLE_FACTOR: f32 = 0.02;

/// Gives a score based on the position in the middlegame
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
pub fn get_middlegame_position_evaluation(game_state: &GameState) -> f32 {
  let mut score: f32 = 0.0;

  score += PIECE_MOBILITY_FACTOR
    * (get_piece_mobility(game_state, Color::White) as f32
      - get_piece_mobility(game_state, Color::Black) as f32);

  score += KING_DANGER_FACTOR
    * (get_king_danger_score(game_state, Color::Black)
      - get_king_danger_score(game_state, Color::White));

  if is_king_xrayed(game_state, Color::White) {
    score -= KING_XRAY_FACTOR;
  }
  if is_king_xrayed(game_state, Color::Black) {
    score += KING_XRAY_FACTOR;
  }
  if is_king_too_adventurous(game_state, Color::White) {
    score -= KING_TOO_ADVENTUROUS_PENALTY;
  }
  if is_king_too_adventurous(game_state, Color::Black) {
    score += KING_TOO_ADVENTUROUS_PENALTY;
  }

  for i in 0..64 {
    // We are excited about hanging pieces when it's our turn :-)
    // Here it could probably be better.
    /*
     */
    if is_hanging(game_state, i) {
      if is_attacked(game_state, i)
        && (game_state.side_to_play
          == Color::opposite(Piece::color_from_u8(game_state.board.squares[i])))
      {
        score -= HANGING_FACTOR
          * Color::score_factor(Piece::color_from_u8(game_state.board.squares[i]))
          * Piece::material_value_from_u8(game_state.board.squares[i]);
      } else {
        // We usually are not the most fan of hanging pieces
        score -=
          HANGING_PENALTY * Color::score_factor(Piece::color_from_u8(game_state.board.squares[i]));
      }
    }
    // Check if we have some good positional stuff
    if has_reachable_outpost(game_state, i) {
      score += REACHABLE_OUTPOST_BONUS
        * Color::score_factor(Piece::color_from_u8(game_state.board.squares[i]));
    }
    if occupies_reachable_outpost(game_state, i) {
      score +=
        OUTPOST_BONUS * Color::score_factor(Piece::color_from_u8(game_state.board.squares[i]));
    }

    // Piece square table:
    /*
     */
    match game_state.board.squares[i] {
      WHITE_KING => score += SQUARE_TABLE_FACTOR * MiddleGameSquareTable::WHITE_KING[i] as f32,
      WHITE_QUEEN => score += SQUARE_TABLE_FACTOR * MiddleGameSquareTable::WHITE_QUEEN[i] as f32,
      WHITE_ROOK => score += SQUARE_TABLE_FACTOR * MiddleGameSquareTable::WHITE_ROOK[i] as f32,
      WHITE_BISHOP => score += SQUARE_TABLE_FACTOR * MiddleGameSquareTable::WHITE_BISHOP[i] as f32,
      WHITE_KNIGHT => score += SQUARE_TABLE_FACTOR * MiddleGameSquareTable::WHITE_KNIGHT[i] as f32,
      WHITE_PAWN => score += SQUARE_TABLE_FACTOR * MiddleGameSquareTable::WHITE_PAWN[i] as f32,
      BLACK_KING => score -= SQUARE_TABLE_FACTOR * MiddleGameSquareTable::BLACK_KING[i] as f32,
      BLACK_QUEEN => score -= SQUARE_TABLE_FACTOR * MiddleGameSquareTable::BLACK_QUEEN[i] as f32,
      BLACK_ROOK => score -= SQUARE_TABLE_FACTOR * MiddleGameSquareTable::BLACK_ROOK[i] as f32,
      BLACK_BISHOP => score -= SQUARE_TABLE_FACTOR * MiddleGameSquareTable::BLACK_BISHOP[i] as f32,
      BLACK_KNIGHT => score -= SQUARE_TABLE_FACTOR * MiddleGameSquareTable::BLACK_KNIGHT[i] as f32,
      BLACK_PAWN => score -= SQUARE_TABLE_FACTOR * MiddleGameSquareTable::BLACK_PAWN[i] as f32,
      _ => {},
    }
  }

  let white_material = get_material_score(game_state, Color::White);
  let black_material = get_material_score(game_state, Color::Black);
  score += MATERIAL_COUNT_FACTOR * (white_material - black_material);

  score
}

//------------------------------------------------------------------------------
// Tests
#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn evaluate_material_advantage() {
    // Middlegame, we do not compute any development advantage anymore.
    let fen = "r1bqkbnr/pppppppp/2n5/8/2B1P3/1P3N2/PBPP1PPP/R2QK2R w KQkq - 3 8";
    let game_state = GameState::from_string(fen);
    let eval = get_middlegame_position_evaluation(&game_state);

    println!("Evaluation: {eval}");
    assert!(-1.0 > eval);
  }
}
