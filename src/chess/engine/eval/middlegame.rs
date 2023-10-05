use super::helpers::king::*;
use super::helpers::mobility::*;
use super::helpers::rook::*;
use super::position::default_position_evaluation;
use crate::engine::square_affinity::*;
use crate::model::game_state::GameState;
use crate::model::piece::*;

const KING_DANGER_FACTOR: f32 = 0.3;
const KING_TOO_ADVENTUROUS_PENALTY: f32 = 0.9;
const SQUARE_TABLE_FACTOR: f32 = 0.1;

/// Gives a score based on the position in the middlegame
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
pub fn get_middlegame_position_evaluation(game_state: &GameState) -> f32 {
  let mut score: f32 = 0.0;

  /*
  score += PIECE_MOBILITY_FACTOR
    * (get_piece_mobility(game_state, Color::White) as f32
      - get_piece_mobility(game_state, Color::Black) as f32);
       */

  score += KING_DANGER_FACTOR
    * (get_king_danger_score(game_state, Color::Black)
      - get_king_danger_score(game_state, Color::White));

  if is_king_too_adventurous(game_state, Color::White) {
    score -= KING_TOO_ADVENTUROUS_PENALTY;
  }
  if is_king_too_adventurous(game_state, Color::Black) {
    score += KING_TOO_ADVENTUROUS_PENALTY;
  }

  for (i, piece) in game_state.board.pieces.white {
    match piece {
      PieceType::King => {
        score += SQUARE_TABLE_FACTOR * MiddleGameSquareTable::WHITE_KING[i as usize] as f32
      },
      PieceType::Queen => {
        score += SQUARE_TABLE_FACTOR * MiddleGameSquareTable::WHITE_QUEEN[i as usize] as f32
      },
      PieceType::Rook => {
        score += SQUARE_TABLE_FACTOR * MiddleGameSquareTable::WHITE_ROOK[i as usize] as f32
      },
      PieceType::Bishop => {
        score += SQUARE_TABLE_FACTOR * MiddleGameSquareTable::WHITE_BISHOP[i as usize] as f32
      },
      PieceType::Knight => {
        score += SQUARE_TABLE_FACTOR * MiddleGameSquareTable::WHITE_KNIGHT[i as usize] as f32
      },
      PieceType::Pawn => {
        score += SQUARE_TABLE_FACTOR * MiddleGameSquareTable::WHITE_PAWN[i as usize] as f32
      },
    }
  }
  for (i, piece) in game_state.board.pieces.black {
    match piece {
      PieceType::King => {
        score -= SQUARE_TABLE_FACTOR * MiddleGameSquareTable::BLACK_KING[i as usize] as f32
      },
      PieceType::Queen => {
        score -= SQUARE_TABLE_FACTOR * MiddleGameSquareTable::BLACK_QUEEN[i as usize] as f32
      },
      PieceType::Rook => {
        score -= SQUARE_TABLE_FACTOR * MiddleGameSquareTable::BLACK_ROOK[i as usize] as f32
      },
      PieceType::Bishop => {
        score -= SQUARE_TABLE_FACTOR * MiddleGameSquareTable::BLACK_BISHOP[i as usize] as f32
      },
      PieceType::Knight => {
        score -= SQUARE_TABLE_FACTOR * MiddleGameSquareTable::BLACK_KNIGHT[i as usize] as f32
      },
      PieceType::Pawn => {
        score -= SQUARE_TABLE_FACTOR * MiddleGameSquareTable::BLACK_PAWN[i as usize] as f32
      },
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
    let game_state = GameState::from_fen(fen);
    let eval = get_middlegame_position_evaluation(&game_state);

    println!("Evaluation: {eval}");
    assert!(-1.0 > eval);
  }

  #[test]
  fn evaluate_outposts() {
    // Compare 3 position, one with nothing, one with the reachable outpost and one with the outpost:
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
