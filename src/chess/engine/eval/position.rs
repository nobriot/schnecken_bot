use log::*;

// From our module
use super::super::cache::EngineCache;
use super::endgame::get_endgame_position_evaluation;
use super::helpers::bishop::*;
use super::helpers::generic::*;
use super::helpers::knight::*;
use super::helpers::pawn::*;
use super::helpers::rook::*;
use super::middlegame::get_middlegame_position_evaluation;
use super::opening::get_opening_position_evaluation;

// From another crate
use crate::chess::model::board_mask::*;
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
const HANGING_FACTOR: f32 = 0.5;
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
        && (game_state.board.side_to_play
          == Color::opposite(Piece::color_from_u8(game_state.board.squares[i])))
      {
        score -= HANGING_FACTOR * Piece::material_value_from_u8(game_state.board.squares[i]);
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
    let value = bishop_attack_with_pins(game_state, i);
    if value.abs() > 3.1 {
      score += score_factor * (value - 3.1) / 1.9;
    } else {
      let value = bishop_attack(game_state, i);
      if value.abs() > 3.1 {
        score += score_factor * (value - 3.1) / 2.3;
      }
    }
  }

  // Basic material count
  let white_material = get_material_score(game_state, Color::White);
  let black_material = get_material_score(game_state, Color::Black);
  score += white_material - black_material;

  // Return our score
  score
}

// Determine the game phrase and update it.
pub fn determine_game_phase(cache: &EngineCache, game_state: &GameState) {
  // Do not recalculate when we calculated already
  if cache.has_game_phase(&game_state.board.hash) {
    return;
  }

  if game_state.move_count > 30 {
    cache.set_game_phase(game_state.board.hash, GamePhase::Endgame);
    return;
  }

  // Basic material count, disregarding pawns.
  let mut material_count: usize = 0;
  let mut development_index: usize = 0;
  for i in 0..64 {
    match game_state.board.squares[i] {
      WHITE_QUEEN | BLACK_QUEEN => material_count += 9,
      WHITE_ROOK | BLACK_ROOK => material_count += 5,
      WHITE_BISHOP | BLACK_BISHOP => material_count += 3,
      WHITE_KNIGHT | BLACK_KNIGHT => material_count += 3,
      _ => {},
    }
  }
  for i in 0..8 {
    match game_state.board.squares[i] {
      WHITE_QUEEN | WHITE_BISHOP | WHITE_KNIGHT => development_index += 1,
      _ => {},
    }
  }
  for i in 56..64 {
    match game_state.board.squares[i] {
      BLACK_QUEEN | BLACK_BISHOP | BLACK_KNIGHT => development_index += 1,
      _ => {},
    }
  }

  if material_count < 17 {
    cache.set_game_phase(game_state.board.hash, GamePhase::Endgame);
  } else if development_index > 2 {
    cache.set_game_phase(game_state.board.hash, GamePhase::Opening);
  } else {
    cache.set_game_phase(game_state.board.hash, GamePhase::Middlegame);
  }
}

