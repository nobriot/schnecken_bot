use std::fmt::Display;
use std::ops::Index;

// Our project
use crate::model::moves::*;

/// How many moves do we keep in a move list.
const MOVE_LIST_LENGTH: usize = 100;

/// Move list. Fixed-size array
///
#[derive(Debug, Clone)]
pub struct MoveList {
  moves: [Move; MOVE_LIST_LENGTH],
  length: u8,
}

impl MoveList {
  /// Creates a new empty move list
  ///
  pub fn new() -> Self {
    MoveList {
      moves: [Move::null(); MOVE_LIST_LENGTH],
      length: 0,
    }
  }

  /// Creates a new move list based on a slice
  ///
  pub fn new_from_slice(moves: &[Move]) -> Self {
    debug_assert!(moves.len() < u8::MAX as usize);
    let mut list = MoveList {
      moves: [Move::null(); MOVE_LIST_LENGTH],
      length: std::cmp::min(MOVE_LIST_LENGTH, moves.len()) as u8,
    };

    unsafe {
      let dest = list.moves.as_mut_ptr();
      let src = moves.as_ptr();
      std::ptr::copy_nonoverlapping(src, dest, list.length as usize);
    }

    list
  }

  /// Adds a move at the end of the move list
  ///
  /// ### Arguments
  ///
  /// * `mv` - Move to add to the list
  ///
  pub fn add(&mut self, mv: Move) {
    if self.length as usize >= MOVE_LIST_LENGTH {
      return;
    }
    // SAFETY: We just check above that the length is within bounds.
    unsafe {
      let element = self.moves.get_unchecked_mut(self.length as usize);
      *element = mv;
    }
    self.length += 1;
  }

  /// Clears all the moves from a move list
  ///
  pub fn clear(&mut self) {
    self.moves = [Move::null(); MOVE_LIST_LENGTH];
    self.length = 0;
  }

  /// Remove the last move from the list
  /// and returns it.
  ///
  pub fn pop(&mut self) -> Option<Move> {
    if self.length == 0 {
      return None;
    }

    self.length -= 1;
    // SAFETY: length variable never goes out of bounds.
    Some(unsafe { *self.moves.get_unchecked(self.length as usize) })
  }

  /// Gets the length of a move list.
  ///
  pub fn len(&self) -> usize {
    self.length as usize
  }

  /// Checks if the move list is empty
  pub fn is_empty(&self) -> bool {
    self.length == 0
  }

  /// Returns the capacity of a move list
  ///
  pub const fn capacity(&self) -> usize {
    MOVE_LIST_LENGTH
  }

  /// Returns an optional slice from the current move list.
  ///  
  pub fn get_moves(&self) -> &[Move] {
    if self.length == 0 {
      return &[];
    }

    &self.moves[..self.length as usize]
  }

  /// Returns a copy of the first move of the move list
  ///
  pub fn get_first_move(&self) -> Option<Move> {
    if self.length == 0 {
      return None;
    }
    Some(self.moves[0])
  }

  pub fn to_vec(&self) -> Vec<Move> {
    let length = self.length as usize;
    let mut vector = Vec::with_capacity(length);

    if length == 0 {
      return vector;
    }

    unsafe {
      // memcpy the content of the array in the vector
      let dest = vector.as_mut_ptr();
      let src = self.moves.as_ptr();
      std::ptr::copy_nonoverlapping(src, dest, length);
      vector.set_len(length);
    }
    vector
  }
}

// -----------------------------------------------------------------------------
// Common traits

impl IntoIterator for MoveList {
  type Item = Move;
  type IntoIter = MoveListIterator;

  fn into_iter(self) -> Self::IntoIter {
    MoveListIterator {
      move_list: self,
      index: 0,
    }
  }
}

pub struct MoveListIterator {
  move_list: MoveList,
  index: usize,
}

impl Iterator for MoveListIterator {
  type Item = Move;
  fn next(&mut self) -> Option<Self::Item> {
    if self.index >= self.move_list.len() {
      return None;
    }

    let item = self.move_list[self.index];
    self.index += 1;
    Some(item)
  }
}

impl Index<usize> for MoveList {
  type Output = Move;

  fn index(&self, index: usize) -> &Self::Output {
    &self.moves[index]
  }
}

impl Display for MoveList {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for mv in &self.moves {
      if mv.is_null() {
        break;
      }
      write!(f, " {}", mv)?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_move_list_container() {
    let mut list = MoveList::new();

    assert_eq!(0, list.len());
    assert_eq!(MOVE_LIST_LENGTH, list.capacity());
    assert_eq!(None, list.pop());
    assert_eq!(None, list.get_first_move());
    assert_eq!(0, list.get_moves().len());

    list.add(Move::from_string("a4b7"));
    list.add(Move::from_string("f8g6"));

    assert_eq!(2, list.len());
    assert_eq!(2, list.get_moves().len());
    let expected_slice = [Move::from_string("a4b7"), Move::from_string("f8g6")];
    assert_eq!(expected_slice, list.get_moves());
    assert_eq!(Move::from_string("a4b7"), list.get_first_move().unwrap());
    assert_eq!(Move::from_string("f8g6"), list.pop().unwrap());
    assert_eq!(1, list.get_moves().len());

    list.clear();
    assert_eq!(0, list.len());
    assert_eq!(None, list.pop());
    assert_eq!(None, list.get_first_move());
    assert_eq!(0, list.get_moves().len());

    let slice = [
      Move::from_string("a4b7"),
      Move::from_string("f8g6"),
      Move::from_string("b2c7"),
      Move::from_string("a1g8"),
      Move::from_string("f1f3"),
    ];
    let mut list = MoveList::new_from_slice(&slice);

    assert_eq!(list.len(), slice.len());
    assert_eq!(slice[0], list.get_first_move().unwrap());
    assert_eq!(*slice.last().unwrap(), list.pop().unwrap());

    let mut count = 0;
    for m in list {
      count += 1;
      assert!(!m.is_null());
    }
    assert_eq!(count, 4);

    let mut list = MoveList::new_from_slice(&slice);
    let vector = list.to_vec();
    assert_eq!(list.len(), vector.len());
    assert_eq!(slice.len(), vector.len());
  }
}
