use super::helpers::king::*;
use super::position::default_position_evaluation;
use crate::engine::development::get_development_score;
use crate::engine::eval::helpers::mobility::*;
use crate::engine::square_affinity::*;
use crate::model::game_state::GameState;
use crate::model::piece::*;

// Constants
const DEVELOPMENT_FACTOR: f32 = 0.30;
const PIECE_MOBILITY_FACTOR: f32 = 0.05;
const KING_DANGER_FACTOR: f32 = 0.3;
const KING_XRAY_FACTOR: f32 = 0.05;
const KING_TOO_ADVENTUROUS_PENALTY: f32 = 2.0;
const SQUARE_TABLE_FACTOR: f32 = 0.02;
const CASTLING_PENATLY: f32 = 1.0;

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

  if are_casling_rights_lost(game_state, Color::White) {
    score -= CASTLING_PENATLY;
  }
  if are_casling_rights_lost(game_state, Color::Black) {
    score += CASTLING_PENATLY;
  }

  for i in 0..64_usize {
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
  }

  score + default_position_evaluation(game_state)
}

//------------------------------------------------------------------------------
// Tests
#[cfg(test)]
mod tests {
  use super::*;
  use crate::model::board::*;
  use crate::model::board_mask::*;

  #[test]
  fn evaluate_opening_positions() {
    let fen = "rnbqkb1r/ppp1pppQ/5n2/3p4/3P4/8/PPP1PPPP/RNB1KBNR b KQkq - 0 3";
    let game_state = GameState::from_fen(fen);
    let eval = get_opening_position_evaluation(&game_state);
    print_board_mask(game_state.board.white_masks.control);
    print_board_mask(game_state.board.black_masks.control);
    println!("Evaluation: {eval}");
    assert!(-2.0 > eval);
  }

  #[test]
  fn evaluate_start_position() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let game_state = GameState::from_fen(fen);
    let eval = get_opening_position_evaluation(&game_state);

    print_board_mask(game_state.board.white_masks.control);
    print_board_mask(game_state.board.black_masks.control);

