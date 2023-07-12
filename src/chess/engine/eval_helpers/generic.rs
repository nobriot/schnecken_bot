use crate::chess::engine::eval_helpers::pawn::*;
use crate::chess::model::board::*;
use crate::chess::model::game_state::*;
use crate::chess::model::piece::*;
use crate::chess::model::piece_moves::*;

use log::*;

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
    (true, false) => true,
    (false, true) => true,
    (_, _) => false,
  }
}

/// Finds the outputs for a color on a position
///
/// ### Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      Color for which we determine outposts
///
/// ### Return value
///
/// Board mask with outposts squares
pub fn get_outposts(game_state: &GameState, color: Color) -> u64 {
  let mut outposts: u64 = 0;

  let opponent_holes = get_holes(game_state, Color::opposite(color));

  for i in 0..64 {
    // Not a hole, not an outpost
    if (1 << i) & opponent_holes == 0 {
      continue;
    }

    // Now to be an outpost we need the hole to be defended by one of our pawns
    if is_square_protected_by_pawn(game_state, i, color) {
      outposts |= 1 << i;
    }
  }

  outposts
}

/// Checks if a bishop or knight has a reachable outpost (going there in 1 move)
///
/// ### Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `index` -      Index of the square on the board
///
/// ### Return value
///
///  True if the piece on the square is a knight or bishop and has a reachable outpost.
///
pub fn has_reachable_outpost(game_state: &GameState, index: usize) -> bool {
  let piece = game_state.board.squares[index];
  let color;
  match piece {
    WHITE_KNIGHT | WHITE_BISHOP => color = Color::White,
    BLACK_KNIGHT | BLACK_BISHOP => color = Color::Black,
    _ => return false,
  }

  let outposts = get_outposts(game_state, color);

  let piece_destinations = match piece {
    WHITE_KNIGHT | BLACK_KNIGHT => {
      let same_side_pieces = game_state.board.get_color_mask(color);
      get_knight_moves(same_side_pieces, 0, index)
    },
    WHITE_BISHOP | BLACK_BISHOP => {
      let same_side_pieces = game_state.board.get_color_mask(color);
      let opponent_pieces = game_state.board.get_color_mask(Color::opposite(color));
      get_bishop_moves(same_side_pieces, opponent_pieces, index)
    },
    _ => 0,
  };

  if piece_destinations & outposts > 0 {
    return true;
  }

  false
}

/// Checks if a bishop or knight is sitting on an outpost
///
/// ### Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `index` -      Index of the square on the board
///
/// ### Return value
///
/// True if the piece on the square is a knight or bishop is located on an outpost
/// False otherwise
///
pub fn occupies_reachable_outpost(game_state: &GameState, index: usize) -> bool {
  let piece = game_state.board.squares[index];
  let color = match piece {
    WHITE_KNIGHT | WHITE_BISHOP => Color::White,
    BLACK_KNIGHT | BLACK_BISHOP => Color::Black,
    _ => return false,
  };

  let outposts = get_outposts(game_state, color);
  if 1 << index & outposts > 0 {
    return true;
  }

  false
}

/// Checks if a piece is hanging
///
/// ### Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `index` -      Index of the square on the board
///
/// ### Return value
///
/// True if the square contains a piece or pawn (knight, bishop, rook, queen or pawn) that is hanging
/// False otherwise
///
pub fn is_hanging(game_state: &GameState, index: usize) -> bool {
  let piece = game_state.board.squares[index];

  let color = match piece {
    NO_PIECE | BLACK_KING | WHITE_KING => return false,
    WHITE_KNIGHT | WHITE_BISHOP | WHITE_ROOK | WHITE_QUEEN | WHITE_PAWN => Color::White,
    BLACK_KNIGHT | BLACK_BISHOP | BLACK_ROOK | BLACK_QUEEN | BLACK_PAWN => Color::Black,
    p => {
      warn!("Unknown piece onthe board! {p}");
      return false;
    },
  };

  let bitmap = if color == Color::White {
    game_state.white_bitmap
  } else {
    game_state.black_bitmap
  };

  if bitmap.is_none() {
    warn!("Unknown color bitmap, we won't know if pieces are hanging");
    return false;
  }

  if (1 << index & bitmap.unwrap()) > 0 {
    return false;
  }

  true
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

  #[test]
  fn test_outposts() {
    let fen = "2k5/pp3ppp/8/8/1r6/K7/Pq2BPPP/R6R w - - 5 26";
    let game_state = GameState::from_string(fen);
    assert_eq!(0, get_outposts(&game_state, Color::White));
    assert_eq!(5497558138880, get_outposts(&game_state, Color::Black));

    let fen = "rnbqkbnr/3pp1pp/3N4/p4p2/1pPP4/1P2PN1P/P4PP1/R1BQKB1R b KQkq - 0 4";
    let game_state = GameState::from_string(fen);
    /*
    let outposts = get_outposts(&game_state, Color::White);
    let outposts = get_outposts(&game_state, Color::Black);
    print_mask(outposts);
    */
    assert_eq!(327680, get_outposts(&game_state, Color::Black));
    assert_eq!(8606711808, get_outposts(&game_state, Color::White));
  }

  #[test]
  fn test_is_hanging() {
    let fen = "2k5/pp3ppp/8/8/1r6/K7/Pq2BPPP/R6R w - - 5 26";
    let game_state = GameState::from_string(fen);

    assert_eq!(false, is_hanging(&game_state, 0));
    assert_eq!(false, is_hanging(&game_state, 7));
    assert_eq!(true, is_hanging(&game_state, 12));

    let fen = "r1bq1rk1/5pb1/p3p1p1/3pn3/QP6/3PP1N1/1P1BB1PP/1R3RK1 b - - 2 20";
    let game_state = GameState::from_string(fen);

    assert_eq!(false, is_hanging(&game_state, 0));
    assert_eq!(false, is_hanging(&game_state, 1));
    assert_eq!(false, is_hanging(&game_state, 2));
    assert_eq!(false, is_hanging(&game_state, 3));
    assert_eq!(false, is_hanging(&game_state, 5));
    assert_eq!(false, is_hanging(&game_state, 6));
    assert_eq!(false, is_hanging(&game_state, 6));
    assert_eq!(true, is_hanging(&game_state, 11));
    assert_eq!(false, is_hanging(&game_state, 12));
    assert_eq!(false, is_hanging(&game_state, 22));
    assert_eq!(true, is_hanging(&game_state, 24));
    assert_eq!(true, is_hanging(&game_state, 56));
    assert_eq!(false, is_hanging(&game_state, 58));
  }
}
