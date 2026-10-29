//! Keeps track of what happened during a game, to decide what to say in the
//! chat and when to do special things like resigning.
use chess::engine::eval::position::{EpicMate, mate_distance_in_plies};
use chess::model::board::Board;
use lichess::types::{ChatLine, ChatRoom, Color};
use std::collections::BTreeSet;

// -----------------------------------------------------------------------------
// Messages

static BOT_VERSION: &str = env!("CARGO_PKG_VERSION");
static BOT_NAME: &str = env!("CARGO_PKG_NAME");
pub static HAVE_FUN: &str = "Have fun";
pub static WELL_PLAYED: &str = "Well played!";
pub static GOOD_GAME: &str = "Good game!";
pub static THANKS_FOR_PLAYING: &str = "Thanks for playing ! :)";
pub static QUIET_ACKNOWLEDGED: &str = "Okay, I will stop commenting";
pub static CHATTY_ACKNOWLEDGED: &str = "Alright, I will comment the game again";
pub static NO_EVAL_YET: &str = "No eval yet, I have not thought about this game yet";
pub static IN_TROUBLE: &str = "oh oh, I think I am in trouble";
pub static OUT_OF_TROUBLE: &str = "Maybe not so much in trouble after all";
pub static SWINDLE: &str = "Wow, this is a swindle!";
pub static ATTEMPTING_TO_FLAG: &str = "Attempting to flag!";

/// We try to flag the opponent only when they have less time than this
pub const FLAG_MAX_OPPONENT_TIME_MS: usize = 60_000;
/// We start trying to flag when the eval (for us) is this bad or worse
pub const FLAG_START_EVAL: f32 = -2.5;
/// We stop trying to flag when the eval (for us) is this good or better
pub const FLAG_STOP_EVAL: f32 = 2.5;
/// Time per move when flagging and the opponent has a minute left
pub const FLAG_MOVE_TIME_MAX_MS: usize = 500;
/// Time per move when flagging and the opponent is about to run out of time
pub const FLAG_MOVE_TIME_MIN_MS: usize = 200;
pub static MATE_ANNOUNCEMENT: &str = "Woo-hoo, mate around the corner";
/// Bots with chat commands usually advertise them with `!help`
pub static OPPONENT_HELP_COMMAND: &str = "!help";
/// Asks such an opponent bot for its eval
pub static ASK_OPPONENT_EVAL: &str = "!eval";

/// Eval (against us) from which we consider being in trouble
const IN_TROUBLE_EVAL: f32 = -3.0;
/// Eval from which we are back to a 0-ish (or better) position after being in
/// trouble
const OUT_OF_TROUBLE_EVAL: f32 = -0.5;

/// Greeting message at the start of the game
pub fn greeting(opponent_name: &str) -> String {
  format!("Hello {opponent_name}, I am {BOT_NAME} v{BOT_VERSION}")
}

/// Highest depth that can be requested with `max-depth` (the engine default)
pub const MAX_REQUESTED_DEPTH: usize = 20;
pub static MAX_DEPTH_USAGE: &str = "Usage: max-depth N, with N between 1 and 20";
pub static MAX_DEPTH_NOT_ADJUSTABLE: &str =
  "Sorry, the depth can only be changed in casual games against humans";

/// Tells how deep we search in this game, and how to change it
pub fn depth_announcement(max_depth: usize) -> String {
  format!("I will search {max_depth} plies deep. Type max-depth N (1-{MAX_REQUESTED_DEPTH}) \
           to change it")
}

/// Answer to a successful `max-depth` command
pub fn max_depth_set(max_depth: usize) -> String {
  format!("Okay, I will now search {max_depth} plies deep")
}

/// Message when we found a forced mate
pub fn mate_announcement(moves: u32, move_number: usize) -> String {
  format!("{MATE_ANNOUNCEMENT} (#{moves}, move {move_number})")
}

