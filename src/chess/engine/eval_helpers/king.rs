use crate::chess::model::board::*;
use crate::chess::model::game_state::*;
use crate::chess::model::piece::*;

use log::*;

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
  let mut attacked_squares: usize = 0;
  let mut total_squares: usize = 0;

  let king_position = match color {
    Color::White => game_state.board.get_white_king_square(),
    Color::Black => game_state.board.get_black_king_square(),
  };

  let (king_file, king_rank) = Board::index_to_fr(king_position as usize);
  //println!("King coordinates: {king_file} {king_rank} ");
  if game_state.white_bitmap.is_none() || game_state.black_bitmap.is_none() {
    warn!("No game state bitmap available. Aborting get_king_danger_score calculation.");
    return 0.0;
  }

  let op_heatmap = match color {
    Color::White => game_state.black_bitmap.unwrap_or(0),
    Color::Black => game_state.white_bitmap.unwrap_or(0),
  };
  //print_mask(op_heatmap);

  for file_offset in -1..2 {
    for rank_offset in -1..2 as isize {
      if file_offset == 0 && rank_offset == 0 {
        continue;
      }
      let file = king_file as isize + file_offset;
      let rank = king_rank as isize + rank_offset;
      if file < 1 || file > 8 || rank < 1 || rank > 8 {
        continue;
      }
      let square = Board::fr_to_index(file as usize, rank as usize);

      total_squares += 1;
      if (1 << square) & op_heatmap != 0 {
        attacked_squares += 1;
      }
    }
  }

  if total_squares == 0 {
    // if the king has no surrounding squares, probably means something bad for the king.
    error!("No squares surrounding the king ?? The king is probably in a bad shape LOL.");
    return 1.0;
  }

  return attacked_squares as f32 / total_squares as f32;
}

/// Checks if the king is x-rayed in some way
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
pub fn is_king_xrayed(game_state: &GameState, color: Color) -> bool {
  let king_position = match color {
    Color::White => game_state.board.get_white_king_square(),
    Color::Black => game_state.board.get_black_king_square(),
  };

  let opponent_mask = game_state.get_color_bitmap_with_xrays(Color::opposite(color));
  return (1 << king_position) & opponent_mask == 0;
}

/// Checks if the king is way too adventurous (noticed that the engine likes to walk)
/// the king up the board, but it should not do that unless opponent has no more
/// major pieces
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine the number of pawn islands
///
/// # Returns
///
/// True if the king has left its home rank and major enemy pieces are still here.
///
pub fn is_king_too_adventurous(game_state: &GameState, color: Color) -> bool {
  let king_position = match color {
    Color::White => game_state.board.get_white_king_square(),
    Color::Black => game_state.board.get_black_king_square(),
  };

  let (_, king_rank) = Board::index_to_fr(king_position as usize);
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
  for i in 0..64 {
    if color == Color::White {
      match game_state.board.squares[i] {
        BLACK_QUEEN | BLACK_ROOK => {
          return true;
        },
        _ => {},
      }
    } else {
      match game_state.board.squares[i] {
        WHITE_QUEEN | WHITE_ROOK => {
          return true;
        },
        _ => {},
      }
    }
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
    let game_state = GameState::from_string(fen);
    assert_eq!(0.0, get_king_danger_score(&game_state, Color::Black));
    assert_eq!(0.0, get_king_danger_score(&game_state, Color::White));

    let fen = "1r1qk1nr/p2bppbp/6p1/1p2N3/3p1P2/1Q4P1/PP1PP1BP/R1B1K2R b KQk - 0 12";
    let game_state = GameState::from_string(fen);
    assert_eq!(2.0 / 5.0, get_king_danger_score(&game_state, Color::Black));
    assert_eq!(0.0 / 5.0, get_king_danger_score(&game_state, Color::White));

    let fen = "6k1/4pp1p/2n3p1/P7/8/6P1/3P1QKP/2q5 b - - 1 33";
    let game_state = GameState::from_string(fen);
    assert_eq!(1.0 / 5.0, get_king_danger_score(&game_state, Color::Black));
    assert_eq!(3.0 / 8.0, get_king_danger_score(&game_state, Color::White));

    let fen = "8/4ppkp/2n3p1/P7/8/6P1/3P1QKP/2q5 w - - 2 34";
    let game_state = GameState::from_string(fen);

    assert_eq!(2.0 / 8.0, get_king_danger_score(&game_state, Color::Black));
    assert_eq!(3.0 / 8.0, get_king_danger_score(&game_state, Color::White));
  }
}
