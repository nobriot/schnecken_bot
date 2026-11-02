use super::eval::position::{Score, determine_game_phase, is_mate_score};
use super::search_result::SearchResult;
use crate::model::game_state::{GamePhase, GameState};
use crate::model::piece::Color;

// -----------------------------------------------------------------------------
// Constants

/// Time kept for the network / GUI lag on each move (milliseconds)
pub const MOVE_OVERHEAD_MS: usize = 100;
/// Never search less than this (milliseconds)
pub const MIN_MOVE_TIME_MS: usize = 50;
/// Increments larger than this are not taken into account (milliseconds)
pub const MAX_INCREMENT_MS: usize = 60_000;
/// Share of the increment that we plan to spend on each move
const INCREMENT_USAGE: f32 = 0.8;
/// Estimated number of moves left in the game, by game phase. It stays high in
/// the endgame: converting a won endgame takes many moves.
const MOVES_LEFT_OPENING: usize = 50;
const MOVES_LEFT_MIDDLEGAME: usize = 40;
const MOVES_LEFT_ENDGAME: usize = 30;
/// Pawn endgames: one tempo decides the game, spend more time.
const PAWN_ENDGAME_FACTOR: f32 = 1.3;
/// How much the ratio of our time vs the opponent time influences the budget,
/// and the bounds of the resulting factor.
const OPPONENT_TIME_INFLUENCE: f32 = 0.5;
const OPPONENT_TIME_FACTOR_MIN: f32 = 0.8;
const OPPONENT_TIME_FACTOR_MAX: f32 = 1.2;
/// The hard limit is at most this many times the soft limit
const HARD_LIMIT_FACTOR: f32 = 4.0;
/// The hard limit is at most this fraction (1/n) of the time left
const HARD_LIMIT_TIME_LEFT_DIVIDER: usize = 5;

/// We do not start a new depth after this share of the soft limit: the next
/// depth takes several times longer than the previous one, it would likely not
/// finish.
pub const NEXT_DEPTH_SOFT_LIMIT_SHARE: f32 = 0.5;
/// Best move unchanged for that many depths: the answer is stable.
const STABLE_DEPTHS: usize = 3;
const STABLE_FACTOR: f32 = 0.7;
const VERY_STABLE_DEPTHS: usize = 5;
const VERY_STABLE_FACTOR: f32 = 0.5;
/// Best move changed at the last depth: unclear position.
const BEST_MOVE_CHANGED_FACTOR: f32 = 1.5;
/// Eval dropped since the previous depth: look for a defence.
const EVAL_DROP: i32 = 30;
const EVAL_DROP_FACTOR: f32 = 1.3;
const BIG_EVAL_DROP: i32 = 100;
const BIG_EVAL_DROP_FACTOR: f32 = 2.0;
/// Moves within this margin of the best move are considered close to it
const CLOSE_MOVE_MARGIN: i32 = 30;
/// Moves this much worse than the best move are considered bad
const BAD_MOVE_MARGIN: i32 = 150;
/// Easy move: the best move is much better than all the others, and stable.
const EASY_MOVE_FACTOR: f32 = 0.4;
/// Critical choice: a few moves are close, all the others are bad.
const CRITICAL_CHOICE_MAX_MOVES: usize = 4;
const CRITICAL_CHOICE_FACTOR: f32 = 1.3;
/// Quiet position: many moves are close, the choice matters little.
const QUIET_POSITION_MIN_MOVES: usize = 6;
const QUIET_POSITION_FACTOR: f32 = 0.8;
/// Few legal moves: little to choose from.
const FEW_LEGAL_MOVES: usize = 10;
const FEW_LEGAL_MOVES_FACTOR: f32 = 0.8;
/// Decisive eval: the result is unlikely to change.
const DECISIVE_EVAL: i32 = 1000;
const DECISIVE_EVAL_FACTOR: f32 = 0.7;
/// Bounds of the combined adjustment factors
const MIN_ADJUSTMENT: f32 = 0.25;
const MAX_ADJUSTMENT: f32 = 2.5;

// -----------------------------------------------------------------------------
// Type definitions

/// Clock situation of the side to play
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Clock {
  /// Time left on our clock (milliseconds)
  pub time_left_ms:          usize,
  /// Increment per move (milliseconds)
  pub increment_ms:          usize,
  /// Time left on the opponent clock (milliseconds), 0 if unknown
  pub opponent_time_left_ms: usize,
  /// Number of moves until the next time control, if any
  pub moves_to_go:           Option<usize>,
}

/// Time we can spend on a move
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimeBudget {
  /// Target time (milliseconds), checked between depths
  pub soft_ms: usize,
  /// Maximum time (milliseconds), never exceeded
  pub hard_ms: usize,
}

