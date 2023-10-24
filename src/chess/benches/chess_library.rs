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

/// Compare how fast we are with the chess crate
/// from a given board position
#[divan::bench(sample_count = 10000)]
fn compute_legal_moves_external_chess_library(bencher: Bencher) {
  // Create a bunch of random boards
  extern crate chess_lib;
  use std::str::FromStr;

  let fen = GameState::from_board(&Board::new_random()).to_fen();
  let board =
    chess_lib::Board::from_str(fen.as_str()).expect(format!("Valid FEN {}", fen).as_str());
  bencher.bench_local(|| {
    let movegen = chess_lib::MoveGen::new_legal(&board);
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

/// Checks how fast we are at computing attackers of a square on the board
#[divan::bench(sample_count = 10000)]
fn determine_pins_for_square(bencher: Bencher) {
  let mut game_state: GameState = GameState::from_board(&Board::new_random());
  let mut rng = rand::thread_rng();

  for i in 0..63 {
    if game_state.board.has_piece(i) {
      continue;
    }
    bencher.bench_local(|| {
      let _ = game_state.board.get_pins(i);
    });
    break;
  }
}