    println!("Evaluation: {eval}");
    assert!(-0.01 < eval);
    assert!(0.01 > eval);
  }

  #[test]
  fn evaluate_better_development() {
    let fen = "rnbqkb1r/pppppppp/5n2/8/2B1P3/2N2N2/PPPP1PPP/R1BQK2R b KQkq - 6 4";
    let game_state = GameState::from_fen(fen);
    let eval = get_opening_position_evaluation(&game_state);

    println!("Evaluation: {eval}");
    assert!(0.5 < eval);
  }

  #[test]
  fn evaluate_material_over_development() {
    // Here black is under-developed, but they are a knight up. We want the material to prevail:
    let fen = "r1bqkbnr/pppppppp/2n5/8/2B1P3/1P3N2/PBPP1PPP/R2QK2R w KQkq - 3 8";
    let game_state = GameState::from_fen(fen);
    let eval = get_opening_position_evaluation(&game_state);

    println!("Evaluation: {eval}");
    assert!(eval < -1.0);
  }

  #[test]
  fn evaluate_white_rook_on_second_rank() {
    // Historically the bot liked to place the rook on the 2nd rank for white / 7th for black, seems like a bug to me
    let fen = "rnbqkb1r/pppppppp/5n2/8/8/7P/PPPPPPPR/RNBQKBN1 b Qkq - 2 2";
    let game_state = GameState::from_fen(fen);
    let eval_1 = get_opening_position_evaluation(&game_state);

    let fen = "rnbqkb1r/pppppppp/5n2/8/8/7P/PPPPPPP1/RNBQKBNR b Qkq - 9 6";
    let game_state = GameState::from_fen(fen);
    let eval_2 = get_opening_position_evaluation(&game_state);

    println!("Evaluation: {eval_1} vs {eval_2}");
    assert!(eval_1 < eval_2);
  }

  #[test]
  fn evaluate_castle_better_than_non_castle() {
    // Here we are not castled.
    let fen = "r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/2N2N2/PPPP1PPP/R1BQK2R w KQkq - 6 5";
    let game_state = GameState::from_fen(fen);
    let eval_1 = get_opening_position_evaluation(&game_state);

    // Here we are castled
    let fen = "r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/2N2N2/PPPP1PPP/R1BQ1RK1 w kq - 6 5";
    let game_state = GameState::from_fen(fen);
    let eval_2 = get_opening_position_evaluation(&game_state);

    println!("Evaluation: {eval_1} vs {eval_2}");
    assert!(eval_1 < eval_2);
  }

  #[test]
  fn evaluate_adventurous_king() {
    // Here we are not castled.
    let fen = "r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/2N2N2/PPPP1PPP/R1BQK2R w KQkq - 6 5";
    let game_state = GameState::from_fen(fen);
    let eval_1 = get_opening_position_evaluation(&game_state);

    // Here the king is trying king of the hill
    let fen = "r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/2N2N2/PPPPKPPP/R1BQ3R w kq - 6 5";
    let game_state = GameState::from_fen(fen);
    let eval_2 = get_opening_position_evaluation(&game_state);

    println!("Evaluation: {eval_1} vs {eval_2}");
    assert!(eval_1 > eval_2);
  }

  #[test]
  fn evaluate_pawn_attack() {
    // Pawn attacking pieces with pawns is kinda good
    let fen = "rnbq1rk1/ppp1bppp/3p1n2/8/3N4/2P5/PP2BPPP/RNBQ1RK1 b - - 7 9";
    let game_state = GameState::from_fen(fen);
    let eval_nothing = get_opening_position_evaluation(&game_state);

    let fen = "rnbq1rk1/pp2bppp/3p1n2/2p5/3N4/2P5/PP2BPPP/RNBQ1RK1 w - - 0 10";
    let game_state = GameState::from_fen(fen);
    let eval_pawn_1_attack = get_opening_position_evaluation(&game_state);

    let fen = "rnbq1rk1/pp2bppp/3p1n2/2p5/1N1N4/2P5/PP2BPPP/R1BQ1RK1 w - - 0 10";
    let game_state = GameState::from_fen(fen);
    let eval_pawn_2_attacks = get_opening_position_evaluation(&game_state);

    println!("Evaluation: {eval_nothing} vs {eval_pawn_1_attack} vs {eval_pawn_2_attacks}");
    assert!(eval_nothing > eval_pawn_1_attack);
    assert!(eval_pawn_1_attack > eval_pawn_2_attacks);

    // Try from the other side (white pawns attacking black pieces)
    let fen = "rnbq1rk1/pp2bppp/3p1n2/2p5/5P2/2P2N2/PPN1B1PP/R1BQ1RK1 w - - 0 10";
    let game_state = GameState::from_fen(fen);
    let eval_nothing = get_opening_position_evaluation(&game_state);

    let fen = "rnbq1rk1/pp2bppp/3p4/2p3n1/5P2/2P2N2/PPN1B1PP/R1BQ1RK1 w - - 0 10";
    let game_state = GameState::from_fen(fen);
    let eval_pawn_1_attack = get_opening_position_evaluation(&game_state);

    let fen = "rnbq1rk1/pp3ppp/3p4/2p1b1n1/5P2/2P2N2/PPN1B1PP/R1BQ1RK1 w - - 0 10";
    let game_state = GameState::from_fen(fen);
    let eval_pawn_2_attacks = get_opening_position_evaluation(&game_state);

    println!("Evaluation: {eval_nothing} vs {eval_pawn_1_attack} vs {eval_pawn_2_attacks}");
    assert!(eval_nothing < eval_pawn_1_attack);
    assert!(eval_pawn_1_attack < eval_pawn_2_attacks);
  }

  #[test]
  fn evaluate_bishop_pin() {
    // https://lichess.org/7oeMxsbq
    let fen = "r6r/1p1k1npp/pBp2pn1/5b1B/8/2P5/PP2RPPP/5KNR b - - 13 18";
    let game_state = GameState::from_fen(fen);
    let eval_no_pin = get_opening_position_evaluation(&game_state);

    let fen = "r6r/1p1k1npp/pBp2pn1/7B/8/2Pb4/PP2RPPP/5KNR w - - 14 19";
    let game_state = GameState::from_fen(fen);
    let eval_pin = get_opening_position_evaluation(&game_state);

    println!("Evaluation: no pin {eval_no_pin} vs pin {eval_pin}");
    assert!(eval_pin < eval_no_pin);
    assert!(eval_pin < 0.5);
  }

  #[test]
  fn evaluate_hanging_knight() {
    let fen = "r1bqkb1r/1ppppp1p/p7/8/4Q3/5N2/nPPP1PPP/RNB1KB1R w KQkq - 0 9";
    let game_state = GameState::from_fen(fen);
    let eval = get_opening_position_evaluation(&game_state);
    println!("Evaluation: {eval}");
    assert!(eval > 4.5);

    // Capturing should be evaluated slightly better:
    let fen = "r1bqkb1r/1ppppp1p/p7/8/4Q3/5N2/RPPP1PPP/1NB1KB1R b Kkq - 0 9";
    let game_state = GameState::from_fen(fen);
    let eval_captured = get_opening_position_evaluation(&game_state);
    println!("Evaluation: hanging: {eval} - vs captured: {eval_captured}");
    assert!(eval_captured > eval);
  }

  #[test]
  fn evaluate_piece_down_in_opening() {
    let fen = "r1bqkb1r/1ppppp1p/p7/8/1n2Q3/5N2/PPPP1PPP/RNB1KB1R b KQkq - 0 8";
    let game_state = GameState::from_fen(fen);
    let eval = get_opening_position_evaluation(&game_state);
    println!("Evaluation: {eval}");
    assert!(eval > 3.0);
  }
}
