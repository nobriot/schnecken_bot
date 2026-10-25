use crate::bot::games::events::*;
use chess::engine::Engine;
use chess::model::board::Board;

#[test]
fn test_messages() {
  assert_eq!("maia1", display_name("BOT maia1"));
  assert_eq!("thibault", display_name("thibault"));
  assert!(greeting("maia1").starts_with("Hello maia1, I am schnecken_bot v"));
  assert_eq!("I smell some mate around the corner (#3, move 23)", mate_announcement(3, 23));
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
  assert!(events.after_search(3.5, true).is_empty());
  // Mate in 3 for us (5 plies: 200 - 4)
  assert_eq!(vec![EvalEvent::AnnounceMate(3)], events.after_search(196.0, false));
  // Announced only once
  assert!(events.after_search(198.0, false).is_empty());
  assert!(events.after_search(200.0, false).is_empty());

  // Mate in one for us (our move mates)
  let mut events = GameEvents::default();
  assert_eq!(vec![EvalEvent::AnnounceMate(1)], events.after_search(200.0, true));
}

#[test]
fn test_resign_against_bots_only() {
  // Mate in one against us: resign against bots (after saying we are in
  // trouble, as it is the first bad eval)
  let mut events = GameEvents::default();
  assert_eq!(vec![EvalEvent::InTrouble, EvalEvent::Resign], events.after_search(-199.0, true));
  // Smothered mate in one against us as well
  assert_eq!(vec![EvalEvent::Resign], events.after_search(-219.0, true));
  // Not against humans
  assert!(events.after_search(-199.0, false).is_empty());
  // Not for longer mates
  assert!(events.after_search(-197.0, true).is_empty());
  // Not when we are just losing
  assert!(events.after_search(-15.0, true).is_empty());
}

#[test]
fn test_trouble_commentary() {
  let mut events = GameEvents::default();
  assert!(events.after_search(0.3, false).is_empty());
  assert!(events.after_search(-2.9, false).is_empty());
  // First time at -3 or worse
  assert_eq!(vec![EvalEvent::InTrouble], events.after_search(-3.0, false));
  // Never again while still in trouble, even if it gets better or worse
  assert!(events.after_search(-5.0, false).is_empty());
  assert!(events.after_search(-1.0, false).is_empty());
  assert!(events.after_search(-8.0, false).is_empty());
  // Back to a 0-ish eval
  assert_eq!(vec![EvalEvent::OutOfTrouble], events.after_search(-0.4, false));
  assert!(events.after_search(0.2, false).is_empty());
  // In trouble again later: we can say it again
  assert_eq!(vec![EvalEvent::InTrouble], events.after_search(-4.0, false));
  // Jumping straight to a winning (non-mate) position also gets us out of
  // trouble
  assert_eq!(vec![EvalEvent::OutOfTrouble], events.after_search(2.5, false));
}

#[test]
fn test_swindle() {
  let mut events = GameEvents::default();
  assert_eq!(vec![EvalEvent::InTrouble], events.after_search(-6.0, false));
  // Opponent blunders, we have a mate: swindle instead of the mate announcement
  assert_eq!(vec![EvalEvent::Swindle], events.after_search(197.0, false));
  // Out of trouble, and the mate counts as announced
  assert!(events.after_search(199.0, false).is_empty());
  assert!(events.after_search(1.0, false).is_empty());

  // Without being in trouble, finding a mate is a normal announcement
  let mut events = GameEvents::default();
  assert_eq!(vec![EvalEvent::AnnounceMate(2)], events.after_search(198.0, false));
}

#[test]
fn test_event_messages() {
  assert_eq!(Some(String::from("oh oh, I think I am in trouble (move 23, eval -3.4)")),
             EvalEvent::InTrouble.message(23, -3.4));
  assert_eq!(Some(String::from("Maybe not so much in trouble after all (move 31, eval -0.3)")),
             EvalEvent::OutOfTrouble.message(31, -0.3));
  assert_eq!(Some(String::from("Maybe not so much in trouble after all (move 31, eval +2.5)")),
             EvalEvent::OutOfTrouble.message(31, 2.5));
  assert_eq!(Some(String::from("I smell some mate around the corner (#4, move 40)")),
             EvalEvent::AnnounceMate(4).message(40, 194.0));
  assert_eq!(Some(String::from("Wow, this is a swindle!")),
             EvalEvent::Swindle.message(40, 197.0));
  assert_eq!(None, EvalEvent::Resign.message(40, -199.0));
}

#[test]
fn test_format_eval() {
  assert_eq!("+1.5", format_eval(1.5));
  assert_eq!("-3.2", format_eval(-3.24));
  assert_eq!("+0.0", format_eval(0.0));
  // Mates: number of moves for the mating side
  assert_eq!("#1", format_eval(200.0));
  assert_eq!("#3", format_eval(196.0));
  assert_eq!("#-1", format_eval(-199.0));
  assert_eq!("#-2", format_eval(-197.0));
}

#[test]
fn test_game_over_message() {
  use lichess::types::Color;
  assert_eq!("Good game!", game_over_message(Some(Color::White), Color::White));
  assert_eq!("Good game!", game_over_message(Some(Color::Black), Color::Black));
  assert_eq!("Well played!", game_over_message(Some(Color::Black), Color::White));
  assert_eq!("Well played!", game_over_message(Some(Color::White), Color::Black));
  assert_eq!("Thanks for playing ! :)", game_over_message(None, Color::White));

  // Said only once
  let mut events = GameEvents::default();
  assert!(events.on_game_over());
  assert!(!events.on_game_over());
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
  let result = events.after_search(eval_for_white, true);
  assert!(result.contains(&EvalEvent::Resign), "eval {eval_for_white}: {result:?}");
}