/// What the search told us after a completed depth.
/// All evals are from the point of view of the side to play.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IterationStats {
  /// Depth that was just completed
  pub depth:                   usize,
  /// Number of consecutive depths (including this one) with the same best move
  pub best_move_stable_depths: usize,
  /// How much the eval dropped since the previous depth (negative if it rose)
  pub eval_drop:               i32,
  /// Eval of the best move
  pub best_eval:               i32,
  /// Number of moves (best included) within `CLOSE_MOVE_MARGIN` of the best
  pub close_moves:             usize,
  /// How much worse than the best move the first move that is not close is.
  /// None if all the moves are close.
  pub gap_after_close:         Option<i32>,
  /// Number of legal moves in the position
  pub legal_moves:             usize,
}

/// Parameters of the UCI `go` command
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GoCommand {
  pub clock:     Option<Clock>,
  /// Fixed time per move (milliseconds)
  pub move_time: Option<usize>,
  pub depth:     Option<usize>,
  pub infinite:  bool,
  pub ponder:    bool,
}

// -----------------------------------------------------------------------------
// Functions

/// Converts a (white point of view) eval to the point of view of a side
fn for_side(eval: Score, color: Color) -> i32 {
  match color {
    Color::White => eval as i32,
    Color::Black => -(eval as i32),
  }
}

/// Number of moves we expect to play until the end of the game (or the next
/// time control)
fn estimated_moves_left(clock: &Clock, game_state: &GameState) -> usize {
  if let Some(moves_to_go) = clock.moves_to_go {
    return moves_to_go.max(1);
  }
  match determine_game_phase(game_state) {
    GamePhase::Opening => MOVES_LEFT_OPENING,
    GamePhase::Middlegame => MOVES_LEFT_MIDDLEGAME,
    GamePhase::Endgame => MOVES_LEFT_ENDGAME,
  }
}

/// Checks if only kings and pawns are left on the board
fn is_pawn_endgame(game_state: &GameState) -> bool {
  let pieces = &game_state.board.pieces;
  (pieces.queens() | pieces.rooks() | pieces.minors()) == 0
}

/// Computes how much time we can spend on a move
///
/// ### Arguments
///
/// * `clock`:       Clock situation of the side to play
/// * `game_state`:  Position to play
/// * `time_factor`: Multiplies the time we spend (< 1.0 to play quicker)
/// * `cap_ms`:      Maximum time to spend on the move, 0 for no cap
///
/// ### Return value
///
/// The soft and hard time limits for the move
pub fn time_budget(clock: &Clock,
                   game_state: &GameState,
                   time_factor: f32,
                   cap_ms: usize)
                   -> TimeBudget {
  let available = clock.time_left_ms.saturating_sub(MOVE_OVERHEAD_MS);
  let increment = clock.increment_ms.min(MAX_INCREMENT_MS);
  let moves_left = estimated_moves_left(clock, game_state);

  let mut soft = available as f32 / moves_left as f32 + increment as f32 * INCREMENT_USAGE;

  if is_pawn_endgame(game_state) {
    soft *= PAWN_ENDGAME_FACTOR;
  }

  // Use a bit more time when we have more time than the opponent, a bit less
  // otherwise.
  if clock.opponent_time_left_ms > 0 {
    let ratio = clock.time_left_ms as f32 / clock.opponent_time_left_ms as f32;
    soft *= (1.0 + (ratio - 1.0) * OPPONENT_TIME_INFLUENCE).clamp(OPPONENT_TIME_FACTOR_MIN,
                                                                  OPPONENT_TIME_FACTOR_MAX);
  }

  soft *= time_factor;

  let mut hard =
    ((soft * HARD_LIMIT_FACTOR) as usize).min(available / HARD_LIMIT_TIME_LEFT_DIVIDER);
  if cap_ms > 0 {
    hard = hard.min(cap_ms);
  }
  let hard = hard.max(MIN_MOVE_TIME_MS);
  let soft = (soft as usize).clamp(MIN_MOVE_TIME_MS, hard);

  TimeBudget { soft_ms: soft, hard_ms: hard }
}