/// Formats an eval (from our point of view) for chat messages: +1.5, -3.2,
/// or #3 / #-2 for mates (number of moves for the mating side).
pub fn format_eval(eval_for_us: f32) -> String {
  match mate_distance_in_plies(eval_for_us) {
    Some(plies) if eval_for_us > 0.0 => format!("#{}", plies.div_ceil(2)),
    Some(plies) => format!("#-{}", (plies / 2).max(1)),
    None => format!("{eval_for_us:+.1}"),
  }
}

/// Message at the end of the game, depending on who won
///
/// ### Arguments
///
/// * `winner` - Color of the winner, None for a draw
/// * `us`     - Color we played
pub fn game_over_message(winner: Option<Color>, us: Color) -> &'static str {
  match winner {
    Some(color) if color == us => GOOD_GAME,
    Some(_) => WELL_PLAYED,
    None => THANKS_FOR_PLAYING,
  }
}

/// Joyful message when we deliver an epic mate
pub fn epic_mate_message(epic_mate: EpicMate) -> String {
  match epic_mate {
    EpicMate::Smothered => String::from("Smothered mate!!! This is just epic, I am so happy \\o/"),
    EpicMate::MatingNet(pieces) => {
      format!("Epic mate!!! {pieces} pieces in the mating net, what a finish \\o/")
    },
  }
}

/// Lichess sends the username of titled opponents with their title, e.g.
/// "BOT maia1". Usernames cannot contain spaces, so we keep the last word.
pub fn display_name(username: &str) -> &str {
  username.split_whitespace().last().unwrap_or(username)
}

// -----------------------------------------------------------------------------
// Chat commands

/// Commands that can be sent to the bot in the game chat
#[derive(Debug, PartialEq, Eq)]
pub enum ChatCommand {
  /// `/eval`: tell the current move number and eval
  Eval,
  /// `/quiet`: stop commenting the game
  Quiet,
  /// `/chatty`: comment the game again (opposite of `/quiet`)
  Chatty,
  /// `max-depth N`: change the search depth (casual games against humans).
  /// None if N is missing or out of range.
  MaxDepth(Option<usize>),
}

/// Lichess accounts of the bot owners: they can use all the chat commands
/// from any room (e.g. when watching the bot's games as a spectator)
pub const BOT_OWNERS: &[&str] = &["schnellschnecke"];

/// Checks if a chat user can turn the commentary on/off (`/quiet`,
/// `/chatty`) or change the depth (`max-depth`): the opponent (player chat) or
/// a bot owner (any room). Other spectators cannot.
///
/// ### Arguments
///
/// * `room`     - Chat room where the command was sent
/// * `username` - Who sent the command
pub fn can_toggle_commentary(room: &ChatRoom, username: &str) -> bool {
  *room == ChatRoom::Player
  || BOT_OWNERS.iter().any(|owner| owner.eq_ignore_ascii_case(display_name(username)))
}

/// Parses a chat line into a command, if it is one
pub fn parse_chat_command(text: &str) -> Option<ChatCommand> {
  let mut words = text.split_whitespace();
  let command = words.next()?.to_lowercase();
  match command.as_str() {
    "/eval" | "eval" => Some(ChatCommand::Eval),
    "/quiet" | "quiet" => Some(ChatCommand::Quiet),
    "/chatty" | "chatty" => Some(ChatCommand::Chatty),
    // Lichess filters chat messages starting with a "/", so no "/" here, but we
    // still accept it in case it gets through
    "max-depth" | "/max-depth" => {
      let depth = words.next()
                       .and_then(|word| word.parse::<usize>().ok())
                       .filter(|depth| (1..=MAX_REQUESTED_DEPTH).contains(depth));
      Some(ChatCommand::MaxDepth(depth))
    },
    _ => None,
  }
}

// -----------------------------------------------------------------------------
// Game events

