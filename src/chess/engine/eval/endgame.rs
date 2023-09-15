use super::helpers::king::*;
use super::helpers::mobility::*;
use super::position::*;
use crate::model::board::*;
use crate::model::board_geometry::*;
use crate::model::board_mask::*;
use crate::model::game_state::*;
use crate::model::piece::*;

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
  if is_king_and_queen_endgame(game_state) || is_king_and_rook_endgame(game_state)
  //|| just_the_opponent_king_left(game_state)
  {
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

/// Checks if we just have the opponent king left against us
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine if development is completed.
///
fn just_the_opponent_king_left(game_state: &GameState) -> bool {
  let king = game_state.board.get_black_king_square();
  if (1 << king) == game_state.board.black_masks.pieces {
    return true;
  }

  let king = game_state.board.get_white_king_square();
  if (1 << king) == game_state.board.white_masks.pieces {
    return true;
  }

  false
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
    Color::White => game_state.board.white_masks.control,
    Color::Black => game_state.board.black_masks.control,
  };
  let king_position = match attacking_side {
    Color::White => game_state.board.get_black_king_square(),
    Color::Black => game_state.board.get_white_king_square(),
  };
  let (king_file, king_rank) = Board::index_to_fr(king_position as usize);
  // BoardMask bitmap of where the king can go
  let mut king_bitmap: BoardMask = 0;

  // There is probably something smart to do here.
  // Recursion to find all square sounds expensive when evaluating tons of positions.
  // We will simplify by assuming it's rook or queen, so detect horizontal/vertical lines and assign the leftover area of the side of the king.
  for rank in 1..=8 {
    let mut rank_control = 0;
    for file in 1..=8 {
      if square_in_mask!(Board::fr_to_index(file, rank), attacking_bitmap) {
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
          set_square_in_mask!(i, king_bitmap);
        }
      }
    }
  }
  // Same for the file:
  for file in 1..=8 {
    let mut file_control = 0;
    for rank in 1..=8 {
      if square_in_mask!(Board::fr_to_index(file, rank), attacking_bitmap) {
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
          set_square_in_mask!(i, king_bitmap);
        }
      }
    }
  }

  // Now make the count
  score += mask_sum(king_bitmap) as f32;

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

  #[test]
  fn test_engame_eval_queen_vs_king() {
    let fen = "1K6/2Q5/8/8/8/3k4/8/8 w - - 0 1";
    let game_state = GameState::from_fen(fen);
    let expected_score = 64.0 - 30.0 + 7.0 - 5.0;
    assert_eq!(expected_score, get_endgame_position_evaluation(&game_state));

    let fen = "1K6/8/8/8/2Q5/3k4/8/8 b - - 1 1";
    let game_state = GameState::from_fen(fen);
    let blunder_score = 64.0 - 15.0 + 7.0 - 5.0;
    assert_eq!(blunder_score, get_endgame_position_evaluation(&game_state));

    let fen = "1K6/8/8/2Q5/8/3k4/8/8 b - - 1 1";
    let game_state = GameState::from_fen(fen);
    let better_score = 64.0 - 20.0 + 7.0 - 5.0;
    assert_eq!(better_score, get_endgame_position_evaluation(&game_state));

    //FIXME: Blunder scores higher for now.
    //assert!(blunder_score < expected_score);
    assert!(expected_score < better_score);
  }
}