/// Evaluates a position and  tells if it seems to be game over or not
///
/// ### Arguments
///
/// * `cache` -      EngineCache to use to save the results
/// * `game_state` - A GameState object representing a position, side to play, etc.
///
/// ### Returns
///
/// * bool -> True if it is a game over (checkmate, stalemate, repetitions, etc.)
/// All cases included.
/// false if the game is ongoing and must be evaluated manually
///
///
pub fn is_game_over(cache: &EngineCache, game_state: &GameState) -> bool {
  if !cache.has_move_list(&game_state.board.hash) {
    cache.set_move_list(game_state.board.hash, &game_state.get_moves());
  }
  if cache.get_move_list(&game_state.board.hash).is_empty() {
    match (game_state.board.side_to_play, game_state.checks) {
      (_, 0) => {
        cache.set_status(game_state.board.hash, GameStatus::Draw);
        cache.set_eval(game_state.board.hash, 0.0);
        return true;
      },
      (Color::Black, _) => {
        cache.set_status(game_state.board.hash, GameStatus::WhiteWon);
        cache.set_eval(game_state.board.hash, 200.0);
        return true;
      },
      (Color::White, _) => {
        cache.set_status(game_state.board.hash, GameStatus::BlackWon);
        cache.set_eval(game_state.board.hash, -200.0);
        return true;
      },
    }
  }
  if game_state.ply >= 100 {
    debug!("100 Ply detected");
    cache.set_status(game_state.board.hash, GameStatus::Draw);
    cache.set_eval(game_state.board.hash, 0.0);
    return true;
  }

  // 2 kings, or 1 king + knight or/bishop vs king is game over:
  if game_state.board.is_game_over_by_insufficient_material() {
    debug!("game over by insufficient material detected");
    cache.set_status(game_state.board.hash, GameStatus::Draw);
    cache.set_eval(game_state.board.hash, 0.0);
    return true;
  }

  // Check the 3-fold repetitions
  if game_state.get_board_repetitions() >= 2 {
    debug!("3-fold repetition detected");
    return true;
  }

  cache.set_status(game_state.board.hash, GameStatus::Ongoing);
  return false;
}

