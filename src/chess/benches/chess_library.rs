use chess::model::board::Board;
use chess::model::game_state::GameState;
use chess::model::moves::Move;
use divan::Bencher;
use rand::Rng;

fn main() {
  // Run registered benchmarks.
  divan::main();
}

/// Checks how fast we are at deriving legal moves
/// from a given board position
#[divan::bench(sample_count = 10000)]
fn compute_legal_moves(bencher: Bencher) {
  // Create a bunch of random boards
  const NUMBER_OF_BOARDS: usize = 10_000;
  let mut game_states: Vec<GameState> = Vec::with_capacity(NUMBER_OF_BOARDS);
  for _ in 0..NUMBER_OF_BOARDS {
    game_states.push(GameState::from_board(&Board::new_random()));
  }

  let mut rng = rand::thread_rng();

  bencher.bench_local(|| {
    let i = rng.gen_range(0..NUMBER_OF_BOARDS);
    let _ = game_states[i].get_moves();
  });
}

/// Checks how fast we are at applying moves on a board
#[divan::bench(sample_count = 10000)]
fn apply_moves_on_the_board(bencher: Bencher) {
  let mut game_state: GameState = GameState::from_board(&Board::new_random());
  let mut rng = rand::thread_rng();

  let moves = game_state.get_moves();
  if !moves.is_empty() {
    let j = rng.gen_range(0..moves.len());

    bencher.bench_local(|| {
      game_state.apply_move(&moves[j]);
    });
  } else {
    game_state = GameState::from_board(&Board::new_random());
  }
}
