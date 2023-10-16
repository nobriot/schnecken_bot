use chess::engine::cache::EngineCache;
use chess::engine::eval::helpers::generic::get_combined_material_score;
use chess::engine::eval::position::*;
use chess::model::board::Board;
use chess::model::game_state::GameState;
use chess::model::moves::Move;

use divan::Bencher;
use rand::Rng;

fn main() {
  // Run registered benchmarks.
  divan::main();
}

/// Checks how fast we are evaluating the board
#[divan::bench]
fn board_evaluation(bencher: Bencher) {
  let cache = EngineCache::new();
  let mut game_state: GameState = GameState::from_board(&Board::new_random());
  let mut rng = rand::thread_rng();

  bencher.bench_local(|| {
    let _ = evaluate_board(&cache, &game_state);
  });
}

/// Benches the common part of all board evaluation (regardless of game phase)
#[divan::bench]
fn board_generic_evaluation(bencher: Bencher) {
  let mut game_state: GameState = GameState::from_board(&Board::new_random());
  let mut rng = rand::thread_rng();

  bencher.bench_local(|| {
    let _ = default_position_evaluation(&game_state);
  });
}

/// Benches the material calculation of the board evaluation
#[divan::bench]
fn board_material_score(bencher: Bencher) {
  let mut game_state: GameState = GameState::from_board(&Board::new_random());
  let mut rng = rand::thread_rng();

  bencher.bench_local(|| {
    let _ = get_combined_material_score(&game_state);
  });
}

/// Checks how fast we detect if the board is a game over situation
#[divan::bench]
fn detect_board_game_over(bencher: Bencher) {
  let cache = EngineCache::new();
  let mut game_state: GameState = GameState::from_board(&Board::new_random());
  let mut rng = rand::thread_rng();

  bencher.bench_local(|| {
    let _ = is_game_over(&cache, &game_state);
  });
}

/// Checks of fast we determine which game phase the board is at
#[divan::bench]
fn compute_game_phase(bencher: Bencher) {
  let cache = EngineCache::new();
  let mut game_state: GameState = GameState::from_board(&Board::new_random());

  bencher.bench_local(|| {
    let _ = determine_game_phase(&cache, &game_state);
  });
}

/// Checks how fast the cache is for storing/retrieving moves lists
#[divan::bench]
fn cache_for_moves(bencher: Bencher) {
  let cache: EngineCache = EngineCache::new();
  let mut game_state: GameState = GameState::from_board(&Board::new_random());
  let move_list = game_state.get_moves();

  bencher.bench_local(|| {
    if false == cache.has_move_list(&game_state.board) {
      cache.set_move_list(&game_state.board, &move_list);
    } else {
      assert_eq!(move_list, cache.get_move_list(&game_state.board));
    }
  });
}
