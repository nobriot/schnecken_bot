use crate::bot::games::engine::*;

#[test]
fn test_max_move_time() {
  // No limit against bots, a maximum against humans
  assert_eq!(0, max_move_time_ms(true, None));
  assert_eq!(HUMAN_MAX_MOVE_TIME_MS, max_move_time_ms(false, None));

  // Flagging makes us play faster
  assert_eq!(300, max_move_time_ms(false, Some(300)));
  assert_eq!(300, max_move_time_ms(true, Some(300)));
  assert_eq!(HUMAN_MAX_MOVE_TIME_MS, max_move_time_ms(false, Some(100_000)));
}
