use chess::model::board::Board;
use chess::model::game_state::GameState;
use chess::model::moves::Move;
use chess::model::piece::Color;
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
fn apply_moves_on_a_game_state(bencher: Bencher) {
  const NUMBER_OF_BOARDS: usize = 10_000;
  let mut game_states: Vec<GameState> = Vec::with_capacity(NUMBER_OF_BOARDS);
  let mut move_lists: Vec<Vec<Move>> = Vec::with_capacity(NUMBER_OF_BOARDS);
  for i in 0..NUMBER_OF_BOARDS {
    game_states.push(GameState::from_board(&Board::new_random()));
    move_lists.push(game_states[i].board.get_moves());
  }

  let mut rng = rand::thread_rng();

  bencher.bench_local(|| {
           let i = rng.gen_range(0..NUMBER_OF_BOARDS);
           if !move_lists[i].is_empty() {
             let j = rng.gen_range(0..move_lists[i].len());
             let old_game = game_states[i].clone();
             game_states[i].apply_move(&move_lists[i][j]);
             game_states[i] = old_game;
           }
         });
}

/// Checks how fast we are at applying moves on a board
#[divan::bench(sample_count = 10000)]
fn apply_moves_on_the_board(bencher: Bencher) {
  const NUMBER_OF_BOARDS: usize = 10_000;
  let mut boards: Vec<Board> = Vec::with_capacity(NUMBER_OF_BOARDS);
  let mut move_lists: Vec<Vec<Move>> = Vec::with_capacity(NUMBER_OF_BOARDS);
  for i in 0..NUMBER_OF_BOARDS {
    boards.push(Board::new_random());
    move_lists.push(boards[i].get_moves());
  }

  let mut rng = rand::thread_rng();

  bencher.bench_local(|| {
           let i = rng.gen_range(0..NUMBER_OF_BOARDS);
           if !move_lists[i].is_empty() {
             let j = rng.gen_range(0..move_lists[i].len());
             let old_board = boards[i].clone();
             boards[i].apply_move(&move_lists[i][j]);
             boards[i] = old_board;
           }
         });
}

/// Checks how fast we are at computing attackers of a square on the board
#[divan::bench(sample_count = 10000)]
fn find_attackers(bencher: Bencher) {
  let mut game_state: GameState = GameState::from_board(&Board::new_random());
  let mut rng = rand::thread_rng();

  let j = rng.gen_range(0..63);

  bencher.bench_local(|| {
           let _ = game_state.board.get_attackers(j, Color::White);
         });
}

/// Checks how fast we are at computing pins for the board
#[divan::bench(sample_count = 10000)]
fn determine_board_pins(bencher: Bencher) {
  let mut game_state: GameState = GameState::from_board(&Board::new_random());
  let mut rng = rand::thread_rng();

  for i in 0..63 {
    if game_state.board.has_piece(i) {
      continue;
    }
    bencher.bench_local(|| {
             let _ = game_state.board.get_pins_rays(Color::White);
           });
    break;
  }
}
