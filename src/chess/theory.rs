use lazy_static::lazy_static;
use std::collections::HashMap;

// Our known chess position and good moves associated to it
lazy_static! {
    static ref CHESS_THEORY: HashMap<&'static str, Vec<&'static str>> = {
        let mut t = HashMap::new();
        // First move for White
        t.insert("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        vec!["e2e4","d2d4","g1f3"]);

        // First move for Black, 1.e4
        t.insert("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
        vec!["c7c5","e7e5","e7e6"]);

        // After 1.e4e5, we play the bongcloud!
        t.insert("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
        vec!["e1e2"]);

        // French defence for black
        t.insert("rnbqkbnr/pppp1ppp/4p3/8/4P3/3P4/PPP2PPP/RNBQKBNR b KQkq - 0 2",
        vec!["d7d5"]);

        // Random position I asked stockfist to analyze:
        // We should probably strip the last numbers from this hashmap key for the endgames
        t.insert("4r3/pppk4/2p1p2p/6PN/2P2PK1/1Pb4P/P3R3/8 w - - 3 38",
        vec!["h5g3","h3h4","e2e3"]);

        // Return our super cache
        t
    };
}

// Check our known book moves, known positions that have been computed with an
// evaluation before, so that we do not need to find moves ourselves.
pub fn get_theory_moves(fen: &str) -> Option<&Vec<&'static str>> {
  CHESS_THEORY.get(fen)
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_theory_lines() {
    use crate::chess::theory::*;
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    // Some eval values, from strongest to weakest (from white perspective)
    let result = get_theory_moves(fen);
    assert_eq!(Some(&vec!["e2e4", "d2d4", "g1g3"]), result);
  }

  #[test]
  fn test_theory_lines_endgame() {
    use crate::chess::theory::*;
    let fen = "4r3/pppk4/2p1p2p/6PN/2P2PK1/1Pb4P/P3R3/8 w - - 3 38";
    // Some eval values, from strongest to weakest (from white perspective)
    let result = get_theory_moves(fen);
    assert_eq!(Some(&vec!["h5g3", "h3h4", "e2e3"]), result);
  }
}
