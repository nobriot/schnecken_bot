use chess::engine::cache::engine_cache::EngineCache;
use chess::engine::cache::evaluation_table::EvaluationCache;
use chess::engine::eval::endgame::*;
use chess::engine::eval::helpers::generic::get_combined_material_score;
use chess::engine::eval::helpers::generic::*;
use chess::engine::eval::helpers::pawn::is_passed;
use chess::engine::eval::middlegame::*;
use chess::engine::eval::opening::*;
use chess::engine::eval::position::*;
use chess::engine::nnue::*;
use chess::model::board::Board;
use chess::model::game_state::GameState;
use chess::model::game_state::GameStatus;
use chess::model::moves::Move;
use chess::model::piece::Color;

use divan::Bencher;

fn main() {
  // Run registered benchmarks.
  divan::main();
}

/// Checks how fast we are evaluating the board
#[divan::bench(sample_count = 10000)]
fn board_evaluation(bencher: Bencher) {
  let mut game_state: GameState = GameState::from_board(&Board::new_random());

  bencher.bench_local(|| {
    let _ = evaluate_board(&game_state);
  });
}

/// Checks how fast we are evaluating the board in the opening
#[divan::bench(sample_count = 10000)]
fn board_evaluation_2(bencher: Bencher) {
  let mut game_state: GameState = GameState::from_board(&Board::new_random());

  bencher.bench_local(|| {
    let _ = evaluate_board(&game_state);
  });
}

/// Checks how fast we are evaluating the board in the opening
#[divan::bench(sample_count = 10000)]
fn opening_evaluation(bencher: Bencher) {
  let mut game_state: GameState = GameState::from_board(&Board::new_random());

  bencher.bench_local(|| {
    let _ = get_opening_position_evaluation(&game_state);
  });
}

/// Checks how fast we are evaluating the board in the middlegame
#[divan::bench(sample_count = 10000)]
fn middlegame_evaluation(bencher: Bencher) {
  let mut game_state: GameState = GameState::from_board(&Board::new_random());

  bencher.bench_local(|| {
    let _ = get_middlegame_position_evaluation(&game_state);
  });
}

/// Checks how fast we are evaluating the board in the endgame
#[divan::bench(sample_count = 10000)]
fn endgame_evaluation(bencher: Bencher) {
  let mut game_state: GameState = GameState::from_board(&Board::new_random());

  bencher.bench_local(|| {
    let _ = get_endgame_position_evaluation(&game_state);
  });
}

/// Benches the common part of all board evaluation (regardless of game phase)
#[divan::bench(sample_count = 10000)]
fn board_default_position_evaluation(bencher: Bencher) {
  let mut game_state: GameState = GameState::from_board(&Board::new_random());

  bencher.bench_local(|| {
    let _ = default_position_evaluation(&game_state);
  });
}

/// Benches the material calculation of the board evaluation
#[divan::bench(sample_count = 10000)]
fn board_material_score(bencher: Bencher) {
  let mut game_state: GameState = GameState::from_board(&Board::new_random());

  bencher.bench_local(|| {
    let _ = get_combined_material_score(&game_state);
  });
}

/// Checks how fast we detect if the board is a game over situation
#[divan::bench(sample_count = 10000)]
fn detect_board_game_over(bencher: Bencher) {
  let cache = EngineCache::new();
  let mut game_state: GameState = GameState::from_board(&Board::new_random());

  bencher.bench_local(|| {
    let _ = is_game_over(&cache, &game_state.board);
  });
}

/// Checks of fast we determine which game phase the board is at
#[divan::bench(sample_count = 10000)]
fn compute_game_phase(bencher: Bencher) {
  let mut game_state: GameState = GameState::from_board(&Board::new_random());

  bencher.bench_local(|| {
    let _ = determine_game_phase(&game_state);
  });
}

/// Checks how fast the cache is for storing/retrieving moves lists
#[divan::bench(sample_count = 10000)]
fn cache_for_moves(bencher: Bencher) {
  let cache: EngineCache = EngineCache::new();
  let game_state: GameState = GameState::from_board(&Board::new_random());
  let move_list = game_state.get_moves();

  bencher.bench_local(|| {
    if false == cache.has_move_list(&game_state.board) {
      cache.set_move_list(&game_state.board, &move_list);
    } else {
      assert_eq!(
        move_list,
        cache.get_move_list(&game_state.board).unwrap().to_vec()
      );
    }
  });
}

/// Checks how fast the cache is for storing/retrieving moves lists
#[divan::bench(sample_count = 10000)]
fn cache_for_game_status(bencher: Bencher) {
  let cache: EngineCache = EngineCache::new();
  let game_state: GameState = GameState::from_board(&Board::new_random());
  let game_status = is_game_over(&cache, &game_state.board);

  bencher.bench_local(|| {
    let eval_cache = cache.get_eval(&game_state.board).unwrap_or_default();
    if eval_cache.depth == 0 {
      cache.set_eval(
        &game_state.board,
        EvaluationCache {
          game_status,
          eval: f32::NAN,
          depth: 1,
        },
      );
    } else {
      assert_eq!(game_status, eval_cache.game_status);
    }
  });
}

