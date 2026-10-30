use crate::bot::games::events::*;
use chess::engine::Engine;
use chess::engine::eval::position::{EPIC_MATE, MATE, Score};
use chess::model::board::Board;
use lichess::types::ChatLine;

#[test]
fn test_messages() {
  assert_eq!("maia1", display_name("BOT maia1"));
  assert_eq!("thibault", display_name("thibault"));
  assert!(greeting("maia1").starts_with("Hello maia1, I am schnecken_bot v"));
  assert_eq!("Woo-hoo, mate around the corner (#3, move 23)", mate_announcement(3, 23));
  assert_eq!("I will search 4 plies deep. Type max-depth N (1-20) to change it",
             depth_announcement(4));
  // Lichess chat messages are limited to 140 characters
  assert!(depth_announcement(20).len() <= 140);
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
  assert!(events.after_search(350, true).is_empty());
  // Mate in 3 for us (5 plies)
  assert_eq!(vec![EvalEvent::AnnounceMate(3)], events.after_search(MATE - 5, false));
  // Announced only once
  assert!(events.after_search(MATE - 3, false).is_empty());
  assert!(events.after_search(MATE - 1, false).is_empty());

  // Mate in one for us (our move mates)
  let mut events = GameEvents::default();
  assert_eq!(vec![EvalEvent::AnnounceMate(1)], events.after_search(MATE - 1, true));
}

#[test]
fn test_resign_against_bots_only() {
  // Mate in one against us: resign against bots (after saying we are in
  // trouble, as it is the first bad eval)
  let mut events = GameEvents::default();
  assert_eq!(vec![EvalEvent::InTrouble, EvalEvent::Resign],
             events.after_search(-(MATE - 2), true));
  // Epic mate in one against us as well
  assert_eq!(vec![EvalEvent::Resign], events.after_search(-(EPIC_MATE - 2), true));
  // Not against humans
  assert!(events.after_search(-(MATE - 2), false).is_empty());
  // Not for longer mates
  assert!(events.after_search(-(MATE - 4), true).is_empty());
  // Not when we are just losing
  assert!(events.after_search(-1500, true).is_empty());
}

#[test]
fn test_trouble_commentary() {
  let mut events = GameEvents::default();
  assert!(events.after_search(30, false).is_empty());
  assert!(events.after_search(-290, false).is_empty());
  // First time at -3 or worse
  assert_eq!(vec![EvalEvent::InTrouble], events.after_search(-300, false));
  // Never again while still in trouble, even if it gets better or worse
  assert!(events.after_search(-500, false).is_empty());
  assert!(events.after_search(-100, false).is_empty());
  assert!(events.after_search(-800, false).is_empty());
  // Back to a 0-ish eval
  assert_eq!(vec![EvalEvent::OutOfTrouble], events.after_search(-40, false));
  assert!(events.after_search(20, false).is_empty());
  // In trouble again later: we can say it again
  assert_eq!(vec![EvalEvent::InTrouble], events.after_search(-400, false));
  // Jumping straight to a winning (non-mate) position also gets us out of
  // trouble
  assert_eq!(vec![EvalEvent::OutOfTrouble], events.after_search(250, false));
}

#[test]
fn test_swindle() {
  let mut events = GameEvents::default();
  assert_eq!(vec![EvalEvent::InTrouble], events.after_search(-600, false));
  // Opponent blunders, we have a mate: swindle instead of the mate announcement
  assert_eq!(vec![EvalEvent::Swindle], events.after_search(MATE - 4, false));
  // Out of trouble, and the mate counts as announced
  assert!(events.after_search(MATE - 2, false).is_empty());
  assert!(events.after_search(100, false).is_empty());

  // Without being in trouble, finding a mate is a normal announcement
  let mut events = GameEvents::default();
  assert_eq!(vec![EvalEvent::AnnounceMate(2)], events.after_search(MATE - 3, false));
}

#[test]
fn test_event_messages() {
  assert_eq!(Some(String::from("oh oh, I think I am in trouble (move 23, eval -3.4)")),
             EvalEvent::InTrouble.message(23, -340));
  assert_eq!(Some(String::from("Maybe not so much in trouble after all (move 31, eval -0.3)")),
             EvalEvent::OutOfTrouble.message(31, -30));
  assert_eq!(Some(String::from("Maybe not so much in trouble after all (move 31, eval +2.5)")),
             EvalEvent::OutOfTrouble.message(31, 250));
  assert_eq!(Some(String::from("Woo-hoo, mate around the corner (#4, move 40)")),
             EvalEvent::AnnounceMate(4).message(40, MATE - 7));
  assert_eq!(Some(String::from("Wow, this is a swindle!")),
             EvalEvent::Swindle.message(40, MATE - 4));
  assert_eq!(None, EvalEvent::Resign.message(40, -(MATE - 2)));
}

