use log::*;

// From our module
use super::eval_helpers::rook::*;
use crate::chess::engine::endgame::*;
use crate::chess::engine::eval_helpers::generic::*;
use crate::chess::engine::eval_helpers::pawn::*;
use crate::chess::engine::middlegame::*;
use crate::chess::engine::opening::*;
use crate::chess::engine::square_affinity::*;
use crate::chess::model::game_state::*;
use crate::chess::model::piece::*;

// Constants
const PIECE_AFFINITY_FACTOR: f32 = 0.005;
const PAWN_ISLAND_FACTOR: f32 = 0.02;
const PASSED_PAWN_FACTOR: f32 = 0.2;
const PROTECTED_PASSED_PAWN_FACTOR: f32 = 0.6;
const PROTECTED_PAWN_FACTOR: f32 = 0.05;
const CLOSENESS_TO_PROMOTION_PAWN_FACTOR: f32 = 0.1;
const BACKWARDS_PAWN_FACTOR: f32 = 0.01;
const CONNECTED_ROOKS_FACTOR: f32 = 0.02;
const ROOK_FILE_FACTOR: f32 = 0.03;
const HANGING_FACTOR: f32 = 0.1;
const REACHABLE_OUTPOST: f32 = 0.08;

// Shows "interesting" squares to control on the board
// Giving them a score
pub const HEATMAP_SCORES: [f32; 64] = [
  0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, // 1st row
  0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, // 2nd row
  0.01, 0.01, 0.03, 0.03, 0.03, 0.03, 0.01, 0.01, // 3rd row
  0.01, 0.01, 0.03, 0.04, 0.04, 0.03, 0.01, 0.01, // 4th row
  0.01, 0.01, 0.03, 0.04, 0.04, 0.03, 0.01, 0.01, // 5th row
  0.01, 0.01, 0.03, 0.03, 0.03, 0.03, 0.01, 0.01, // 6th row
  0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, // 7th row
  0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, // 8th row
];

fn get_free_piece_value(game_state: &GameState) -> f32 {
  let mut highest_free_piece_value: f32 = 0.0;
  //println!("Side to play: {}", game_state.side_to_play);
  let op_color = Color::opposite(game_state.side_to_play);
  //println!("Opposite side : {}", op_color);
  let op_heatmap = game_state.get_heatmap(op_color, false);
  let ss_heatmap = game_state.get_heatmap(game_state.side_to_play, false);

  for i in 0..64 {
    if game_state.board.has_piece_with_color(i, op_color) == true {
      if ss_heatmap[i as usize] > 0
        && op_heatmap[i as usize] == 0
        && highest_free_piece_value.abs()
          < Piece::material_value_from_u8(game_state.board.squares[i as usize]).abs()
      {
        highest_free_piece_value =
          Piece::material_value_from_u8(game_state.board.squares[i as usize]);
      }
    }
  }
  //println!("Free piece value: {highest_free_piece_value}");
  highest_free_piece_value
}

