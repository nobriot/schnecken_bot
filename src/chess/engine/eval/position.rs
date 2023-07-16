use log::*;

// From our module
use super::endgame::get_endgame_position_evaluation;
use super::helpers::bishop::*;
use super::helpers::generic::*;
use super::helpers::knight::*;
use super::helpers::pawn::*;
use super::helpers::rook::*;
use super::middlegame::get_middlegame_position_evaluation;
use super::opening::get_opening_position_evaluation;

// From another crate
use crate::chess::model::game_state::*;
use crate::chess::model::piece::*;

// Constants
const PAWN_ISLAND_FACTOR: f32 = 0.01;
const PASSED_PAWN_FACTOR: f32 = 0.2;
const PROTECTED_PASSED_PAWN_FACTOR: f32 = 0.6;
const PROTECTED_PAWN_FACTOR: f32 = 0.05;
const BACKWARDS_PAWN_FACTOR: f32 = 0.005;
const CONNECTED_ROOKS_FACTOR: f32 = 0.01;
const ROOK_FILE_FACTOR: f32 = 0.015;
const HANGING_FACTOR: f32 = 0.2;
const HANGING_PENALTY: f32 = 0.1;
const REACHABLE_OUTPOST_BONUS: f32 = 0.2;
const OUTPOST_BONUS: f32 = 0.9;

// Shows "interesting" squares to control on the board
// Giving them a score
pub const HEATMAP_SCORES: [f32; 64] = [
  0.005, 0.005, 0.005, 0.005, 0.005, 0.005, 0.005, 0.005, // 1st row
  0.005, 0.005, 0.005, 0.005, 0.005, 0.005, 0.005, 0.005, // 2nd row
  0.005, 0.005, 0.01, 0.01, 0.01, 0.01, 0.005, 0.005, // 3rd row
  0.005, 0.01, 0.015, 0.02, 0.02, 0.015, 0.01, 0.005, // 4th row
  0.005, 0.01, 0.015, 0.02, 0.02, 0.015, 0.01, 0.005, // 5th row
  0.005, 0.005, 0.01, 0.01, 0.01, 0.01, 0.005, 0.005, // 6th row
  0.005, 0.005, 0.005, 0.005, 0.005, 0.005, 0.005, 0.005, // 7th row
  0.005, 0.005, 0.005, 0.005, 0.005, 0.005, 0.005, 0.005, // 8th row
];

/// Default way to look at a position regardless of the game phase
///
/// ### Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
///
/// ### Returns
///
/// Score assigned to the position, applicable in all game phases
///
pub fn default_position_evaluation(game_state: &GameState) -> f32 {
  let mut score: f32 = 0.0;

  // Pawn structure comparisons
  score += PAWN_ISLAND_FACTOR
    * (get_number_of_pawn_islands(game_state, Color::Black) as f32
      - get_number_of_pawn_islands(game_state, Color::White) as f32);

  score += PASSED_PAWN_FACTOR
    * (get_number_of_passers(game_state, Color::White) as f32
      - get_number_of_passers(game_state, Color::Black) as f32);

  score += PROTECTED_PASSED_PAWN_FACTOR
    * (get_number_of_protected_passers(game_state, Color::White) as f32
      - get_number_of_protected_passers(game_state, Color::Black) as f32);

  score += PROTECTED_PAWN_FACTOR
    * (get_number_of_protected_pawns(game_state, Color::White) as f32
      - get_number_of_protected_pawns(game_state, Color::Black) as f32);

  score += BACKWARDS_PAWN_FACTOR
    * (mask_sum(get_backwards_pawns(game_state, Color::Black)) as f32
      - mask_sum(get_backwards_pawns(game_state, Color::White)) as f32);

  // Evaluate the quality of our rooks:
  if are_rooks_connected(game_state, Color::White) {
    score += CONNECTED_ROOKS_FACTOR;
  }
  if are_rooks_connected(game_state, Color::Black) {
    score -= CONNECTED_ROOKS_FACTOR;
  }

  score += ROOK_FILE_FACTOR
    * (get_rooks_file_score(game_state, Color::Black)
      - get_rooks_file_score(game_state, Color::White));

  // Get a pressure score, if one side has more attackers than defenders on a square, they get bonus points
  let white_heatmap = game_state.get_heatmap(Color::White, false);
  let black_heatmap = game_state.get_heatmap(Color::Black, false);

  for i in 0..64_usize {
    score += HEATMAP_SCORES[i] * white_heatmap[i] as f32;
    score -= HEATMAP_SCORES[i] * black_heatmap[i] as f32;

    if !game_state.board.has_piece(i as u8) {
      continue;
    }
    let score_factor = Color::score_factor(Piece::color_from_u8(game_state.board.squares[i]));
    /*
     */
    // We are excited about hanging pieces when it's our turn :-)
    // Here it could probably be better.
    if is_hanging(game_state, i) {
      if is_attacked(game_state, i)
        && (game_state.side_to_play
          == Color::opposite(Piece::color_from_u8(game_state.board.squares[i])))
      {
        score -= HANGING_FACTOR
          * score_factor
          * Piece::material_value_from_u8(game_state.board.squares[i]);
      } else {
        // We usually are not the most fan of hanging pieces
        score -= HANGING_PENALTY * score_factor;
      }
    }
    // Check if we have some good positional stuff
    if has_reachable_outpost(game_state, i) {
      score += REACHABLE_OUTPOST_BONUS * score_factor;
    }
    if occupies_reachable_outpost(game_state, i) {
      score += OUTPOST_BONUS * score_factor;
    }

    // Piece attacks
    score += score_factor * pawn_attack(game_state, i) / 3.1;
    let value = knight_attack(game_state, i);
    if value.abs() > 3.0 {
      score += score_factor * (value - 3.0) / 2.3;
    }
    let value = bishop_attack(game_state, i);
    if value.abs() > 3.1 {
      score += score_factor * (value - 3.1) / 2.3;
    }
  }

  // Basic material count
  let white_material = get_material_score(game_state, Color::White);
  let black_material = get_material_score(game_state, Color::Black);
  score += white_material - black_material;

  // Return our score
  score
}

