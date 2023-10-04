use super::pawn::*;
use crate::model::board::*;
use crate::model::board_geometry::*;
use crate::model::board_mask::*;
use crate::model::game_state::*;
use crate::model::piece::*;
use crate::model::piece_moves::*;
use crate::model::tables::pawn_destinations::*;

use log::*;

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

  match color {
    Color::White => {
      score += game_state.board.pieces.white.queen.count_ones() as f32 * QUEEN_VALUE;
      score += game_state.board.pieces.white.rook.count_ones() as f32 * ROOK_VALUE;
      score += game_state.board.pieces.white.bishop.count_ones() as f32 * BISHOP_VALUE;
      score += game_state.board.pieces.white.knight.count_ones() as f32 * KNIGHT_VALUE;
      score += game_state.board.pieces.white.pawn.count_ones() as f32 * PAWN_VALUE;
    },
    Color::Black => {
      score += game_state.board.pieces.black.queen.count_ones() as f32 * QUEEN_VALUE;
      score += game_state.board.pieces.black.rook.count_ones() as f32 * ROOK_VALUE;
      score += game_state.board.pieces.black.bishop.count_ones() as f32 * BISHOP_VALUE;
      score += game_state.board.pieces.black.knight.count_ones() as f32 * KNIGHT_VALUE;
      score += game_state.board.pieces.black.pawn.count_ones() as f32 * PAWN_VALUE;
    },
  }

  score
}

