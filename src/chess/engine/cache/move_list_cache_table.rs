// Dependencies
use log::*;
use std::mem;

// From our project
use crate::model::moves::Move;
use crate::model::tables::zobrist::BoardHash;

// FIXME: Many similarities between evaluation_table and move_list_cache_table, we should template it.

#[derive(Clone)]
struct MoveListCacheEntry {
  hash: BoardHash,
  move_list: Option<Vec<Move>>,
}

/// Default values for EvaluationCacheEntry
///
impl Default for MoveListCacheEntry {
  fn default() -> Self {
    MoveListCacheEntry {
      hash: 0,
      move_list: None,
    }
  }
}

pub struct MoveListCacheTable {
  table: Box<[MoveListCacheEntry]>,
  max_index_mask: usize,
}

impl MoveListCacheTable {
  /// Initializes an MoveList Cache Table.
  ///
  /// ### Arguments
  ///
  /// * `Capacity`: Total size of the table, in MB.
  ///
  /// ### Return value
  ///
  /// An Evaluation Cache table
  ///
  #[inline]
  pub fn new(capacity_mb: usize) -> MoveListCacheTable {
    debug!(
      "Creating new MoveList cache table with capacity {} MB",
      capacity_mb
    );
    let entry_size = mem::size_of::<MoveListCacheEntry>();
    let number_of_entries = capacity_mb * 1024 * 1024 / entry_size;

    // Find the power of 2 immediately under
    let mut size: usize = 2;
    while 2_usize.pow((size + 1) as u32) < number_of_entries {
      size += 1;
    }
    let size = 2_usize.pow(size as u32);

    debug!("MoveListCacheTable will be able to store {} entries", size);

    // Create a vector with default entries
    let entries = vec![MoveListCacheEntry::default(); size];
    MoveListCacheTable {
      table: entries.into_boxed_slice(),
      max_index_mask: size - 1,
    }
  }

  /// Get a particular entry with the hash specified
  #[inline]
  pub fn get(&self, hash: BoardHash) -> Option<Vec<Move>> {
    let entry = unsafe { self.table.get_unchecked((hash as usize) & self.max_index_mask) };
    if entry.hash != hash {
      return None;
    }
    if entry.move_list.is_none() {
      return None;
    }

    Some(entry.move_list.as_ref().unwrap().clone())
  }

  /// Adds (or update) an evaluation cache entry.
  #[inline]
  pub fn add(&mut self, hash: BoardHash, list: &[Move]) {
    let e = unsafe { self.table.get_unchecked_mut((hash as usize) & self.max_index_mask) };
    *e = MoveListCacheEntry {
      hash: hash,
      move_list: Some(list.to_vec().clone()),
    };
  }

  /// Zeroes out all the board hashes in the table and fill with default values.
  #[inline]
  pub fn clear(&mut self) {
    for i in 0..self.max_index_mask {
      let e = unsafe { self.table.get_unchecked_mut(i) };
      *e = MoveListCacheEntry::default();
    }
  }

  /// Resize the table with a new capacity
  /// Note that the previous data will be zero'ed out
  ///
  /// ### Arguments
  ///
  /// * `self`:     Table to update.
  /// * `Capacity`: New size for the table, in MB.
  ///
  ///
  #[inline]
  pub fn resize(&mut self, capacity_mb: usize) {
    debug!(
      "Resizing MoveListCacheTable with capacity {} MB",
      capacity_mb
    );
    let new_table = MoveListCacheTable::new(capacity_mb);
    *self = new_table;
  }
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {

  use super::*;
  use crate::model::game_state::GameState;

  #[test]
  fn test_using_move_list_cache_table() {
    let mut cache_table = MoveListCacheTable::new(1024);

    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    let game_state = GameState::from_fen(fen);
    let moves = game_state.get_moves();

    assert_eq!(None, cache_table.get(game_state.board.hash));
    cache_table.add(game_state.board.hash, &moves);
    assert!(cache_table.get(game_state.board.hash).is_some());

    let saved_moves = cache_table.get(game_state.board.hash).unwrap();
    assert_eq!(saved_moves, moves);
  }
}