/// Determines if a position is a game over due to insufficient material or not
///
/// ### Arguments
///
/// * `game_state` - A GameState object reference representing a position, side to play, etc.
///
/// ### Returns
///
/// True if is it a game over (draw) by insufficient material
/// false otherwise
///
pub fn is_game_over_by_insufficient_material(game_state: &GameState) -> bool {
  let mut minor_piece_count = 0;
  for i in 0..64 {
    match game_state.board.squares[i] {
      NO_PIECE | WHITE_KING | BLACK_KING => {},
      WHITE_BISHOP | WHITE_KNIGHT | BLACK_BISHOP | BLACK_KNIGHT => {
        minor_piece_count += 1;
        if minor_piece_count > 1 {
          return false;
        }
      },
      _ => {
        return false;
      },
    }
  }
  true
}

pub fn is_game_over_by_repetition(game_state: &GameState) -> bool {
  let current_position = game_state.board.to_string();
  let mut repetition_count = 0;
  for position in &game_state.last_positions {
    if current_position.as_str() == position {
      repetition_count += 1;
    }
  }

  // We need to find 2 occurences of the same as the current to make it a 3 fold repetition
  repetition_count >= 2
}

/// Evaluates a position and  tells if it seems to be game over or not
///
/// ### Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
///
/// ### Returns
///
/// * f32 -> Score assigned to the position (+200.0 for white win, -200.0 for black win, 0.0 for draw)
///       Can be ignored if the `bool` is false.
///
/// * bool -> True if it is a game over (checkmate, stalemate, repetitions, etc.) All cases included
/// false if the game is ongoing and must be evaluated manually
///
///
pub fn is_game_over(game_state: &GameState) -> (f32, bool) {
  if !game_state.available_moves_computed {
    warn!("Evaluating a position without move list computed, cannot determine if it is a game over position.");
  }
  if game_state.available_moves_computed && game_state.move_list.is_empty() {
    match (game_state.side_to_play, game_state.checks) {
      (_, 0) => return (0.0, true),
      (Color::Black, _) => return (200.0, true),
      (Color::White, _) => return (-200.0, true),
    }
  }
  if game_state.ply >= 100 {
    debug!("100 Ply detected");
    return (0.0, true);
  }
  // 2 kings, or 1 king + knight or/bishop vs king is game over:
  if is_game_over_by_insufficient_material(game_state) {
    debug!("game over by insufficient material detected");
    return (0.0, true);
  }

  // Check the 3-fold repetitions
  if is_game_over_by_repetition(game_state) {
    debug!("3-fold repetition detected");
    return (0.0, true);
  }

  (0.0, false)
}

