use log::*;

use crate::chess::model::board::INVALID_SQUARE;
use crate::chess::model::board::{Board, Move};
use crate::chess::model::piece::Color;

/// Start game state for a standard chess game.
pub const START_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
pub struct CastlingRights {
  pub K: bool,
  pub Q: bool,
  pub k: bool,
  pub q: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct GameState {
  pub side_to_play: Color,
  pub is_check: bool,
  pub is_checkmate: bool,
  pub en_passant_square: u8,
  pub castling_rights: CastlingRights,
  pub board: Board,
  pub ply: u8,
  pub move_count: u8,
}

// -----------------------------------------------------------------------------
// Game state implementation

impl GameState {
  /// Takes a full FEN notation and converts it into a Game State
  pub fn from_string(fen: &str) -> Self {
    let fen_parts: Vec<&str> = fen.split(" ").collect();
    if fen_parts.len() < 6 {
      error!("Fen too small to generate a game state");
      return GameState::default();
    }

    let board = Board::from_string(fen_parts[0]);
    let side_to_move;
    if fen_parts[1] == "w" {
      side_to_move = Color::White;
    } else {
      side_to_move = Color::Black;
    }
    let casting_rights = CastlingRights {
      K: fen_parts[2].contains("K"),
      Q: fen_parts[2].contains("Q"),
      k: fen_parts[2].contains("k"),
      q: fen_parts[2].contains("q"),
    };

    let en_passant_square;
    if fen_parts[3] != "-" {
      en_passant_square = Board::string_to_square(fen_parts[3]);
    } else {
      en_passant_square = INVALID_SQUARE;
    }

    let ply: u8 = fen_parts[4].parse::<u8>().unwrap_or(0);
    let move_count: u8 = fen_parts[5].parse::<u8>().unwrap_or(0);

    // FIXME: Determine if we're check / Checkmate
    GameState {
      side_to_play: side_to_move,
      is_check: false,
      is_checkmate: false,
      en_passant_square: en_passant_square,
      castling_rights: casting_rights,
      board: board,
      ply: ply,
      move_count: move_count,
    }
  }

  pub fn to_string(&self) -> String {
    let mut fen = String::new();

    fen += self.board.to_string().as_str();
    fen.push(' ');
    match self.side_to_play {
      Color::White => fen.push('w'),
      Color::Black => fen.push('b'),
    }
    fen.push(' ');

    if self.castling_rights.K == true {
      fen.push('K');
    }
    if self.castling_rights.Q == true {
      fen.push('Q');
    }
    if self.castling_rights.k == true {
      fen.push('k');
    }
    if self.castling_rights.q == true {
      fen.push('q');
    }
    if self.castling_rights.K == false
      && self.castling_rights.Q == false
      && self.castling_rights.q == false
      && self.castling_rights.k == false
    {
      fen.push('-');
    }
    fen.push(' ');

    if self.en_passant_square != INVALID_SQUARE {
      fen += Board::square_to_string(self.en_passant_square).as_str();
    } else {
      fen.push('-');
    }
    fen.push(' ');
    fen += self.ply.to_string().as_str();
    fen.push(' ');
    fen += self.move_count.to_string().as_str();

    fen
  }

  // Get all the possible moves in a position
  pub fn get_moves(&self) -> Vec<Move> {
    match self.side_to_play {
      Color::White => self.get_white_moves(),
      Color::Black => self.get_black_moves(),
    }
  }

  // Get all the possible moves for white in a position
  pub fn get_white_moves(&self) -> Vec<Move> {
    let legal_moves = Vec::new();

    for square in 0..64 {
      if !self.board.has_piece_with_color(square, Color::White) {
        continue;
      }
    }

    legal_moves
  }

  // Get all the possible moves for black in a position
  pub fn get_black_moves(&self) -> Vec<Move> {
    let legal_moves = Vec::new();

    legal_moves
  }
}

// -----------------------------------------------------------------------------
// Display/Default implementations for our game state

impl std::fmt::Display for GameState {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut message = String::from("\n");
    message += format!(
      "Move: {}, Ply: {} Side to play {} - Check {} - Checkmate {}\n",
      self.move_count, self.ply, self.side_to_play, self.is_check, self.is_checkmate
    )
    .as_str();
    message += format!(
      "En passant: {} - Castling rights: ({},{},{},{})\n",
      Board::square_to_string(self.en_passant_square),
      self.castling_rights.K,
      self.castling_rights.Q,
      self.castling_rights.k,
      self.castling_rights.q
    )
    .as_str();

    message += format!("Board: {}\n", self.board.to_string()).as_str();

    f.write_str(message.as_str())
  }
}

impl Default for GameState {
  fn default() -> Self {
    GameState {
      side_to_play: Color::White,
      is_check: false,
      is_checkmate: false,
      en_passant_square: INVALID_SQUARE,
      castling_rights: CastlingRights {
        K: true,
        Q: true,
        k: true,
        q: true,
      },
      board: Board::from_string(START_POSITION_FEN),
      ply: 0,
      move_count: 1,
    }
  }
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn game_state_display_test() {
    let game_state = GameState::from_string(START_POSITION_FEN);
    assert_eq!(START_POSITION_FEN, game_state.to_string().as_str());
    println!("{game_state}");

    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    let game_state = GameState::from_string(fen);
    assert_eq!(fen, game_state.to_string().as_str());
    println!("{game_state}");

    let fen = "5rk1/3b1p2/1r3p1p/p1pPp3/8/1P6/P3BPPP/R1R3K1 w - c6 0 23";
    let game_state = GameState::from_string(fen);
    assert_eq!(fen, game_state.to_string().as_str());
    println!("{game_state}");

    let fen = "r2qk2r/p1pb1ppp/3bpn2/8/2BP4/2N2Q2/PP3PPP/R1B2RK1 b kq - 2 12";
    let game_state = GameState::from_string(fen);
    assert_eq!(fen, game_state.to_string().as_str());
    println!("{game_state}");
  }
}
