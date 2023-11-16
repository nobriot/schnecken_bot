// From our module
use super::endgame::get_endgame_position_evaluation;
use super::helpers::bishop::get_bishop_victims;
use super::helpers::generic::*;
use super::helpers::knight::get_knight_victims;
use super::helpers::pawn::*;
use super::helpers::rook::*;
use super::middlegame::get_middlegame_position_evaluation;
use super::opening::get_opening_position_evaluation;
use crate::engine::cache::engine_cache::EngineCache;
use crate::engine::Engine;
use crate::model::board::Board;
use crate::model::board_geometry::*;
use crate::model::board_mask::CountFewOnes;
use crate::model::game_state::*;
use crate::model::piece::*;

// Constants
const PAWN_ISLAND_FACTOR: f32 = 0.05;
const CONNECTED_ROOKS_FACTOR: f32 = 0.03;
const ROOK_FILE_FACTOR: f32 = 0.06;
const HANGING_FACTOR: f32 = 0.4;
const HANGING_PENALTY: f32 = 0.15;
const PIN_PENALTY: f32 = 0.25;

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

  let mut white_pieces =
    game_state.board.pieces.white.minors() & game_state.board.pieces.white.majors();
  while white_pieces != 0 {
    let i = white_pieces.trailing_zeros() as u8;
    let defenders = game_state.board.get_attackers(i, Color::White);
    let attackers = game_state.board.get_attackers(i, Color::Black);

    if defenders == 0 {
      score -= HANGING_PENALTY;
    }
    if attackers.count_ones() > defenders.count_ones()
      && game_state.board.side_to_play == Color::Black
    {
      // Lowest value of any piece is 3.0.
      score -= HANGING_FACTOR * 3.0;
    }
    white_pieces &= white_pieces - 1;
  }
  /*
  // Check if we have some good positional stuff
  if has_reachable_outpost(game_state, i as usize) {
    score += REACHABLE_OUTPOST_BONUS;
  }
  if occupies_reachable_outpost(game_state, i as usize) {
    score += OUTPOST_BONUS;
  }
  */

  let mut black_pieces =
    game_state.board.pieces.black.minors() & game_state.board.pieces.black.majors();
  while black_pieces != 0 {
    let i = black_pieces.trailing_zeros() as u8;
    let defenders = game_state.board.get_attackers(i, Color::Black);
    let attackers = game_state.board.get_attackers(i, Color::White);

    if defenders == 0 {
      score -= HANGING_PENALTY;
    }
    if attackers.count_ones() > defenders.count_ones()
      && game_state.board.side_to_play == Color::White
    {
      // Lowest value of any piece is 3.0.
      score += HANGING_FACTOR * 3.0;
    }
    black_pieces &= black_pieces - 1;
  }

  // Look for pawns attacking pieces, or forking
  score += get_pawn_victims(game_state, Color::White) as f32;
  score -= get_pawn_victims(game_state, Color::Black) as f32;

  // Look for knight spans
  score += 0.5 * get_knight_victims(game_state, Color::White) as f32;
  score -= 0.5 * get_knight_victims(game_state, Color::Black) as f32;

  // Look for bishop tricks
  score += 0.5 * get_bishop_victims(game_state, Color::White) as f32;
  score -= 0.5 * get_bishop_victims(game_state, Color::Black) as f32;

  // Look for rook attacks
  score += 0.3 * get_rook_victims(game_state, Color::White) as f32;
  score -= 0.3 * get_rook_victims(game_state, Color::Black) as f32;

  /*
  // Check if we have some good positional stuff
  if has_reachable_outpost(game_state, i as usize) {
    score -= REACHABLE_OUTPOST_BONUS;
  }
  if occupies_reachable_outpost(game_state, i as usize) {
    score -= OUTPOST_BONUS;
  }
  */

  // Pinned pieces is never confortable
  if game_state.board.get_pins_rays(Color::White) != 0 {
    score -= PIN_PENALTY;
  }
  if game_state.board.get_pins_rays(Color::Black) != 0 {
    score += PIN_PENALTY;
  }

  // Check on the material imbalance
  score += get_combined_material_score(game_state);

  // Return our score
  score
}

