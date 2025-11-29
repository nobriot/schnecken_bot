use crate::model::board_mask::*;

#[test]
fn test_mask_macros() {
  let mut mask: BoardMask = 0;
  set_square_in_mask!(2, mask);
  set_square_in_mask!(5, mask);

  assert!(square_in_mask!(2, mask));
  assert!(square_in_mask!(5, mask));
  assert!(!square_in_mask!(1, mask));
  assert!(!square_in_mask!(3, mask));
  assert!(!square_in_mask!(4, mask));
  assert!(!square_in_mask!(6, mask));

  unset_square_in_mask!(5, mask);
  assert_eq!(1, mask.count_ones());

  assert!(square_in_mask!(2, mask));
  assert!(!square_in_mask!(5, mask));
}
