/// Makes the sum of a board mask
///
/// # Arguments
///
/// * `mask` - u64 bitmask representing a board with 0 and 1s.
///
/// # Return value
///
/// the sum of all bits set to 1.
pub fn mask_sum(mask: u64) -> usize {
  let mut sum: usize = 0;
  for i in 0..64 {
    if mask >> i & 1 == 1 {
      sum += 1;
    }
  }
  sum
}
