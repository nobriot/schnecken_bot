use crate::model::board::*;
use std::cmp::max;

/// Finds how many moves/squares away is a king from a particular square, regardless
/// of "obstacle pieces" along the way
///
/// # Arguments
///
/// * `king_position` - Square index where the king is located.
/// * `destination`   - Square index where the king wants to go.
///
/// # Return value
///
/// The number of moves the king needs to do to reach the destination. With valid
/// square indexes, it will be in [0..7]
///
pub fn get_king_distance(king_position: u8, destination: u8) -> u8 {
  let (sf, sr) = Board::index_to_fr(king_position);
  let (df, dr) = Board::index_to_fr(destination);

  // basically the distance will be the max of file/rank difference.
  max(sf.abs_diff(df), sr.abs_diff(dr))
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn check_king_distance() {
    let start = Board::fr_to_index(4, 3);
    let destination = Board::fr_to_index(6, 6);
    assert_eq!(get_king_distance(start, destination), 3);
    assert_eq!(get_king_distance(destination, start), 3);

    let start = Board::fr_to_index(1, 1);
    let destination = Board::fr_to_index(8, 8);
    assert_eq!(get_king_distance(start, destination), 7);
    assert_eq!(get_king_distance(destination, start), 7);

    let start = Board::fr_to_index(7, 1);
    let destination = Board::fr_to_index(8, 8);
    assert_eq!(get_king_distance(start, destination), 7);
    assert_eq!(get_king_distance(destination, start), 7);

    let start = Board::fr_to_index(7, 1);
    let destination = Board::fr_to_index(8, 8);
    assert_eq!(get_king_distance(start, destination), 7);
    assert_eq!(get_king_distance(destination, start), 7);

    let start = Board::fr_to_index(1, 2);
    let destination = Board::fr_to_index(1, 3);
    assert_eq!(get_king_distance(start, destination), 1);
    assert_eq!(get_king_distance(destination, start), 1);

    let start = Board::fr_to_index(3, 2);
    let destination = Board::fr_to_index(3, 2);
    assert_eq!(get_king_distance(start, destination), 0);
    assert_eq!(get_king_distance(destination, start), 0);
  }
}