/// Things that happened after a search, based on the eval
#[derive(Debug, PartialEq, Eq)]
pub enum EvalEvent {
  /// We just found a forced mate, in this number of moves
  AnnounceMate(u32),
  /// The eval dropped to a bad position for us
  InTrouble,
  /// We were in trouble, and the position is back to 0-ish (or better)
  OutOfTrouble,
  /// We were in trouble, and now we have a forced mate
  Swindle,
  /// The opponent mates us in one whatever we play
  Resign,
}

impl EvalEvent {
  /// Chat message for this event, if any
  ///
  /// ### Arguments
  ///
  /// * `move_number` - Move number (as in the game notation) we are playing
  /// * `eval_for_us` - Best eval, from our point of view
  pub fn message(&self, move_number: usize, eval_for_us: f32) -> Option<String> {
    let context = format!("(move {move_number}, eval {})", format_eval(eval_for_us));
    match self {
      EvalEvent::AnnounceMate(moves) => Some(mate_announcement(*moves, move_number)),
      EvalEvent::InTrouble => Some(format!("{IN_TROUBLE} {context}")),
      EvalEvent::OutOfTrouble => Some(format!("{OUT_OF_TROUBLE} {context}")),
      EvalEvent::Swindle => Some(SWINDLE.to_string()),
      // Resigning needs a confirmation first, the game sends the message
      EvalEvent::Resign => None,
    }
  }
}

/// What already happened during the game
#[derive(Debug, Default)]
pub struct GameEvents {
  /// We said "Have fun" when the game started
  have_fun_sent:  bool,
  /// We announced that we found a forced mate
  mate_announced: bool,
  /// We said we are in trouble, and did not get out of it yet
  in_trouble:     bool,
  /// We already said something at the end of the game (or when resigning /
  /// leaving)
  game_over:      bool,
  /// We were asked to stop commenting (`/quiet`)
  quiet:          bool,
  /// Move number and eval (from our point of view) of our last search
  last_eval:      Option<(usize, f32)>,
  /// We are trying to win on time against a human (playing very fast)
  flagging:       bool,
  /// Comments we already posted in this game, so that we never post the same
  /// one twice (also restored from the chat history after a restart)
  said:           BTreeSet<String>,
  /// The opponent mentioned `!help` in the chat, so it likely understands
  /// `!eval`
  opponent_help:  bool,
  /// We already asked the opponent for its eval (`!eval`)
  opponent_eval:  bool,
  /// Last valid depth the opponent asked for with `max-depth`, found in the
  /// chat history when rejoining a game
  max_depth:      Option<usize>,
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

  /// Restores what already happened from the chat history of the game, when
  /// we join a game that is already ongoing (e.g. after a restart), so that we
  /// do not repeat ourselves.
  ///
  /// ### Arguments
  ///
  /// * `chat`         - Player chat history, oldest first
  /// * `our_username` - Our Lichess username
  ///
  /// ### Returns
  ///
  /// True if we already wrote in this game, i.e. it is not a new game and we
  /// should not greet again.
  pub fn restore_from_chat(&mut self, chat: &[ChatLine], our_username: &str) -> bool {
    let is_ours =
      |line: &ChatLine| display_name(&line.user).eq_ignore_ascii_case(display_name(our_username));

    let mut already_talked = false;
    for line in chat {
      if is_ours(line) {
        already_talked = true;
        self.said.insert(line.text.clone());
        if line.text.starts_with(MATE_ANNOUNCEMENT) || line.text == SWINDLE {
          self.mate_announced = true;
        }
        // The last trouble-related message tells where we are
        if line.text.starts_with(IN_TROUBLE) {
          self.in_trouble = true;
        } else if line.text.starts_with(OUT_OF_TROUBLE) || line.text == SWINDLE {
          self.in_trouble = false;
        }
        if line.text == ASK_OPPONENT_EVAL {
          self.opponent_eval = true;
        }
      } else {
        self.on_opponent_message(&line.text);
        // The last /quiet or /chatty from the opponent wins
        match parse_chat_command(&line.text) {
          Some(ChatCommand::Quiet) => self.quiet = true,
          Some(ChatCommand::Chatty) => self.quiet = false,
          Some(ChatCommand::MaxDepth(Some(depth))) => self.max_depth = Some(depth),
          _ => {},
        }
      }
    }

    if already_talked {
      self.mark_ongoing_game();
    }
    already_talked
  }

