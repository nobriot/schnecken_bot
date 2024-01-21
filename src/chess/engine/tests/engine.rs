//------------------------------------------------------------------------------
// Engine black-box Tests, checking sanity
use crate::engine::*;

#[test]
#[allow(non_snake_case)]
fn engine_search_real_game_2Dxi9wZH() {
  /*
      Engine came up with sacking the knight on this game. This should be avoided.
      [2023-10-05T06:38:53.763Z INFO  schnecken_bot::bot::state] Using 639 ms to find a move for position rn1qkbnr/p1p2ppp/8/1N1pNb2/8/8/PPPPPPPP/R1BQKB1R w KQkq - 1 5
  Starting depth 2
  Starting depth 3
  Score for position rn1qkbnr/p1p2ppp/8/1N1pNb2/8/8/PPPPPPPP/R1BQKB1R w KQkq - 1 5: 4.8500004
  Line 0 : Eval 4.85       - e5f7 e8f7 e2e3
  Line 1 : Eval 4.67       - d2d4 g8h6
  Line 2 : Eval 4.62       - b5c3 b8a6
  Line 3 : Eval 4.52       - b5d4 f5e6
  Line 4 : Eval 4.47       - e2e3 g8h6
  Line 5 : Eval 4.27       - a2a4 d8d7
  Line 6 : Eval 4.22       - c2c4 d8d7
  Line 7 : Eval 4.22       - d2d3 d8d7
  Line 8 : Eval 4.10       - e5d3 g8h6
  Line 9 : Eval 4.07       - g2g4 f5e4 f2f3
  ---- SNIP ----
  */

  let mut engine = Engine::new();
  engine.set_position("rn1qkbnr/p1p2ppp/8/1N1pNb2/8/8/PPPPPPPP/R1BQKB1R w KQkq - 1 5");
  engine.set_search_time_limit(639);
  engine.go();
  engine.print_evaluations();
  let analysis = engine.get_analysis();
  assert!(!analysis.is_empty());
  assert!(engine.get_best_move().to_string() != "e5f7");
  let eval = analysis[0].1;
  assert!(eval > 2.0);
}

#[test]
#[allow(non_snake_case)]
fn engine_search_real_game_w1mLoTRZ() {
  /*
    [2023-10-26T10:13:15.054Z INFO  schnecken_bot::bot::state] Using 1758 ms to find a move for position r2q1rk1/pp2bppp/2p2n2/4p3/3pP1bP/2NQ4/PPPP1PPR/RNB2K2 w - - 0 13
    Score for position r2q1rk1/pp2bppp/2p2n2/4p3/3pP1bP/2NQ4/PPPP1PPR/RNB2K2 w - - 0 13: -2.2133334
    Line 0 : Eval -2.21      - h2h1 d8d6 f1g1 d4c3
    Line 1 : Eval -2.36      - a2a3 d8d6 f1g1 d4c3
    Line 2 : Eval -2.41      - c3a4 d8d6 b1c3
    Line 3 : Eval -2.46      - a2a4 d8d6 f1g1 d4c3
    Line 4 : Eval -2.56      - h4h5 d8d6 h5h6
    Line 5 : Eval -2.60      - f2f3 g4e6 f1g1 d4c3
    Line 6 : Eval -2.61      - b2b3 d8d6 c1b2
    Line 7 : Eval -2.66      - g2g3 d8d6 f1g1 d4c3
    Line 8 : Eval -2.75      - c3e2 g4e2 f1e2
    Line 9 : Eval -2.76      - f1e1 d8d6 b1a3 d4c3
    Line 10: Eval -3.09      - f2f4 e5f4 f1g1
    Line 11: Eval -3.16      - b2b4 e7b4 c1b2
    Line 12: Eval -3.34      - d3g3 d4c3 b1c3
    Line 13: Eval -3.49      - d3c4 d4c3 b1c3
    Line 14: Eval -4.15      - f1g1 g4h5 c3e2 h5e2
    Line 15: Eval -4.42      - c3d1 g4d1 b1c3
    Line 16: Eval -4.59      - h2h3 g4h3 g2h3
    Line 17: Eval -4.77      - c3b5 c6b5 b1c3 d4c3
    Line 18: Eval -5.32      - c3d5 c6d5 b1c3
    Line 19: Eval -5.37      - b1a3 d4c3 d3c3 e7a3
    Line 20: Eval -10.03     - d3h3 g4h3 h2h3
    Line 21: Eval -10.23     - d3e2 g4e2 f1e2
    Line 22: Eval -10.78     - d3f3 g4f3 g2f3
    Line 23: Eval -11.62     - d3e3 d4e3 d2e3
    Line 24: Eval -11.78     - d3d4 d8d4 d2d3
    Line 25: Eval -12.44     - d3a6 b7a6 f1g1
    Line 26: Eval -12.74     - d3b5 c6b5 f1g1
  */
  let mut engine = Engine::new();
  engine.set_position("r2q1rk1/pp2bppp/2p2n2/4p3/3pP1bP/2NQ4/PPPP1PPR/RNB2K2 w - - 0 13");
  engine.set_search_time_limit(1758);
  engine.go();
  engine.print_evaluations();
  let analysis = engine.get_analysis();
  assert!(!analysis.is_empty());
  assert!(
    engine.get_best_move().to_string() == "c3e2" || engine.get_best_move().to_string() == "f2f3"
  );
  let eval = analysis[0].1;
  assert!(eval < -2.0);
}

