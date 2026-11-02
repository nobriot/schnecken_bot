//------------------------------------------------------------------------------
// Time management tests
use crate::engine::search_result::{SearchResult, VariationWithEval};
use crate::engine::time_management::*;
use crate::engine::*;
use crate::model::game_state::{GameState, START_POSITION_FEN};
use crate::model::moves::Move;
use crate::model::piece::Color;

const MIDDLEGAME_FEN: &str = "r1bq1rk1/pp2bppp/2n1pn2/3p4/2PP4/2N1PN2/PP2BPPP/R2QKB1R w KQ - 0 8";
const ENDGAME_FEN: &str = "8/5k2/8/8/8/8/3R4/4K3 w - - 0 50";
const PAWN_ENDGAME_FEN: &str = "8/5kp1/8/8/8/8/3P2K1/8 w - - 0 50";

fn clock(time_left_ms: usize, increment_ms: usize) -> Clock {
  Clock { time_left_ms,
          increment_ms,
          opponent_time_left_ms: time_left_ms,
          moves_to_go: None }
}

fn budget(clock: Clock, fen: &str) -> TimeBudget {
  time_budget(&clock, &GameState::from_fen(fen), 1.0, 0)
}

#[test]
fn time_budget_stays_within_bounds() {
  for fen in [START_POSITION_FEN, MIDDLEGAME_FEN, ENDGAME_FEN, PAWN_ENDGAME_FEN] {
    for time_left in [0, 50, 500, 3_000, 10_000, 60_000, 180_000, 600_000, 3_600_000] {
      for increment in [0, 1_000, 5_000, 120_000] {
        let b = budget(clock(time_left, increment), fen);
        let available = time_left.saturating_sub(MOVE_OVERHEAD_MS);
        assert!(b.soft_ms >= MIN_MOVE_TIME_MS, "{b:?} for {time_left}+{increment}");
        assert!(b.soft_ms <= b.hard_ms, "{b:?} for {time_left}+{increment}");
        assert!(b.hard_ms <= (available / 5).max(MIN_MOVE_TIME_MS),
                "{b:?} for {time_left}+{increment}");
      }
    }
  }
}

#[test]
fn time_budget_grows_with_time_and_increment() {
  let mut previous = budget(clock(1_000, 0), MIDDLEGAME_FEN);
  for time_left in [5_000, 30_000, 60_000, 300_000] {
    let b = budget(clock(time_left, 0), MIDDLEGAME_FEN);
    assert!(b.soft_ms >= previous.soft_ms && b.hard_ms >= previous.hard_ms);
    previous = b;
  }

  let mut previous = budget(clock(60_000, 0), MIDDLEGAME_FEN);
  for increment in [500, 1_000, 3_000, 10_000] {
    let b = budget(clock(60_000, increment), MIDDLEGAME_FEN);
    assert!(b.soft_ms >= previous.soft_ms && b.hard_ms >= previous.hard_ms);
    previous = b;
  }
}

#[test]
fn time_budget_endgame_keeps_time_for_the_conversion() {
  let b = budget(clock(60_000, 0), ENDGAME_FEN);
  // At least 25 moves left are expected in the endgame
  assert!(b.soft_ms <= 60_000 / 25, "{b:?}");
  // But we still spend more time than in the opening, where we expect more moves
  assert!(b.soft_ms > budget(clock(60_000, 0), START_POSITION_FEN).soft_ms);
}

#[test]
fn time_budget_pawn_endgame_spends_more() {
  assert!(budget(clock(60_000, 0), PAWN_ENDGAME_FEN).soft_ms
          > budget(clock(60_000, 0), ENDGAME_FEN).soft_ms);
}

#[test]
fn time_budget_low_time_uses_increment() {
  let without_increment = budget(clock(5_000, 0), MIDDLEGAME_FEN);
  let with_increment = budget(clock(5_000, 2_000), MIDDLEGAME_FEN);
  assert!(with_increment.soft_ms > 5 * without_increment.soft_ms,
          "{with_increment:?} vs {without_increment:?}");
}

#[test]
fn time_budget_moves_to_go() {
  let mut c = clock(10_000, 0);
  let sudden_death = budget(c, MIDDLEGAME_FEN);
  c.moves_to_go = Some(2);
  let last_moves = budget(c, MIDDLEGAME_FEN);
  assert!(last_moves.soft_ms > 5 * sudden_death.soft_ms);
}

#[test]
fn time_budget_cap_and_factor() {
  let game_state = GameState::from_fen(MIDDLEGAME_FEN);
  let c = clock(3_600_000, 0);
  let b = time_budget(&c, &game_state, 1.0, 0);
  assert!(b.hard_ms > 60_000);

  let capped = time_budget(&c, &game_state, 1.0, 60_000);
  assert_eq!(60_000, capped.hard_ms);
  assert!(capped.soft_ms <= 60_000);

  let quicker = time_budget(&c, &game_state, 0.75, 0);
  assert!(quicker.soft_ms < b.soft_ms);
}

#[test]
fn time_budget_opponent_time() {
  let mut c = clock(60_000, 0);
  let equal = budget(c, MIDDLEGAME_FEN);
  c.opponent_time_left_ms = 30_000;
  let ahead = budget(c, MIDDLEGAME_FEN);
  c.opponent_time_left_ms = 120_000;
  let behind = budget(c, MIDDLEGAME_FEN);
  assert!(ahead.soft_ms > equal.soft_ms);
  assert!(behind.soft_ms < equal.soft_ms);
  // The influence stays mild
  assert!(ahead.soft_ms <= equal.soft_ms * 13 / 10);
  assert!(behind.soft_ms >= equal.soft_ms * 7 / 10);
}