  /// Checks if we can post a comment: true if we never posted it in this game,
  /// and remembers it so that it is not posted again.
  ///
  /// ### Arguments
  ///
  /// * `message` - The comment we would like to post
  pub fn should_say(&mut self, message: &str) -> bool {
    self.said.insert(message.to_string())
  }

  /// Remembers if the opponent mentioned `!help` in one of its messages
  /// (e.g. "Type !help for a list of commands").
  ///
  /// ### Arguments
  ///
  /// * `text` - Message the opponent sent in the player chat
  pub fn on_opponent_message(&mut self, text: &str) {
    if text.to_lowercase().contains(OPPONENT_HELP_COMMAND) {
      self.opponent_help = true;
    }
  }

  /// Checks if we should ask the opponent for its eval (`!eval`) along with
  /// this event's comment: only if it mentioned `!help`, on our first eval
  /// comment (mate announcement or being in trouble), and only once.
  ///
  /// ### Arguments
  ///
  /// * `event` - Event we are commenting
  pub fn should_ask_opponent_eval(&mut self, event: &EvalEvent) -> bool {
    if !self.opponent_help
       || self.opponent_eval
       || !matches!(event, EvalEvent::AnnounceMate(_) | EvalEvent::InTrouble)
    {
      return false;
    }
    self.opponent_eval = true;
    true
  }

  /// Marks the game as already ongoing when we join it: "Have fun" is said at
  /// the start of the game, never later.
  pub fn mark_ongoing_game(&mut self) {
    self.have_fun_sent = true;
  }

  /// Stops the commentary. Returns true only the first time, so that we
  /// acknowledge it only once.
  pub fn set_quiet(&mut self) -> bool {
    !std::mem::replace(&mut self.quiet, true)
  }

  /// Resumes the commentary. Returns true only if we were quiet, so that we
  /// acknowledge it only when something changes.
  pub fn set_chatty(&mut self) -> bool {
    std::mem::replace(&mut self.quiet, false)
  }

  /// Depth the opponent asked for with `max-depth` before we rejoined the
  /// game, if any
  pub fn requested_max_depth(&self) -> Option<usize> {
    self.max_depth
  }

  /// True if we were asked to stop commenting
  pub fn is_quiet(&self) -> bool {
    self.quiet
  }

  /// Saves the move number and eval of our last search, for `/eval`
  pub fn set_last_eval(&mut self, move_number: usize, eval_for_us: f32) {
    if !eval_for_us.is_nan() {
      self.last_eval = Some((move_number, eval_for_us));
    }
  }

  /// Decides if we should try to flag the opponent (win on time): against a
  /// human short on time (less than a minute, and less than us) who is
  /// winning on the board. We stop only when we are clearly better, an eval
  /// around 0 keeps us flagging.
  ///
  /// Uses the eval of our last search (from our point of view).
  ///
  /// ### Arguments
  ///
  /// * `opponent_is_human` - We never try to flag bots
  /// * `our_time_ms`       - Time left on our clock
  /// * `opponent_time_ms`  - Time left on the opponent clock
  ///
  /// ### Returns
  ///
  /// True only when we just started flagging (to announce it)
  pub fn update_flagging(&mut self,
                         opponent_is_human: bool,
                         our_time_ms: usize,
                         opponent_time_ms: usize)
                         -> bool {
    let Some(eval) = self.last_eval() else {
      return false;
    };

    if self.flagging {
      if eval >= FLAG_STOP_EVAL {
        self.flagging = false;
      }
      return false;
    }

    if opponent_is_human
       && opponent_time_ms < FLAG_MAX_OPPONENT_TIME_MS
       && opponent_time_ms < our_time_ms
       && eval <= FLAG_START_EVAL
    {
      self.flagging = true;
      return true;
    }
    false
  }

