use std::fmt::Display;

use crate::model::tables::zobrist::BoardHash;

/// How many positions hash do we keep in a list.
const POSITION_LIST_LENGTH: usize = 30;

/// Position Hash list. Fixed-size array
///
#[derive(Debug, Clone)]
pub struct PositionList {
  list: [BoardHash; POSITION_LIST_LENGTH],
  index: u8,
}

impl PositionList {
  /// Creates a new empty position list
  ///
  pub fn new() -> Self {
    PositionList {
      list: [0; POSITION_LIST_LENGTH],
      index: 0,
    }
  }

  /// Adds a position hash in the list list
  ///
  /// ### Arguments
  ///
  /// * `board_hash` - board hash to add to the list
  ///
  pub fn add(&mut self, board_hash: BoardHash) {
    if self.index as usize >= POSITION_LIST_LENGTH {
      self.index = 0;
    }
    // SAFETY: We just check above that the index is within bounds.
    unsafe {
      let element = self.list.get_unchecked_mut(self.index as usize);
      *element = board_hash;
    }
    self.index += 1;
  }

  /// Clears all the moves from a move list
  ///
  pub fn clear(&mut self) {
    self.list.fill(0);
    self.index = 0;
  }

  /// Gets the length of a move list.
  ///
  pub fn len(&self) -> usize {
    self.list.into_iter().filter(|e| *e != 0).count()
  }

  /// Checks how many time the hash is present in the list.
  ///
  pub fn count(&self, hash: BoardHash) -> usize {
    self.list.into_iter().filter(|e| *e == hash).count()
  }

  /// Returns the capacity of a move list
  ///
  pub const fn capacity(&self) -> usize {
    POSITION_LIST_LENGTH
  }
}

impl Display for PositionList {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "[")?;
    for hash in &self.list {
      if *hash == 0 {
        continue;
      }
      write!(f, "{} ", hash)?;
    }
    write!(f, "]")
  }
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_position_list() {
    let mut list = PositionList::new();

    assert_eq!(0, list.len());
    assert_eq!(POSITION_LIST_LENGTH, list.capacity());
    assert_eq!(POSITION_LIST_LENGTH, list.count(0));
    assert_eq!(0, list.count(1));

    list.add(3);
    list.add(4);

    assert_eq!(2, list.len());
    assert_eq!(POSITION_LIST_LENGTH, list.capacity());
    assert_eq!(0, list.count(1));
    assert_eq!(1, list.count(3));
    assert_eq!(1, list.count(4));

    list.add(4);
    assert_eq!(0, list.count(1));
    assert_eq!(1, list.count(3));
    assert_eq!(2, list.count(4));

    for _i in 1..POSITION_LIST_LENGTH {
      list.add(5 as BoardHash);
    }

    assert_eq!(POSITION_LIST_LENGTH, list.len());
    assert_eq!(0, list.count(1));
    assert_eq!(0, list.count(3));
    assert_eq!(POSITION_LIST_LENGTH - 1, list.count(5));

    list.clear();
    assert_eq!(0, list.count(1));
    assert_eq!(0, list.count(3));
    assert_eq!(0, list.count(4));
  }
}
