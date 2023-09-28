use crate::model::board::*;
use crate::model::game_state::*;
use crate::model::piece::*;
use crate::model::piece_moves::KING_MOVES;

/// Determine the number of attacked squares surrounding the king
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine the number of pawn islands
///
/// # Returns
///
/// The number of squares surrounding the king attacked by enemy pieces
/// divided by the total number of squares around the king.
///
pub fn get_king_danger_score(game_state: &GameState, color: Color) -> f32 {
  if game_state.board.pieces.white.king == 0 || game_state.board.pieces.black.king == 0 {
    println!("King disappeared {}", game_state.to_fen());
    return 0.0;
  }

  let surrounding_squares = match color {
    Color::White => KING_MOVES[game_state.board.pieces.white.king.trailing_zeros() as usize],
    Color::Black => KING_MOVES[game_state.board.pieces.black.king.trailing_zeros() as usize],
  };

  let total_squares: f32 = surrounding_squares.count_ones() as f32;
  let attacked_squares: f32 = game_state
    .board
    .get_attacked_squares(surrounding_squares, Color::opposite(color))
    .count_ones() as f32;

  attacked_squares as f32 / total_squares as f32
}

/// Checks if the king is way too adventurous
///
/// Noticed that the engine likes to walk
/// the king up the board, but it should not do that unless opponent has no more
/// major pieces
///
/// ### Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine the number of pawn islands
///
/// ### Returns
///
/// True if the king has left its home rank and major enemy pieces are still here.
///
pub fn is_king_too_adventurous(game_state: &GameState, color: Color) -> bool {
  let king_position = match color {
    Color::White => game_state.board.get_white_king_square(),
    Color::Black => game_state.board.get_black_king_square(),
  };

  let (_, king_rank) = Board::index_to_fr(king_position);
  match (king_rank, color) {
    (1, Color::White) => {
      return false;
    },
    (8, Color::Black) => {
      return false;
    },
    (_, _) => {},
  }

  // Check for major enemy pieces
  if color == Color::White {
    return game_state.board.pieces.black.majors().count_ones() > 0;
  } else {
    return game_state.board.pieces.white.majors().count_ones() > 0;
  }
}

/// Returns true if the king does not seem castled and has lost his castling rights
///
/// ### Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine the number of pawn islands
///
/// ### Returns
///
/// True if the king is not on a castling destination square and has no castling rights
///
pub fn are_casling_rights_lost(game_state: &GameState, color: Color) -> bool {
  match color {
    Color::White => {
      if game_state.board.castling_rights.K() || game_state.board.castling_rights.Q() {
        return false;
      }
    },
    Color::Black => {
      if game_state.board.castling_rights.k() || game_state.board.castling_rights.q() {
        return false;
      }
    },
  }

  let king_square = match color {
    Color::White => game_state.board.get_white_king_square(),
    Color::Black => game_state.board.get_black_king_square(),
  };

  if color == Color::White && (king_square != 2 && king_square != 6) {
    return true;
  } else if color == Color::Black && (king_square != 62 && king_square != 58) {
    return true;
  }

  false
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_king_danger_score() {
    let fen = "rnb1kbnr/pppp1ppp/5q2/4p3/4P3/5Q2/PPPP1PPP/RNB1KBNR w KQkq - 2 3";
    let game_state = GameState::from_fen(fen);
    assert_eq!(0.0, get_king_danger_score(&game_state, Color::Black));
    assert_eq!(0.0, get_king_danger_score(&game_state, Color::White));

    let fen = "1r1qk1nr/p2bppbp/6p1/1p2N3/3p1P2/1Q4P1/PP1PP1BP/R1B1K2R b KQk - 0 12";
    let game_state = GameState::from_fen(fen);
    assert_eq!(2.0 / 5.0, get_king_danger_score(&game_state, Color::Black));
    assert_eq!(0.0 / 5.0, get_king_danger_score(&game_state, Color::White));

    let fen = "6k1/4pp1p/2n3p1/P7/8/6P1/3P1QKP/2q5 b - - 1 33";
    let game_state = GameState::from_fen(fen);
    assert_eq!(1.0 / 5.0, get_king_danger_score(&game_state, Color::Black));
    assert_eq!(3.0 / 8.0, get_king_danger_score(&game_state, Color::White));

    let fen = "8/4ppkp/2n3p1/P7/8/6P1/3P1QKP/2q5 w - - 2 34";
    let game_state = GameState::from_fen(fen);

    assert_eq!(2.0 / 8.0, get_king_danger_score(&game_state, Color::Black));
    assert_eq!(3.0 / 8.0, get_king_danger_score(&game_state, Color::White));
  }
}