#[test]
#[allow(non_snake_case)]
fn engine_earch_real_game_W89VkRfp() {
  let mut engine = Engine::new();
  engine.set_position("4r1k1/2p2ppp/8/p1b5/P3n3/2N4P/1P1B1PP1/R5K1 w - - 1 22");
  engine.set_search_time_limit(1624);
  engine.go();
  engine.print_evaluations();
  let analysis = engine.get_analysis();
  assert!(!analysis.is_empty());
  assert!(engine.get_best_move().to_string() == "c3e4");
  let eval = analysis[0].1;
  assert!(eval > 1.0);
}

#[test]
fn engine_select_best_move_checkmate_in_one() {
  // This is a forced checkmate in 1:
  let mut engine = Engine::new();
  engine.set_position("1n4nr/5ppp/1N6/1P2p3/1P6/4kP2/1B1NP1PP/R3KB1R w KQ - 1 36");
  engine.set_maximum_depth(2);
  engine.go();

  // println!("engine analysis: {:#?}", engine.analysis.scores);
  engine.print_evaluations();
  let expected_move = Move::from_string("b6d5");
  assert_eq!(expected_move, engine.get_best_move());
}

#[test]
fn engine_select_best_move_checkmate_in_one_for_black() {
  // This is a forced checkmate in 1 for black:
  let mut engine = Engine::new();
  engine.set_position("8/8/2p1pkp1/p3p3/P1P1P1P1/6q1/7q/3K4 b - - 2 55");
  engine.set_maximum_depth(2);
  engine.go();

  //println!("engine analysis: {:#?}", engine.analysis.scores);
  engine.print_evaluations();
  let expected_move = Move::from_string("g3g1");
  assert_eq!(expected_move, engine.get_best_move());
}

#[test]
fn engine_select_best_move_checkmate_in_two() {
  // This is a forced checkmate in 2: c1b2 d4e3 b6d5
  let mut engine = Engine::new();
  engine.set_position("1n4nr/5ppp/1N6/1P2p3/1P1k4/5P2/1p1NP1PP/R1B1KB1R w KQ - 0 35");
  engine.set_search_time_limit(5000);
  engine.set_maximum_depth(3);
  engine.go();

  engine.print_evaluations();
  let expected_move = "c1b2";
  assert_eq!(expected_move, engine.get_best_move().to_string());
  let analysis = engine.get_analysis();
  assert_eq!(analysis[0].1, 198.0);
}

#[test]
fn engine_select_find_best_defensive_move() {
  // Only good defense is : h8f8
  let mut engine = Engine::new();
  engine.set_position("r1bqk2r/ppppbp1p/2n5/3Bp1pQ/4P3/3P4/PPPN1PPP/R3K1NR b KQq - 0 7");
  engine.set_search_time_limit(5000);
  engine.set_maximum_depth(8);
  engine.go();

  engine.print_evaluations();
  let expected_move = "h8f8";
  assert_eq!(expected_move, engine.get_best_move().to_string());
}

#[test]
fn engine_save_the_last_knight() {
  // Game: https://lichess.org/iavzLpKc
  let mut engine = Engine::new();
  engine.set_position("4r1k1/1p6/7p/p4p2/Pb1p1P2/1PN3P1/2P1P1K1/r7 w - - 0 34");
  engine.set_maximum_depth(20);
  engine.set_search_time_limit(7863);
  engine.go();

  let good_moves = [Move::from_string("c3b5"), Move::from_string("c3d5")];
  let engine_move = engine.get_best_move();
  engine.print_evaluations();
  if !good_moves.contains(&engine_move) {
    assert!(
      false,
      "Expected either c3b5 or c3d5, but instead we have {}",
      engine_move.to_string()
    );
  }
}

