use log::*;

// From our module
use super::super::cache::EngineCache;
use super::endgame::get_endgame_position_evaluation;
use super::helpers::generic::*;
use super::helpers::pawn::*;
use super::helpers::rook::*;
use super::middlegame::get_middlegame_position_evaluation;
use super::opening::get_opening_position_evaluation;
use crate::engine::Engine;

use crate::model::board_geometry::*;
use crate::model::game_state::*;
use crate::model::piece::*;

// Constants
const PAWN_ISLAND_FACTOR: f32 = 0.01;
const PASSED_PAWN_FACTOR: f32 = 0.2;
const PROTECTED_PASSED_PAWN_FACTOR: f32 = 0.6;
const PROTECTED_PAWN_FACTOR: f32 = 0.05;
const BACKWARDS_PAWN_FACTOR: f32 = 0.005;
const CONNECTED_ROOKS_FACTOR: f32 = 0.01;
const ROOK_FILE_FACTOR: f32 = 0.03;
const HANGING_FACTOR: f32 = 0.4;
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

  /*
  FIXME: These computations are slow
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
  * (get_backwards_pawns(game_state, Color::Black).count_ones() as f32
  - get_backwards_pawns(game_state, Color::White).count_ones() as f32);
  */

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

  for (i, piece) in game_state.board.pieces.white {
    let defenders = game_state.board.get_attackers(i, Color::White);
    let attackers = game_state.board.get_attackers(i, Color::Black);
    if defenders == 0 {
      score -= HANGING_PENALTY;
    }
    if attackers.count_ones() > defenders.count_ones()
      && game_state.board.side_to_play == Color::Black
    {
      score -= HANGING_FACTOR * Piece::material_value_from_type(piece);
    }

    // Check if we have some good positional stuff
    /*
    FIXME: This is slow
    if has_reachable_outpost(game_state, i as usize) {
      score += REACHABLE_OUTPOST_BONUS;
    }
    if occupies_reachable_outpost(game_state, i as usize) {
      score += OUTPOST_BONUS;
    }
    */
  }

  for (i, piece) in game_state.board.pieces.black {
    let defenders = game_state.board.get_attackers(i, Color::Black);
    let attackers = game_state.board.get_attackers(i, Color::White);
    if defenders == 0 {
      score += HANGING_PENALTY;
    }
    if attackers.count_ones() > defenders.count_ones()
      && game_state.board.side_to_play == Color::White
    {
      score += HANGING_FACTOR * Piece::material_value_from_type(piece);
    }

    // Check if we have some good positional stuff
    /*
    FIXME: This is slow
        if has_reachable_outpost(game_state, i as usize) {
          score -= REACHABLE_OUTPOST_BONUS;
        }
        if occupies_reachable_outpost(game_state, i as usize) {
          score -= OUTPOST_BONUS;
        }
        */
  }

  // Check on the material imbalance
  score += get_combined_material_score(game_state);

  // Return our score
  score
}

