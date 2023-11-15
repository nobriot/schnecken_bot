use super::helpers::generic::get_material_score;
use super::position::*;
use crate::engine::eval::helpers::pawn::is_passed;
use crate::engine::square_affinity::*;
use crate::model::board_geometry::*;
use crate::model::board_mask::*;
use crate::model::game_state::*;
use crate::model::piece::*;
use crate::model::piece_moves::KING_MOVES;

//const PIECE_MOBILITY_FACTOR: f32 = 0.01;
//const KING_DANGER_FACTOR: f32 = 2.0;
const SQUARE_TABLE_FACTOR: f32 = 0.03;
const PASSED_PAWN_FACTOR: f32 = 0.02;

// TODO: Consider this https://lichess.org/blog/W3WeMyQAACQAdfAL/7-piece-syzygy-tablebases-are-complete
// Or maybe just try as much as I can without any external resources.

/// Computes a total score based on the square where pieces are located in the
/// endgame.
///
/// ### Arguments
///
/// * `game_state`: GameState reference
///
/// ### Return value
///
/// f32 score that can be applied to the evaluation
///
pub fn get_square_table_endgame_score(game_state: &GameState) -> f32 {
  let mut score = 0.0;
  for (i, piece) in game_state.board.pieces.white {
    match piece {
      PieceType::King => score += EndgameSquareTable::KING[i as usize] as f32,
      PieceType::Queen => score += EndgameSquareTable::QUEEN[i as usize] as f32,
      PieceType::Rook => score += EndgameSquareTable::WHITE_ROOK[i as usize] as f32,
      PieceType::Bishop => score += SquareTable::WHITE_BISHOP[i as usize] as f32,
      PieceType::Knight => score += SquareTable::KNIGHT[i as usize] as f32,
      PieceType::Pawn => score += SquareTable::WHITE_PAWN[i as usize] as f32,
    }
  }
  for (i, piece) in game_state.board.pieces.black {
    match piece {
      PieceType::King => score -= EndgameSquareTable::KING[i as usize] as f32,
      PieceType::Queen => score -= EndgameSquareTable::QUEEN[i as usize] as f32,
      PieceType::Rook => score -= EndgameSquareTable::BLACK_ROOK[i as usize] as f32,
      PieceType::Bishop => score -= SquareTable::BLACK_BISHOP[i as usize] as f32,
      PieceType::Knight => score -= SquareTable::KNIGHT[i as usize] as f32,
      PieceType::Pawn => score -= SquareTable::BLACK_PAWN[i as usize] as f32,
    }
  }
  score * SQUARE_TABLE_FACTOR
}

/// Gives a score based on the endgame situation.
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine if development is completed.
pub fn get_endgame_position_evaluation(game_state: &GameState) -> f32 {
  if just_the_opponent_king_left(game_state) {
    return get_king_vs_queen_or_rook_score(game_state);
  }

  // Check if we cannot win, cap the score to 0:
  let max_score = if one_minor_left(game_state, Color::White) { 0.0 } else { 200.0 };
  let min_score = if one_minor_left(game_state, Color::Black) { 0.0 } else { -200.0 };

  // TODO: Implement a proper evaluation here
  let mut score: f32 = 0.0;

  // Check if we have good passed pawns for white.
  let mut pawns = game_state.board.pieces.white.pawn;
  let mut passed_pawns_score = 0;
  while pawns != 0 {
    let pawn = pawns.trailing_zeros() as usize;

    if is_passed(game_state, pawn as u8) {
      passed_pawns_score += SquareTable::WHITE_PASSED_PAWN[pawn];
    }

    pawns &= pawns - 1
  }
  score += PASSED_PAWN_FACTOR * passed_pawns_score as f32;

  // Check if we have good passed pawns for black.
  let mut pawns = game_state.board.pieces.black.pawn;
  let mut passed_pawns_score = 0;
  while pawns != 0 {
    let pawn = pawns.trailing_zeros() as usize;

    if is_passed(game_state, pawn as u8) {
      passed_pawns_score += SquareTable::BLACK_PASSED_PAWN[pawn];
    }

    pawns &= pawns - 1
  }
  score -= PASSED_PAWN_FACTOR * passed_pawns_score as f32;

  /*
  score += PIECE_MOBILITY_FACTOR
    * ((get_piece_mobility(game_state, Color::White) as f32)
      - (get_piece_mobility(game_state, Color::Black) as f32));

      score += KING_DANGER_FACTOR
      * (get_king_danger_score(game_state, Color::Black)
      - get_king_danger_score(game_state, Color::White));
      */

  score += get_square_table_endgame_score(game_state);

  score += default_position_evaluation(game_state);
  if score < min_score {
    score = min_score;
  } else if score > max_score {
    score = max_score;
  }

  score
}

/// Checks if we just have the opponent king left against us
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine if development is completed.
///
#[inline]
fn just_the_opponent_king_left(game_state: &GameState) -> bool {
  (game_state.board.pieces.black.all() == game_state.board.pieces.black.king)
    | (game_state.board.pieces.white.all() == game_state.board.pieces.white.king)
}