#[test]
fn engine_promote_this_pawn() {
  let mut engine = Engine::new();
  engine.set_position("8/P7/4kN2/4P3/1K3P2/4P3/8/8 w - - 7 76");
  engine.set_maximum_depth(20);
  engine.set_search_time_limit(855);
  engine.go();

  engine.print_evaluations();
  let expected_move = Move::from_string("a7a8Q");
  assert_eq!(expected_move, engine.get_best_move());
}

#[test]
fn engine_go_and_stop() {
  let mut engine = Engine::new();
  // Note: Avoid book moves here, it will return immediately no matter what.
  engine.set_position("rn2kbnr/ppp1pppp/8/3p4/P7/2NPPP1N/1PP1b1PR/R1B1KB2 b Qkq - 0 7");
  engine.set_maximum_depth(0);
  engine.set_search_time_limit(0);
  engine.set_ponder(true);

  let engine_clone = engine.clone();
  let _handle = std::thread::spawn(move || engine_clone.go());

  std::thread::sleep(std::time::Duration::from_millis(10));

  assert_eq!(true, engine.is_active());
  std::thread::sleep(std::time::Duration::from_millis(1000));
  assert_eq!(true, engine.is_active());
  engine.stop();
  assert_eq!(true, engine.is_active());

  std::thread::sleep(std::time::Duration::from_millis(50));
  assert_eq!(false, engine.is_active());

  // It actually takes super long before handle will be marked as finished.
  //std::thread::sleep(std::time::Duration::from_millis(300));
  //assert_eq!(true, handle.is_finished());
}

#[test]
fn engine_bench_positions_per_second() {
  let mut engine = Engine::new();
  engine.set_position("4r1k1/1p6/7p/p4p2/Pb1p1P2/1PN3P1/2P1P1K1/r7 w - - 0 34");
  engine.set_search_time_limit(1000);
  engine.go();

  println!("Engine cache length: {}", engine.cache.len());
  // 100 kNPS would be nice. Right now we are at a very low number LOL
  assert!(
    engine.cache.len() > 100_000,
    "Number of NPS for engine analysis: {}",
    engine.cache.len()
  );
}

#[test]
fn test_dont_hang_pieces_1() {
  /* Got this in a game, hanging a knight, after thinking for 16_000 ms :
   Line 0 Eval: 0.79999995 - f8h6 d5e4 d7d5 e4d3
   Line 1 Eval: -0.30000085 - e4f6 d5d3
   Line 2 Eval: 2.3999996 - b7b5 d5e4 d7d5 e4d3 e7e5 b1c3
   Line 3 Eval: 2.5499997 - b7b6 d5e4 d7d5 e4d3 e7e5 b1c3
   Line 4 Eval: 3.2999995 - c6b8 d5e4 d7d5 e4d3 b8c6 b1c3
  */
  let mut engine = Engine::new();
  engine.set_position("r1bqkb1r/1ppppp1p/p1n5/3Q4/4n3/5N2/PPPP1PPP/RNB1KB1R b KQkq - 0 7");
  engine.set_search_time_limit(3000);
  engine.go();
  engine.print_evaluations();

  let best_move = engine.get_best_move().to_string();

  if "e4f6" != best_move && "e4d6" != best_move {
    assert!(
      false,
      "Should have been either e4f6 or e4d6, instead we have: {best_move}"
    );
  }
}

#[test]
fn test_dont_hang_pieces_2() {
  /*
    https://lichess.org/zcQesp7F#69
    Here we blundered a rook playing e2f2
    2k5/pp5p/2p3p1/8/1PpP4/P5KP/4r2P/8 b - - 1 35
    Using 1355 ms to find a move
    Line 0 Eval: -9.860003 - e2f2 g3f2 c8b8 f2g1 c4c3 g1g2 c3c2 g2g1 c2c1Q
    Line 1 Eval: -9.250003 - e2e5 d4e5 c8b8 g3g2 c4c3 e5e6 c3c2 e6e7 c2c1Q
    Line 2 Eval: -7.820003 - e2a2 g3f3 a2a3 f3g2
    Line 3 Eval: -8.105003 - e2h2 g3g4 h2e2
    Line 4 Eval: -7.9150023 - e2d2 b4b5 d2d4
    [2023-05-12T06:06:18Z INFO  schnecken_bot] Playing move e2f2 for game id zcQesp7F
  */

  let mut engine = Engine::new();
  engine.set_position("2k5/pp5p/2p3p1/8/1PpP4/P5KP/4r2P/8 b - - 1 35");
  engine.set_search_time_limit(1000);
  engine.go();
  engine.print_evaluations();
  let not_expected_move = Move::from_string("e2f2");
  assert!(
    not_expected_move != engine.get_best_move(),
    "e2f2 should not be played!!"
  );
}