// Determine the game phrase and update it.
pub fn determine_game_phase(game_state: &GameState) -> GamePhase {
  if game_state.move_count > 30 {
    return GamePhase::Endgame;
  }

  // Basic material count, disregarding pawns.
  let mut material_count = 0;
  let mut development_index = 0;

  material_count += game_state.board.pieces.queens().count_few_ones() * 9;
  material_count += game_state.board.pieces.rooks().count_few_ones() * 5;
  material_count += game_state.board.pieces.minors().count_ones() * 3;

  development_index += ((game_state.board.pieces.white.minors()
    | game_state.board.pieces.white.queen)
    & BOARD_DOWN_EDGE)
    .count_ones();

  development_index += ((game_state.board.pieces.black.minors()
    | game_state.board.pieces.black.queen)
    & BOARD_UP_EDGE)
    .count_ones();

  if material_count < 20 {
    return GamePhase::Endgame;
  } else if development_index > 6 {
    return GamePhase::Opening;
  } else {
    return GamePhase::Middlegame;
  }
}

/// Looks at a board and verifies if the game is over.
/// Does not count game specific sequences like 3-fold repetitions and 100 ply.
///
/// ### Arguments
///
/// * `cache` -      EngineCache to use to save the results
/// * `board` -      Board reference to look at.
///
/// ### Returns
///
/// * GameStatus indicating if the game is ongoing or not.
///
pub fn is_game_over(cache: &EngineCache, board: &Board) -> GameStatus {
  Engine::find_move_list(cache, board);
  if cache.get_move_list(board).unwrap().is_empty() {
    match (board.side_to_play, board.checkers.count_ones()) {
      (_, 0) => {
        return GameStatus::Stalemate;
      },
      (Color::Black, _) => {
        return GameStatus::WhiteWon;
      },
      (Color::White, _) => {
        return GameStatus::BlackWon;
      },
    }
  }

  // 2 kings, or 1 king + knight or/bishop vs king is game over:
  if board.is_game_over_by_insufficient_material() {
    //debug!("game over by insufficient material detected");
    return GameStatus::Draw;
  }

  return GameStatus::Ongoing;
}

/// Returns evaluation scores based on the game status.
///
/// ### Arguments
///
/// * `cache` -      EngineCache to use to save the results
/// * `board` -      Board reference to look at.
///
/// ### Returns
///
/// * GameStatus indicating if the game is ongoing or not.
///
#[inline]
pub fn get_eval_from_game_status(game_status: GameStatus) -> f32 {
  match game_status {
    GameStatus::Ongoing => f32::NAN,
    GameStatus::WhiteWon => 200.0,
    GameStatus::BlackWon => -200.0,
    GameStatus::ThreeFoldRepetition | GameStatus::Stalemate | GameStatus::Draw => 0.0,
  }
}
/// Returns a decrement of the evaluation scores if we are in a mating sequence.
///
/// ### Arguments
///
/// * `cache` -      EngineCache to use to save the results
/// * `eval` -       Eval to decrement if mating.
///
/// ### Returns
///
/// * New eval value
///
#[inline]
pub fn decrement_eval_if_mating_sequence(eval: f32) -> f32 {
  if eval.abs() > 100.0 {
    eval - eval.signum() * 1.0
  } else {
    eval
  }
}

/// Looks at a game state and check if the game can be declared a draw
/// (3 fold repetitions and 100-ply)
///
/// ### Arguments
///
/// * `cache` -      EngineCache to use to save the results
/// * `game_state` - A GameState object representing a position, side to play, etc.
///
/// ### Returns
///
/// * `GameStatus::OnGoing` if draw cannot be declared.
/// * `GameStatus::Draw` if we have exceeded the 100-ply
/// * `GameStatus::ThreeFoldRepetition` if we have repeated the position
///
pub fn can_declare_draw(game_state: &GameState) -> GameStatus {
  if game_state.ply >= 100 {
    return GameStatus::Draw;
  }

  // Check the 3-fold repetitions
  if game_state.get_board_repetitions() >= 2 {
    return GameStatus::ThreeFoldRepetition;
  }

  return GameStatus::Ongoing;
}