/// Checks if a color just has one minor piece left
///
/// ### Arguments
///
/// * `game_state`: State of the game
/// * `color`:      Side for which we want to look at minor pieces.
///
/// ### Return value
///
/// True if we have the king and one minor, false otherwhise.
///
#[inline]
pub fn one_minor_left(game_state: &GameState, color: Color) -> bool {
  match color {
    Color::White => {
      (game_state.board.pieces.white.pawn | game_state.board.pieces.white.majors()) == 0
        && game_state.board.pieces.white.minors().count_few_ones() == 1
    },
    Color::Black => {
      (game_state.board.pieces.black.pawn | game_state.board.pieces.black.majors()) == 0
        && game_state.board.pieces.black.minors().count_few_ones() == 1
    },
  }
}

/// Gives a score based on the endgame consisting of a King-Queen or Rook vs King
///
/// ### Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine if development is completed.
///
/// ### Return value
///
/// f32 evaluation score for the position
///
pub fn get_king_vs_queen_or_rook_score(game_state: &GameState) -> f32 {
  // Try to assign a better score as we are getting closer to corner the king

  let attacking_side = if game_state.board.pieces.black.all() == game_state.board.pieces.black.king
  {
    Color::White
  } else {
    Color::Black
  };

  let mut score = get_material_score(game_state, attacking_side);

  let king_position = match attacking_side {
    Color::White => game_state.board.get_black_king_square(),
    Color::Black => game_state.board.get_white_king_square(),
  } as usize;

  if king_position > 64 {
    return 0.0;
  }

  // BoardMask bitmap of where the king can go. We want as few squares as possible
  score += game_state
    .board
    .get_attacked_squares(KING_MOVES[king_position], attacking_side)
    .count_ones() as f32;

  // Now check how many square are available for each king
  score += 7.0
    - get_king_distance(
      game_state.board.get_white_king_square(),
      game_state.board.get_black_king_square(),
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
    let expected_score = -(QUEEN_VALUE + 4.0 + 3.0);
    assert_eq!(expected_score, get_king_vs_queen_or_rook_score(&game_state));

    // Another one from white's perpective
    // King is on a controlled rank, so it will go either side it likes.
    let fen = "8/8/3k1Q2/8/4K3/8/8/8 b - - 0 1";
    let game_state = GameState::from_fen(fen);
    let expected_score = QUEEN_VALUE + 5.0 + 5.0;
    assert_eq!(expected_score, get_king_vs_queen_or_rook_score(&game_state));

    // From a game with another bot:
    let fen = "8/1k6/6Q1/5K2/8/8/8/8 w - - 7 80";
    let game_state = GameState::from_fen(fen);
    let expected_score = QUEEN_VALUE + 3.0 + 3.0;
    assert_eq!(expected_score, get_king_vs_queen_or_rook_score(&game_state));

    // Check if the incentive to box the king better is good:
    let fen = "8/1k6/3Q4/5K2/8/8/8/8 b - - 8 80";
    let game_state = GameState::from_fen(fen);
    let expected_score = QUEEN_VALUE + 5.0 + 3.0;
    assert_eq!(expected_score, get_king_vs_queen_or_rook_score(&game_state));
  }

  #[test]
  fn test_engame_eval_queen_vs_king() {
    let fen = "1K6/2Q5/8/8/8/3k4/8/8 w - - 0 1";
    let game_state = GameState::from_fen(fen);
    let expected_score = QUEEN_VALUE + 3.0 + 2.0;
    assert_eq!(expected_score, get_endgame_position_evaluation(&game_state));

    let fen = "1K6/8/8/8/2Q5/3k4/8/8 b - - 1 1";
    let game_state = GameState::from_fen(fen);
    let blunder_score = QUEEN_VALUE + 5.0 + 2.0;
    assert_eq!(blunder_score, get_endgame_position_evaluation(&game_state));

    let fen = "1K6/8/8/2Q5/8/3k4/8/8 b - - 1 1";
    let game_state = GameState::from_fen(fen);
    let better_score = QUEEN_VALUE + 5.0 + 2.0;
    assert_eq!(better_score, get_endgame_position_evaluation(&game_state));

    //FIXME: Blunder scores higher for now.
    //assert!(blunder_score < expected_score);
    assert!(expected_score < better_score);
  }

  #[test]
  fn test_max_min_score_with_one_minor_left() {
    let fen = "2k5/2p5/8/8/3N4/2K5/8/8 w - - 0 1";
    let game_state = GameState::from_fen(fen);
    let expected_score = 0.0;
    assert_eq!(expected_score, get_endgame_position_evaluation(&game_state));

    let fen = "2k5/2p5/8/8/3B4/2K5/8/8 w - - 0 1";
    let game_state = GameState::from_fen(fen);
    assert_eq!(expected_score, get_endgame_position_evaluation(&game_state));

    let fen = "2k5/3b4/8/8/3P4/2K5/8/8 w - - 0 1";
    let game_state = GameState::from_fen(fen);
    assert_eq!(expected_score, get_endgame_position_evaluation(&game_state));

    let fen = "2k5/3n4/8/8/3P4/2K5/8/8 w - - 0 1";
    let game_state = GameState::from_fen(fen);
    assert_eq!(expected_score, get_endgame_position_evaluation(&game_state));
  }
}