// From game : https://lichess.org/SKF7qgMu -
// Did not capture the knight, it was very obvious to capture.
// Spent 2450 ms to come up with this crap: e5f5
#[test]
fn save_the_queen() {
  let mut engine = Engine::new();
  engine.set_position("rnbqk2r/pp3ppp/2pbpn2/3pQ3/B3P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 6");
  engine.set_search_time_limit(2450);
  engine.go();
  engine.print_evaluations();

  let game_state1 =
    GameState::from_fen("rnbqk2r/pp3ppp/2pQpn2/3p4/B3P3/8/PPPP1PPP/RNB1K1NR b KQkq - 0 6");
  println!(
    "Static intermediate:  {}",
    engine.cache.get_eval(&game_state1.board).unwrap_or_default().eval
  );

  let game_state =
    GameState::from_fen("rnb1k2r/pp3ppp/2pqpn2/3p4/B3P3/8/PPPP1PPP/RNB1K1NR w KQkq - 0 7");
  println!(
    "Static from cache:  {}",
    engine.cache.get_eval(&game_state.board).unwrap_or_default().eval
  );

  let static_eval = evaluate_board(&game_state);
  println!("Static eval: {static_eval}");
  assert_eq!(true, engine.cache.has_eval(&game_state.board));

  let best_move = engine.get_best_move().to_string();
  if "e5g5" != best_move && "e5d4" != best_move && "e5c3" != best_move {
    assert!(
      false,
      "Should have been either e5g5, e5d4 or e5c3, instead we have: {best_move}"
    );
  }
}

// From game : https://lichess.org/47V8eE5x -
// Did not capture the knight, it was very obvious to capture.
// Spent 2900 ms to come up with this crap: d7d5
#[test]
fn capture_the_damn_knight_1() {
  let mut engine = Engine::new();
  engine.set_position("rnb2r1k/pppp2pp/5N2/8/1bB5/8/PPPPQPPP/RNB1K2R b KQ - 0 9");
  engine.set_search_time_limit(2900);
  engine.go();
  engine.print_evaluations();

  let best_move = engine.get_best_move().to_string();
  if "f8f6" != best_move && "g7f6" != best_move {
    assert!(
      false,
      "Should have been either f8f6 or g7f6, instead we have: {best_move}"
    );
  }
}

#[test]
fn evaluate_checkmate_with_castle() {
  let mut engine = Engine::new();
  engine.set_position("8/8/8/8/2nN4/1q6/ppP1NPPP/1k2K2R w K - 0 1");
  engine.set_search_time_limit(10);
  engine.go();
  engine.print_evaluations();

  assert_eq!("e1g1", engine.get_best_move().to_string());
}

// Game https://lichess.org/Xjgkf4pp seemed really off. Testing some of the positions here
#[test]
fn test_select_pawn_capture() {
  let mut engine = Engine::new();
  engine.set_position("r2q1rk1/1pp1ppbp/p2p1np1/P7/6bP/R1N1Pn2/1PPP1PP1/2BQKB1R w K - 0 11");
  engine.set_search_time_limit(2000);
  engine.go();
  engine.print_evaluations();

  assert_eq!("g2f3", engine.get_best_move().to_string());
}

#[test]
fn test_select_best_move_checkmate_in_two() {
  // This is a forced checkmate in 2: c1b2 d4e3 b6d5
  let mut engine = Engine::new();
  engine.set_position("1n4nr/5ppp/1N6/1P2p3/1P1k4/5P2/1p1NP1PP/R1B1KB1R w KQ - 0 35");
  engine.set_search_time_limit(5000);
  engine.go();
  engine.print_evaluations();

  let expected_move = "c1b2";
  assert_eq!(expected_move, engine.get_best_move().to_string());
}

#[test]
fn test_select_best_move_checkmate_in_one() {
  // This is a forced checkmate in 1:
  let mut engine = Engine::new();
  engine.set_position("1n4nr/5ppp/1N6/1P2p3/1P6/4kP2/1B1NP1PP/R3KB1R w KQ - 1 36");
  engine.set_search_time_limit(5000);
  engine.go();
  engine.print_evaluations();
  let expected_move = Move::from_string("b6d5");
  assert_eq!(expected_move, engine.get_best_move());
}