fn find_most_interesting_capture(game_state: &GameState) -> f32 {
  let mut highest_value_gain: f32 = 0.0;
  let op_color = Color::opposite(game_state.side_to_play);
  //println!("Opposite side : {}", op_color);
  let (op_heatmap, op_sources) = game_state.get_heatmap_with_sources(op_color, false);
  let (ss_heatmap, ss_sources) =
    game_state.get_heatmap_with_sources(game_state.side_to_play, false);

  for i in 0..64 {
    if game_state.board.has_piece_with_color(i, op_color) == false {
      continue;
    }
    if ss_heatmap[i as usize] == 0 {
      continue;
    }

    let target_value = Piece::material_value_from_u8(game_state.board.squares[i as usize]).abs();
    // Same number or less attackers than defenders.
    // Check if we have a lesser value piece that can capture a higher value piece.
    if ss_heatmap[i as usize] <= op_heatmap[i as usize] {
      let mut min_value_attacker = 200.0;
      for j in 0..64 {
        if ((1 << j) & ss_sources[i as usize] != 0)
          && min_value_attacker
            > Piece::material_value_from_u8(game_state.board.squares[j as usize]).abs()
        {
          min_value_attacker =
            Piece::material_value_from_u8(game_state.board.squares[j as usize]).abs();
        }
      }
      if (min_value_attacker < target_value)
        && (target_value - min_value_attacker) > highest_value_gain
      {
        highest_value_gain = target_value - min_value_attacker;
      }
    } else if ss_heatmap[i as usize] >= op_heatmap[i as usize] {
      // Let's just calculate the gain if we chop everything (in ascending piece value order)
      let mut defender_values: Vec<f32> = Vec::new();
      defender_values.push(target_value);
      let mut attacker_values: Vec<f32> = Vec::new();
      for j in 0..64 {
        if (1 << j) & ss_sources[i as usize] != 0 {
          attacker_values
            .push(Piece::material_value_from_u8(game_state.board.squares[j as usize]).abs());
        }
        if (1 << j) & op_sources[i as usize] != 0 {
          defender_values
            .push(Piece::material_value_from_u8(game_state.board.squares[j as usize]).abs());
        }
      }
      defender_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
      attacker_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

      for chop_stop in 1..attacker_values.len() {
        let mut defender_sum = target_value;
        let mut attacker_sum = 0.0;
        for j in 1..=chop_stop {
          attacker_sum += attacker_values[j];
          if j > 1 {
            defender_sum += attacker_values[j - 1];
          }
        }
        if (attacker_sum < defender_sum) && (defender_sum - attacker_sum) > highest_value_gain {
          highest_value_gain = defender_sum - attacker_sum;
        }
      }
    }
  }
  //println!("Highest value gain: {highest_value_gain}");
  if game_state.side_to_play == Color::White {
    highest_value_gain = highest_value_gain * -1.0;
  }
  highest_value_gain
}

/// Default way to look at a position if we are not in a special situation.
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
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
    * (mask_sum(get_backwards_pawns(&game_state, Color::Black)) as f32
      - mask_sum(get_backwards_pawns(&game_state, Color::White)) as f32);

  // Evaluate the quality of our rooks:
  if are_rooks_connected(game_state, Color::White) {
    score += CONNECTED_ROOKS_FACTOR;
  }
  if are_rooks_connected(game_state, Color::Black) {
    score -= CONNECTED_ROOKS_FACTOR;
  }

  score += ROOK_FILE_FACTOR
    * (get_rooks_file_score(&game_state, Color::Black)
      - get_rooks_file_score(&game_state, Color::White));

  if game_state.game_phase.unwrap_or(GamePhase::Opening) == GamePhase::Endgame {
    score += CLOSENESS_TO_PROMOTION_PAWN_FACTOR
      * (get_distance_left_for_closest_pawn_to_promotion(game_state, Color::Black) as f32
        - get_distance_left_for_closest_pawn_to_promotion(game_state, Color::White) as f32);
  }

  // Find the highest free piece, if any:
  /*
  let mut capture_gain = get_free_piece_value(game_state);
  if capture_gain == 0.0 {
    // Divide the capture gain by 2, so it stays more interesting to actually capture than to just have a capture possibility.
    capture_gain = find_most_interesting_capture(game_state) / 2.0;
  }
  score -= capture_gain;
  */

  /*

  for i in 0..64 {
    if is_hanging(game_state, i) == true {
      score += HANGING_FACTOR * Piece::material_value_from_u8(game_state.board.squares[i]);
    }
    if has_reachable_outpost(game_state, i) == true {
      score +=
        REACHABLE_OUTPOST * Color::score_factor(Piece::color_from_u8(game_state.board.squares[i]));
    }
    if occupies_reachable_outpost(game_state, i) == true {
      score += 1.8 * Color::score_factor(Piece::color_from_u8(game_state.board.squares[i]));
    }
  } */

  // Get a pressure score, if one side has more attackers than defenders on a square, they get bonus points
  let white_heatmap = game_state.get_heatmap(Color::White, false);
  let black_heatmap = game_state.get_heatmap(Color::Black, false);

  for i in 0..64 {
    score += HEATMAP_SCORES[i] * white_heatmap[i] as f32;
    score -= HEATMAP_SCORES[i] * black_heatmap[i] as f32;
  }

  // Piece affinity offsets, do not apply this in the endgame
  if game_state.game_phase.unwrap_or(GamePhase::Endgame) != GamePhase::Endgame {
    for i in 0..64 {
      match game_state.board.squares[i] {
        WHITE_KING => score += PIECE_AFFINITY_FACTOR * WHITE_KING_SQUARE_AFFINITY[i] as f32,
        WHITE_QUEEN => score += PIECE_AFFINITY_FACTOR * QUEEN_SQUARE_AFFINITY[i] as f32,
        WHITE_ROOK => score += PIECE_AFFINITY_FACTOR * WHITE_ROOK_SQUARE_AFFINITY[i] as f32,
        WHITE_BISHOP => score += PIECE_AFFINITY_FACTOR * WHITE_BISHOP_SQUARE_AFFINITY[i] as f32,
        WHITE_KNIGHT => score += PIECE_AFFINITY_FACTOR * KNIGHT_SQUARE_AFFINITY[i] as f32,
        WHITE_PAWN => score += PIECE_AFFINITY_FACTOR * WHITE_PAWN_SQUARE_AFFINITY[i] as f32,
        BLACK_KING => score -= PIECE_AFFINITY_FACTOR * BLACK_KING_SQUARE_AFFINITY[i] as f32,
        BLACK_QUEEN => score -= PIECE_AFFINITY_FACTOR * QUEEN_SQUARE_AFFINITY[i] as f32,
        BLACK_ROOK => score -= PIECE_AFFINITY_FACTOR * BLACK_ROOK_SQUARE_AFFINITY[i] as f32,
        BLACK_BISHOP => score -= PIECE_AFFINITY_FACTOR * BLACK_BISHOP_SQUARE_AFFINITY[i] as f32,
        BLACK_KNIGHT => score -= PIECE_AFFINITY_FACTOR * KNIGHT_SQUARE_AFFINITY[i] as f32,
        BLACK_PAWN => score -= PIECE_AFFINITY_FACTOR * BLACK_PAWN_SQUARE_AFFINITY[i] as f32,
        _ => {},
      }
    }
  }
  // Return our score
  score
}

