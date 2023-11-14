// Dependencies
use log::*;
use std::mem;

// From our project
use crate::model::game_state::GameStatus;
use crate::model::tables::zobrist::BoardHash;

/// Struct of evaluation data we save for a given board position
///
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct EvaluationCache {
  pub game_status: GameStatus,
  pub eval: f32,
  pub depth: usize,
}

/// Default values for EvaluationCache
///
impl Default for EvaluationCache {
  fn default() -> Self {
    EvaluationCache {
      game_status: GameStatus::Ongoing,
      eval: f32::NAN,
      depth: 0,
    }
  }
}

#[derive(Copy, Clone, PartialEq, PartialOrd)]
struct EvaluationCacheEntry {
  hash: BoardHash,
  evaluation_cache: EvaluationCache,
}

/// Default values for EvaluationCacheEntry
///
impl Default for EvaluationCacheEntry {
  fn default() -> Self {
    EvaluationCacheEntry {
      hash: 0,
      evaluation_cache: EvaluationCache::default(),
    }
  }
}

pub struct EvaluationCacheTable {
  table: Box<[EvaluationCacheEntry]>,
  max_index_mask: usize,
  /// Keeps track of how many time we access the cache. (both read and write)
  counter: usize,
}

impl EvaluationCacheTable {
  /// Initializes an Evaluation Cache Table.
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
  pub fn new(capacity_mb: usize) -> EvaluationCacheTable {
    debug!(
      "Creating new EvaluationCacheTable with capacity {} MB",
      capacity_mb
    );
    let entry_size = mem::size_of::<EvaluationCacheEntry>();
    let number_of_entries = capacity_mb * 1024 * 1024 / entry_size;

    // Find the power of 2 immediately under
    let mut size: usize = 2;
    while 2_usize.pow((size + 1) as u32) < number_of_entries {
      size += 1;
    }
    let size = 2_usize.pow(size as u32);

    debug!(
      "EvaluationCacheTable will be able to store {} entries",
      size
    );

    // Create a vector with default entries
    let entries = vec![EvaluationCacheEntry::default(); size];
    EvaluationCacheTable {
      table: entries.into_boxed_slice(),
      max_index_mask: size - 1,
      counter: 0,
    }
  }

  /// Get a particular entry with the hash specified
  #[inline]
  pub fn get(&mut self, hash: BoardHash) -> Option<EvaluationCache> {
    self.counter = self.counter.wrapping_add(1);
    let entry = unsafe { *self.table.get_unchecked((hash as usize) & self.max_index_mask) };
    if entry.hash != hash {
      return None;
    }
    Some(entry.evaluation_cache)
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
      "Resizing EvaluationCacheTable with capacity {} MB",
      capacity_mb
    );
    let new_table = EvaluationCacheTable::new(capacity_mb);
    *self = new_table;
  }

  /// Adds (or update) an evaluation cache entry.
  #[inline]
  pub fn add(&mut self, hash: BoardHash, evaluation: EvaluationCache) {
    let e = unsafe { self.table.get_unchecked_mut((hash as usize) & self.max_index_mask) };
    *e = EvaluationCacheEntry {
      hash: hash,
      evaluation_cache: evaluation,
    };
    self.counter = self.counter.wrapping_add(1);
  }

  /// Checks how many time we accessed the cache.
  #[inline]
  pub fn len(&self) -> usize {
    self.counter
  }

  /// Zeroes out all the board hashes in the table and fill with default values.
  #[inline]
  pub fn clear(&mut self) {
    for i in 0..self.max_index_mask {
      let e = unsafe { self.table.get_unchecked_mut(i) };
      *e = EvaluationCacheEntry::default();
    }
    self.counter = 0;
  }
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {

  use super::*;
  use crate::model::game_state::GameState;

  #[test]
  fn test_using_cache_table() {
    //use crate::engine::evaluate_board;
    let mut cache_table = EvaluationCacheTable::new(1024);

    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    let game_state = GameState::from_fen(fen);
    let boardcache = EvaluationCache {
      game_status: GameStatus::WhiteWon,
      eval: 1.0,
      depth: 3,
    };

    cache_table.add(game_state.board.hash, boardcache);

    let board_cache_2 = cache_table.get(game_state.board.hash).unwrap_or_default();
    assert_eq!(boardcache, board_cache_2);

    for i in 3..100000 {
      assert!(cache_table.get(i).is_none());
      cache_table.add(i, boardcache);
      assert!(cache_table.get(i).is_some());
    }

    cache_table.clear();
    for i in 3..100000 {
      assert!(cache_table.get(i).is_none());
      //println!("Pointer: {:p}", cache_table.pointer(i));
    }
  }

  #[test]
  fn test_resize() {
    //use crate::engine::evaluate_board;
    let mut cache_table = EvaluationCacheTable::new(20);

    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    let game_state = GameState::from_fen(fen);
    let boardcache = EvaluationCache {
      game_status: GameStatus::WhiteWon,
      eval: 1.0,
      depth: 3,
    };

    cache_table.add(game_state.board.hash, boardcache);

    let board_cache_2 = cache_table.get(game_state.board.hash).unwrap_or_default();
    assert_eq!(boardcache, board_cache_2);

    for i in 3..100000 {
      assert!(cache_table.get(i).is_none());
      cache_table.add(i, boardcache);
      assert!(cache_table.get(i).is_some());
    }

    cache_table.resize(10);
    for i in 3..100000 {
      assert!(cache_table.get(i).is_none());
    }

    for i in 3..100000 {
      assert!(cache_table.get(i).is_none());
      cache_table.add(i, boardcache);
      assert!(cache_table.get(i).is_some());
    }
  }
}