  /// True if we are trying to flag the opponent
  pub fn is_flagging(&self) -> bool {
    self.flagging
  }

  /// Eval (from our point of view) of our last search, if any
  pub fn last_eval(&self) -> Option<f32> {
    self.last_eval.map(|(_, eval)| eval)
  }

  /// Answer to the `/eval` command
  pub fn eval_report(&self) -> String {
    match self.last_eval {
      Some((move_number, eval)) => format!("Move {move_number}, eval {}", format_eval(eval)),
      None => NO_EVAL_YET.to_string(),
    }
  }

  /// Marks the game as over. Returns true only the first time, so that we say
  /// goodbye only once (e.g. not after we already said "Well played!" when
  /// resigning).
  pub fn on_game_over(&mut self) -> bool {
    !std::mem::replace(&mut self.game_over, true)
  }

  /// Decides what happened after a search, based on the best eval.
  ///
  /// ### Arguments
  ///
  /// * `eval_for_us`     - Best eval, from our point of view (positive is good
  ///   for us)
  /// * `opponent_is_bot` - We only resign against bots
  ///
  /// ### Returns
  ///
  /// The events to act on, in order. Usually empty.
  pub fn after_search(&mut self, eval_for_us: f32, opponent_is_bot: bool) -> Vec<EvalEvent> {
    let mut events = Vec::new();
    if eval_for_us.is_nan() {
      return events;
    }
    let mate_plies = mate_distance_in_plies(eval_for_us);

    if eval_for_us > 0.0
       && let Some(plies) = mate_plies
    {
      if self.in_trouble {
        // Going from trouble to a forced mate: the opponent blundered
        self.in_trouble = false;
        self.mate_announced = true;
        events.push(EvalEvent::Swindle);
      } else if !self.mate_announced {
        self.mate_announced = true;
        // Our moves left: we play the 1st, 3rd, 5th... ply
        events.push(EvalEvent::AnnounceMate(plies.div_ceil(2)));
      }
      return events;
    }

    if !self.in_trouble && eval_for_us <= IN_TROUBLE_EVAL {
      self.in_trouble = true;
      events.push(EvalEvent::InTrouble);
    } else if self.in_trouble && eval_for_us >= OUT_OF_TROUBLE_EVAL {
      self.in_trouble = false;
      events.push(EvalEvent::OutOfTrouble);
    }

    // We play a move, and the opponent mates us with the next one
    if eval_for_us < 0.0 && mate_plies == Some(2) && opponent_is_bot {
      events.push(EvalEvent::Resign);
    }
    events
  }
}

/// Time to spend on a move when trying to flag the opponent: 500ms when they
/// have a minute left, going down to 200ms as they get close to 0.
///
/// ### Arguments
///
/// * `opponent_time_ms` - Time left on the opponent clock
pub fn flagging_move_time_ms(opponent_time_ms: usize) -> usize {
  let opponent_time_ms = opponent_time_ms.min(FLAG_MAX_OPPONENT_TIME_MS);
  FLAG_MOVE_TIME_MIN_MS
  + (FLAG_MOVE_TIME_MAX_MS - FLAG_MOVE_TIME_MIN_MS) * opponent_time_ms / FLAG_MAX_OPPONENT_TIME_MS
}

/// Above this eval (from our point of view), we do not claim draws: we are
/// better and should play on
pub const CLAIM_DRAW_MAX_EVAL: f32 = 0.5;

/// Checks if we should claim a draw that is available on the board (threefold
/// repetition, 50 moves rule).
///
/// ### Arguments
///
/// * `last_eval_for_us` - Eval of our last search, from our point of view. If
///   we do not know, we claim.
pub fn should_claim_draw(last_eval_for_us: Option<f32>) -> bool {
  last_eval_for_us.is_none_or(|eval| eval.is_nan() || eval <= CLAIM_DRAW_MAX_EVAL)
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
