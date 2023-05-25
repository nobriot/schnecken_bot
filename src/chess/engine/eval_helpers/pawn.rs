use crate::chess::engine::eval_helpers::*;
use crate::chess::model::board::*;
use crate::chess::model::board_geometry::*;
use crate::chess::model::game_state::*;
use crate::chess::model::piece::*;

/// Determine if a pawn on the board can be protected by another pawn
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `index` -      Index of the square on the board where the pawn is located
///
/// # Return value
///
/// true if the pawn is protected, false if there is no pawn on the square index
/// or it is not protected.
///
pub fn can_become_protected(game_state: &GameState, index: usize) -> bool {
  // Check if there is no other pawn behind:
  let pawn = game_state.board.squares[index];
  let (file, mut rank) = Board::index_to_fr(index);

  // Check if it can be defended by another pawn behind or on the side:
  while (rank >= 1) && (rank <= 8) {
    // Check on the left side:
    if file > 1 {
      let s = Board::fr_to_index(file - 1, rank);
      if game_state.board.squares[s] == pawn {
        return true;
      }
    }
    // Check on the right side:
    if file < 8 {
      let s = Board::fr_to_index(file + 1, rank);
      if game_state.board.squares[s] == pawn {
        return true;
      }
    }

    // white pawns are behind in ranks, black pawns ahead
    if pawn == WHITE_PAWN {
      rank -= 1;
    } else {
      rank += 1;
    }
  }

  return false;
}

/// Returns a board mask with backwards pawns for a color
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine backwards pawns
pub fn get_backwards_pawns(game_state: &GameState, color: Color) -> u64 {
  let mut mask: u64 = 0;

  let pawn = match color {
    Color::White => WHITE_PAWN,
    Color::Black => BLACK_PAWN,
  };
  let op_pawn = match color {
    Color::White => BLACK_PAWN,
    Color::Black => WHITE_PAWN,
  };

  for i in 0..64 {
    if game_state.board.squares[i] != pawn {
      continue;
    }

    if can_become_protected(game_state, i) {
      continue;
    }

    // Check if a opposite pawn controls the square in front:
    let (file, rank) = Board::index_to_fr(i);
    let new_rank = match color {
      Color::White => rank + 2,
      Color::Black => rank - 2,
    };

    if (new_rank > 8) || (new_rank < 1) {
      continue;
    }

    // Check on the left side:
    if file > 1 {
      let s = Board::fr_to_index(file - 1, new_rank);
      if game_state.board.squares[s] == op_pawn {
        mask |= 1 << i;
        continue;
      }
    }

    // Check on the right side:
    if file < 8 {
      let s = Board::fr_to_index(file + 1, new_rank);
      if game_state.board.squares[s] == op_pawn {
        mask |= 1 << i;
        continue;
      }
    }
  }
  mask
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use crate::chess::engine::eval_helpers::generic::mask_sum;

  use super::*;
  #[test]
  fn test_backwards_pawns() {
    // 2 backwards pawn in here: d7 and d4
    let fen = "rnbqkbnr/pp1p3p/2p3p1/2Pp2p1/PP1P4/8/6PP/RNBQKBNR w KQkq - 0 4";
    let mut game_state = GameState::from_string(fen);
    let mask = get_backwards_pawns(&game_state, Color::White);
    print_mask(mask);
    assert_eq!(1, mask_sum(mask));
    assert_eq!(1 << 27, mask);

    let mask = get_backwards_pawns(&game_state, Color::Black);
    print_mask(mask);
    assert_eq!(1, mask_sum(mask));
    assert_eq!(1 << 51, mask)
  }
}