#[test]
fn test_avoid_threefold_repetitions() {
  use crate::model::board::Board;
  /* Looks like we had a permutation bug that lead us into some 3-fold repetitions
   [2023-07-04T12:36:47Z INFO  schnecken_bot::chess::engine::core] Using 1211 ms to find a move
     Line 0 Eval: 10.71348 - d1e2 / Permutation
     Line 1 Eval: 6.581044 - h2h3 / Permutation
     Line 2 Eval: 6.461045 - g3g2 / Permutation
     Line 3 Eval: 6.431045 - a1b1 / Permutation
     Line 4 Eval: 6.391044 - g3g1 / Permutation
  */

  let mut engine = Engine::new();
  engine.set_position("r7/1p4p1/5p1p/b3n1k1/p3P1P1/PbN3R1/1P1K3P/R1BB4 w - - 10 45");
  engine.set_search_time_limit(1200);
  engine
    .position
    .last_positions
    .push_back(Board::from_fen("r7/1p4p1/5p1p/b3n1k1/p3P1P1/PbN3R1/1P1K3P/R1BB4").hash);
  engine
    .position
    .last_positions
    .push_back(Board::from_fen("r7/1p4p1/5p1p/b3n1k1/p1b1P1P1/P1N3R1/1P1K3P/R1BB4").hash);
  engine
    .position
    .last_positions
    .push_back(Board::from_fen("r7/1p4p1/5p1p/b3n1k1/p1b1P1P1/P1N3R1/1P1KB2P/R1B5").hash);
  engine
    .position
    .last_positions
    .push_back(Board::from_fen("r7/1p4p1/5p1p/b3n1k1/p3P1P1/PbN3R1/1P1KB2P/R1B5").hash);
  engine
    .position
    .last_positions
    .push_back(Board::from_fen("r7/1p4p1/5p1p/b3n1k1/p3P1P1/PbN3R1/1P1K3P/R1BB4").hash);
  engine
    .position
    .last_positions
    .push_back(Board::from_fen("r7/1p4p1/5p1p/b3n1k1/p1b1P1P1/P1N3R1/1P1K3P/R1BB4").hash);
  engine
    .position
    .last_positions
    .push_back(Board::from_fen("r7/1p4p1/5p1p/b3n1k1/p1b1P1P1/P1N3R1/1P1KB2P/R1B5").hash);
  engine
    .position
    .last_positions
    .push_back(Board::from_fen("r7/1p4p1/2n2p1p/b5k1/p1b1P1P1/P1N3R1/1P1KB2P/R1B5").hash);

  engine.go();
  engine.print_evaluations();
  assert!(engine.get_best_move() != Move::from_string("d1e2"));
}

#[test]
fn test_only_one_legal_move() {
  let mut engine = Engine::new();
  engine.set_position("5k2/R6P/8/2PKB3/1P6/1P1P1N2/5PP1/R7 b - - 0 67");
  engine.set_search_time_limit(942);

  engine.go();
  engine.print_evaluations();
  let analysis = engine.get_analysis();
  assert!(!analysis.is_empty());
  assert!(engine.get_best_move() == Move::from_string("f8e8"));
}

#[test]
fn capture_the_bishop() {
  let mut engine = Engine::new();
  engine.set_position("rnbqk1nr/pp3ppp/2p5/1Q1p4/1b1Pp3/2N2N2/PPP1PPPP/R1B1KB1R w KQkq - 0 6");
  engine.set_search_time_limit(1875);
  engine.go();
  engine.print_evaluations();
  let analysis = engine.get_analysis();
  assert!(!analysis.is_empty());
  assert!(engine.get_best_move().to_string() == "b5b4");
}

#[test]
fn endgame_evaluation_search() {
  let mut engine = Engine::new();
  engine.set_position("1K6/2Q5/8/8/8/3k4/8/8 w - - 0 1");
  engine.set_search_time_limit(800);
  engine.go();
  engine.print_evaluations();
  let analysis = engine.get_analysis();

  // 26 moves.
  assert_eq!(analysis.len(), 26);
  let bad_moves = vec![
    "c7c4", "c7c3", "c7c2", "c7d8", "c7c8", "c7b7", "c7a7", "c7e7", "c7f7", "c7d7", "c7g7", "c7h7",
    "b8a8", "b8a7",
  ];
  assert!(!bad_moves.contains(&engine.get_best_move().to_string().as_str()));
}

#[test]
#[allow(non_snake_case)]
fn evaluate_real_game_0BYxLu3V_example_1() {
  // https://lichess.org/0BYxLu3V has plently of blunders.
  //
  let mut engine = Engine::new();
  engine.set_position("r1b1kbnr/pppp1p1p/4pqp1/8/3nP3/2NQ1N2/PPPP1PPP/R1B1KB1R b KQkq - 7 6");
  engine.set_search_time_limit(1897);
  //engine.set_maximum_depth(3);
  engine.go();
  engine.print_evaluations();
  let analysis = engine.get_analysis();
  assert!(!analysis.is_empty());
  assert!(engine.get_best_move() != Move::from_string("f8d6"));
  assert!(engine.get_best_move() != Move::from_string("f6d8"));
}