/// Evaluates a position and returns a score and if the game is over.
///
/// ### Arguments
///
/// * `cache` -      EngineCache to use to store calculations
/// * `game_state` - A GameState object representing a position, side to play, etc.
///
/// ### Returns
///
/// Score assigned to the position.
pub fn evaluate_position(cache: &EngineCache, game_state: &GameState) -> (f32, bool) {
  // Check if the evaluation is due to a game over:
  if is_game_over(cache, game_state) {
    if cache.has_eval(&game_state.board.hash) {
      return (cache.get_eval(&game_state.board.hash), true);
    } else {
      return (0.0, true);
    }
  }

  let mut score = 0.0;
  if !cache.has_game_phase(&game_state.board.hash) {
    determine_game_phase(cache, game_state);
  }
  match cache.get_game_phase(&game_state.board.hash) {
    GamePhase::Opening => score = get_opening_position_evaluation(game_state),
    GamePhase::Middlegame => score = get_middlegame_position_evaluation(game_state),
    GamePhase::Endgame => score = get_endgame_position_evaluation(game_state),
  }

  //score = default_position_evaluation(game_state);
  cache.set_eval(game_state.board.hash, score);
  cache.set_status(game_state.board.hash, GameStatus::Ongoing);
  (score, false)
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use crate::chess::engine::cache::EngineCache;
  use crate::chess::model::moves::Move;

  use super::*;
  #[test]
  fn test_evaluate_position() {
    // This is a forced checkmate in 2:
    let cache = EngineCache::new();
    let fen = "1n4nr/5ppp/1N6/1P2p3/1P1k4/5P2/1p1NP1PP/R1B1KB1R w KQ - 0 35";
    let game_state = GameState::from_fen(fen);
    let (evaluation, game_over) = evaluate_position(&cache, &game_state);
    assert_eq!(false, game_over);
    println!("Evaluation {evaluation}");
  }

  #[test]
  fn test_evaluate_position_checkmate_in_one() {
    // This is a forced checkmate in 1:
    let cache = EngineCache::new();
    let fen = "1n4nr/5ppp/1N6/1P2p3/1P6/4kP2/1B1NP1PP/R3KB1R w KQ - 1 36";
    let game_state = GameState::from_fen(fen);
    let (evaluation, game_over) = evaluate_position(&cache, &game_state);
    assert_eq!(false, game_over);
    println!("Evaluation {evaluation}");
  }

  #[test]
  fn test_evaluate_position_checkmate() {
    // This is a "game over" position
    let cache = EngineCache::new();
    let fen = "1n4nr/5ppp/8/1P1Np3/1P6/4kP2/1B1NP1PP/R3KB1R b KQ - 2 37";
    let game_state = GameState::from_fen(fen);
    game_state.get_moves();
    let (evaluation, game_over) = evaluate_position(&cache, &game_state);
    assert_eq!(true, game_over);
    assert_eq!(200.0, evaluation);
  }
  #[test]
  fn test_evaluate_position_hanging_queen() {
    // This should obviously be very bad for white:
    let cache = EngineCache::new();
    let fen = "rnbqkb1r/ppp1pppQ/5n2/3p4/3P4/8/PPP1PPPP/RNB1KBNR b KQkq - 0 3";
    let game_state = GameState::from_fen(fen);
    game_state.get_moves();
    let (evaluation, game_over) = evaluate_position(&cache, &game_state);
    println!("Evaluation : {evaluation} - Game Over: {game_over}");
    assert_eq!(false, game_over);
    assert!(evaluation < -3.0);
  }

  #[test]
  fn test_evaluate_position_queen_standoff() {
    // This should obviously be okay because queen is defended and attacked by a queen.
    let cache = EngineCache::new();
    let fen = "rnb1kbnr/pppp1ppp/5q2/4p3/4P3/5Q2/PPPP1PPP/RNB1KBNR w KQkq - 2 3";
    let game_state = GameState::from_fen(fen);
    game_state.get_moves();
    let (evaluation, game_over) = evaluate_position(&cache, &game_state);
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
    let cache = EngineCache::new();
    let fen = "Qn2q2r/2p2pb1/p2k1n1p/5Bp1/8/2NP4/PPPB1PPP/R4RK1 b - - 0 15";
    let game_state = GameState::from_fen(fen);
    game_state.get_moves();
    let (evaluation, game_over) = evaluate_position(&cache, &game_state);
    assert_eq!(false, game_over);
    println!("Evaluation: {}", evaluation);
    assert!(evaluation > 7.0);
  }

  #[test]
  fn test_game_over_threefold_repetition_1() {
    let fen = "8/8/8/5P2/Q1k2KP1/8/p7/8 b - - 1 87";
    let mut game_state = GameState::from_fen(fen);
    assert_eq!(0, game_state.get_board_repetitions());

    game_state.apply_move(&Move::from_string("c4c3"));
    assert_eq!(0, game_state.get_board_repetitions());

    game_state.apply_move(&Move::from_string("a4a2"));
    assert_eq!(0, game_state.get_board_repetitions());

    game_state.apply_move(&Move::from_string("c3d4"));
    assert_eq!(0, game_state.get_board_repetitions());

    game_state.apply_move(&Move::from_string("a2c2"));
    assert_eq!(0, game_state.get_board_repetitions());

    game_state.apply_move(&Move::from_string("d4d5"));
    assert_eq!(0, game_state.get_board_repetitions());

    game_state.apply_move(&Move::from_string("c2g2"));
    assert_eq!(0, game_state.get_board_repetitions());

    game_state.apply_move(&Move::from_string("d5d6"));
    assert_eq!(0, game_state.get_board_repetitions());

    game_state.apply_move(&Move::from_string("g2c2"));
    assert_eq!(0, game_state.get_board_repetitions());

    game_state.apply_move(&Move::from_string("d6d5"));
    assert_eq!(1, game_state.get_board_repetitions());

    game_state.apply_move(&Move::from_string("c2c1"));
    assert_eq!(0, game_state.get_board_repetitions());

    game_state.apply_move(&Move::from_string("d5d4"));
    assert_eq!(0, game_state.get_board_repetitions());

    game_state.apply_move(&Move::from_string("c1c2"));
    assert_eq!(1, game_state.get_board_repetitions());

    game_state.apply_move(&Move::from_string("d4d5"));
    println!("{:?}", game_state);
    assert_eq!(2, game_state.get_board_repetitions());
  }

  #[test]
  fn test_game_over_threefold_repetition_2() {
    // This three-fold repetition was not understood during the game: https://lichess.org/oBjYp62P/white
    let fen = "r2q1b1r/1pp1pkpp/2n1p3/p2p4/3PnB2/2NQ1NP1/PPP1PP1P/R3K2R w KQ - 2 9";
    let mut game_state = GameState::from_fen(fen);
    game_state.apply_move(&Move::from_string("c3e4"));
    game_state.apply_move(&Move::from_string("d5e4"));
    game_state.apply_move(&Move::from_string("f3g5"));
    game_state.apply_move(&Move::from_string("f7f6"));
    game_state.apply_move(&Move::from_string("g5e4"));
    game_state.apply_move(&Move::from_string("f6f7"));
    game_state.apply_move(&Move::from_string("e4g5"));
    game_state.apply_move(&Move::from_string("f7f6"));
    game_state.apply_move(&Move::from_string("g5h7"));
    game_state.apply_move(&Move::from_string("f6f7"));
    game_state.apply_move(&Move::from_string("h7g5"));
    game_state.apply_move(&Move::from_string("f7f6"));
    game_state.apply_move(&Move::from_string("g5e4"));
    game_state.apply_move(&Move::from_string("f6f7"));
    game_state.apply_move(&Move::from_string("e4g5"));
    game_state.apply_move(&Move::from_string("f7f6"));
    game_state.apply_move(&Move::from_string("g5h7"));
    assert_eq!(1, game_state.get_board_repetitions());
    game_state.apply_move(&Move::from_string("g5h7"));
    game_state.apply_move(&Move::from_string("f6f7"));
    assert_eq!(1, game_state.get_board_repetitions());
    game_state.apply_move(&Move::from_string("h7g5"));
    assert_eq!(2, game_state.get_board_repetitions());
  }

  #[test]
  fn position_bench_evaluations_per_second() {
    use rand::Rng;
    use std::time::{Duration, Instant};

    let cache = EngineCache::new();
    let fens = [
      "8/P7/4kN2/4P3/1K3P2/4P3/8/8 w - - 7 76",
      "r2q1b1r/1pp1pkpp/2n1p3/p2p4/3PnB2/2NQ1NP1/PPP1PP1P/R3K2R w KQ - 2 9",
      "8/8/8/5P2/Q1k2KP1/8/p7/8 b - - 1 87",
      "rn1qkb1r/1bp1pppp/p2p1n2/1p6/3PP3/4B1P1/PPPN1PBP/R2QK1NR b KQkq - 5 6",
      "rnbqk1nr/ppp2ppp/8/3pp3/B2bP3/8/P1PP1PPP/R3K1NR b - - 0 1",
      "r2qk2r/2pb1ppp/3bpn2/p7/2BP4/2N2Q2/PP3PPP/R1B2RK1 w kq - 0 13",
      "r2q1rk1/p2b1ppp/2Pbpn2/8/2B5/2N2Q2/PP3PPP/R1B2RK1 b - - 0 14",
      "5k2/P7/2p5/1p6/3P2NR/1p2p3/1P4q1/1K6 w - - 0 53",
      "r2qk2r/p1pb1ppp/3bpn2/8/2BP4/2N2Q2/PP3PPP/R1B2RK1 b kq - 2 12",
      "5rk1/3b1p2/1r3p1p/p1pPp3/8/1P6/P3BPPP/R1R3K1 w - c6 0 23",
      "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43",
      "8/8/8/8/2nN4/1q6/ppP1NPPP/1k2K2R w K - 0 1",
    ];

    let mut positions_evaluated = 0;
    let start_time = Instant::now();

    // Spin at it for 1 second
    while Instant::now() < (start_time + Duration::from_millis(1000)) {
      let i = rand::thread_rng().gen_range(0..fens.len());
      let game_state = GameState::from_fen(fens[i]);

      // FIXME: Now evaluations are all cached, the number probably does not make sense as it will keep skipping.
      let _ = evaluate_position(&cache, &game_state);
      positions_evaluated += 1;
    }

    // 1000 kNPS would be nice. Right now we are at a very low number LOL
    assert!(
      positions_evaluated > 1_000_000,
      "Number of NPS for evaluating positions: {}",
      positions_evaluated
    );
  }
}
