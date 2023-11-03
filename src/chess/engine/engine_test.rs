//------------------------------------------------------------------------------
// Engine black-box Tests, checking sanity
#[cfg(test)]
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
fn engine_earch_real_game_w1mLoTRZ() {
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
  assert!(engine.get_best_move().to_string() == "c3e2");
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