#[test]
#[allow(non_snake_case)]
fn evaluate_real_game_0BYxLu3V_example_2() {
  // https://lichess.org/0BYxLu3V has plently of blunders.
  //
  let mut engine = Engine::new();
  engine.set_position("r1b1k1nr/pppp1p1p/3bpqp1/8/3QP3/2N2N2/PPPP1PPP/R1B1KB1R b KQkq - 0 7");
  engine.set_search_time_limit(1870);
  engine.go();
  engine.print_evaluations();
  let analysis = engine.get_analysis();
  assert!(!analysis.is_empty());
  assert!(engine.get_best_move() != Move::from_string("d6e5"));
}

#[test]
fn evaluate_real_game_no8g7oup_example() {
  // https://lichess.org/no8g7oup
  //
  let mut engine = Engine::new();
  engine.set_position("r4rk1/2p5/p2pq2p/1p4p1/3Qb1n1/2N5/PPn1K1PP/R1B2B1R b - - 1 22");
  engine.set_search_time_limit(423);
  engine.go();
  engine.print_evaluations();
  let analysis = engine.get_analysis();
  assert!(!analysis.is_empty());
  assert!(engine.get_best_move().to_string() == "c2d4");
}

#[test]
#[allow(non_snake_case)]
fn evaluate_real_game_ov5SZJLX_example() {
  // https://lichess.org/ov5SZJLX
  // Engine came up with this:
  // Depth 2 completed
  // Score for position rn2kbnr/ppp1pppp/3q4/3p4/P7/2N1P2N/1PPP1PPP/R1BbKB1R w KQkq - 0 5: 21.355005
  // Line 0 : Eval 21.355005  - f1b5 d6c6
  // Line 1 : Eval -6.16      - e1d1 d6h2
  // Line 2 : Eval -6.4399996 - c3d1 d6h2
  // Line 3 : Eval -8.295     - f1d3 d1c2
  // Line 4 : Eval -8.605     - d2d4 d1c

  let mut engine = Engine::new();
  engine.set_position("rn2kbnr/ppp1pppp/3q4/3p4/P7/2N1P2N/1PPP1PPP/R1BbKB1R w KQkq - 0 5");
  engine.set_search_time_limit(6426);
  engine.go();
  engine.print_evaluations();
  let analysis = engine.get_analysis();
  assert!(!analysis.is_empty());
  assert!(analysis[0].1 < -5.0);
}

#[ignore]
#[test]
fn test_sorting_moves_without_eval() {
  let fen = "r1bqk2r/pp3ppp/n1pbpn2/3pQ3/B3P3/5N2/PPPP1PPP/RNB1K2R w KQkq - 6 7";
  let game_state = GameState::from_fen(fen);

  let engine = Engine::new();
  Engine::find_move_list(&engine.cache, &game_state.board);
  let move_list = engine.cache.get_move_list(&game_state.board).unwrap();
  for m in &move_list {
    println!("Move: {}", m.to_string());
  }

  assert_eq!(Move::from_string("e5d6"), move_list[0]);
  assert_eq!(Move::from_string("e5f6"), move_list[1]);
  assert_eq!(Move::from_string("e4d5"), move_list[2]);
  assert_eq!(Move::from_string("a4c6"), move_list[3]);
  assert_eq!(Move::from_string("e5e6"), move_list[4]);
}

#[test]
fn test_drawn_pawn_and_king_endgame() {
  // From game : https://lichess.org/KHECoP9m - The bot thought it is completely winning
  let fen = "8/8/3K2k1/6p1/6P1/8/8/8 w - - 6 50";
  let game_state = GameState::from_fen(fen);

  let eval = evaluate_board(&game_state);
  println!("Eval is: {}", eval);

  let game_state_2 = GameState::from_fen("8/8/5K1k/6p1/6P1/8/8/8 b - - 11 52");
  let eval_2 = evaluate_board(&game_state_2);
  println!("Eval is: {}", eval_2);

  let mut engine = Engine::new();
  engine.set_position(fen);
  engine.set_search_time_limit(500);
  engine.go();
  engine.print_evaluations();
  let analysis = engine.get_analysis();
  assert!(!analysis.is_empty());
  assert!(analysis[0].1 < 1.0);
  assert!(analysis[0].1 > -1.0);
}

