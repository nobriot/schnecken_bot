use super::helpers::bishop::*;
use super::helpers::generic::*;
use super::helpers::king::*;
use super::helpers::knight::*;
use super::helpers::mobility::*;
use super::helpers::pawn::*;
use super::helpers::rook::*;
use super::position::default_position_evaluation;
use super::position::HEATMAP_SCORES;
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
const ROOKS_FILE_BONUS: f32 = 0.3;
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

  // Check if rooks are placed on interesting files
  score += ROOKS_FILE_BONUS
    * (get_rooks_file_score(game_state, Color::White) as f32
      - get_rooks_file_score(game_state, Color::Black) as f32);

  for i in 0..64_usize {
    if !game_state.board.has_piece(i as u8) {
      continue;
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

  score + default_position_evaluation(game_state)
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

  #[test]
  fn evaluate_outposts() {
    // Compare 3 position, one with nothing, one with the reachable outpost and one with the outpost:
    let fen = "r1bqk2r/ppp2ppp/2n2n2/3p4/1bPPp3/2N1P2P/PP3PPN/R1BQKB1R w KQkq - 8 11";
    let game_state = GameState::from_string(fen);
    let eval_nothing = get_middlegame_position_evaluation(&game_state);

    let fen = "r1bqk2r/ppp2ppp/2n2n2/3p4/1bPPp1N1/2N1P2P/PP3PP1/R1BQKB1R w KQkq - 3 9";
    let game_state = GameState::from_string(fen);
    let eval_reachable_outpost = get_middlegame_position_evaluation(&game_state);

    let fen = "r1bqk2r/ppp2ppp/2n2n2/3pN3/1bPPp3/2N1P2P/PP3PP1/R1BQKB1R w KQkq - 0 7";
    let game_state = GameState::from_string(fen);
    let eval_outpost = get_middlegame_position_evaluation(&game_state);

    println!("Evaluation: Nothing: {eval_nothing} - Reachable outpost: {eval_reachable_outpost} - Outpost: {eval_outpost}");
    assert!(eval_reachable_outpost > eval_nothing);
    assert!(eval_outpost > eval_reachable_outpost);
  }

  #[test]
  fn evaluate_well_placed_rooks() {
    // Compare 3 position, one with rook on closed file, half open and then open file
    let fen = "6k1/5ppp/6p1/8/8/8/5PPP/5RK1 w - - 0 12";
    let game_state = GameState::from_string(fen);
    let eval_closed = get_middlegame_position_evaluation(&game_state);

    let fen = "6k1/5ppp/6p1/8/8/8/4P1PP/5RK1 w - - 0 12";
    let game_state = GameState::from_string(fen);
    let eval_half_open = get_middlegame_position_evaluation(&game_state);

    let fen = "6k1/4p1pp/6p1/8/8/8/4P1PP/5RK1 w - - 0 12";
    let game_state = GameState::from_string(fen);
    let eval_open = get_middlegame_position_evaluation(&game_state);

    println!("Evaluation: closed: {eval_closed} - half open: {eval_half_open} - open: {eval_open}");
    assert!(eval_open > eval_half_open);
    assert!(eval_half_open > eval_closed);
  }
}
