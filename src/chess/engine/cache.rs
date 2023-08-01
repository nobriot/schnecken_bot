use lazy_static::lazy_static;
use log::*;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::chess::model::game_state::GamePhase;
use crate::chess::model::moves::*;

// How large do we want the cache to grow before we purge it.
const DEFAULT_CACHE_MAX_SIZE: usize = 100_000_000;

#[derive(Debug, Clone, Default)]
pub struct PositionCache {
  // List of moves available for a position
  pub move_list: Option<Vec<Move>>,
  // Evaluation for a position
  pub eval: Option<f32>,
  // game phase for a position
  pub game_phase: Option<GamePhase>,
}

// Short alias for the engine cache
type EngineCacheData = Arc<Mutex<HashMap<u64, PositionCache>>>;

pub struct EngineCache {
  cache: EngineCacheData,
}

impl EngineCache {
  pub fn new() -> Self {
    EngineCache {
      cache: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  pub fn add(&self, board_hash: u64, position_cache: PositionCache) {
    if self.len() > DEFAULT_CACHE_MAX_SIZE {
      // Ideally we should purge old entries
      error!("Clearing the cache due to large size.");
      self.clear();
    }
    self
      .cache
      .lock()
      .unwrap()
      .insert(board_hash, position_cache);
  }

  pub fn len(&self) -> usize {
    return self.cache.lock().unwrap().len();
  }

  pub fn clear(&self) {
    self.cache.lock().unwrap().clear();
  }

  pub fn has_key(&self, board_hash: u64) -> bool {
    return self.cache.lock().unwrap().contains_key(&board_hash);
  }

  pub fn get_move_list(&self, board_hash: &u64) -> Option<Vec<Move>> {
    return self
      .cache
      .lock()
      .unwrap()
      .get(board_hash)
      .unwrap_or(&PositionCache::default())
      .move_list
      .clone();
  }

  pub fn set_move_list(&self, board_hash: u64, move_list: &[Move]) {
    if !self.has_key(board_hash) {
      self.add(board_hash, PositionCache::default());
    }

    if let Some(entry) = self.cache.lock().unwrap().get_mut(&board_hash) {
      entry.move_list = Some(move_list.to_owned());
    } else {
      error!("Error updating move list in the cache for hash {board_hash}");
    }
  }

  pub fn get_eval(&self, board_hash: &u64) -> Option<f32> {
    return self
      .cache
      .lock()
      .unwrap()
      .get(&board_hash)
      .unwrap_or(&PositionCache::default())
      .eval;
  }

  pub fn set_eval(&self, board_hash: u64, eval: f32) {
    if !self.has_key(board_hash) {
      self.add(board_hash, PositionCache::default());
    }

    if let Some(entry) = self.cache.lock().unwrap().get_mut(&board_hash) {
      entry.eval = Some(eval);
    } else {
      error!("Error updating eval in the cache for hash {board_hash}");
    }
  }

  pub fn get_game_phase(&self, board_hash: &u64) -> Option<GamePhase> {
    return self
      .cache
      .lock()
      .unwrap()
      .get(board_hash)
      .unwrap_or(&PositionCache::default())
      .game_phase;
  }

  pub fn set_game_phase(&self, board_hash: u64, game_phase: GamePhase) {
    if !self.has_key(board_hash) {
      self.add(board_hash, PositionCache::default());
    }

    if let Some(entry) = self.cache.lock().unwrap().get_mut(&board_hash) {
      entry.game_phase = Some(game_phase);
    } else {
      error!("Error updating Game Phase in the cache for hash {board_hash}");
    }
  }
}

lazy_static! {
  static ref ENGINE_CACHE: EngineCache = EngineCache::new();
}

pub fn get_engine_cache() -> &'static EngineCache {
  &ENGINE_CACHE
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use super::*;
  use crate::chess::model::game_state::GameState;

  #[test]
  fn test_cache_has_key() {
    let engine_cache: EngineCache = EngineCache::new();

    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    let game_state = GameState::from_fen(fen);
    engine_cache.add(game_state.board.hash, PositionCache::default());

    // Same position, different move number
    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 46";
    let game_state = GameState::from_fen(fen);
    engine_cache.add(game_state.board.hash, PositionCache::default());

    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 0 0";
    let game_state = GameState::from_fen(fen);
    assert_eq!(true, engine_cache.has_key(game_state.board.hash));

    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/7P b - - 8 43";
    let game_state = GameState::from_fen(fen);
    assert_eq!(false, engine_cache.has_key(game_state.board.hash));
  }

  #[test]
  fn test_cache_get_set_data() {
    let engine_cache: EngineCache = EngineCache::new();

    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    let game_state = GameState::from_fen(fen);

    // Empty cache:
    assert_eq!(0, engine_cache.len());
    assert_eq!(None, engine_cache.get_move_list(&game_state.board.hash));

    // Now add the data:
    let position_cache = PositionCache {
      move_list: Some(Vec::new()),
      eval: Some(20.0),
      game_phase: Some(GamePhase::Opening),
    };

    engine_cache.add(game_state.board.hash, position_cache);
    assert_eq!(1, engine_cache.len());

    // Read the cache
    assert_eq!(
      Vec::<Move>::new(),
      engine_cache.get_move_list(&game_state.board.hash).unwrap()
    );
    assert_eq!(Some(20.0), engine_cache.get_eval(&game_state.board.hash));
    assert_eq!(
      Some(GamePhase::Opening),
      engine_cache.get_game_phase(&game_state.board.hash)
    );

    // Add manually:
    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/7P b - - 8 43";
    let game_state = GameState::from_fen(fen);
    assert_eq!(None, engine_cache.get_move_list(&game_state.board.hash));
    assert_eq!(None, engine_cache.get_eval(&game_state.board.hash));
    assert_eq!(None, engine_cache.get_game_phase(&game_state.board.hash));

    // First the move list
    let mut move_list = Vec::new();
    move_list.push(Move::from_string("h3g7"));
    move_list.push(Move::from_string("a1a8Q"));

    engine_cache.set_move_list(game_state.board.hash, &move_list);

    assert_eq!(
      move_list,
      engine_cache.get_move_list(&game_state.board.hash).unwrap()
    );
    assert_eq!(None, engine_cache.get_eval(&game_state.board.hash));
    assert_eq!(None, engine_cache.get_game_phase(&game_state.board.hash));

    // Then the eval:
    engine_cache.set_eval(game_state.board.hash, 99.9);
    assert_eq!(
      move_list,
      engine_cache.get_move_list(&game_state.board.hash).unwrap()
    );
    assert_eq!(Some(99.9), engine_cache.get_eval(&game_state.board.hash));
    assert_eq!(None, engine_cache.get_game_phase(&game_state.board.hash));

    // Finally the game phase
    engine_cache.set_game_phase(game_state.board.hash, GamePhase::Endgame);
    assert_eq!(
      move_list,
      engine_cache.get_move_list(&game_state.board.hash).unwrap()
    );
    assert_eq!(Some(99.9), engine_cache.get_eval(&game_state.board.hash));
    assert_eq!(
      Some(GamePhase::Endgame),
      engine_cache.get_game_phase(&game_state.board.hash)
    );

    // Clear the cache:
    engine_cache.clear();
    assert_eq!(0, engine_cache.len());
    assert_eq!(None, engine_cache.get_move_list(&game_state.board.hash));
    assert_eq!(None, engine_cache.get_eval(&game_state.board.hash));
    assert_eq!(None, engine_cache.get_game_phase(&game_state.board.hash));
  }
}
