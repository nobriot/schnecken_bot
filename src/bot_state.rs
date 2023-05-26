use log::*;

use crate::chess::model::piece::Color;

pub struct BotState {
  pub susername: String,
  pub games: Vec<BotGame>,
}

#[derive(Debug, Clone)]
pub struct BotGame {
  // Color played by the bot in the ongoing game
  pub color: Color,
  // Start FEN
  pub start_fen: String,
  // Full Lichess Game ID
  pub full_id: String,
  // Short Lichess Game ID, used in URLs
  pub id: String,
  // Whether it got started, ever
  pub has_moved: bool,
  // If it is our turn or not
  pub is_my_turn: bool,
  // If it is our turn or not
  pub last_move: String,
  pub rated: bool,
}

impl BotState {
  pub fn add_game(&mut self, game: &BotGame) {
    for existing_game in &self.games {
      if existing_game.full_id == game.full_id {
        debug!("Game ID {} already in the cache. Ignoring", game.full_id);
        return;
      }
    }
    debug!("Adding Game ID {} in the cache", game.full_id);
    self.games.push(game.clone());
  }

  pub fn remove_game(&mut self, game_full_id: &str) {
    for i in 0..self.games.len() {
      if self.games[i].full_id == game_full_id {
        self.games.remove(i);
        return;
      }
    }
  }
}