/// Checks how fast the cache is for storing/retrieving evaluations
#[divan::bench(sample_count = 10000)]
fn cache_for_evals(bencher: Bencher) {
  let cache: EngineCache = EngineCache::new();
  let game_state: GameState = GameState::from_board(&Board::new_random());
  let eval = evaluate_board(&game_state);

  assert!(
    !eval.is_nan(),
    "Evaluate board returned NaN eval for : {}",
    game_state.to_fen()
  );

  bencher.bench_local(|| {
    let eval_cache = cache.get_eval(&game_state.board);
    if eval_cache.is_none() {
      cache.set_eval(
        &game_state.board,
        EvaluationCache {
          game_status: GameStatus::Ongoing,
          eval,
          depth: 1,
        },
      );
    } else {
      let eval_cache = eval_cache.unwrap();
      if eval_cache.eval.is_nan() {
        assert!(
          false,
          "Eval is Nan For board: {} - hash {}",
          &game_state.to_fen(),
          &game_state.board.hash
        )
      }
      assert_eq!(eval, eval_cache.eval);
    }
  });
}

/// Checks how fast the piece square table lookup is - Opening
#[divan::bench(sample_count = 10000)]
fn opening_piece_square_table_lookup(bencher: Bencher) {
  let cache: EngineCache = EngineCache::new();
  let mut game_state: GameState = GameState::from_board(&Board::new_random());
  let move_list = game_state.get_moves();

  bencher.bench_local(|| {
    let _ = get_square_table_opening_score(&game_state);
  });
}

/// Checks how fast the piece square table lookup is - Middlegame
#[divan::bench(sample_count = 10000)]
fn middlegame_piece_square_table_lookup(bencher: Bencher) {
  let cache: EngineCache = EngineCache::new();
  let mut game_state: GameState = GameState::from_board(&Board::new_random());
  let move_list = game_state.get_moves();

  bencher.bench_local(|| {
    let _ = get_square_table_middlegame_score(&game_state);
  });
}

/// Checks how fast the piece square table lookup is - Endgame
#[divan::bench(sample_count = 10000)]
fn endgame_piece_square_table_lookup(bencher: Bencher) {
  let cache: EngineCache = EngineCache::new();
  let mut game_state: GameState = GameState::from_board(&Board::new_random());
  let move_list = game_state.get_moves();

  bencher.bench_local(|| {
    let _ = get_square_table_endgame_score(&game_state);
  });
}

/// Checks how fast we convert a game state to NNUE input layer
#[divan::bench(sample_count = 10000)]
fn nnue_input_layer_conversion(bencher: Bencher) {
  let mut nnue = NNUE::default();
  let cache: EngineCache = EngineCache::new();
  let mut game_state: GameState = GameState::from_board(&Board::new_random());

  bencher.bench_local(|| {
    nnue.game_state_to_input_layer(&vec![&game_state]);
  });
}

/// Checks how fast the NNUE is evaluating a board position
#[divan::bench(sample_count = 10000)]
fn nnue_board_evaluation(bencher: Bencher) {
  let mut nnue = NNUE::default();
  let cache: EngineCache = EngineCache::new();
  let mut game_state: GameState = GameState::from_board(&Board::new_random());

  bencher.bench_local(|| {
    let _ = nnue.eval(&game_state);
  });
}

/// Checks how fast some of the evaluation functions are
#[divan::bench(sample_count = 10000)]
fn file_open_detection(bencher: Bencher) {
  let mut nnue = NNUE::default();
  let cache: EngineCache = EngineCache::new();
  let mut game_state: GameState = GameState::from_board(&Board::new_random());

  bencher.bench_local(|| {
    let _ = is_file_open(&game_state, 3);
  });
}

/// Checks how fast some of the evaluation functions are
#[divan::bench(sample_count = 10000)]
fn file_half_open_detection(bencher: Bencher) {
  let mut nnue = NNUE::default();
  let cache: EngineCache = EngineCache::new();
  let mut game_state: GameState = GameState::from_board(&Board::new_random());

  bencher.bench_local(|| {
    let _ = is_file_half_open(&game_state, 3);
  });
}

/// Checks how fast some of the evaluation functions are
#[divan::bench(sample_count = 10000)]
fn file_state_detection(bencher: Bencher) {
  let mut nnue = NNUE::default();
  let cache: EngineCache = EngineCache::new();
  let mut game_state: GameState = GameState::from_board(&Board::new_random());

  bencher.bench_local(|| {
    let _ = get_file_state(&game_state, 3);
  });
}

/// Checks how fast we detect the list of passed pawns in a board
#[divan::bench(sample_count = 10000)]
fn passed_pawn_detection(bencher: Bencher) {
  let mut nnue = NNUE::default();
  let cache: EngineCache = EngineCache::new();
  let mut game_state: GameState = GameState::from_board(&Board::new_random());

  bencher.bench_local(|| {
    let mut pawns = game_state.board.pieces.black.pawn;
    let mut passed_pawns_score = 0;
    while pawns != 0 {
      let pawn = pawns.trailing_zeros() as usize;

      if is_passed(&game_state, pawn as u8) {
        passed_pawns_score += 1;
      }

      pawns &= pawns - 1
    }
  });
}

/// Checks how fast we are evaluating the board in the "simple" mode
#[divan::bench(sample_count = 10000)]
fn board_simple_evaluation(bencher: Bencher) {
  let mut game_state: GameState = GameState::from_board(&Board::new_random());

  bencher.bench_local(|| {
    let _ = evaluate_board_simple(&game_state);
  });
}
