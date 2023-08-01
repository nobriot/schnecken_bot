use lazy_static::lazy_static;
use log::*;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::chess::model::moves::*;
use crate::chess::model::game_state::GamePhase;

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
type EngineCacheData = Arc<Mutex<HashMap<String, PositionCache>>>;

pub struct EngineCache {
  cache: EngineCacheData,
}

impl EngineCache {
  pub fn new() -> Self {
    EngineCache {
      cache: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  fn clip_fen(fen: &str) -> String {
    let mut clipped_fen = String::new();
    let fen_parts: Vec<&str> = fen.split(' ').collect();
    for part in fen_parts.iter().take(4) {
      clipped_fen.push_str(part);
    }
    clipped_fen
  }

  pub fn add(&self, fen: &str, position_cache: PositionCache) {
    if self.len() > DEFAULT_CACHE_MAX_SIZE {
      self.clear();
    }
    self
      .cache
      .lock()
      .unwrap()
      .insert(EngineCache::clip_fen(fen), position_cache);
  }

  pub fn len(&self) -> usize {
    return self.cache.lock().unwrap().len();
  }

  pub fn clear(&self) {
    self.cache.lock().unwrap().clear();
  }

  pub fn has_fen(&self, fen: &str) -> bool {
    return self
      .cache
      .lock()
      .unwrap()
      .contains_key(EngineCache::clip_fen(fen).as_str());
  }

  pub fn get_move_list(&self, fen: &str) -> Option<Vec<Move>> {
    return self
      .cache
      .lock()
      .unwrap()
      .get(EngineCache::clip_fen(fen).as_str())
      .unwrap_or(&PositionCache::default())
      .move_list
      .clone();
  }

  pub fn set_move_list(&self, fen: &str, move_list: &[Move]) {
    if !self.has_fen(fen) {
      self.add(fen, PositionCache::default());
    }

    if let Some(entry) = self
      .cache
      .lock()
      .unwrap()
      .get_mut(EngineCache::clip_fen(fen).as_str())
    {
      entry.move_list = Some(move_list.to_owned());
    } else {
      error!("Error updating move list in the cache for {fen}");
    }
  }

  pub fn get_eval(&self, fen: &str) -> Option<f32> {
    return self
      .cache
      .lock()
      .unwrap()
      .get(EngineCache::clip_fen(fen).as_str())
      .unwrap_or(&PositionCache::default())
      .eval;
  }

  pub fn set_eval(&self, fen: &str, eval: f32) {
    if !self.has_fen(fen) {
      self.add(fen, PositionCache::default());
    }

    if let Some(entry) = self
      .cache
      .lock()
      .unwrap()
      .get_mut(EngineCache::clip_fen(fen).as_str())
    {
      entry.eval = Some(eval);
    } else {
      error!("Error updating eval in the cache for {fen}");
    }
  }

  pub fn get_game_phase(&self, fen: &str) -> Option<GamePhase> {
    return self
      .cache
      .lock()
      .unwrap()
      .get(EngineCache::clip_fen(fen).as_str())
      .unwrap_or(&PositionCache::default())
      .game_phase;
  }

  pub fn set_game_phase(&self, fen: &str, game_phase: GamePhase) {
    if !self.has_fen(fen) {
      self.add(fen, PositionCache::default());
    }

    if let Some(entry) = self
      .cache
      .lock()
      .unwrap()
      .get_mut(EngineCache::clip_fen(fen).as_str())
    {
      entry.game_phase = Some(game_phase);
    } else {
      error!("Error updating Game Phase in the cache for {fen}");
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

  #[test]
  fn test_cache_has_key() {
    let engine_cache: EngineCache = EngineCache::new();

    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    engine_cache.add(fen, PositionCache::default());

    // Same position, different move number
    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 46";
    engine_cache.add(fen, PositionCache::default());

    assert_eq!(
      true,
      engine_cache.has_fen("8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 0 0")
    );

    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/7P b - - 8 43";
    assert_eq!(false, engine_cache.has_fen(fen));
  }

  #[test]
  fn test_cache_get_set_data() {
    let engine_cache: EngineCache = EngineCache::new();

    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";

    // Empty cache:
    assert_eq!(0, engine_cache.len());
    assert_eq!(None, engine_cache.get_move_list(fen));

    // Now add the data:
    let position_cache = PositionCache {
      move_list: Some(Vec::new()),
      eval: Some(20.0),
      game_phase: Some(GamePhase::Opening),
    };

    engine_cache.add(fen, position_cache);
    assert_eq!(1, engine_cache.len());

    // Read the cache
    assert_eq!(Vec::<Move>::new(), engine_cache.get_move_list(fen).unwrap());
    assert_eq!(Some(20.0), engine_cache.get_eval(fen));
    assert_eq!(Some(GamePhase::Opening), engine_cache.get_game_phase(fen));

    // Add manually:
    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/7P b - - 8 43";
    assert_eq!(None, engine_cache.get_move_list(fen));
    assert_eq!(None, engine_cache.get_eval(fen));
    assert_eq!(None, engine_cache.get_game_phase(fen));

    // First the move list
    let mut move_list = Vec::new();
    move_list.push(Move::from_string("h3g7"));
    move_list.push(Move::from_string("a1a8Q"));

    engine_cache.set_move_list(fen, &move_list);

    assert_eq!(move_list, engine_cache.get_move_list(fen).unwrap());
    assert_eq!(None, engine_cache.get_eval(fen));
    assert_eq!(None, engine_cache.get_game_phase(fen));

    // Then the eval:
    engine_cache.set_eval(fen, 99.9);
    assert_eq!(move_list, engine_cache.get_move_list(fen).unwrap());
    assert_eq!(Some(99.9), engine_cache.get_eval(fen));
    assert_eq!(None, engine_cache.get_game_phase(fen));

    // Finally the game phase
    engine_cache.set_game_phase(fen, GamePhase::Endgame);
    assert_eq!(move_list, engine_cache.get_move_list(fen).unwrap());
    assert_eq!(Some(99.9), engine_cache.get_eval(fen));
    assert_eq!(Some(GamePhase::Endgame), engine_cache.get_game_phase(fen));

    // Clear the cache:
    engine_cache.clear();
    assert_eq!(0, engine_cache.len());
    assert_eq!(None, engine_cache.get_move_list(fen));
    assert_eq!(None, engine_cache.get_eval(fen));
    assert_eq!(None, engine_cache.get_game_phase(fen));
  }
}
