use crate::chess::model::game_state::*;
use crate::chess::model::piece::*;

/// Gives a score penalty for lack of development.
/// 0 for full development
/// +1 if connected rooks are missing
/// +1 per pieces that is still in the 1th / 8th row
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine if development is completed.
pub fn get_development_score(game_state: &GameState, color: Color) -> usize {
  let mut score: usize = 0;
  let rank = match color {
    Color::White => 0,
    Color::Black => 7,
  };

  // Check for trailing pieces first:
  for file in 0..8 {
    match game_state.board.squares[rank * 8 + file] {
      WHITE_BISHOP | WHITE_QUEEN | WHITE_KNIGHT | BLACK_BISHOP | BLACK_QUEEN | BLACK_KNIGHT => {
        score += 1
      },
      _ => {},
    }
  }

  // If pieces are around, we can conclude that rook are not connected
  if score != 0 {
    score += 1;
    return score;
  }

  // Check for trailing pieces first:
  let mut first_rook_found = false;
  for file in 0..8 {
    match game_state.board.squares[rank * 8 + file] {
      WHITE_ROOK | BLACK_ROOK => {
        if first_rook_found == true {
          // We just found the second rook, we are happy!
          return 0;
        } else {
          // First rook found
          first_rook_found = true;
        }
      },
      NO_PIECE => {},
      _ => {
        // Rooks are not connected!
        if first_rook_found == true {
          score += 1;
          return score;
        }
      },
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
  fn get_development_score_test() {
    // Not developed at all
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let game_state = GameState::from_string(fen);

    assert_eq!(6, get_development_score(&game_state, Color::Black));
    assert_eq!(6, get_development_score(&game_state, Color::White));

    // 1 piece developed
    let fen = "rnbqkb1r/pppppppp/5n2/8/8/5N2/PPPPPPPP/RNBQKB1R w KQkq - 2 2";
    let game_state = GameState::from_string(fen);

    assert_eq!(5, get_development_score(&game_state, Color::Black));
    assert_eq!(5, get_development_score(&game_state, Color::White));

    // 2 pieces developed
    let fen = "rn1qkb1r/pppppppp/5n2/5b2/2B5/5N2/PPPPPPPP/RNBQK2R w KQkq - 2 2";
    let game_state = GameState::from_string(fen);

    assert_eq!(4, get_development_score(&game_state, Color::Black));
    assert_eq!(4, get_development_score(&game_state, Color::White));

    // 3 pieces developed
    let fen = "r2qkb1r/pppppppp/2n2n2/5b2/2B5/2N2N2/PPPPPPPP/R1BQK2R w KQkq - 2 2";
    let game_state = GameState::from_string(fen);

    assert_eq!(3, get_development_score(&game_state, Color::Black));
    assert_eq!(3, get_development_score(&game_state, Color::White));

    // 4 pieces developed
    let fen = "r3kb1r/pppppppp/1qn2n2/5b2/2B5/2N2NQ1/PPPPPPPP/R1B1K2R w KQkq - 2 2";
    let game_state = GameState::from_string(fen);

    assert_eq!(2, get_development_score(&game_state, Color::Black));
    assert_eq!(2, get_development_score(&game_state, Color::White));

    // 5 pieces developed - no rook connection
    let fen = "r3k2r/ppppppbp/1qn2np1/5b2/2B5/1PN2NQ1/PBPPPPPP/R3K2R w KQkq - 2 2";
    let game_state = GameState::from_string(fen);

    assert_eq!(1, get_development_score(&game_state, Color::Black));
    assert_eq!(1, get_development_score(&game_state, Color::White));

    // Full development
    let fen = "r4rk1/ppppppbp/1qn2np1/5b2/2B5/1PN2NQ1/PBPPPPPP/2KR3R w - - 4 3";
    let game_state = GameState::from_string(fen);

    assert_eq!(0, get_development_score(&game_state, Color::White));
    assert_eq!(0, get_development_score(&game_state, Color::Black));
  }
}