/// Evaluates a position and returns a score, assuming that the game is Ongoing
///
/// ### Arguments
///
/// * `cache` -      EngineCache to use to store calculations
/// * `game_state` - A GameState object representing a position, side to play, etc.
///
/// ### Returns
///
/// Score assigned to the position.
///
pub fn evaluate_board(game_state: &GameState) -> f32 {
  let score = match determine_game_phase(game_state) {
    GamePhase::Opening => get_opening_position_evaluation(game_state),
    GamePhase::Middlegame => get_middlegame_position_evaluation(game_state),
    GamePhase::Endgame => get_endgame_position_evaluation(game_state),
  };

  score
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_evaluate_board() {
    // This is a forced checkmate in 2:
    let fen = "1n4nr/5ppp/1N6/1P2p3/1P1k4/5P2/1p1NP1PP/R1B1KB1R w KQ - 0 35";
    let game_state = GameState::from_fen(fen);
    let evaluation = evaluate_board(&game_state);
    println!("Evaluation {evaluation}");
    assert!(evaluation > 4.0);
  }

  #[test]
  fn test_evaluate_board_checkmate_in_one() {
    // This is a forced checkmate in 1:
    let fen = "1n4nr/5ppp/1N6/1P2p3/1P6/4kP2/1B1NP1PP/R3KB1R w KQ - 1 36";
    let game_state = GameState::from_fen(fen);
    let evaluation = evaluate_board(&game_state);
    println!("Evaluation {evaluation}");
    assert!(evaluation > 4.0);
  }

  #[test]
  fn test_evaluate_board_checkmate() {
    // This is a "game over" position
    let cache = EngineCache::new();
    let fen = "1n4nr/5ppp/8/1P1Np3/1P6/4kP2/1B1NP1PP/R3KB1R b KQ - 2 37";
    let game_state = GameState::from_fen(fen);
    game_state.get_moves();
    let game_status = is_game_over(&cache, &game_state.board);
    assert_eq!(game_status, GameStatus::WhiteWon);
  }

  #[test]
  fn test_evaluate_board_queen_standoff() {
    // This should obviously be okay because queen is defended and attacked by a queen.
    let fen = "rnb1kbnr/pppp1ppp/5q2/4p3/4P3/5Q2/PPPP1PPP/RNB1KBNR w KQkq - 2 3";
    let game_state = GameState::from_fen(fen);
    game_state.get_moves();
    let evaluation = evaluate_board(&game_state);
    println!("Evaluation: {}", evaluation);
    assert!(evaluation < 1.0);
    assert!(evaluation > -1.0);
  }

  #[test]
  fn test_evaluate_queen_down() {
    let fen = "rnbqk2r/pp3ppp/2pb1n2/3p4/B3P3/8/PPPP1PPP/RNB1K1NR w KQkq - 0 7";
    let game_state = GameState::from_fen(fen);
    game_state.get_moves();
    let evaluation = evaluate_board(&game_state);
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
    let fen = "Qn2q2r/2p2pb1/p2k1n1p/5Bp1/8/2NP4/PPPB1PPP/R4RK1 b - - 0 15";
    let game_state = GameState::from_fen(fen);
    game_state.get_moves();
    let evaluation = evaluate_board(&game_state);
    println!("Evaluation: {}", evaluation);
    assert!(evaluation > 7.0);
  }

  #[test]
  fn test_game_over_threefold_repetition_1() {
    let fen = "8/8/8/5P2/Q1k2KP1/8/p7/8 b - - 1 87";
    let mut game_state = GameState::from_fen(fen);
    assert_eq!(0, game_state.get_board_repetitions());

    game_state.apply_move_from_notation("c4c3");
    assert_eq!(0, game_state.get_board_repetitions());

    game_state.apply_move_from_notation("a4a2");
    assert_eq!(0, game_state.get_board_repetitions());

    game_state.apply_move_from_notation("c3d4");
    assert_eq!(0, game_state.get_board_repetitions());

    game_state.apply_move_from_notation("a2c2");
    assert_eq!(0, game_state.get_board_repetitions());

    game_state.apply_move_from_notation("d4d5");
    assert_eq!(0, game_state.get_board_repetitions());

    game_state.apply_move_from_notation("c2g2");
    assert_eq!(0, game_state.get_board_repetitions());

    game_state.apply_move_from_notation("d5d6");
    assert_eq!(0, game_state.get_board_repetitions());

    game_state.apply_move_from_notation("g2c2");
    assert_eq!(0, game_state.get_board_repetitions());

    game_state.apply_move_from_notation("d6d5");
    assert_eq!(1, game_state.get_board_repetitions());

    game_state.apply_move_from_notation("c2c1");
    assert_eq!(0, game_state.get_board_repetitions());

    game_state.apply_move_from_notation("d5d4");
    assert_eq!(0, game_state.get_board_repetitions());

    game_state.apply_move_from_notation("c1c2");
    assert_eq!(1, game_state.get_board_repetitions());

    game_state.apply_move_from_notation("d4d5");
    println!("{:?}", game_state);
    assert_eq!(2, game_state.get_board_repetitions());
  }

  #[test]
  fn test_game_over_threefold_repetition_2() {
    // This three-fold repetition was not understood during the game: https://lichess.org/oBjYp62P/white
    let fen = "r2q1b1r/1pp1pkpp/2n1p3/p2p4/3PnB2/2NQ1NP1/PPP1PP1P/R3K2R w KQ - 2 9";
    let mut game_state = GameState::from_fen(fen);
    game_state.apply_move_from_notation("c3e4");
    game_state.apply_move_from_notation("d5e4");
    game_state.apply_move_from_notation("f3g5");
    game_state.apply_move_from_notation("f7f6");
    game_state.apply_move_from_notation("g5e4");
    game_state.apply_move_from_notation("f6f7");
    game_state.apply_move_from_notation("e4g5");
    game_state.apply_move_from_notation("f7f6");
    game_state.apply_move_from_notation("g5h7");
    game_state.apply_move_from_notation("f6f7");
    game_state.apply_move_from_notation("h7g5");
    game_state.apply_move_from_notation("f7f6");
    game_state.apply_move_from_notation("g5e4");
    game_state.apply_move_from_notation("f6f7");
    game_state.apply_move_from_notation("e4g5");
    game_state.apply_move_from_notation("f7f6");
    game_state.apply_move_from_notation("g5h7");
    assert_eq!(1, game_state.get_board_repetitions());
    game_state.apply_move_from_notation("f6f7");
    assert_eq!(1, game_state.get_board_repetitions());
    game_state.apply_move_from_notation("h7g5");
    assert_eq!(2, game_state.get_board_repetitions());
  }

  #[test]
  fn test_moves_and_game_over() {
    // Saw something weird in the log here : https://lichess.org/rrAELqBT
    // both Line 0 and 1 are not checkmates, but evaluated to 200.0
    /*
    [2023-10-13T07:08:59.308Z INFO  schnecken_bot::bot::state] Using 1889 ms to find a move for position r1bqkb1r/pp1ppppp/2n2n2/8/8/N7/PPP1QNPP/R1B1KB1R w KQkq - 1 7
    Starting depth 2
    Starting depth 3
    Starting depth 4
    Score for position r1bqkb1r/pp1ppppp/2n2n2/8/8/N7/PPP1QNPP/R1B1KB1R w KQkq - 1 7: 200
    Line 0 : Eval 200.00     - a3c4 d7d5 c1f4 d5c4
    Line 1 : Eval 200.00     - a3b5 d7d5 e2e4 f6e4
    Line 2 : Eval -4.54      - a1b1 c6b4 e2c4
    Line 3 : Eval -4.76      - e1d2 d8b6 d2d1
    Line 4 : Eval -4.79      - e2e3 d7d5 a3c4 d5c4
    Line 5 : Eval -4.84      - h2h3 e7e5 a3c4 d7d
         */
    let fen = "r1bqkb1r/pp2pppp/2n5/1N1p4/4n3/8/PPP2NPP/R1B1KB1R w KQkq - 0 9";
    let game_state = GameState::from_fen(fen);

    let cache = EngineCache::new();
    assert_eq!(GameStatus::Ongoing, is_game_over(&cache, &game_state.board));

    assert!(evaluate_board(&game_state) < 0.0);
  }

  #[test]
  fn test_game_over_checkmate() {
    let fen = "4r1k1/5ppp/p1p5/1QP5/3p2b1/P7/2P1rqPP/2R2NKR w - - 5 26";
    let game_state = GameState::from_fen(fen);
    let cache = EngineCache::new();
    assert_eq!(
      GameStatus::BlackWon,
      is_game_over(&cache, &game_state.board)
    );
  }

  #[test]
  fn evaluate_position_material_down() {
    let game_state = GameState::from_fen("4r1k1/2p2ppp/8/p1b5/P7/2N3PP/1P1n1P2/R5K1 w - - 0 23");
    let eval = evaluate_board(&game_state);
    assert!(eval < -2.0);
  }
}
