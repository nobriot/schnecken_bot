use crate::chess::model::board::*;
use crate::chess::model::game_state::*;
use crate::chess::model::piece::*;

/// Makes the sum of a board mask
///
/// # Arguments
///
/// * `mask` - u64 bitmask representing a board with 0 and 1s.
///
/// # Return value
///
/// the sum of all bits set to 1.
pub fn mask_sum(mask: u64) -> usize {
  let mut sum: usize = 0;
  for i in 0..64 {
    if mask >> i & 1 == 1 {
      sum += 1;
    }
  }
  sum
}

/// Computes the material score of a side
///
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine the material score
///
/// # Return value
///
/// Score for material
pub fn get_material_score(game_state: &GameState, color: Color) -> f32 {
  // Basic material count
  let mut score: f32 = 0.0;
  for i in 0..64 {
    if color == Color::White {
      match game_state.board.squares[i] {
        WHITE_QUEEN => score += 9.5,
        WHITE_ROOK => score += 5.0,
        WHITE_BISHOP => score += 3.05,
        WHITE_KNIGHT => score += 3.0,
        WHITE_PAWN => score += 1.0,
        _ => {},
      }
    } else {
      match game_state.board.squares[i] {
        BLACK_QUEEN => score += 9.5,
        BLACK_ROOK => score += 5.0,
        BLACK_BISHOP => score += 3.05,
        BLACK_KNIGHT => score += 3.0,
        BLACK_PAWN => score += 1.0,
        _ => {},
      }
    }
  }
  score
}

/// Checks if a file is open
///
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `file` -      File number (must be between 1 and 8)
///
/// # Return value
///
/// true if the file is open. false otherwise
pub fn is_file_open(game_state: &GameState, file: usize) -> bool {
  fr_bounds_or_return!(file, false);

  for rank in 1..9 {
    let i = Board::fr_to_index(file, rank);

    match game_state.board.squares[i] {
      WHITE_PAWN | BLACK_PAWN => {
        return false;
      },
      _ => {},
    }
  }

  true
}

/// Checks if a file is half-open
///
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `file` -      File number (must be between 1 and 8)
///
/// # Return value
///
/// true if the file is half-open. false otherwise
pub fn is_file_half_open(game_state: &GameState, file: usize) -> bool {
  fr_bounds_or_return!(file, false);

  let mut black_pawn = false;
  let mut white_pawn = false;

  for rank in 1..9 {
    let i = Board::fr_to_index(file, rank);

    match game_state.board.squares[i] {
      WHITE_PAWN => white_pawn = true,
      BLACK_PAWN => {
        black_pawn = true;
      },
      _ => {},
    }
  }

  match (white_pawn, black_pawn) {
    (true, false) => return true,
    (false, true) => return true,
    (_, _) => return false,
  }
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_open_files() {
    let fen = "2k5/pp3ppp/8/8/1r6/K7/Pq2BPPP/R6R w - - 5 26";
    let game_state = GameState::from_string(fen);
    assert_eq!(false, is_file_open(&game_state, 1));
    assert_eq!(false, is_file_open(&game_state, 2));
    assert_eq!(true, is_file_open(&game_state, 3));
    assert_eq!(true, is_file_open(&game_state, 4));
    assert_eq!(true, is_file_open(&game_state, 5));
    assert_eq!(false, is_file_open(&game_state, 6));
    assert_eq!(false, is_file_open(&game_state, 7));
    assert_eq!(false, is_file_open(&game_state, 8));

    // Out of bounds:
    assert_eq!(false, is_file_open(&game_state, 0));
    assert_eq!(false, is_file_open(&game_state, 9));

    // Also test half open files:
    assert_eq!(false, is_file_half_open(&game_state, 1));
    assert_eq!(true, is_file_half_open(&game_state, 2));
    assert_eq!(false, is_file_half_open(&game_state, 3));
    assert_eq!(false, is_file_half_open(&game_state, 4));
    assert_eq!(false, is_file_half_open(&game_state, 5));
    assert_eq!(false, is_file_half_open(&game_state, 6));
    assert_eq!(false, is_file_half_open(&game_state, 7));
    assert_eq!(false, is_file_half_open(&game_state, 8));

    // Out of bounds:
    assert_eq!(false, is_file_half_open(&game_state, 0));
    assert_eq!(false, is_file_half_open(&game_state, 9));

    // Try with another position
    let fen = "2k5/pp2p1p1/4p3/4p3/1r6/K5P1/Pq2B1PP/R6R w - - 5 26";
    let game_state = GameState::from_string(fen);
    assert_eq!(false, is_file_open(&game_state, 1));
    assert_eq!(false, is_file_open(&game_state, 2));
    assert_eq!(true, is_file_open(&game_state, 3));
    assert_eq!(true, is_file_open(&game_state, 4));
    assert_eq!(false, is_file_open(&game_state, 5));
    assert_eq!(true, is_file_open(&game_state, 6));
    assert_eq!(false, is_file_open(&game_state, 7));
    assert_eq!(false, is_file_open(&game_state, 8));
    assert_eq!(false, is_file_half_open(&game_state, 1));
    assert_eq!(true, is_file_half_open(&game_state, 2));
    assert_eq!(false, is_file_half_open(&game_state, 3));
    assert_eq!(false, is_file_half_open(&game_state, 4));
    assert_eq!(true, is_file_half_open(&game_state, 5));
    assert_eq!(false, is_file_half_open(&game_state, 6));
    assert_eq!(false, is_file_half_open(&game_state, 7));
    assert_eq!(true, is_file_half_open(&game_state, 8));
  }
}