/// Determines if a position is a game over due to insufficient material or not
///
/// # Arguments
///
/// * `game_state` - A GameState object reference representing a position, side to play, etc.
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
  return true;
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
  return repetition_count >= 2;
}

/// Evaluates a position and  tells if it seems to be game over or not
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
pub fn is_game_over(game_state: &GameState) -> (f32, bool) {
  if game_state.available_moves_computed == false {
    warn!("Evaluating a position without move list computed, cannot determine if it is a game over position.");
  }
  if game_state.available_moves_computed == true && game_state.move_list.len() == 0 {
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
  if is_game_over_by_insufficient_material(game_state) == true {
    debug!("game over by insufficient material detected");
    return (0.0, true);
  }

  // Check the 3-fold repetitions
  if is_game_over_by_repetition(game_state) == true {
    debug!("3-fold repetition detected");
    return (0.0, true);
  }

  return (0.0, false);
}

/// Evaluates a position and returns a score and if the game is over.
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
pub fn evaluate_position(game_state: &GameState) -> (f32, bool) {
  // Check if the evaluation is due to a game over:
  let (mut score, game_over) = is_game_over(game_state);
  if game_over == true {
    return (score, game_over);
  }

  if game_state.game_phase.is_some() {
    match game_state.game_phase.unwrap() {
      GamePhase::Opening => score = get_opening_position_evaluation(&game_state),
      GamePhase::Middlegame => score = get_middlegame_position_evaluation(&game_state),
      GamePhase::Endgame => score = get_endgame_position_evaluation(&game_state),
    }
  } else {
    warn!("Evaluating a position in an unknown game phase");
    score = default_position_evaluation(&game_state);
  }

  (score, false)
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use crate::chess::model::board::Move;
  use crate::chess::model::game_state;

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
    // This should obviously be very bad for black:
    let fen = "rnbqkb1r/ppp1pppQ/5n2/3p4/3P4/8/PPP1PPPP/RNB1KBNR b KQkq - 0 3";
    let mut game_state = GameState::from_string(fen);
    game_state.get_moves();
    let (evaluation, game_over) = evaluate_position(&game_state);
    println!("Evaluation : {evaluation} - Game Over: {game_over}");
    assert_eq!(false, game_over);
    assert!(evaluation < -8.0);
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
