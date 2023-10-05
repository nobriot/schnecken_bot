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
