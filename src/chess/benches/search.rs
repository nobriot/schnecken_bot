use chess::engine::Engine;
use divan::Bencher;
use divan::counter::ItemsCount;

fn main() {
  // Run registered benchmarks.
  divan::main();
}

/// Depth at which the positions are searched. Keep it low enough that one
/// search stays in the 10-100 ms range.
const SEARCH_DEPTH: usize = 4;

/// Positions out of the opening books, so that the engine has to search them.
const POSITIONS: [&str; 3] = [// Middlegame with a lot of pieces
                              "r1bqk2r/ppp2ppp/2n2n2/3p4/1bPPp3/2N1P2P/PP3PPN/R1BQKB1R w KQkq - 8 11",
                              // Tactical position, white is winning material
                              "rn1qkbnr/p1p2ppp/8/1N1pNb2/8/8/PPPPPPPP/R1BQKB1R w KQkq - 1 5",
                              // Pawn endgame
                              "8/2k5/1pp2p2/p3p1p1/P1P1P1P1/1PK2P2/8/8 w - - 0 40"];

/// Creates an engine ready to search a position, with empty caches.
fn new_engine(fen: &str) -> Engine {
  let mut engine = Engine::new(false);
  engine.options.max_depth = SEARCH_DEPTH;
  engine.options.max_search_time = 0;
  engine.set_position(fen);
  engine
}

/// Checks how fast we search a position to a fixed depth, starting from cold
/// caches. Throughput is reported in evaluated nodes per second.
#[divan::bench(args = POSITIONS, sample_count = 20)]
fn search_fixed_depth(bencher: Bencher, fen: &str) {
  // The search is deterministic with cold caches: count the nodes once.
  let engine = new_engine(fen);
  engine.go();
  let nodes = engine.get_nodes_visited();
  assert!(nodes > 0, "No node visited for {fen}, is it a book position?");

  bencher.counter(ItemsCount::new(nodes))
         .with_inputs(|| new_engine(fen))
         .bench_local_values(|engine| {
           engine.go();
           // Return the engine so that it is dropped outside of the timing.
           engine
         });
}