// Determine the game phrase and update it.
pub fn determine_game_phase(cache: &EngineCache, game_state: &GameState) {
  // Do not recalculate when we calculated already
  if cache.has_game_phase(&game_state.board) {
    return;
  }

  if game_state.move_count > 30 {
    cache.set_game_phase(&game_state.board, GamePhase::Endgame);
    return;
  }

  // Basic material count, disregarding pawns.
  let mut material_count: usize = 0;
  let mut development_index: usize = 0;

  material_count += game_state.board.pieces.queens().count_ones() as usize * 9;
  material_count += game_state.board.pieces.rooks().count_ones() as usize * 5;
  material_count += game_state.board.pieces.minors().count_ones() as usize * 3;

  development_index += ((game_state.board.pieces.white.minors()
    | game_state.board.pieces.white.queen)
    & BOARD_DOWN_EDGE)
    .count_ones() as usize;

  development_index += ((game_state.board.pieces.black.minors()
    | game_state.board.pieces.black.queen)
    & BOARD_UP_EDGE)
    .count_ones() as usize;

  if material_count < 17 {
    cache.set_game_phase(&game_state.board, GamePhase::Endgame);
  } else if development_index > 6 {
    cache.set_game_phase(&game_state.board, GamePhase::Opening);
  } else {
    cache.set_game_phase(&game_state.board, GamePhase::Middlegame);
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
pub fn is_game_over(cache: &EngineCache, game_state: &GameState) -> GameStatus {
  Engine::find_move_list(cache, &game_state.board);
  if cache.get_move_list(&game_state.board).is_empty() {
    match (
      game_state.board.side_to_play,
      game_state.board.checkers.count_ones(),
    ) {
      (_, 0) => {
        cache.set_status(game_state, GameStatus::Stalemate);
        cache.set_eval(&game_state.board, 0.0);
        return GameStatus::Stalemate;
      },
      (Color::Black, _) => {
        cache.set_status(game_state, GameStatus::WhiteWon);
        cache.set_eval(&game_state.board, 200.0);
        return GameStatus::WhiteWon;
      },
      (Color::White, _) => {
        cache.set_status(game_state, GameStatus::BlackWon);
        cache.set_eval(&game_state.board, -200.0);
        return GameStatus::BlackWon;
      },
    }
  }
  if game_state.ply >= 100 {
    debug!("100 Ply detected");
    cache.set_status(game_state, GameStatus::Draw);
    return GameStatus::Draw;
  }

  // 2 kings, or 1 king + knight or/bishop vs king is game over:
  if game_state.board.is_game_over_by_insufficient_material() {
    debug!("game over by insufficient material detected");
    cache.set_status(game_state, GameStatus::Draw);
    return GameStatus::Draw;
  }

  // Check the 3-fold repetitions
  if game_state.get_board_repetitions() >= 2 {
    debug!("3-fold repetition detected");
    cache.set_status(game_state, GameStatus::Draw);
    return GameStatus::Draw;
  }

  cache.set_status(game_state, GameStatus::Ongoing);
  return GameStatus::Ongoing;
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
pub fn evaluate_board(cache: &EngineCache, game_state: &GameState) -> f32 {
  if !cache.has_game_phase(&game_state.board) {
    determine_game_phase(cache, game_state);
  }
  let mut score = match cache.get_game_phase(&game_state.board) {
    GamePhase::Opening => get_opening_position_evaluation(game_state),
    GamePhase::Middlegame => get_middlegame_position_evaluation(game_state),
    GamePhase::Endgame => get_endgame_position_evaluation(game_state),
  };

  score = default_position_evaluation(game_state);
  cache.set_eval(&game_state.board, score);
  cache.set_status(game_state, GameStatus::Ongoing);
  score
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use crate::engine::cache::EngineCache;
  use crate::model::board::Board;
  use crate::model::moves::Move;
  use rand::Rng;
  use std::time::{Duration, Instant};

  use super::*;
  #[test]
  fn test_evaluate_board() {
    // This is a forced checkmate in 2:
    let cache = EngineCache::new();
    let fen = "1n4nr/5ppp/1N6/1P2p3/1P1k4/5P2/1p1NP1PP/R1B1KB1R w KQ - 0 35";
    let game_state = GameState::from_fen(fen);
    let evaluation = evaluate_board(&cache, &game_state);
    println!("Evaluation {evaluation}");
  }

  #[test]
  fn test_evaluate_board_checkmate_in_one() {
    // This is a forced checkmate in 1:
    let cache = EngineCache::new();
    let fen = "1n4nr/5ppp/1N6/1P2p3/1P6/4kP2/1B1NP1PP/R3KB1R w KQ - 1 36";
    let game_state = GameState::from_fen(fen);
    let evaluation = evaluate_board(&cache, &game_state);
    println!("Evaluation {evaluation}");
  }

  #[test]
  fn test_evaluate_board_checkmate() {
    // This is a "game over" position
    let cache = EngineCache::new();
    let fen = "1n4nr/5ppp/8/1P1Np3/1P6/4kP2/1B1NP1PP/R3KB1R b KQ - 2 37";
    let game_state = GameState::from_fen(fen);
    game_state.get_moves();
    let game_status = is_game_over(&cache, &game_state);
    assert_eq!(game_status, GameStatus::WhiteWon);
  }
  #[test]
  fn test_evaluate_board_hanging_queen() {
    // This should obviously be very bad for white:
    let cache = EngineCache::new();
    let fen = "rnbqkb1r/ppp1pppQ/5n2/3p4/3P4/8/PPP1PPPP/RNB1KBNR b KQkq - 0 3";
    let game_state = GameState::from_fen(fen);
    game_state.get_moves();
    let evaluation = evaluate_board(&cache, &game_state);
    println!("Evaluation : {evaluation}");
    assert!(evaluation < -((HANGING_FACTOR * QUEEN_VALUE) - 1.0));
  }

  #[test]
  fn test_evaluate_board_queen_standoff() {
    // This should obviously be okay because queen is defended and attacked by a queen.
    let cache = EngineCache::new();
    let fen = "rnb1kbnr/pppp1ppp/5q2/4p3/4P3/5Q2/PPPP1PPP/RNB1KBNR w KQkq - 2 3";
    let game_state = GameState::from_fen(fen);
    game_state.get_moves();
    let evaluation = evaluate_board(&cache, &game_state);
    println!("Evaluation: {}", evaluation);
    assert!(evaluation < 1.0);
    assert!(evaluation > -1.0);
  }

  #[test]
  fn test_evaluate_queen_down() {
    let cache = EngineCache::new();
    let fen = "rnbqk2r/pp3ppp/2pb1n2/3p4/B3P3/8/PPPP1PPP/RNB1K1NR w KQkq - 0 7";
    let game_state = GameState::from_fen(fen);
    game_state.get_moves();
    let evaluation = evaluate_board(&cache, &game_state);
    println!("Evaluation: {}", evaluation);
    assert!(evaluation < -7.0);
  }

  #[test]
  fn test_evaluate_board_losing() {
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
    let evaluation = evaluate_board(&cache, &game_state);
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
    use crate::model::board::Board;
    use std::time::{Duration, Instant};

    let cache = EngineCache::new();

    // Create a bunch of random boards
    const NUMBER_OF_BOARDS: usize = 1_000_000;
    let mut game_states: Vec<GameState> = Vec::with_capacity(NUMBER_OF_BOARDS);
    for _ in 0..NUMBER_OF_BOARDS {
      game_states.push(GameState::from_board(&Board::new_random()));
    }

    let mut rng = rand::thread_rng();
    let mut positions_evaluated = 0;
    let start_time = Instant::now();

    // Spin at it for 1 second
    while Instant::now() < (start_time + Duration::from_millis(1000)) {
      let _ = evaluate_board(&cache, &game_states[rng.gen_range(0..NUMBER_OF_BOARDS)]);
      positions_evaluated += 1;
    }

    // 1000 kNPS would be nice. Right now we are at a very low number LOL
    assert!(
      positions_evaluated > 1_000_000,
      "Number of NPS for evaluating positions using evaluate_board: {}",
      positions_evaluated
    );
  }

  #[test]
  fn position_bench_is_game_over_per_second() {
    use crate::model::board::Board;
    use std::time::{Duration, Instant};

    let cache = EngineCache::new();

    // Create a bunch of random boards
    const NUMBER_OF_BOARDS: usize = 1_000_000;
    let mut game_states: Vec<GameState> = Vec::with_capacity(NUMBER_OF_BOARDS);
    for _ in 0..NUMBER_OF_BOARDS {
      game_states.push(GameState::from_board(&Board::new_random()));
    }

    let mut rng = rand::thread_rng();
    let mut positions_evaluated = 0;
    let start_time = Instant::now();

    // Spin at it for 1 second
    while Instant::now() < (start_time + Duration::from_millis(1000)) {
      let _ = is_game_over(&cache, &game_states[rng.gen_range(0..NUMBER_OF_BOARDS)]);
      positions_evaluated += 1;
    }

    // 1000 kNPS would be nice. Right now we are at a very low number LOL
    assert!(
      positions_evaluated > 1_000_000,
      "Number of NPS for determining if it is game_over: {}",
      positions_evaluated
    );
  }

  #[test]
  fn position_bench_default_position_evaluation_per_second() {
    let cache = EngineCache::new();

    // Create a bunch of random boards
    const NUMBER_OF_BOARDS: usize = 1_000_000;
    let mut game_states: Vec<GameState> = Vec::with_capacity(NUMBER_OF_BOARDS);
    for _ in 0..NUMBER_OF_BOARDS {
      game_states.push(GameState::from_board(&Board::new_random()));
    }

    let mut rng = rand::thread_rng();
    let mut positions_evaluated = 0;
    let start_time = Instant::now();

    // Spin at it for 1 second
    while Instant::now() < (start_time + Duration::from_millis(1000)) {
      let _ = default_position_evaluation(&game_states[rng.gen_range(0..NUMBER_OF_BOARDS)]);
      positions_evaluated += 1;
    }

    // 1000 kNPS would be nice. Right now we are at a very low number LOL
    assert!(
      positions_evaluated > 1_000_000,
      "Number of NPS for evaluating positions using default_position_evaluation:: {}",
      positions_evaluated
    );
  }

  #[test]
  fn position_bench_get_material_score_per_second() {
    // Create a bunch of random boards
    const NUMBER_OF_BOARDS: usize = 100_000;
    let mut game_states: Vec<GameState> = Vec::with_capacity(NUMBER_OF_BOARDS);
    for _ in 0..NUMBER_OF_BOARDS {
      game_states.push(GameState::from_board(&Board::new_random()));
    }

    let mut rng = rand::thread_rng();
    let mut positions_evaluated = 0;
    let start_time = Instant::now();
    // Spin at it for 1 second
    while Instant::now() < (start_time + Duration::from_millis(1000)) {
      let _ = get_combined_material_score(&game_states[rng.gen_range(0..NUMBER_OF_BOARDS)]);
      positions_evaluated += 1;
    }

    // 1000 kNPS would be nice. Right now we are at a very low number LOL
    assert!(
      positions_evaluated > 1_000_000,
      "Number of NPS for evaluating positions using get_combined_material_score: {}",
      positions_evaluated
    );
  }
}