/// Evaluates a position and returns a score and if the game is over.
///
/// ### Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
///
/// ### Returns
///
/// Score assigned to the position.
pub fn evaluate_position(game_state: &GameState) -> (f32, bool) {
  // Check if the evaluation is due to a game over:
  let (mut score, game_over) = is_game_over(game_state);
  if game_over {
    return (score, game_over);
  }

  if game_state.game_phase.is_some() {
    match game_state.game_phase.unwrap() {
      GamePhase::Opening => score = get_opening_position_evaluation(game_state),
      GamePhase::Middlegame => score = get_middlegame_position_evaluation(game_state),
      GamePhase::Endgame => score = get_endgame_position_evaluation(game_state),
    }
  } else {
    warn!("Evaluating a position in an unknown game phase");
    score = default_position_evaluation(game_state);
  }

  (score, false)
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use crate::chess::model::board::Move;

  use super::*;
  #[test]
  fn test_evaluate_position() {
    // This is a forced checkmate in 2:
    let fen = "1n4nr/5ppp/1N6/1P2p3/1P1k4/5P2/1p1NP1PP/R1B1KB1R w KQ - 0 35";
    let mut game_state = GameState::from_string(fen);
    let (evaluation, game_over) = evaluate_position(&game_state);
    assert_eq!(false, game_over);
    println!("Evaluation {evaluation}");
  }

  #[test]
  fn test_evaluate_position_checkmate_in_one() {
    // This is a forced checkmate in 1:
    let fen = "1n4nr/5ppp/1N6/1P2p3/1P6/4kP2/1B1NP1PP/R3KB1R w KQ - 1 36";
    let mut game_state = GameState::from_string(fen);
    let (evaluation, game_over) = evaluate_position(&game_state);
    assert_eq!(false, game_over);
    println!("Evaluation {evaluation}");
  }

  #[test]
  fn test_evaluate_position_checkmate() {
    // This is a "game over" position
    let fen = "1n4nr/5ppp/8/1P1Np3/1P6/4kP2/1B1NP1PP/R3KB1R b KQ - 2 37";
    let mut game_state = GameState::from_string(fen);
    game_state.get_moves();
    let (evaluation, game_over) = evaluate_position(&game_state);
    assert_eq!(true, game_over);
    assert_eq!(200.0, evaluation);
  }
  #[test]
  fn test_evaluate_position_hanging_queen() {
    // This should obviously be very bad for white:
    let fen = "rnbqkb1r/ppp1pppQ/5n2/3p4/3P4/8/PPP1PPPP/RNB1KBNR b KQkq - 0 3";
    let mut game_state = GameState::from_string(fen);
    game_state.get_moves();
    let (evaluation, game_over) = evaluate_position(&game_state);
    println!("Evaluation : {evaluation} - Game Over: {game_over}");
    assert_eq!(false, game_over);
    assert!(evaluation < -3.0);
  }

  #[test]
  fn test_evaluate_position_queen_standoff() {
    // This should obviously be okay because queen is defended and attacked by a queen.
    let fen = "rnb1kbnr/pppp1ppp/5q2/4p3/4P3/5Q2/PPPP1PPP/RNB1KBNR w KQkq - 2 3";
    let mut game_state = GameState::from_string(fen);
    game_state.get_moves();
    let (evaluation, game_over) = evaluate_position(&game_state);
    assert_eq!(false, game_over);
    println!("Evaluation: {}", evaluation);
    assert!(evaluation < 1.0);
    assert!(evaluation > -1.0);
  }

  #[test]
  fn test_evaluate_position_losing() {
    // We had this evaluated in favor of black in a game: (after a certain continuation)
    /* INFO  schnecken_bot::chess::engine::core] Using 5866 ms to find a move
    Line 0 Eval: -2.0450013 - h8f8 a8d5 f6d5
    Line 1 Eval: -1.9150015 - d6c5 a8d5 f6d5
    Line 2 Eval: -1.8200021 - e8g8 a8d5 f6d5
    Line 3 Eval: 7.735001 - d6e5 a8f3 g7f8 d2c1
    Line 4 Eval: 7.7650003 - e8c6 a8c6 b8c6 f1e1
     */
    let fen = "Qn2q2r/2p2pb1/p2k1n1p/5Bp1/8/2NP4/PPPB1PPP/R4RK1 b - - 0 15";
    let mut game_state = GameState::from_string(fen);
    game_state.get_moves();
    let (evaluation, game_over) = evaluate_position(&game_state);
    assert_eq!(false, game_over);
    println!("Evaluation: {}", evaluation);
    assert!(evaluation > 7.0);
  }

  #[test]
  fn test_game_over_insufficient_material() {
    let fen = "8/4nk2/8/8/8/2K5/8/8 w - - 0 1";
    let game_state = GameState::from_string(fen);
    assert_eq!(true, is_game_over_by_insufficient_material(&game_state));

    let fen = "8/5k2/8/8/8/2KB4/8/8 w - - 0 1";
    let game_state = GameState::from_string(fen);
    assert_eq!(true, is_game_over_by_insufficient_material(&game_state));

    let fen = "8/4nk2/8/8/8/2KB4/8/8 w - - 0 1";
    let game_state = GameState::from_string(fen);
    assert_eq!(false, is_game_over_by_insufficient_material(&game_state));

    let fen = "8/4nk2/8/8/8/2KR4/8/8 w - - 0 1";
    let game_state = GameState::from_string(fen);
    assert_eq!(false, is_game_over_by_insufficient_material(&game_state));
  }

  #[test]
  fn test_game_over_threefold_repetition_1() {
    let fen = "8/8/8/5P2/Q1k2KP1/8/p7/8 b - - 1 87";
    let mut game_state = GameState::from_string(fen);
    assert_eq!(false, is_game_over_by_repetition(&game_state));

    game_state.apply_move(&Move::from_string("c4c3"), false);
    assert_eq!(false, is_game_over_by_repetition(&game_state));

    game_state.apply_move(&Move::from_string("a4a2"), false);
    assert_eq!(false, is_game_over_by_repetition(&game_state));

    game_state.apply_move(&Move::from_string("c3d4"), false);
    assert_eq!(false, is_game_over_by_repetition(&game_state));

    game_state.apply_move(&Move::from_string("a2c2"), false);
    assert_eq!(false, is_game_over_by_repetition(&game_state));

    game_state.apply_move(&Move::from_string("d4d5"), false);
    assert_eq!(false, is_game_over_by_repetition(&game_state));

    game_state.apply_move(&Move::from_string("c2g2"), false);
    assert_eq!(false, is_game_over_by_repetition(&game_state));

    game_state.apply_move(&Move::from_string("d5d6"), false);
    assert_eq!(false, is_game_over_by_repetition(&game_state));

    game_state.apply_move(&Move::from_string("g2c2"), false);
    assert_eq!(false, is_game_over_by_repetition(&game_state));

    game_state.apply_move(&Move::from_string("d6d5"), false);
    assert_eq!(false, is_game_over_by_repetition(&game_state));

    game_state.apply_move(&Move::from_string("c2c1"), false);
    assert_eq!(false, is_game_over_by_repetition(&game_state));

    game_state.apply_move(&Move::from_string("d5d4"), false);
    assert_eq!(false, is_game_over_by_repetition(&game_state));

    game_state.apply_move(&Move::from_string("c1c2"), false);
    assert_eq!(false, is_game_over_by_repetition(&game_state));

    game_state.apply_move(&Move::from_string("d4d5"), false);
    println!("{:?}", game_state);
    assert_eq!(true, is_game_over_by_repetition(&game_state));
  }

  #[test]
  fn test_game_over_threefold_repetition_2() {
    // This three-fold repetition was not understood during the game: https://lichess.org/oBjYp62P/white
    let fen = "r2q1b1r/1pp1pkpp/2n1p3/p2p4/3PnB2/2NQ1NP1/PPP1PP1P/R3K2R w KQ - 2 9";
    let mut game_state = GameState::from_string(fen);
    game_state.apply_move(&Move::from_string("c3e4"), false);
    game_state.apply_move(&Move::from_string("d5e4"), false);
    game_state.apply_move(&Move::from_string("f3g5"), false);
    game_state.apply_move(&Move::from_string("f7f6"), false);
    game_state.apply_move(&Move::from_string("g5e4"), false);
    game_state.apply_move(&Move::from_string("f6f7"), false);
    game_state.apply_move(&Move::from_string("e4g5"), false);
    game_state.apply_move(&Move::from_string("f7f6"), false);
    game_state.apply_move(&Move::from_string("g5h7"), false);
    game_state.apply_move(&Move::from_string("f6f7"), false);
    game_state.apply_move(&Move::from_string("h7g5"), false);
    game_state.apply_move(&Move::from_string("f7f6"), false);
    game_state.apply_move(&Move::from_string("g5e4"), false);
    game_state.apply_move(&Move::from_string("f6f7"), false);
    game_state.apply_move(&Move::from_string("e4g5"), false);
    game_state.apply_move(&Move::from_string("f7f6"), false);
    game_state.apply_move(&Move::from_string("g5h7"), false);
    assert_eq!(false, is_game_over_by_repetition(&game_state));
    game_state.apply_move(&Move::from_string("g5h7"), false);
    game_state.apply_move(&Move::from_string("f6f7"), false);
    assert_eq!(false, is_game_over_by_repetition(&game_state));
    game_state.apply_move(&Move::from_string("h7g5"), false);
    assert_eq!(true, is_game_over_by_repetition(&game_state));
  }
}
