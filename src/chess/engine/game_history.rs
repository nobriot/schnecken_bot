use crate::engine::search_result::Variation;
use crate::model::moves::*;
use std::fmt;
use std::fmt::Display;

// Keeping here a table of how the game went
#[derive(Debug, Clone)]
pub struct GameHistoryEntry {
  /// FEN string of a given position
  pub position:  String,
  /// Last move.
  /// Will be a NULL move for the first position of the game.
  pub last_move: Move,
  /// Evaluation in centipawns.
  pub eval:      isize,
  /// Evaluation in centipawns.
  pub pv:        Variation,
}

/// Keeps track of the historical evaluations during a game.
#[derive(Debug, Clone)]
pub struct GameHistory {
  entries: Vec<GameHistoryEntry>,
}

pub struct GameHistoryIterator {
  game_history: GameHistory,
  index:        usize,
}

impl GameHistory {
  /// Creates a new instance of a game history.
  ///
  /// ### Return value
  pub fn new() -> Self {
    GameHistory { entries: Vec::<GameHistoryEntry>::new(), }
  }

  /// Adds a position and its evaluation from the engine in the Game History
  ///
  /// ### Arguments
  ///
  /// * `fen`:
  pub fn add(&mut self, fen: String, last_move: Move, eval: isize, pv: Variation) {
    self.entries.push(GameHistoryEntry { position: fen,
                                         last_move,
                                         eval,
                                         pv: pv.clone() })
  }

  /// Pops the last entry in the game history
  pub fn pop(&mut self) -> Option<GameHistoryEntry> {
    self.entries.pop()
  }

  /// Gets the length of the current game history
  pub fn len(&self) -> usize {
    self.entries.len()
  }

  /// Clears the game history. Can be used to start a new game
  pub fn clear(&mut self) {
    self.entries.clear()
  }
}

impl IntoIterator for GameHistory {
  type IntoIter = GameHistoryIterator;
  type Item = GameHistoryEntry;

  fn into_iter(self) -> Self::IntoIter {
    GameHistoryIterator { game_history: self,
                          index:        0, }
  }
}

impl Display for GameHistory {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for entry in &self.entries {
      if entry.last_move.is_null() {
        writeln!(f, "{:6} - start position / {} - [{}]", entry.eval, entry.position, entry.pv)?;
      } else {
        writeln!(f,
                 "{:6} - {:14} / {} - [{}]",
                 entry.eval,
                 entry.last_move.to_string(),
                 entry.position,
                 entry.pv)?;
      }
    }
    Ok(())
  }
}

impl Default for GameHistory {
  fn default() -> Self {
    GameHistory::new()
  }
}

impl Iterator for GameHistoryIterator {
  type Item = GameHistoryEntry;

  fn next(&mut self) -> Option<GameHistoryEntry> {
    if self.index < self.game_history.len() {
      self.index += 1;
      return Some(self.game_history.entries[self.index - 1].clone());
    }
    None
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::model::game_state::*;

  #[test]
  fn test_game_history() {
    let mut history = GameHistory::new();

    let fen = "rnbqkb1r/1p2pppp/p2p1n2/8/3NP3/2N5/PPP2PPP/R1BQKB1R w KQkq - 0 6";
    let mut game_state = GameState::from_fen(fen);

    history.add(game_state.to_fen(), Move::null(), 40, Variation::new());

    let mv = "f2f3";
    game_state.apply_move_from_notation(mv);
    history.add(game_state.to_fen(), Move::from_string(mv), 30, Variation::new());

    assert_eq!(history.len(), 2);

    let mv = "e7e5";
    game_state.apply_move_from_notation(mv);
    history.add(game_state.to_fen(), Move::from_string(mv), -50, Variation::new());

    assert_eq!(history.len(), 3);

    println!("History is:\n{}", history);

    let entry = history.pop();
    let entry = entry.unwrap();
    assert_eq!(entry.eval, -50);
    assert_eq!(history.len(), 2);
  }
}
