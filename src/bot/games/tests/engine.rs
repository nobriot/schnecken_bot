use crate::bot::games::engine::*;

#[test]
fn test_max_move_time() {
  // No limit against bots, a maximum against humans
  assert_eq!(0, max_move_time_ms(true, None, false));
  assert_eq!(HUMAN_MAX_MOVE_TIME_MS, max_move_time_ms(false, None, false));

  // Flagging makes us play faster
  assert_eq!(300, max_move_time_ms(false, Some(300), false));
  assert_eq!(300, max_move_time_ms(true, Some(300), false));
  assert_eq!(HUMAN_MAX_MOVE_TIME_MS, max_move_time_ms(false, Some(100_000), false));

  // Our first move is always quick
  assert_eq!(FIRST_MOVE_MAX_TIME_MS, max_move_time_ms(true, None, true));
  assert_eq!(FIRST_MOVE_MAX_TIME_MS, max_move_time_ms(false, None, true));
  assert_eq!(300, max_move_time_ms(true, Some(300), true));
}