impl IterationStats {
  /// Gathers the stats of a completed depth
  ///
  /// ### Arguments
  ///
  /// * `previous`:                Result of the previous depth, if any
  /// * `current`:                 Result of the depth that was just completed
  /// * `depth`:                   Depth that was just completed
  /// * `best_move_stable_depths`: Number of consecutive depths with the same best move
  /// * `legal_moves`:             Number of legal moves in the position
  /// * `side_to_play`:            Side to play in the position
  pub fn new(previous: Option<&SearchResult>,
             current: &SearchResult,
             depth: usize,
             best_move_stable_depths: usize,
             legal_moves: usize,
             side_to_play: Color)
             -> Self {
    let best_eval = current.get_eval().map_or(0, |e| for_side(e, side_to_play));
    let eval_drop =
      previous.and_then(|p| p.get_eval()).map_or(0, |e| for_side(e, side_to_play) - best_eval);

    let evals: Vec<i32> =
      current.variations.iter().map(|v| for_side(v.eval, side_to_play)).collect();
    let close_moves = evals.iter().take_while(|&&e| best_eval - e <= CLOSE_MOVE_MARGIN).count();
    let gap_after_close = evals.get(close_moves).map(|e| best_eval - e);

    IterationStats { depth,
                     best_move_stable_depths,
                     eval_drop,
                     best_eval,
                     close_moves,
                     gap_after_close,
                     legal_moves }
  }
}

/// Adjusts the soft limit of a move, based on what the search found so far
///
/// ### Arguments
///
/// * `soft_ms`: Soft limit computed at the start of the search
/// * `stats`:   Stats of the last completed depth
///
/// ### Return value
///
/// The adjusted soft limit, in milliseconds
pub fn adjust_soft_limit(soft_ms: usize, stats: &IterationStats) -> usize {
  let mut factor = 1.0;

  // Stability of the best move
  if stats.depth > 1 && stats.best_move_stable_depths <= 1 {
    factor *= BEST_MOVE_CHANGED_FACTOR;
  } else if stats.best_move_stable_depths >= VERY_STABLE_DEPTHS {
    factor *= VERY_STABLE_FACTOR;
  } else if stats.best_move_stable_depths >= STABLE_DEPTHS {
    factor *= STABLE_FACTOR;
  }

  // Eval dropping
  if stats.eval_drop > BIG_EVAL_DROP {
    factor *= BIG_EVAL_DROP_FACTOR;
  } else if stats.eval_drop > EVAL_DROP {
    factor *= EVAL_DROP_FACTOR;
  }

  // How many moves are worth considering
  let all_others_bad = stats.gap_after_close.is_some_and(|gap| gap >= BAD_MOVE_MARGIN);
  if stats.close_moves == 1 && all_others_bad && stats.best_move_stable_depths >= 2 {
    factor *= EASY_MOVE_FACTOR;
  } else if (2..=CRITICAL_CHOICE_MAX_MOVES).contains(&stats.close_moves) && all_others_bad {
    factor *= CRITICAL_CHOICE_FACTOR;
  } else if stats.close_moves >= QUIET_POSITION_MIN_MOVES {
    factor *= QUIET_POSITION_FACTOR;
  }

  if stats.legal_moves < FEW_LEGAL_MOVES {
    factor *= FEW_LEGAL_MOVES_FACTOR;
  }

  if stats.best_eval.abs() > DECISIVE_EVAL || is_mate_score(stats.best_eval as Score) {
    factor *= DECISIVE_EVAL_FACTOR;
  }

  (soft_ms as f32 * factor.clamp(MIN_ADJUSTMENT, MAX_ADJUSTMENT)) as usize
}

/// Parses the parameters of a UCI `go` command
///
/// ### Arguments
///
/// * `line`:         The full command, e.g. `go wtime 60000 btime 60000 winc 1000 binc 1000`
/// * `side_to_play`: Side to play in the position, to pick its clock
pub fn parse_go_command(line: &str, side_to_play: Color) -> GoCommand {
  let mut command = GoCommand::default();
  let (mut wtime, mut btime, mut winc, mut binc, mut moves_to_go) = (None, None, 0, 0, None);

  let mut tokens = line.split_whitespace().skip(1);
  while let Some(token) = tokens.next() {
    let mut value = || tokens.next().and_then(|v| v.parse::<i64>().ok()).map(|v| v.max(0) as usize);
    match token {
      "wtime" => wtime = value(),
      "btime" => btime = value(),
      "winc" => winc = value().unwrap_or(0),
      "binc" => binc = value().unwrap_or(0),
      "movestogo" => moves_to_go = value(),
      "movetime" => command.move_time = value(),
      "depth" => command.depth = value(),
      "infinite" => command.infinite = true,
      "ponder" => command.ponder = true,
      _ => {},
    }
  }

  let (time_left, increment, opponent_time_left) = match side_to_play {
    Color::White => (wtime, winc, btime),
    Color::Black => (btime, binc, wtime),
  };
  command.clock = time_left.map(|time_left_ms| Clock { time_left_ms,
                                                       increment_ms: increment,
                                                       opponent_time_left_ms:
                                                         opponent_time_left.unwrap_or(0),
                                                       moves_to_go });
  command
}