#[test]
fn test_under_promotion_got_evaluated_better() {
  /*
    Got this in game uPyVb71j

    [2023-11-15T10:18:28.537Z INFO  schnecken_bot::bot::state] Trying to find a move for game id uPyVb71j
  [2023-11-15T10:18:28.537Z INFO  schnecken_bot::bot::state] Using 1067 ms to find a move for position 8/8/4K3/7k/8/8/6Rp/8 b - - 2 58
  info score cp 180 depth 5 seldepth 7 nodes 14909878 time 126 multipv 1 pv h2h1r e6d5 h1d1 d5e4 d1d2
  info score cp 900 depth 5 seldepth 7 nodes 14909878 time 126 multipv 2 pv h5h4 g2h2 h4g3 h2h1
  info score cp 1100 depth 5 seldepth 7 nodes 14909878 time 126 multipv 3 pv h2h1q
  bestmove h2h1r
  Score for position 8/8/4K3/7k/8/8/6Rp/8 b - - 2 58: 1.8
  Line 0 : Eval 1.80    @ depth 5 - h2h1r e6d5 h1d1 d5e4 d1d2
  Line 1 : Eval 9.00    @ depth 4 - h5h4 g2h2 h4g3 h2h1
  Line 2 : Eval 11.00   @ depth 1 - h2h1q
  Line 3 : Eval 11.00   @ depth 4 - h2h1b g2g1 h1d5 e6d5
  Line 4 : Eval 12.00   @ depth 4 - h2h1n g2g1 h1g3 g1g3
  Line 5 : Eval 14.00   @ depth 3 - h5h6 g2h2 h6g7
  */

  let queen_fen = "8/8/4K3/7k/8/8/6R1/7q w - - 0 59";
  let queen_game_state = GameState::from_fen(queen_fen);

  let rook_fen = "8/8/4K3/7k/8/8/6R1/7r w - - 0 59";
  let rook_game_state = GameState::from_fen(rook_fen);

  let queen_eval = evaluate_board(&queen_game_state);
  let rook_eval = evaluate_board(&rook_game_state);
  println!("Queen eval : {} - Rook Eval : {}", queen_eval, rook_eval);
  assert!(queen_eval < rook_eval);

  let fen = "8/8/4K3/7k/8/8/6Rp/8 b - - 2 58";
  let mut engine = Engine::new();
  engine.set_position(fen);
  engine.set_search_time_limit(1067);
  engine.go();
  engine.print_evaluations();
  let best_move = engine.get_best_move();
  let analysis = engine.get_analysis();
  assert!(!analysis.is_empty());
  assert_eq!(best_move.to_string(), "h2h1q");
  assert_eq!(analysis[1].0[0].to_string(), "h2h1r");
  assert!(analysis[1].1 < 1.0);
  assert!(analysis[1].1 > -1.0);

  // Same but from the next move perspective:
  let fen = "8/8/4K3/7k/8/8/6R1/7r w - - 0 59";
  let mut engine = Engine::new();
  engine.set_position(fen);
  engine.set_search_time_limit(1067);
  engine.go();
  engine.print_evaluations();
  let best_move = engine.get_best_move();
  let analysis = engine.get_analysis();
  assert!(!analysis.is_empty());
  assert!(analysis[0].1 == 0.0);
  assert!(analysis[1].1 == 0.0);
  assert!(analysis[2].1 == 0.0);
}

