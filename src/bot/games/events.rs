//! Keeps track of what happened during a game, to decide what to say in the
//! chat and when to do special things like resigning.
use chess::engine::eval::position::mate_distance_in_plies;
use chess::model::board::Board;

// -----------------------------------------------------------------------------
// Messages

static BOT_VERSION: &str = env!("CARGO_PKG_VERSION");
static BOT_NAME: &str = env!("CARGO_PKG_NAME");
pub static HAVE_FUN: &str = "Have fun";
pub static WELL_PLAYED: &str = "Well played!";

/// Greeting message at the start of the game
pub fn greeting(opponent_name: &str) -> String {
  format!("Hello {opponent_name}, I am {BOT_NAME} v{BOT_VERSION}")
}

/// Message when we found a forced mate
pub fn mate_announcement(moves: u32) -> String {
  format!("I smell some mate around the corner (#{moves})")
}

/// Lichess sends the username of titled opponents with their title, e.g.
/// "BOT maia1". Usernames cannot contain spaces, so we keep the last word.
pub fn display_name(username: &str) -> &str {
  username.split_whitespace().last().unwrap_or(username)
}

// -----------------------------------------------------------------------------
// Game events

/// What to do after a search, based on the eval
#[derive(Debug, PartialEq, Eq)]
pub enum EvalOutcome {
  /// Nothing special
  None,
  /// We just found a forced mate, in this number of moves
  AnnounceMate(u32),
  /// The opponent mates us in one whatever we play
  Resign,
}

/// What already happened during the game
#[derive(Debug, Default)]
pub struct GameEvents {
  /// We said "Have fun" when the game started
  have_fun_sent:  bool,
  /// We announced that we found a forced mate
  mate_announced: bool,
}

impl GameEvents {
  /// Checks if the game just started, so we should say "Have fun".
  /// Only true once, and only on the first moves, so that we do not say it
  /// when joining an ongoing game (e.g. after a restart).
  ///
  /// ### Arguments
  ///
  /// * `move_count` - Number of moves (plies) played in the game
  pub fn on_moves_played(&mut self, move_count: usize) -> bool {
    if self.have_fun_sent || !(1..=2).contains(&move_count) {
      return false;
    }
    self.have_fun_sent = true;
    true
  }

  /// Decides what to do after a search, based on the best eval.
  ///
  /// ### Arguments
  ///
  /// * `eval_for_us`     - Best eval, from our point of view (positive is good
  ///   for us)
  /// * `opponent_is_bot` - We only resign against bots
  pub fn after_search(&mut self, eval_for_us: f32, opponent_is_bot: bool) -> EvalOutcome {
    let Some(plies) = mate_distance_in_plies(eval_for_us) else {
      return EvalOutcome::None;
    };

    if eval_for_us > 0.0 {
      if self.mate_announced {
        return EvalOutcome::None;
      }
      self.mate_announced = true;
      // Our moves left: we play the 1st, 3rd, 5th... ply
      return EvalOutcome::AnnounceMate(plies.div_ceil(2));
    }

    // We play a move, and the opponent mates us with the next one
    if plies == 2 && opponent_is_bot {
      return EvalOutcome::Resign;
    }
    EvalOutcome::None
  }
}

/// Checks if every legal move of the side to play allows a mate in one.
/// Used to confirm the engine eval before resigning: cached mate scores can be
/// off by a ply or two.
///
/// ### Arguments
///
/// * `board` - Board position, with us to play
///
/// ### Returns
///
/// True if all our moves lose to a mate in one. False if we have no legal
/// move (game is already over).
pub fn is_forced_mate_in_one_against(board: &Board) -> bool {
  let our_moves = board.get_moves();
  if our_moves.is_empty() {
    return false;
  }

  our_moves.iter().all(|our_move| {
                    let mut after_our_move = *board;
                    after_our_move.apply_move(our_move);
                    after_our_move.get_moves().iter().any(|reply| {
                                                       let mut after_reply = after_our_move;
                                                       after_reply.apply_move(reply);
                                                       after_reply.checkers != 0
                                                       && after_reply.get_moves().is_empty()
                                                     })
                  })
}
