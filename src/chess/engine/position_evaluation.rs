use log::*;

// From our module
use crate::chess::engine::development::*;
use crate::chess::engine::endgame::*;
use crate::chess::engine::eval_helpers::generic::mask_sum;
use crate::chess::engine::eval_helpers::pawn::*;
use crate::chess::engine::pawn_structure::*;
use crate::chess::engine::square_affinity::*;
use crate::chess::model::game_state::*;
use crate::chess::model::piece::*;

// Constants
const PIECE_AFFINITY_FACTOR: f32 = 0.1;
const PAWN_ISLAND_FACTOR: f32 = 0.2;
const PASSED_PAWN_FACTOR: f32 = 0.5;
const PROTECTED_PASSED_PAWN_FACTOR: f32 = 0.7;
const PROTECTED_PAWN_FACTOR: f32 = 0.15;
const CLOSENESS_TO_PROMOTION_PAWN_FACTOR: f32 = 0.1;
const BACKWARDS_PAWN_FACTOR: f32 = 0.11;
const DEVELOPMENT_FACTOR: f32 = 0.20;

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

/// Makes a material count of the position
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
pub fn get_material_score(game_state: &GameState) -> f32 {
  // Basic material count
  let mut score: f32 = 0.0;
  for i in 0..64 {
    match game_state.board.squares[i] {
      WHITE_QUEEN => score += 9.5,
      WHITE_ROOK => score += 5.0,
      WHITE_BISHOP => score += 3.05,
      WHITE_KNIGHT => score += 3.0,
      WHITE_PAWN => score += 1.0,
      BLACK_QUEEN => score -= 9.5,
      BLACK_ROOK => score -= 5.0,
      BLACK_BISHOP => score -= 3.05,
      BLACK_KNIGHT => score -= 3.0,
      BLACK_PAWN => score -= 1.0,
      _ => {},
    }
  }
  score
}

/// Evaluates a position and returns a score and if the game is over.
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
pub fn evaluate_position(game_state: &GameState) -> (f32, bool) {
  // println!("evaluate_position");
  // Check if we are checkmated or stalemated
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

  let mut score: f32 = 0.0;
  // Basic material count
  score += get_material_score(game_state);

  // Measure if we are developed.
  if game_state.game_phase.unwrap_or(GamePhase::Opening) == GamePhase::Opening {
    score += DEVELOPMENT_FACTOR * (get_development_score(game_state, Color::White) as f32)
      - (get_development_score(game_state, Color::Black) as f32);
  }

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

  if game_state.game_phase.unwrap_or(GamePhase::Opening) == GamePhase::Endgame {
    score += CLOSENESS_TO_PROMOTION_PAWN_FACTOR
      * (get_distance_left_for_closest_pawn_to_promotion(game_state, Color::Black) as f32
        - get_distance_left_for_closest_pawn_to_promotion(game_state, Color::White) as f32);
  }

  // Find the highest free piece, if any:
  let mut capture_gain = get_free_piece_value(game_state);
  if capture_gain == 0.0 {
    capture_gain = find_most_interesting_capture(game_state);
  }
  score -= capture_gain;

  // This is an expensive calculation, for now we skip this.
  // Compare the mobility of both sides. Give +1 if one side has 15 available moves.
  // score +=
  //  (self.get_white_moves().len() as isize - self.get_black_moves().len() as isize) as f32 / 15.0;

  // Get a pressure score, if one side has more attackers than defenders on a square, they get bonus points
  let white_heatmap = game_state.get_heatmap(Color::White, false);
  let black_heatmap = game_state.get_heatmap(Color::Black, false);

  for i in 0..64 {
    score += HEATMAP_SCORES[i] * white_heatmap[i] as f32;
    score -= HEATMAP_SCORES[i] * black_heatmap[i] as f32;
  }

  // Are we in an endgame:
  if game_state.game_phase.unwrap_or(GamePhase::Opening) == GamePhase::Endgame {
    score += get_endgame_score(game_state);
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

  (score, false)
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
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
}