/// Computes the combined material score (white score - black score)
///
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
///
/// # Return value
///
/// Combined score for material
///
pub fn get_combined_material_score(game_state: &GameState) -> f32 {
  // Basic material count
  let mut score: f32 = 0.0;

  score += (game_state.board.pieces.white.queen.count_ones() as f32
    - game_state.board.pieces.black.queen.count_ones() as f32)
    * QUEEN_VALUE
    + (game_state.board.pieces.white.rook.count_ones() as f32
      - game_state.board.pieces.black.rook.count_ones() as f32)
      * ROOK_VALUE
    + (game_state.board.pieces.white.bishop.count_ones() as f32
      - game_state.board.pieces.black.bishop.count_ones() as f32)
      * BISHOP_VALUE
    + (game_state.board.pieces.white.knight.count_ones() as f32
      - game_state.board.pieces.black.knight.count_ones() as f32)
      * KNIGHT_VALUE
    + (game_state.board.pieces.white.pawn.count_ones() as f32
      - game_state.board.pieces.black.pawn.count_ones() as f32)
      * PAWN_VALUE;

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
pub fn is_file_open(game_state: &GameState, file: u8) -> bool {
  fr_bounds_or_return!(file, false);

  (FILES[(file - 1) as usize] & game_state.board.pieces.pawns()) == 0
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
pub fn is_file_half_open(game_state: &GameState, file: u8) -> bool {
  fr_bounds_or_return!(file, false);

  let black_pawn = (FILES[(file - 1) as usize] & game_state.board.pieces.black.pawn) != 0;
  let white_pawn = (FILES[(file - 1) as usize] & game_state.board.pieces.white.pawn) != 0;

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
pub fn get_outposts(game_state: &GameState, color: Color) -> BoardMask {
  let mut opponent_holes = get_holes(game_state, Color::opposite(color));
  let mut outposts: BoardMask = 0;

  while opponent_holes != 0 {
    let i = opponent_holes.trailing_zeros() as usize;

    match color {
      Color::White => {
        if BLACK_PAWN_CONTROL[i] & game_state.board.pieces.white.pawn != 0 {
          set_square_in_mask!(i, outposts);
        }
      },
      Color::Black => {
        if WHITE_PAWN_CONTROL[i] & game_state.board.pieces.black.pawn != 0 {
          set_square_in_mask!(i, outposts);
        }
      },
    }

    opponent_holes &= opponent_holes - 1;
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
  let piece = game_state.board.pieces.get(index as u8);
  let color;
  match piece {
    WHITE_KNIGHT | WHITE_BISHOP => color = Color::White,
    BLACK_KNIGHT | BLACK_BISHOP => color = Color::Black,
    _ => return false,
  }

  let outposts = get_outposts(game_state, color);

  let piece_destinations = match piece {
    WHITE_KNIGHT | BLACK_KNIGHT => {
      let same_side_pieces = game_state.board.get_piece_color_mask(color);
      get_knight_moves(same_side_pieces, 0, index)
    },
    WHITE_BISHOP | BLACK_BISHOP => {
      let same_side_pieces = game_state.board.get_piece_color_mask(color);
      let opponent_pieces = game_state
        .board
        .get_piece_color_mask(Color::opposite(color));
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
  let piece = game_state.board.pieces.get(index as u8);
  let color = match piece {
    WHITE_KNIGHT | WHITE_BISHOP => Color::White,
    BLACK_KNIGHT | BLACK_BISHOP => Color::Black,
    _ => return false,
  };

  let outposts = get_outposts(game_state, color);

  return square_in_mask!(index, outposts);
}

/// Checks if a piece or pawn is hanging
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
pub fn is_hanging(game_state: &GameState, index: u8) -> bool {
  let piece = game_state.board.pieces.get(index);

  let color = match piece {
    NO_PIECE | BLACK_KING | WHITE_KING => return false,
    WHITE_KNIGHT | WHITE_BISHOP | WHITE_ROOK | WHITE_QUEEN | WHITE_PAWN => Color::White,
    BLACK_KNIGHT | BLACK_BISHOP | BLACK_ROOK | BLACK_QUEEN | BLACK_PAWN => Color::Black,
    p => {
      warn!("Unknown piece onthe board! {p}");
      return false;
    },
  };

  let bitmap = game_state.board.get_attackers(index, color);

  bitmap == 0
}

/// Checks if a piece or pawn is attacked by enemy pieces
///
/// ### Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `index` -      Index of the square on the board
///
/// ### Return value
///
/// True if the square contains a piece or pawn (knight, bishop, rook, queen or pawn)
/// that is attacked by enemy pieces
/// False otherwise
///
pub fn is_attacked(game_state: &GameState, index: u8) -> bool {
  let color = match game_state.board.pieces.get(index) {
    NO_PIECE | BLACK_KING | WHITE_KING => return false,
    WHITE_KNIGHT | WHITE_BISHOP | WHITE_ROOK | WHITE_QUEEN | WHITE_PAWN => Color::Black,
    BLACK_KNIGHT | BLACK_BISHOP | BLACK_ROOK | BLACK_QUEEN | BLACK_PAWN => Color::White,
    p => {
      error!("Unknown piece onthe board! {p}");
      return false;
    },
  };

  let bitmap = game_state.board.get_attackers(index, color);

  bitmap != 0
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_open_files() {
    let fen = "2k5/pp3ppp/8/8/1r6/K7/Pq2BPPP/R6R w - - 5 26";
    let game_state = GameState::from_fen(fen);
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
    let game_state = GameState::from_fen(fen);
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
    let game_state = GameState::from_fen(fen);
    assert_eq!(0, get_outposts(&game_state, Color::White));
    assert_eq!(5497558138880, get_outposts(&game_state, Color::Black));

    let fen = "rnbqkbnr/3pp1pp/3N4/p4p2/1pPP4/1P2PN1P/P4PP1/R1BQKB1R b KQkq - 0 4";
    let game_state = GameState::from_fen(fen);
    /*
    let outposts = get_outposts(&game_state, Color::White);
    let outposts = get_outposts(&game_state, Color::Black);
    print_board_mask(outposts);
    */
    assert_eq!(327680, get_outposts(&game_state, Color::Black));
    assert_eq!(8606711808, get_outposts(&game_state, Color::White));
  }

  #[test]
  fn test_is_hanging() {
    let fen = "2k5/pp3ppp/8/8/1r6/K7/Pq2BPPP/R6R w - - 5 26";
    let game_state = GameState::from_fen(fen);

    assert_eq!(false, is_hanging(&game_state, 0));
    assert_eq!(false, is_hanging(&game_state, 7));
    assert_eq!(true, is_hanging(&game_state, 12));

    let fen = "r1bq1rk1/5pb1/p3p1p1/3pn3/QP6/3PP1N1/1P1BB1PP/1R3RK1 b - - 2 20";
    let game_state = GameState::from_fen(fen);

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

  #[test]
  fn test_material_scores() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let game_state = GameState::from_fen(fen);
    assert_eq!(get_combined_material_score(&game_state), 0.0);
    assert_eq!(get_material_score(&game_state, Color::White), 39.6);
    assert_eq!(get_material_score(&game_state, Color::Black), 39.6);

    let fen = "rnbqk1nr/pppppppp/8/8/8/8/PPPPP2P/RNBQKBNR w KQkq - 0 1";
    let game_state = GameState::from_fen(fen);
    assert_eq!(get_combined_material_score(&game_state), 1.05);
    assert_eq!(get_material_score(&game_state, Color::White), 37.6);
    assert_eq!(get_material_score(&game_state, Color::Black), 36.55);
  }
}
