use crate::chess::engine::position_evaluation::*;
use crate::chess::model::board::*;
use crate::chess::model::board_geometry::*;
use crate::chess::model::game_state::*;
use crate::chess::model::piece::*;
use log::*;

/// Gives a score based on the endgame situation.
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine if development is completed.
pub fn get_endgame_score(game_state: &GameState) -> f32 {
  if is_king_and_queen_endgame(game_state) || is_king_and_rook_endgame(game_state) {
    return get_king_vs_queen_or_rook_score(game_state);
  }

  // TODO: Implement a proper evaluation here
  return 0.0;
}

/// Checks if it is a King-Queen vs King endgame
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine if development is completed.
fn is_king_and_queen_endgame(game_state: &GameState) -> bool {
  let mut queen_color = Color::White;
  let mut queen_found = false;
  for i in 0..64 {
    match game_state.board.squares[i] {
      WHITE_KING => {},
      BLACK_KING => {},
      WHITE_QUEEN => {
        if queen_found == true && queen_color == Color::Black {
          return false;
        } else {
          queen_color = Color::White;
          queen_found = true;
        }
      },
      BLACK_QUEEN => {
        if queen_found == true && queen_color == Color::White {
          return false;
        } else {
          queen_color = Color::Black;
          queen_found = true;
        }
      },
      NO_PIECE => {},
      _ => return false,
    }
  }

  return true;
}

/// Checks if it is a King-Queen vs King endgame
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine if development is completed.
fn is_king_and_rook_endgame(game_state: &GameState) -> bool {
  let mut rook_color = Color::White;
  let mut rook_found = false;
  for i in 0..64 {
    match game_state.board.squares[i] {
      WHITE_KING => {},
      BLACK_KING => {},
      WHITE_ROOK => {
        if rook_found == true && rook_color == Color::Black {
          return false;
        } else {
          rook_color = Color::White;
          rook_found = true;
        }
      },
      BLACK_ROOK => {
        if rook_found == true && rook_color == Color::White {
          return false;
        } else {
          rook_color = Color::Black;
          rook_found = true;
        }
      },
      NO_PIECE => {},
      _ => return false,
    }
  }

  return true;
}

/// Gives a score based on the endgame consisting of a King-Queen or Rook vs King
/// Note: This function assumes the board is in this configuration for its calculations
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine if development is completed.
pub fn get_king_vs_queen_or_rook_score(game_state: &GameState) -> f32 {
  // Find the rook and/or queen for the attacking side. Compute a cross from their position,
  // deduct in which section the king is, and how many squares it can navigate to.

  // In order to checkmate, we want the number of available square for the opponent
  // king to be reduced as much as possible.
  // King should come as close to the other king as possible.

  let mut score = 0.0;
  let mut attacking_side = Color::White;

  for i in 0..64 {
    match game_state.board.squares[i] {
      WHITE_ROOK | WHITE_QUEEN => {
        attacking_side = Color::White;
        break;
      },
      BLACK_ROOK | BLACK_QUEEN => {
        attacking_side = Color::Black;
        break;
      },
      _ => {},
    }
  }

  let attacking_bitmap = game_state.get_color_bitmap(attacking_side, false);
  let king_position = match attacking_side {
    Color::White => game_state.board.get_black_king_square(),
    Color::Black => game_state.board.get_white_king_square(),
  };
  let (king_file, king_rank) = Board::index_to_fr(king_position as usize);
  let mut king_bitmap: u64 = 0; // Inverted bitmap of where the king can go 0 it can go, 1 it cannot

  // There is probably something smart to do here.
  // Recursion to find all square sounds expensive when evaluating tons of positions.
  // We will simplify by assuming it's rook or queen, so detect horizontal/vertical lines and assign the leftover area of the side of the king.
  for rank in 1..=8 {
    let mut rank_control = 0;
    for file in 1..=8 {
      if (1 << Board::fr_to_index(file, rank)) & attacking_bitmap != 0 {
        rank_control += 1;
      }
    }

    if rank_control >= 7 {
      // Rank control is from 7 squares because
      if rank == king_rank {
        continue;
      }
      for i in 0..64 {
        let (_, current_rank) = Board::index_to_fr(i);
        if (current_rank <= rank && king_rank > rank) || (current_rank >= rank && king_rank < rank)
        {
          king_bitmap |= 1 << i;
        }
      }
    }
  }
  // Same for the file:
  for file in 1..=8 {
    let mut file_control = 0;
    for rank in 1..=8 {
      if (1 << Board::fr_to_index(file, rank)) & attacking_bitmap != 0 {
        file_control += 1;
      }
    }

    if file_control >= 7 {
      if file == king_file {
        continue;
      }
      for i in 0..64 {
        let (current_file, _) = Board::index_to_fr(i);
        if (current_file <= file && king_file > file) || (current_file >= file && king_file < file)
        {
          king_bitmap |= 1 << i;
        }
      }
    }
  }
  // Now make the count
  //print_mask(king_bitmap);
  let mut available_squares = 0;
  for i in 0..64 {
    if (king_bitmap & (1 << i)) == 0 {
      available_squares += 1;
    }
  }

  score += (64 - available_squares) as f32;

  // Now check how many square are available for each king
  score += 7.0
    - get_king_distance(
      game_state.board.get_white_king_square() as usize,
      game_state.board.get_black_king_square() as usize,
    ) as f32;

  if attacking_side == Color::Black {
    score = -score;
  }

  return score;
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_king_vs_queen_or_rook_score() {
    // Simple position, the king is boxed with 12 squares and the kings are 4 steps apart.
    let fen = "8/8/3k4/8/8/6q1/3K4/8 w - - 0 1";
    let game_state = GameState::from_string(fen);
    let expected_score = -(64.0 - 12.0 + 7.0 - 4.0);
    assert_eq!(expected_score, get_king_vs_queen_or_rook_score(&game_state));

    // Another one from white's perpective
    // King is on a controlled rank, so it will go either side it likes.
    let fen = "8/8/3k1Q2/8/4K3/8/8/8 b - - 0 1";
    let game_state = GameState::from_string(fen);
    let expected_score = 64.0 - 40.0 + 7.0 - 2.0;
    assert_eq!(expected_score, get_king_vs_queen_or_rook_score(&game_state));
  }
}