#[test]
fn test_format_eval() {
  assert_eq!("+1.5", format_eval(150));
  assert_eq!("-3.2", format_eval(-324));
  assert_eq!("+0.0", format_eval(0));
  // Mates: number of moves for the mating side
  assert_eq!("#1", format_eval(MATE - 1));
  assert_eq!("#3", format_eval(MATE - 5));
  assert_eq!("#-1", format_eval(-(MATE - 2)));
  assert_eq!("#-2", format_eval(-(MATE - 4)));
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

#[test]
fn test_parse_chat_command() {
  assert_eq!(Some(ChatCommand::Eval), parse_chat_command("/eval"));
  assert_eq!(Some(ChatCommand::Eval), parse_chat_command("  /EVAL please"));
  assert_eq!(Some(ChatCommand::Quiet), parse_chat_command("/quiet"));
  assert_eq!(Some(ChatCommand::Chatty), parse_chat_command("/chatty"));
  assert_eq!(Some(ChatCommand::Chatty), parse_chat_command("/Chatty again"));
  assert_eq!(Some(ChatCommand::MaxDepth(Some(5))), parse_chat_command("max-depth 5"));
  assert_eq!(Some(ChatCommand::MaxDepth(Some(5))), parse_chat_command("/max-depth 5"));
  assert_eq!(None, parse_chat_command("what max-depth 5"));
  assert_eq!(Some(ChatCommand::MaxDepth(Some(20))),
             parse_chat_command("/MAX-DEPTH 20 please"));
  assert_eq!(Some(ChatCommand::MaxDepth(None)), parse_chat_command("max-depth"));
  assert_eq!(Some(ChatCommand::MaxDepth(None)), parse_chat_command("max-depth 0"));
  assert_eq!(Some(ChatCommand::MaxDepth(None)), parse_chat_command("max-depth 21"));
  assert_eq!(Some(ChatCommand::MaxDepth(None)), parse_chat_command("max-depth deep"));
  assert_eq!(None, parse_chat_command("eval"));
  assert_eq!(None, parse_chat_command("what is the /eval?"));
  assert_eq!(None, parse_chat_command("/evaluate"));
  assert_eq!(None, parse_chat_command(""));
}

#[test]
fn test_parse_max_depth_tricky_input() {
  // Anything that is not a plain number between 1 and 20 is rejected
  let rejected = ["max-depth -1",
                  "max-depth -0",
                  "max-depth 5.5",
                  "max-depth 1e3",
                  "max-depth 0x10",
                  "max-depth 5abc",
                  "max-depth ５",
                  "max-depth ٥",
                  "max-depth 18446744073709551615",
                  "max-depth 18446744073709551616",
                  "max-depth 99999999999999999999999999999999999999999",
                  "max-depth NaN",
                  "max-depth inf",
                  "max-depth \u{0}",
                  "max-depth ; /quiet",
                  "max-depth <script>alert(1)</script>",
                  "max-depth %s%n",
                  "max-depth\t\n"];
  for text in rejected {
    assert_eq!(Some(ChatCommand::MaxDepth(None)), parse_chat_command(text), "{text:?}");
  }

  // Leading zeros and a + sign are just the number, extra words are ignored
  assert_eq!(Some(ChatCommand::MaxDepth(Some(7))), parse_chat_command("max-depth 007"));
  assert_eq!(Some(ChatCommand::MaxDepth(Some(7))), parse_chat_command("max-depth +7"));
  assert_eq!(Some(ChatCommand::MaxDepth(Some(1))), parse_chat_command("max-depth 1 2 3"));
  assert_eq!(Some(ChatCommand::MaxDepth(Some(3))), parse_chat_command("\tmax-depth\u{a0}3"));

  // Long garbage does not crash
  let long = format!("max-depth {}", "9".repeat(100_000));
  assert_eq!(Some(ChatCommand::MaxDepth(None)), parse_chat_command(&long));
}

#[test]
fn test_engine_plays_at_lowest_max_depth() {
  let mut engine = Engine::new(false);
  engine.set_position("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1");
  engine.options.max_depth = 1;
  engine.go();
  assert!(engine.get_best_move().is_some());
}

#[test]
fn test_eval_report() {
  let mut events = GameEvents::default();
  assert_eq!(NO_EVAL_YET, events.eval_report());
  events.set_last_eval(12, 125);
  assert_eq!("Move 12, eval +1.2", events.eval_report());
  events.set_last_eval(30, -(MATE - 4));
  assert_eq!("Move 30, eval #-2", events.eval_report());
}

#[test]
fn test_quiet() {
  let mut events = GameEvents::default();
  assert!(!events.is_quiet());
  // Acknowledged only once
  assert!(events.set_quiet());
  assert!(!events.set_quiet());
  assert!(events.is_quiet());

  // The trouble state is still tracked while quiet, so that we do not comment
  // on an old situation if this changes later
  assert_eq!(vec![EvalEvent::InTrouble], events.after_search(-400, false));
  assert!(events.after_search(-600, false).is_empty());
}

#[test]
fn test_chatty() {
  let mut events = GameEvents::default();
  // Already chatty: nothing to acknowledge
  assert!(!events.set_chatty());
  assert!(!events.is_quiet());

  // Quiet, then chatty again: acknowledged once
  assert!(events.set_quiet());
  assert!(events.set_chatty());
  assert!(!events.is_quiet());
  assert!(!events.set_chatty());

  // And quiet can be asked again afterwards
  assert!(events.set_quiet());
  assert!(events.is_quiet());
}

fn line(user: &str, text: &str) -> ChatLine {
  ChatLine { user: user.to_string(),
             text: text.to_string(), }
}

#[test]
fn test_restore_from_chat_new_game() {
  // Nobody talked yet, or only the opponent: new game, we greet
  let mut events = GameEvents::default();
  assert!(!events.restore_from_chat(&[], "schnecken_bot"));
  assert!(events.on_moves_played(1));

  let mut events = GameEvents::default();
  assert!(!events.restore_from_chat(&[line("oli567", "hi!")], "schnecken_bot"));
  assert!(events.on_moves_played(2));
}

#[test]
fn test_restore_from_chat_ongoing_game() {
  // We already greeted: no greeting, no "Have fun" again
  let chat = [line("schnecken_bot", &greeting("oli567")), line("schnecken_bot", HAVE_FUN)];
  let mut events = GameEvents::default();
  assert!(events.restore_from_chat(&chat, "schnecken_bot"));
  assert!(!events.on_moves_played(1));
  assert!(!events.on_moves_played(2));
  // Nothing else happened: trouble and mates can still be commented
  assert_eq!(vec![EvalEvent::InTrouble], events.after_search(-400, false));

  // Usernames are matched without case and title ("BOT schnecken_bot")
  let chat = [line("BOT Schnecken_Bot", &greeting("oli567"))];
  let mut events = GameEvents::default();
  assert!(events.restore_from_chat(&chat, "schnecken_bot"));
}

#[test]
fn test_restore_from_chat_state() {
  // In trouble, and a mate was already announced earlier
  let chat = [line("schnecken_bot", &greeting("oli567")),
              line("schnecken_bot", &mate_announcement(3, 20)),
              line("schnecken_bot", &EvalEvent::InTrouble.message(30, -400).unwrap())];
  let mut events = GameEvents::default();
  assert!(events.restore_from_chat(&chat, "schnecken_bot"));
  // Still in trouble: no new "in trouble" message, but we can get out of it
  assert!(events.after_search(-500, false).is_empty());
  assert_eq!(vec![EvalEvent::OutOfTrouble], events.after_search(0, false));
  // Mate already announced
  assert!(events.after_search(MATE - 4, false).is_empty());

  // Got out of trouble already
  let chat = [line("schnecken_bot", &EvalEvent::InTrouble.message(30, -400).unwrap()),
              line("schnecken_bot", &EvalEvent::OutOfTrouble.message(35, 10).unwrap())];
  let mut events = GameEvents::default();
  events.restore_from_chat(&chat, "schnecken_bot");
  assert_eq!(vec![EvalEvent::InTrouble], events.after_search(-400, false));

  // Opponent asked us to be quiet, then chatty, then quiet again
  let chat = [line("schnecken_bot", &greeting("oli567")),
              line("oli567", "/quiet"),
              line("oli567", "/chatty"),
              line("oli567", "/quiet")];
  let mut events = GameEvents::default();
  events.restore_from_chat(&chat, "schnecken_bot");
  assert!(events.is_quiet());

  // Our own messages are never taken as commands
  let chat = [line("schnecken_bot", "/quiet")];
  let mut events = GameEvents::default();
  events.restore_from_chat(&chat, "schnecken_bot");
  assert!(!events.is_quiet());

  // The last valid depth the opponent asked for is kept
  let chat = [line("schnecken_bot", &depth_announcement(4)),
              line("oli567", "max-depth 6"),
              line("oli567", "max-depth 8"),
              line("oli567", "max-depth 99")];
  let mut events = GameEvents::default();
  events.restore_from_chat(&chat, "schnecken_bot");
  assert_eq!(Some(8), events.requested_max_depth());

  let mut events = GameEvents::default();
  events.restore_from_chat(&[line("schnecken_bot", "max-depth 6")], "schnecken_bot");
  assert_eq!(None, events.requested_max_depth());
}

#[test]
fn test_ask_opponent_eval() {
  // The opponent never mentioned !help: we do not ask
  let mut events = GameEvents::default();
  events.on_opponent_message("hello");
  assert!(!events.should_ask_opponent_eval(&EvalEvent::InTrouble));

  // Only on the first mate announcement / trouble comment
  let mut events = GameEvents::default();
  events.on_opponent_message("Type !HELP for a list of commands");
  assert!(!events.should_ask_opponent_eval(&EvalEvent::OutOfTrouble));
  assert!(!events.should_ask_opponent_eval(&EvalEvent::Swindle));
  assert!(events.should_ask_opponent_eval(&EvalEvent::AnnounceMate(3)));
  assert!(!events.should_ask_opponent_eval(&EvalEvent::InTrouble));

  // After a restart: the opponent said !help, we did not ask yet
  let chat = [line("schnecken_bot", &greeting("botty")), line("BOT botty", "!help for commands")];
  let mut events = GameEvents::default();
  events.restore_from_chat(&chat, "schnecken_bot");
  assert!(events.should_ask_opponent_eval(&EvalEvent::InTrouble));

  // After a restart: we already asked
  let chat = [line("BOT botty", "!help for commands"),
              line("schnecken_bot", &EvalEvent::InTrouble.message(30, -400).unwrap()),
              line("schnecken_bot", ASK_OPPONENT_EVAL)];
  let mut events = GameEvents::default();
  events.restore_from_chat(&chat, "schnecken_bot");
  assert!(!events.should_ask_opponent_eval(&EvalEvent::AnnounceMate(2)));
}

#[test]
fn test_mark_ongoing_game() {
  let mut events = GameEvents::default();
  events.mark_ongoing_game();
  assert!(!events.on_moves_played(1));
}

#[test]
fn test_never_say_the_same_comment_twice() {
  let mut events = GameEvents::default();
  assert!(events.should_say(HAVE_FUN));
  assert!(!events.should_say(HAVE_FUN));
  assert!(events.should_say(WELL_PLAYED));

  // Comments with a different context (move, eval) are different comments
  let trouble_1 = EvalEvent::InTrouble.message(23, -340).unwrap();
  let trouble_2 = EvalEvent::InTrouble.message(40, -400).unwrap();
  assert!(events.should_say(&trouble_1));
  assert!(events.should_say(&trouble_2));
  assert!(!events.should_say(&trouble_1));
}

#[test]
fn test_never_repeat_comments_after_restart() {
  // Everything we said before a restart is not said again
  let goodbye = "Sorry, I have to leave. I'll resign now!";
  let chat = [line("schnecken_bot", &greeting("oli567")),
              line("schnecken_bot", HAVE_FUN),
              line("schnecken_bot", goodbye),
              line("oli567", WELL_PLAYED)];
  let mut events = GameEvents::default();
  events.restore_from_chat(&chat, "schnecken_bot");
  assert!(!events.should_say(&greeting("oli567")));
  assert!(!events.should_say(HAVE_FUN));
  assert!(!events.should_say(goodbye));
  // Only our own comments count: the opponent said "Well played!", we can too
  assert!(events.should_say(WELL_PLAYED));
}

#[test]
fn test_can_toggle_commentary() {
  use lichess::types::ChatRoom;
  // The opponent, in the player chat
  assert!(can_toggle_commentary(&ChatRoom::Player, "oli567"));
  // Not any spectator
  assert!(!can_toggle_commentary(&ChatRoom::Spectator, "oli567"));
  // The bot owner, from any room, whatever the case
  assert!(can_toggle_commentary(&ChatRoom::Spectator, "SchnellSchnecke"));
  assert!(can_toggle_commentary(&ChatRoom::Player, "schnellschnecke"));
}

#[test]
fn test_should_claim_draw() {
  // We do not know how we stand: claim
  assert!(should_claim_draw(None));
  // Worse, equal or only slightly better: claim
  assert!(should_claim_draw(Some(-400)));
  assert!(should_claim_draw(Some(0)));
  assert!(should_claim_draw(Some(CLAIM_DRAW_MAX_EVAL)));
  // Better: play on
  assert!(!should_claim_draw(Some(150)));
  assert!(!should_claim_draw(Some(MATE - 4)));
}

#[test]
fn test_last_eval() {
  let mut events = GameEvents::default();
  assert_eq!(None, events.last_eval());
  events.set_last_eval(18, -300);
  assert_eq!(Some(-300), events.last_eval());
}

#[test]
fn test_resign_with_single_legal_move() {
  // https://lichess.org/zVGdOBiO/black : our only legal move is Ne7, then
  // Qxe7#. We play Black against a bot: we must resign.
  let fen = "r1bk3r/1p1p1B2/p2P1Qn1/2p1p1Np/1P6/P7/2P2PPP/R3K2R b KQ - 1 21";
  let mut engine = Engine::new(false);
  engine.set_position(fen);
  engine.options.max_search_time = 2000;
  engine.go();
  let eval_for_black = -engine.get_eval().unwrap();

  let mut events = GameEvents::default();
  let result = events.after_search(eval_for_black, true);
  assert!(result.contains(&EvalEvent::Resign), "eval {eval_for_black}: {result:?}");
  assert!(is_forced_mate_in_one_against(&Board::from_fen(fen)));
}

/// Game events where our last search gave this eval (from our point of view)
fn events_with_eval(eval_for_us: Score) -> GameEvents {
  let mut events = GameEvents::default();
  events.set_last_eval(30, eval_for_us);
  events
}

#[test]
fn test_flagging_start_conditions() {
  // Human, 30s left (less than our 2 minutes), and winning: we flag
  let mut events = events_with_eval(-300);
  assert!(events.update_flagging(true, 120_000, 30_000));
  assert!(events.is_flagging());
  // Announced only once
  assert!(!events.update_flagging(true, 120_000, 25_000));
  assert!(events.is_flagging());

  // Never against bots
  let mut events = events_with_eval(-300);
  assert!(!events.update_flagging(false, 120_000, 30_000));
  assert!(!events.is_flagging());
  // Not when the opponent has a minute or more
  let mut events = events_with_eval(-300);
  assert!(!events.update_flagging(true, 120_000, 61_000));
  assert!(!events.is_flagging());
  // Not when the opponent has more time than us
  let mut events = events_with_eval(-300);
  assert!(!events.update_flagging(true, 20_000, 30_000));
  assert!(!events.is_flagging());
  // Not when the opponent is not clearly winning
  let mut events = events_with_eval(-200);
  assert!(!events.update_flagging(true, 120_000, 30_000));
  assert!(!events.is_flagging());
  // Not before we have an eval
  let mut events = GameEvents::default();
  assert!(!events.update_flagging(true, 120_000, 30_000));
  assert!(!events.is_flagging());
}

#[test]
fn test_flagging_stop_conditions() {
  let mut events = events_with_eval(-250);
  assert!(events.update_flagging(true, 120_000, 30_000));

  // Getting back to an even or still losing position: keep flagging
  events.set_last_eval(31, 0);
  events.update_flagging(true, 120_000, 20_000);
  assert!(events.is_flagging());
  events.set_last_eval(32, -500);
  events.update_flagging(true, 120_000, 15_000);
  assert!(events.is_flagging());
  // Even if the time conditions are not met anymore
  events.update_flagging(true, 5_000, 70_000);
  assert!(events.is_flagging());

  // Clearly better: stop, silently
  events.set_last_eval(33, 250);
  assert!(!events.update_flagging(true, 120_000, 10_000));
  assert!(!events.is_flagging());

  // And start again if the opponent is winning again later
  events.set_last_eval(40, -400);
  assert!(events.update_flagging(true, 120_000, 8_000));
  assert!(events.is_flagging());
}

#[test]
fn test_flagging_move_time() {
  assert_eq!(500, flagging_move_time_ms(60_000));
  assert_eq!(500, flagging_move_time_ms(120_000));
  assert_eq!(350, flagging_move_time_ms(30_000));
  assert_eq!(200, flagging_move_time_ms(0));
  assert!(flagging_move_time_ms(10_000) < flagging_move_time_ms(20_000));
}

#[test]
fn test_epic_mate_messages() {
  use chess::engine::eval::position::EpicMate;
  assert!(epic_mate_message(EpicMate::Smothered).starts_with("Smothered mate!!!"));
  assert_eq!("Epic mate!!! 6 pieces in the mating net, what a finish \\o/",
             epic_mate_message(EpicMate::MatingNet(6)));
}
