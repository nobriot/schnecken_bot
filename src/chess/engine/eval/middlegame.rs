use super::helpers::king::get_king_attack_units;
use super::position::{Score, default_position_evaluation, square_table_score};
use crate::engine::tables::squares::*;
use crate::model::game_state::GameState;
use crate::model::piece::*;

// const KING_DANGER_FACTOR: Score = 30;
// const KING_TOO_ADVENTUROUS_PENALTY: Score = 90;
/// Penalty (centipawns) per squared king danger unit. Squared because attacks
/// get much more dangerous as attackers and weaknesses add up.
const KING_ATTACK_FACTOR: Score = 2;
/// Maximum penalty (centipawns) for king danger
const KING_ATTACK_MAX_PENALTY: Score = 500;

/// Computes a total score based on the square where pieces are located in the
/// middlegame.
///
/// ### Arguments
///
/// * `game_state`: GameState reference
///
/// ### Return value
///
/// Score (centipawns) that can be applied to the evaluation
pub fn get_square_table_middlegame_score(game_state: &GameState) -> Score {
  let mut score: isize = 0;
  for (i, piece) in game_state.board.pieces.white {
    match piece {
      PieceType::King => score += MiddleGameSquareTable::WHITE_KING[i as usize],
      PieceType::Queen => score += SquareTable::QUEEN[i as usize],
      PieceType::Rook => score += MiddleGameSquareTable::WHITE_ROOK[i as usize],
      PieceType::Bishop => score += SquareTable::WHITE_BISHOP[i as usize],
      PieceType::Knight => score += SquareTable::KNIGHT[i as usize],
      PieceType::Pawn => score += SquareTable::WHITE_PAWN[i as usize],
    }
  }
  for (i, piece) in game_state.board.pieces.black {
    match piece {
      PieceType::King => score -= MiddleGameSquareTable::BLACK_KING[i as usize],
      PieceType::Queen => score -= SquareTable::QUEEN[i as usize],
      PieceType::Rook => score -= MiddleGameSquareTable::BLACK_ROOK[i as usize],
      PieceType::Bishop => score -= SquareTable::BLACK_BISHOP[i as usize],
      PieceType::Knight => score -= SquareTable::KNIGHT[i as usize],
      PieceType::Pawn => score -= SquareTable::BLACK_PAWN[i as usize],
    }
  }
  square_table_score(score)
}

/// Gives a score based on the position in the middlegame
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
pub fn get_middlegame_position_evaluation(game_state: &GameState) -> Score {
  let mut score: Score = 0;

  // score += PIECE_MOBILITY_FACTOR
  // (get_piece_mobility(game_state, Color::White) as Score
  // - get_piece_mobility(game_state, Color::Black) as Score);
  //
  // score += KING_DANGER_FACTOR
  // (get_king_danger_score(game_state, Color::Black)
  // - get_king_danger_score(game_state, Color::White));
  //
  // if is_king_too_adventurous(game_state, Color::White) {
  // score -= KING_TOO_ADVENTUROUS_PENALTY;
  // }
  // if is_king_too_adventurous(game_state, Color::Black) {
  // score += KING_TOO_ADVENTUROUS_PENALTY;
  // }

  score += get_king_attack_penalty(game_state, Color::Black)
           - get_king_attack_penalty(game_state, Color::White);

  score += get_square_table_middlegame_score(game_state);

  score + default_position_evaluation(game_state)
}

/// Converts the danger units around the king into a penalty (positive value)
///
/// ### Arguments
///
/// * `game_state` - A GameState object representing a position
/// * `color` -      Color of the king for which we compute the penalty
#[inline]
fn get_king_attack_penalty(game_state: &GameState, color: Color) -> Score {
  let units = get_king_attack_units(game_state, color) as i32;
  (units * units * KING_ATTACK_FACTOR as i32).min(KING_ATTACK_MAX_PENALTY as i32) as Score
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
    let game_state = GameState::from_fen(fen);
    let eval = get_middlegame_position_evaluation(&game_state);

    println!("Evaluation: {eval}");
    assert!(-100 >= eval);
  }

  #[test]
  fn evaluate_outposts() {
    // Compare 3 position, one with nothing, one with the reachable outpost and
    // one with the outpost:
    let fen = "r1bqk2r/ppp2ppp/2n2n2/3p4/1bPPp3/2N1P2P/PP3PPN/R1BQKB1R w KQkq - 8 11";
    let game_state = GameState::from_fen(fen);
    let eval_nothing = get_middlegame_position_evaluation(&game_state);

    let fen = "r1bqk2r/ppp2ppp/2n2n2/3p4/1bPPp1N1/2N1P2P/PP3PP1/R1BQKB1R w KQkq - 3 9";
    let game_state = GameState::from_fen(fen);
    let eval_reachable_outpost = get_middlegame_position_evaluation(&game_state);

    let fen = "r1bqk2r/ppp2ppp/2n2n2/3pN3/1bPPp3/2N1P2P/PP3PP1/R1BQKB1R w KQkq - 0 7";
    let game_state = GameState::from_fen(fen);
    let eval_outpost = get_middlegame_position_evaluation(&game_state);

    println!("Evaluation: Nothing: {eval_nothing} - Reachable outpost: {eval_reachable_outpost} - Outpost: {eval_outpost}");
    assert!(eval_reachable_outpost > eval_nothing);
    assert!(eval_outpost > eval_reachable_outpost);
  }
}
