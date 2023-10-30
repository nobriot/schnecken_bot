use super::generic::*;
use crate::model::board::*;
use crate::model::board_mask::*;
use crate::model::game_state::*;
use crate::model::piece::*;
use crate::square_in_mask;

/// Determine if rooks are connected for a color
///
pub fn are_rooks_connected(game_state: &GameState, color: Color) -> bool {
  let mut rooks = match color {
    Color::White => game_state.board.pieces.white.rook,
    Color::Black => game_state.board.pieces.black.rook,
  };

  if rooks.count_few_ones() != 2 {
    return false;
  }
  let rook_1 = rooks.trailing_zeros() as u8;
  rooks &= rooks - 1;
  let rook_2 = rooks.trailing_zeros() as u8;

  let destinations = game_state.board.get_piece_control_mask(rook_1);

  return square_in_mask!(rook_2, destinations);
}

/// Assigns a score to a rooks based on if it is located on:
/// * a closed file: 0.0
/// * a half-open file: 0.5
/// * an open file: 1.0
///
pub fn get_rooks_file_score(game_state: &GameState, color: Color) -> f32 {
  let mut score: f32 = 0.0;

  let mut rooks = match color {
    Color::White => game_state.board.pieces.white.rook,
    Color::Black => game_state.board.pieces.black.rook,
  };

  while rooks != 0 {
    let rook = rooks.trailing_zeros() as u8;
    let (file, _) = Board::index_to_fr(rook);

    match get_file_state(game_state, file) {
      FileState::Open => score += 1.0,
      FileState::HalfOpen => score += 0.5,
      FileState::Closed => {},
    }

    rooks &= rooks - 1;
  }

  score
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_rooks_connected() {
    // Not developed at all
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let game_state = GameState::from_fen(fen);
    assert_eq!(false, are_rooks_connected(&game_state, Color::Black));
    assert_eq!(false, are_rooks_connected(&game_state, Color::White));

    let fen = "r4rk1/pp1q1ppn/3bb2p/2pNp3/2PnP2N/3P2P1/PB3PBP/R2Q1RK1 w - - 5 15";
    let game_state = GameState::from_fen(fen);
    assert_eq!(true, are_rooks_connected(&game_state, Color::Black));
    assert_eq!(false, are_rooks_connected(&game_state, Color::White));

    let fen = "8/p1b3pk/q3bp1p/5Nn1/PR1pP3/2rP2P1/4Q1BP/1R4K1 b - - 0 28";
    let game_state = GameState::from_fen(fen);
    assert_eq!(false, are_rooks_connected(&game_state, Color::Black));
    assert_eq!(true, are_rooks_connected(&game_state, Color::White));
  }

  #[test]
  fn evaluate_well_placed_rooks() {
    // Compare 3 position, one with rook on closed file, half open and then open file
    let fen = "6k1/5ppp/6p1/8/8/8/5PPP/5RK1 w - - 0 12";
    let game_state = GameState::from_fen(fen);
    let eval_closed = get_rooks_file_score(&game_state, Color::White);

    let fen = "6k1/5ppp/6p1/8/8/8/4P1PP/5RK1 w - - 0 12";
    let game_state = GameState::from_fen(fen);
    let eval_half_open = get_rooks_file_score(&game_state, Color::White);

    let fen = "6k1/4p1pp/6p1/8/8/8/4P1PP/5RK1 w - - 0 12";
    let game_state = GameState::from_fen(fen);
    let eval_open = get_rooks_file_score(&game_state, Color::White);

    println!("Evaluation: closed: {eval_closed} - half open: {eval_half_open} - open: {eval_open}");
    assert!(eval_open > eval_half_open);
    assert!(eval_half_open > eval_closed);
  }
}
