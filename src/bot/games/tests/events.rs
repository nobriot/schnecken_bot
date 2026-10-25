use crate::bot::games::events::*;
use chess::engine::Engine;
use chess::model::board::Board;

#[test]
fn test_messages() {
  assert_eq!("maia1", display_name("BOT maia1"));
  assert_eq!("thibault", display_name("thibault"));
  assert!(greeting("maia1").starts_with("Hello maia1, I am schnecken_bot v"));
  assert_eq!("I smell some mate around the corner (#3)", mate_announcement(3));
}

#[test]
fn test_have_fun_once_at_game_start() {
  let mut events = GameEvents::default();
  // Game not started yet
  assert!(!events.on_moves_played(0));
  // First move played
  assert!(events.on_moves_played(1));
  // Only once
  assert!(!events.on_moves_played(2));
  assert!(!events.on_moves_played(3));

  // Opponent played first, we see 2 moves at once
  let mut events = GameEvents::default();
  assert!(events.on_moves_played(2));

  // Joining an ongoing game (e.g. after a restart): no "Have fun"
  let mut events = GameEvents::default();
  assert!(!events.on_moves_played(30));
  assert!(!events.on_moves_played(31));
}

#[test]
fn test_mate_announcement_once() {
  let mut events = GameEvents::default();
  assert_eq!(EvalOutcome::None, events.after_search(3.5, true));
  // Mate in 3 for us (5 plies: 200 - 4)
  assert_eq!(EvalOutcome::AnnounceMate(3), events.after_search(196.0, false));
  // Announced only once
  assert_eq!(EvalOutcome::None, events.after_search(198.0, false));
  assert_eq!(EvalOutcome::None, events.after_search(200.0, false));

  // Mate in one for us (our move mates)
  let mut events = GameEvents::default();
  assert_eq!(EvalOutcome::AnnounceMate(1), events.after_search(200.0, true));
}

#[test]
fn test_resign_against_bots_only() {
  // Mate in one against us: resign against bots
  let mut events = GameEvents::default();
  assert_eq!(EvalOutcome::Resign, events.after_search(-199.0, true));
  // Smothered mate in one against us as well
  assert_eq!(EvalOutcome::Resign, events.after_search(-219.0, true));
  // Not against humans
  assert_eq!(EvalOutcome::None, events.after_search(-199.0, false));
  // Not for longer mates
  assert_eq!(EvalOutcome::None, events.after_search(-197.0, true));
  // Not when we are just losing
  assert_eq!(EvalOutcome::None, events.after_search(-15.0, true));
}

/// White to play: b3, b4 and bxa3 all allow Qg2#
const FORCED_MATE_IN_ONE: &str = "8/8/8/8/8/p4kq1/PP6/7K w - - 0 1";

#[test]
fn test_is_forced_mate_in_one_against() {
  assert!(is_forced_mate_in_one_against(&Board::from_fen(FORCED_MATE_IN_ONE)));

  // The rook can take the queen with check: no forced mate
  let defended = Board::from_fen("6R1/8/8/8/8/p4kq1/PP6/7K w - - 0 1");
  assert!(!is_forced_mate_in_one_against(&defended));

  // Start position: obviously not
  let start = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
  assert!(!is_forced_mate_in_one_against(&start));

  // Stalemate: no legal move, nothing to resign
  let stalemate = Board::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1");
  assert!(!is_forced_mate_in_one_against(&stalemate));
}

#[test]
fn test_engine_eval_leads_to_resign() {
  // End-to-end with the engine: the eval it finds must trigger a resignation
  let mut engine = Engine::new(false);
  engine.set_position(FORCED_MATE_IN_ONE);
  engine.options.max_search_time = 0;
  engine.options.max_depth = 3;
  engine.go();
  let eval_for_white = engine.get_eval().unwrap();
  let mut events = GameEvents::default();
  assert_eq!(EvalOutcome::Resign,
             events.after_search(eval_for_white, true),
             "eval {eval_for_white}");
}
