use crate::chess::engine::development::get_development_score;
use crate::chess::engine::eval_helpers::generic::*;
use crate::chess::engine::eval_helpers::king::*;
use crate::chess::engine::eval_helpers::mobility::*;
use crate::chess::engine::square_affinity::*;
use crate::chess::model::game_state::print_mask;
use crate::chess::model::game_state::GameState;
use crate::chess::model::piece::*;

// Constants
const DEVELOPMENT_FACTOR: f32 = 0.30;
const PIECE_MOBILITY_FACTOR: f32 = 0.05;
const KING_DANGER_FACTOR: f32 = 0.3;
const KING_XRAY_FACTOR: f32 = 0.05;
const KING_TOO_ADVENTUROUS_PENALTY: f32 = 2.0;
const MATERIAL_COUNT_FACTOR: f32 = 1.0;
const HANGING_PENALTY: f32 = 0.1;
const HANGING_FACTOR: f32 = 0.5;
const REACHABLE_OUTPOST_BONUS: f32 = 0.2;
const OUTPOST_BONUS: f32 = 0.9;
const SQUARE_TABLE_FACTOR: f32 = 0.01;

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
    match game_state.board.squares[i] {
      WHITE_KING => score += SQUARE_TABLE_FACTOR * OpeningSquareTable::WHITE_KING[i] as f32,
      WHITE_QUEEN => score += SQUARE_TABLE_FACTOR * OpeningSquareTable::WHITE_QUEEN[i] as f32,
      WHITE_ROOK => score += SQUARE_TABLE_FACTOR * OpeningSquareTable::WHITE_ROOK[i] as f32,
      WHITE_BISHOP => score += SQUARE_TABLE_FACTOR * OpeningSquareTable::WHITE_BISHOP[i] as f32,
      WHITE_KNIGHT => score += SQUARE_TABLE_FACTOR * OpeningSquareTable::WHITE_KNIGHT[i] as f32,
      WHITE_PAWN => score += SQUARE_TABLE_FACTOR * OpeningSquareTable::WHITE_PAWN[i] as f32,
      BLACK_KING => score -= SQUARE_TABLE_FACTOR * OpeningSquareTable::BLACK_KING[i] as f32,
      BLACK_QUEEN => score -= SQUARE_TABLE_FACTOR * OpeningSquareTable::BLACK_QUEEN[i] as f32,
      BLACK_ROOK => score -= SQUARE_TABLE_FACTOR * OpeningSquareTable::BLACK_ROOK[i] as f32,
      BLACK_BISHOP => score -= SQUARE_TABLE_FACTOR * OpeningSquareTable::BLACK_BISHOP[i] as f32,
      BLACK_KNIGHT => score -= SQUARE_TABLE_FACTOR * OpeningSquareTable::BLACK_KNIGHT[i] as f32,
      BLACK_PAWN => score -= SQUARE_TABLE_FACTOR * OpeningSquareTable::BLACK_PAWN[i] as f32,
      _ => {},
    }
    /*
     */
  }

  // Basic material count
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
  fn evaluate_opening_positions() {
    let fen = "rnbqkb1r/ppp1pppQ/5n2/3p4/3P4/8/PPP1PPPP/RNB1KBNR b KQkq - 0 3";
    let game_state = GameState::from_string(fen);
    let eval = get_opening_position_evaluation(&game_state);
    //print_mask(game_state.white_bitmap.unwrap());
    //print_mask(game_state.black_bitmap.unwrap());
    println!("Evaluation: {eval}");
    assert!(-2.0 > eval);
  }

  #[test]
  fn evaluate_start_position() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let game_state = GameState::from_string(fen);
    let eval = get_opening_position_evaluation(&game_state);

    println!("Evaluation: {eval}");
    assert!(-1.0 < eval);
    assert!(1.0 > eval);
  }

  #[test]
  fn evaluate_better_development() {
    let fen = "rnbqkb1r/pppppppp/5n2/8/2B1P3/2N2N2/PPPP1PPP/R1BQK2R b KQkq - 6 4";
    let game_state = GameState::from_string(fen);
    let eval = get_opening_position_evaluation(&game_state);

    println!("Evaluation: {eval}");
    assert!(0.5 < eval);
  }

  #[test]
  fn evaluate_material_over_development() {
    // Here black is under-developed, but they are a knigh up. We want the material to prevail:
    let fen = "r1bqkbnr/pppppppp/2n5/8/2B1P3/1P3N2/PBPP1PPP/R2QK2R w KQkq - 3 8";
    let game_state = GameState::from_string(fen);
    let eval = get_opening_position_evaluation(&game_state);

    println!("Evaluation: {eval}");
    assert!(1.0 < eval);
  }
}
