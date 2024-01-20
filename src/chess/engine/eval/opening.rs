use super::position::default_position_evaluation;
use crate::engine::development::get_development_score;
use crate::engine::square_affinity::*;
use crate::model::game_state::GameState;
use crate::model::piece::*;

// Constants
const DEVELOPMENT_FACTOR: f32 = 0.03;
//const KING_DANGER_FACTOR: f32 = 0.3;
//const KING_TOO_ADVENTUROUS_PENALTY: f32 = 0.9;
const SQUARE_TABLE_FACTOR: f32 = 0.02;
const _CASTLING_PENATLY: f32 = 1.0;

/// Computes a total score based on the square where pieces are located in the
/// opening.
///
/// ### Arguments
///
/// * `game_state`: GameState reference
///
/// ### Return value
///
/// f32 score that can be applied to the evaluation
///
pub fn get_square_table_opening_score(game_state: &GameState) -> f32 {
  let mut score = 0.0;
  for (i, piece) in game_state.board.pieces.white {
    match piece {
      PieceType::King => score += OpeningSquareTable::WHITE_KING[i as usize] as f32,
      PieceType::Queen => score += OpeningSquareTable::QUEEN[i as usize] as f32,
      PieceType::Rook => score += OpeningSquareTable::WHITE_ROOK[i as usize] as f32,
      PieceType::Bishop => score += SquareTable::WHITE_BISHOP[i as usize] as f32,
      PieceType::Knight => score += SquareTable::KNIGHT[i as usize] as f32,
      PieceType::Pawn => score += SquareTable::WHITE_PAWN[i as usize] as f32,
    }
  }
  for (i, piece) in game_state.board.pieces.black {
    match piece {
      PieceType::King => score -= OpeningSquareTable::BLACK_KING[i as usize] as f32,
      PieceType::Queen => score -= OpeningSquareTable::QUEEN[i as usize] as f32,
      PieceType::Rook => score -= OpeningSquareTable::BLACK_ROOK[i as usize] as f32,
      PieceType::Bishop => score -= SquareTable::WHITE_BISHOP[i as usize] as f32,
      PieceType::Knight => score -= SquareTable::KNIGHT[i as usize] as f32,
      PieceType::Pawn => score -= SquareTable::BLACK_PAWN[i as usize] as f32,
    }
  }
  score * SQUARE_TABLE_FACTOR
}

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

  /*
  score += KING_DANGER_FACTOR
    * (get_king_danger_score(game_state, Color::Black)
      - get_king_danger_score(game_state, Color::White));

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
   */

  score += get_square_table_opening_score(game_state);

  score + default_position_evaluation(game_state)
}

//------------------------------------------------------------------------------
// Tests
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn evaluate_start_position() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let game_state = GameState::from_fen(fen);
    let eval = get_opening_position_evaluation(&game_state);

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
    assert!(eval < -0.5);
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
    use crate::engine::eval::helpers::generic::get_combined_material_score;
    let fen = "r1bqkb1r/1ppppp1p/p7/8/4Q3/5N2/nPPP1PPP/RNB1KB1R w KQkq - 0 9";
    let game_state = GameState::from_fen(fen);
    let material_score = get_combined_material_score(&game_state);
    println!("Material score: {material_score}");
    assert!(material_score == (KNIGHT_VALUE - PAWN_VALUE));

    let eval = get_opening_position_evaluation(&game_state);
    println!("Evaluation: {eval}");
    assert!(eval > (material_score + KNIGHT_VALUE * 0.25));

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
    assert!(eval > 2.5);
  }

  #[test]
  fn evaluate_weird_opening_moves() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let game_state = GameState::from_fen(fen);
    let score_e4 = get_square_table_opening_score(&game_state);

    let fen = "rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq - 0 1";
    let game_state = GameState::from_fen(fen);
    let score_d4 = get_square_table_opening_score(&game_state);

    let fen = "rnbqkbnr/pppppppp/8/8/P7/8/1PPPPPPP/RNBQKBNR b KQkq - 0 1";
    let game_state = GameState::from_fen(fen);
    let score_a4 = get_square_table_opening_score(&game_state);

    let fen = "rnbqkbnr/pppppppp/8/8/8/5N2/PPPPPPPP/RNBQKB1R b KQkq - 1 1";
    let game_state = GameState::from_fen(fen);
    let score_nf3 = get_square_table_opening_score(&game_state);

    let fen = "rnbqkbnr/pppppppp/8/8/8/7N/PPPPPPPP/RNBQKB1R b KQkq - 1 1";
    let game_state = GameState::from_fen(fen);
    let score_nh3 = get_square_table_opening_score(&game_state);

    println!("E4: {score_e4}");
    println!("D4: {score_d4}");
    println!("a4: {score_a4}");
    println!("Nf3: {score_nf3}");
    println!("Nh3: {score_nh3}");

    assert!(score_e4 > score_a4);
    assert!(score_nf3 > score_nh3);
  }

  #[test]
  fn evaluate_chaotic_sacrifice() {
    let fen = "rn1q1bnr/p1p2kpp/8/1N1p1b2/8/8/PPPPPPPP/R1BQKB1R w KQ - 0 6";
    let game_state = GameState::from_fen(fen);

    let eval = get_opening_position_evaluation(&game_state);
    println!("Eval: {eval}");

    assert!(eval < 1.5);
    assert!(eval > -1.0);
  }
}
