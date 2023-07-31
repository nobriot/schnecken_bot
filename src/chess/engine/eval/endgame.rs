use super::helpers::bishop::*;
use super::helpers::generic::*;
use super::helpers::king::*;
use super::helpers::knight::*;
use super::helpers::mobility::*;
use super::helpers::pawn::*;
use super::position::*;
use crate::chess::model::board::*;
use crate::chess::model::board_geometry::*;
use crate::chess::model::game_state::*;
use crate::chess::model::piece::*;

const PIECE_MOBILITY_FACTOR: f32 = 0.01;
const KING_DANGER_FACTOR: f32 = 2.0;

// TODO: Consider this https://lichess.org/blog/W3WeMyQAACQAdfAL/7-piece-syzygy-tablebases-are-complete
// Or maybe just try as much as I can without any external resources.

/// Gives a score based on the endgame situation.
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine if development is completed.
pub fn get_endgame_position_evaluation(game_state: &GameState) -> f32 {
  if is_king_and_queen_endgame(game_state) || is_king_and_rook_endgame(game_state) {
    //debug!("Queen and/or rook vs King detected");
    return get_king_vs_queen_or_rook_score(game_state);
  }

  if is_king_and_pawn_endgame(game_state) {}

  // TODO: Implement a proper evaluation here
  let mut score: f32 = 0.0;

  score += PIECE_MOBILITY_FACTOR
    * ((get_piece_mobility(game_state, Color::White) as f32)
      - (get_piece_mobility(game_state, Color::Black) as f32));

  score += KING_DANGER_FACTOR
    * (get_king_danger_score(game_state, Color::Black)
      - get_king_danger_score(game_state, Color::White));

  score + default_position_evaluation(game_state)
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
        if queen_found && queen_color == Color::Black {
          return false;
        } else {
          queen_color = Color::White;
          queen_found = true;
        }
      },
      BLACK_QUEEN => {
        if queen_found && queen_color == Color::White {
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

  true
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
        if rook_found && rook_color == Color::Black {
          return false;
        } else {
          rook_color = Color::White;
          rook_found = true;
        }
      },
      BLACK_ROOK => {
        if rook_found && rook_color == Color::White {
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

  true
}

/// Checks if it is a King and pawns endgame
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
fn is_king_and_pawn_endgame(game_state: &GameState) -> bool {
  for i in 0..64 {
    match game_state.board.squares[i] {
      WHITE_ROOK | WHITE_BISHOP | WHITE_QUEEN | WHITE_KNIGHT | BLACK_BISHOP | BLACK_KNIGHT
      | BLACK_QUEEN | BLACK_ROOK => return false,
      _ => {},
    }
  }

  true
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

  let attacking_bitmap = match attacking_side {
    Color::White => game_state.white_bitmap.unwrap_or(0),
    Color::Black => game_state.black_bitmap.unwrap_or(0),
  };
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
      // Rank control is from 7 squares because the piece controlling the rank makes a hole if undefended.
      //println!("Rank {rank} is controlled {rank_control}");

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

  score
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
    let game_state = GameState::from_fen(fen);
    //println!("{}", game_state.board);
    let expected_score = -(64.0 - 12.0 + 7.0 - 4.0);
    assert_eq!(expected_score, get_king_vs_queen_or_rook_score(&game_state));

    // Another one from white's perpective
    // King is on a controlled rank, so it will go either side it likes.
    let fen = "8/8/3k1Q2/8/4K3/8/8/8 b - - 0 1";
    let game_state = GameState::from_fen(fen);
    let expected_score = 64.0 - 40.0 + 7.0 - 2.0;
    assert_eq!(expected_score, get_king_vs_queen_or_rook_score(&game_state));

    // From a game with another bot:
    let fen = "8/1k6/6Q1/5K2/8/8/8/8 w - - 7 80";
    let game_state = GameState::from_fen(fen);
    let expected_score = 64.0 - 12.0 + 7.0 - 4.0;
    assert_eq!(expected_score, get_king_vs_queen_or_rook_score(&game_state));

    // Check if the incentive to box the king better is good:
    let fen = "8/1k6/3Q4/5K2/8/8/8/8 b - - 8 80";
    let game_state = GameState::from_fen(fen);
    let expected_score = 64.0 - 6.0 + 7.0 - 4.0;
    assert_eq!(expected_score, get_king_vs_queen_or_rook_score(&game_state));
  }
}
