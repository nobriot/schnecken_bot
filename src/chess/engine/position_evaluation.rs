use log::*;

// From our module
use crate::chess::engine::pawn_structure::*;
use crate::chess::engine::square_affinity::*;
use crate::chess::model::game_state::*;
use crate::chess::model::piece::*;

// Constants
const PIECE_AFFINITY_FACTOR: f32 = 0.3;
const PAWN_ISLAND_FACTOR: f32 = 0.2;

// Shows "interesting" squares to control on the board
// Giving them a score
pub const HEATMAP_SCORES: [f32; 64] = [
  0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, // 1st row
  0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, // 2nd row
  0.01, 0.01, 0.03, 0.03, 0.03, 0.03, 0.01, 0.01, // 3rd row
  0.01, 0.01, 0.03, 0.04, 0.04, 0.03, 0.01, 0.01, // 4th row
  0.01, 0.01, 0.03, 0.04, 0.04, 0.03, 0.01, 0.01, // 5th row
  0.01, 0.01, 0.03, 0.03, 0.03, 0.03, 0.01, 0.01, // 6th row
  0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, // 7th row
  0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, // 8th row
];

/// Evaluates a position and returns a score and if the game is over.
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
pub fn evaluate_position(game_state: &GameState) -> (f32, bool) {
  // Check if we are checkmated or stalemated
  if game_state.available_moves_computed == false {
    warn!("Evaluating a position without move list computed, cannot determine if it is a game over position.");
  }
  if game_state.available_moves_computed == true && game_state.move_list.len() == 0 {
    match (game_state.side_to_play, game_state.checks) {
      (_, 0) => return (0.0, true),
      (Color::Black, _) => return (200.0, true),
      (Color::White, _) => return (-200.0, true),
    }
  }

  // Basic material count
  let mut score: f32 = 0.0;
  for i in 0..64 {
    match game_state.board.squares[i] {
      WHITE_QUEEN => score += 9.5,
      WHITE_ROOK => score += 5.0,
      WHITE_BISHOP => score += 3.05,
      WHITE_KNIGHT => score += 3.0,
      WHITE_PAWN => score += 1.0,
      BLACK_QUEEN => score -= 9.5,
      BLACK_ROOK => score -= 5.0,
      BLACK_BISHOP => score -= 3.05,
      BLACK_KNIGHT => score -= 3.0,
      BLACK_PAWN => score -= 1.0,
      _ => {},
    }
  }

  // Compare pawn islands.
  score += PAWN_ISLAND_FACTOR
    * (get_number_of_pawn_islands(game_state, Color::Black) as f32
      - get_number_of_pawn_islands(game_state, Color::White) as f32);

  // This is an expensive calculation, for now we skip this.
  // Compare the mobility of both sides. Give +1 if one side has 15 available moves.
  // score +=
  //  (self.get_white_moves().len() as isize - self.get_black_moves().len() as isize) as f32 / 15.0;

  // Get a pressure score, if one side has more attackers than defenders on a square, they get bonus points
  //let white_heatmap = self.get_heatmap(Color::White, false);
  //let black_heatmap = self.get_heatmap(Color::Black, false);

  // Piece affinity offsets
  for i in 0..64 {
    match game_state.board.squares[i] {
      WHITE_KING => score += PIECE_AFFINITY_FACTOR * WHITE_KING_SQUARE_AFFINITY[i] as f32,
      WHITE_QUEEN => score += PIECE_AFFINITY_FACTOR * QUEEN_SQUARE_AFFINITY[i] as f32,
      WHITE_ROOK => score += PIECE_AFFINITY_FACTOR * WHITE_ROOK_SQUARE_AFFINITY[i] as f32,
      WHITE_BISHOP => score += PIECE_AFFINITY_FACTOR * WHITE_BISHOP_SQUARE_AFFINITY[i] as f32,
      WHITE_KNIGHT => score += PIECE_AFFINITY_FACTOR * KNIGHT_SQUARE_AFFINITY[i] as f32,
      WHITE_PAWN => score += PIECE_AFFINITY_FACTOR * WHITE_PAWN_SQUARE_AFFINITY[i] as f32,
      BLACK_KING => score -= PIECE_AFFINITY_FACTOR * BLACK_KING_SQUARE_AFFINITY[i] as f32,
      BLACK_QUEEN => score -= PIECE_AFFINITY_FACTOR * QUEEN_SQUARE_AFFINITY[i] as f32,
      BLACK_ROOK => score -= PIECE_AFFINITY_FACTOR * BLACK_ROOK_SQUARE_AFFINITY[i] as f32,
      BLACK_BISHOP => score -= PIECE_AFFINITY_FACTOR * BLACK_BISHOP_SQUARE_AFFINITY[i] as f32,
      BLACK_KNIGHT => score -= PIECE_AFFINITY_FACTOR * KNIGHT_SQUARE_AFFINITY[i] as f32,
      BLACK_PAWN => score -= PIECE_AFFINITY_FACTOR * BLACK_PAWN_SQUARE_AFFINITY[i] as f32,
      _ => {},
    }
  }

  (score, false)
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_evaluate_position() {
    // This is a forced checkmate in 2:
    let fen = "1n4nr/5ppp/1N6/1P2p3/1P1k4/5P2/1p1NP1PP/R1B1KB1R w KQ - 0 35";
    let mut game_state = GameState::from_string(fen);
    let (evaluation, game_over) = evaluate_position(&game_state);
    assert_eq!(false, game_over);
    println!("Evaluation {evaluation}");
  }

  #[test]
  fn test_evaluate_position_checkmate_in_one() {
    // This is a forced checkmate in 1:
    let fen = "1n4nr/5ppp/1N6/1P2p3/1P6/4kP2/1B1NP1PP/R3KB1R w KQ - 1 36";
    let mut game_state = GameState::from_string(fen);
    let (evaluation, game_over) = evaluate_position(&game_state);
    assert_eq!(false, game_over);
    println!("Evaluation {evaluation}");
  }

  #[test]
  fn test_evaluate_position_checkmate() {
    // This is a "game over" position
    let fen = "1n4nr/5ppp/8/1P1Np3/1P6/4kP2/1B1NP1PP/R3KB1R b KQ - 2 37";
    let mut game_state = GameState::from_string(fen);
    game_state.get_moves();
    let (evaluation, game_over) = evaluate_position(&game_state);
    assert_eq!(true, game_over);
    assert_eq!(200.0, evaluation);
  }
  #[test]
  fn test_evaluate_position_hanging_queen() {
    // This should obviously be very bad for black:
    let fen = "rnbqkb1r/ppp1pppQ/5n2/3p4/3P4/8/PPP1PPPP/RNB1KBNR b KQkq - 0 3";
    let mut game_state = GameState::from_string(fen);
    let (evaluation, game_over) = evaluate_position(&game_state);
    assert_eq!(false, game_over);
    assert!(evaluation < 4.0);
  }

  #[test]
  fn test_evaluate_position_queen_standoff() {
    // This should obviously be okay because queen is defended and attacked by a queen.
    let fen = "rnb1kbnr/pppp1ppp/5q2/4p3/4P3/5Q2/PPPP1PPP/RNB1KBNR w KQkq - 2 3";
    let mut game_state = GameState::from_string(fen);
    game_state.get_moves();
    let (evaluation, game_over) = evaluate_position(&game_state);
    assert_eq!(false, game_over);
    println!("Evaluation: {}", evaluation);
    assert!(evaluation < 1.0);
    assert!(evaluation > -1.0);
  }
}