#[test]
fn test_knight_bonanza_on_real_game() {
  /*
    [2023-11-18T16:08:38.256Z INFO  schnecken_bot::bot::state] Trying to find a move for game id g5m6cPhb
    [2023-11-18T16:08:38.256Z INFO  schnecken_bot::bot::state] Using 7509 ms to find a move for position r1bq1rk1/ppp2pbp/3ppnp1/3Pn3/2P1PP2/2N1B2P/PP4P1/RQ2KBNR b KQ - 0 9
    info score cp 9 depth 4 seldepth 7 nodes 38111518 time 13 multipv 1 pv e5d7 g1f3 e6d5 c4d5
    info score cp 65 depth 4 seldepth 7 nodes 38111518 time 13 multipv 2 pv e5c4 f1c4 f6g4 e3a7
    info score cp 76 depth 4 seldepth 7 nodes 38111518 time 13 multipv 3 pv e6d5 f4e5 d6e5 c3d5
    info score cp -110 depth 5 seldepth 8 nodes 38186672 time 82 multipv 1 pv e5d7 g1f3 e6d5 e4d5 f6d5
    info score cp -59 depth 5 seldepth 8 nodes 38186672 time 82 multipv 2 pv e5g4 h3g4 g6g5 h1h7 g8h7 e3a7
    info score cp -58 depth 5 seldepth 8 nodes 38186672 time 82 multipv 3 pv f6g4 h3g4 e6d5 h1h7 g8h7 c4d5
    info score cp 51 depth 6 seldepth 9 nodes 39647298 time 1481 multipv 1 pv f6g4 e3a7 d8h4 e1d1 a8a7 g1f3
    info score cp 59 depth 6 seldepth 9 nodes 39647298 time 1481 multipv 2 pv e5g4 e3d2 g4h6 g1f3 e6d5 c4d5
    info score cp 73 depth 6 seldepth 9 nodes 39647298 time 1481 multipv 3 pv e5d7 d5e6 f7e6 g1f3 d7c5 e4e5

  [2023-11-18T16:08:45.765Z INFO  schnecken_bot::bot::state] Playing Line 0 (f6g4) for GameID g5m6cPhb
  [2023-11-18T16:08:45.765Z INFO  lichess::api::game] Trying chess move f6g4 on game id g5m6cPhb - Draw offer: false
  [2023-11-18T16:08:45.765Z DEBUG lichess::api] Lichess POST request at https://lichess.org/api/bot/game/g5m6cPhb/move/f6g4?offeringDraw=false
  [2023-11-18T16:08:45.765Z DEBUG reqwest::connect] starting new connection: https://lichess.org/
  [2023-11-18T16:08:45.875Z DEBUG lichess::api] Lichess post answer: {"ok":true}
   */
  let fen = "r1bq1rk1/ppp2pbp/3ppnp1/3Pn3/2P1PP2/2N1B2P/PP4P1/RQ2KBNR b KQ - 0 9";
  let mut engine = Engine::new();
  engine.set_position(fen);
  engine.set_search_time_limit(7509);
  engine.go();
  engine.print_evaluations();
  let best_move = engine.get_best_move();
  let analysis = engine.get_analysis();
  assert!(!analysis.is_empty());
  assert_eq!(best_move.to_string(), "e5d7");
}

#[test]
fn real_game_was_winning() {
  /*
      2023-11-26T17:16:06.000Z INFO  schnecken_bot::bot::state] Trying to find a move for game id VwCbe7lf
    [2023-11-26T17:16:06.000Z INFO  schnecken_bot::bot::state] Using 3627 ms to find a move for position 6r1/Bkp1Nbp1/1p6/5R2/8/4bP2/PPP3PK/8 b - - 7 31
    info score cp 202 depth 5 seldepth 8 nodes 17914126 time 267 multipv 1 pv f7e6 f5h5 g8a8 h5e5 e6a2
    info score cp 202 depth 5 seldepth 8 nodes 17914126 time 267 multipv 2 pv g8h8 h2g3 f7a2 f5e5 a2d5
    info score cp 248 depth 5 seldepth 8 nodes 17914126 time 267 multipv 3 pv g8f8 e7g6 f7g6 f5f8 b7a7 h2g3
    [2023-11-26T17:16:06.719Z DEBUG lichess::api] Received keep-alive message for Game State stream
    info score cp 248 depth 6 seldepth 9 nodes 18375657 time 2535 multipv 1 pv g8f8 e7g6 f7g6 f5f8 b7a7 h2g3
    info score cp 298 depth 6 seldepth 9 nodes 18375657 time 2535 multipv 2 pv e3g1 h2g1 g8f8 f5f7 f8f7 a7b6
    info score cp 298 depth 6 seldepth 9 nodes 18375657 time 2535 multipv 3 pv e3f4 f5f4 g8h8 h2g1 h8f8 f4f7 f8f7 a7b6
    bestmove g8f8
    Score for position 6r1/Bkp1Nbp1/1p6/5R2/8/4bP2/PPP3PK/8 b - - 7 31: 2.485385
  */
  let fen = "6r1/Bkp1Nbp1/1p6/5R2/8/4bP2/PPP3PK/8 b - - 7 31";
  let mut engine = Engine::new();
  engine.set_position(fen);
  engine.set_search_time_limit(0);
  engine.set_maximum_depth(6);
  engine.go();
  engine.print_evaluations();
  let best_move = engine.get_best_move();
  let analysis = engine.get_analysis();
  assert!(!analysis.is_empty());
  assert!(best_move.to_string() == "f7e6" || best_move.to_string() == "g8h8");
}

#[ignore]
#[test]
fn king_disappeared() {
  let fen = "r1b3k1/2Bp1ppp/p1p5/2P5/3b1K2/P7/1P3rPP/4q3 w - - 2 24";
  let mut engine = Engine::new();
  engine.set_position(fen);
  engine.set_search_time_limit(3000);
  engine.go();
  engine.print_evaluations();
  let best_move = engine.get_best_move();
  let analysis = engine.get_analysis();
  assert!(!analysis.is_empty());
}