/// Stats of a normal position: nothing special
fn neutral_stats() -> IterationStats {
  IterationStats { depth:                   4,
                   best_move_stable_depths: 2,
                   eval_drop:               0,
                   best_eval:               20,
                   close_moves:             3,
                   gap_after_close:         Some(40),
                   legal_moves:             30, }
}

#[test]
fn adjust_soft_limit_neutral() {
  assert_eq!(1000, adjust_soft_limit(1000, &neutral_stats()));
}

#[test]
fn adjust_soft_limit_stability() {
  let mut stats = neutral_stats();
  stats.best_move_stable_depths = 3;
  let stable = adjust_soft_limit(1000, &stats);
  stats.best_move_stable_depths = 6;
  let very_stable = adjust_soft_limit(1000, &stats);
  stats.best_move_stable_depths = 1;
  let changed = adjust_soft_limit(1000, &stats);
  assert!(very_stable < stable && stable < 1000 && changed > 1000);
}

#[test]
fn adjust_soft_limit_eval_drop() {
  let mut stats = neutral_stats();
  stats.eval_drop = 50;
  let drop = adjust_soft_limit(1000, &stats);
  stats.eval_drop = 200;
  let big_drop = adjust_soft_limit(1000, &stats);
  stats.eval_drop = -200;
  let rise = adjust_soft_limit(1000, &stats);
  assert!(1000 < drop && drop < big_drop);
  assert_eq!(1000, rise);
}

#[test]
fn adjust_soft_limit_move_choice() {
  let mut stats = neutral_stats();
  // Recapture: one move, the others lose material
  stats.close_moves = 1;
  stats.gap_after_close = Some(300);
  assert!(adjust_soft_limit(1000, &stats) < 500);

  // Only a few moves hold
  stats.close_moves = 3;
  assert!(adjust_soft_limit(1000, &stats) > 1000);

  // Many equivalent moves
  stats.close_moves = 10;
  stats.gap_after_close = Some(10);
  assert!(adjust_soft_limit(1000, &stats) < 1000);

  // Decisive eval
  let mut stats = neutral_stats();
  stats.best_eval = -1500;
  assert!(adjust_soft_limit(1000, &stats) < 1000);

  // Few legal moves
  let mut stats = neutral_stats();
  stats.legal_moves = 4;
  assert!(adjust_soft_limit(1000, &stats) < 1000);
}

#[test]
fn iteration_stats_from_search_results() {
  // Black to play: evals are from white point of view
  let mut previous = SearchResult::new(5, Color::Black);
  previous.update(VariationWithEval::new_from_move(-100, Move::from_string("e7e5")));
  let mut current = SearchResult::new(5, Color::Black);
  for (eval, mv) in [(-50, "e7e5"), (-40, "d7d5"), (-30, "c7c5"), (200, "a7a6"), (300, "h7h6")] {
    current.update(VariationWithEval::new_from_move(eval, Move::from_string(mv)));
  }

  let stats = IterationStats::new(Some(&previous), &current, 5, 2, 20, Color::Black);
  assert_eq!(50, stats.best_eval);
  assert_eq!(50, stats.eval_drop);
  assert_eq!(3, stats.close_moves);
  assert_eq!(Some(250), stats.gap_after_close);
  assert_eq!(5, stats.depth);
  assert_eq!(2, stats.best_move_stable_depths);
  assert_eq!(20, stats.legal_moves);
}

#[test]
fn parse_go_command_clock() {
  let go =
    parse_go_command("go wtime 60000 btime 30000 winc 1000 binc 2000 movestogo 20", Color::White);
  assert_eq!(Some(Clock { time_left_ms:          60_000,
                          increment_ms:          1_000,
                          opponent_time_left_ms: 30_000,
                          moves_to_go:           Some(20), }),
             go.clock);
  assert!(!go.infinite);

  let go = parse_go_command("go wtime 60000 btime 30000 winc 1000 binc 2000", Color::Black);
  assert_eq!(Some(Clock { time_left_ms:          30_000,
                          increment_ms:          2_000,
                          opponent_time_left_ms: 60_000,
                          moves_to_go:           None, }),
             go.clock);
}

#[test]
fn parse_go_command_other_parameters() {
  let go = parse_go_command("go movetime 1500 depth 8", Color::White);
  assert_eq!(None, go.clock);
  assert_eq!(Some(1500), go.move_time);
  assert_eq!(Some(8), go.depth);

  let go = parse_go_command("go infinite", Color::White);
  assert!(go.infinite);
  assert_eq!(None, go.clock);

  // Negative times can be sent by GUIs when we are out of time
  let go = parse_go_command("go ponder wtime -20 btime 1000", Color::White);
  assert!(go.ponder);
  assert_eq!(0, go.clock.unwrap().time_left_ms);
}

#[test]
fn engine_plays_easy_move_quickly() {
  // Knight takes the queen, everything else loses it
  let mut engine = Engine::new(false);
  engine.set_position("4k3/8/8/3q4/8/2N5/8/4K3 w - - 0 1");
  engine.options.clock = Some(clock(60_000, 0));
  let budget = engine.time_budget().unwrap();

  let start = std::time::Instant::now();
  engine.go();
  let elapsed = start.elapsed().as_millis() as usize;

  assert_eq!("c3d5", engine.get_best_move().unwrap().to_string());
  assert!(elapsed < budget.soft_ms, "searched {elapsed}ms, budget {budget:?}");
}
