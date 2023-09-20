use super::generic::*;
use crate::model::board::*;
use crate::model::game_state::*;
use crate::model::piece::*;

/// Determine if rooks are connected for a color
///
pub fn are_rooks_connected(game_state: &GameState, color: Color) -> bool {
  let rook = match color {
    Color::White => WHITE_ROOK,
    Color::Black => BLACK_ROOK,
  };

  let mut rook_1 = INVALID_SQUARE;
  let mut rook_2 = INVALID_SQUARE;

  for i in 0..64_u8 {
    if game_state.board.pieces.get(i) == rook {
      if rook_1 == INVALID_SQUARE {
        rook_1 = i;
      } else {
        rook_2 = i;
        break;
      }
    }
  }

  // We need the 2 rooks on the board for them to be connected
  if rook_2 == INVALID_SQUARE {
    return false;
  }

  let (f1, r1) = Board::index_to_fr(rook_1);
  let (f2, r2) = Board::index_to_fr(rook_2);

  if f1 != f2 && r1 != r2 {
    // Rooks neither on same file or rank, they cannot be connected
    return false;
  }

  if f1 == f2 {
    // Walk the ranks and check that they are no piece in between
    let min = std::cmp::min(r1, r2);
    let max = std::cmp::max(r1, r2);
    if max == min + 1 {
      return true;
    }
    for rank in (min + 1)..max {
      let i = Board::fr_to_index(f1, rank);
      if game_state.board.pieces.get(i as u8) != NO_PIECE {
        return false;
      }
    }
    return true;
  }

  // Same procedure if they are on the same rank
  if r1 == r2 {
    // Walk the ranks and check that they are no piece in between
    let min = std::cmp::min(f1, f2);
    let max = std::cmp::max(f1, f2);
    if max == min + 1 {
      return true;
    }

    for file in (min + 1)..max {
      let i = Board::fr_to_index(file, r1);
      if game_state.board.pieces.get(i as u8) != NO_PIECE {
        return false;
      }
    }
    return true;
  }

  false
}

/// Assigns a score to a rooks based on if it is located on:
/// * a closed file: 0.0
/// * a half-open file: 0.5
/// * an open file: 1.0
///
pub fn get_rooks_file_score(game_state: &GameState, color: Color) -> f32 {
  let mut score: f32 = 0.0;

  let rook = match color {
    Color::White => WHITE_ROOK,
    Color::Black => BLACK_ROOK,
  };

  for i in 0..64 {
    if game_state.board.pieces.get(i as u8) == rook {
      let (file, _) = Board::index_to_fr(i);
      if is_file_open(game_state, file) {
        score += 1.0;
      } else if is_file_half_open(game_state, file) {
        score += 0.5;
      }
    }
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
}
