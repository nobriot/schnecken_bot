use std::borrow::Borrow;

use log::*;

use crate::chess::model::board::*;
use crate::chess::model::piece::NO_PIECE;
use crate::chess::model::piece::*;
use crate::chess::model::piece_moves::*;

// Shows "interesting" squares to control on the board
// Giving them a score
pub const HEATMAP_SCORES: [f32; 64] = [
  // 1st row
  0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, // 2nd row
  0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, // 3rd row
  0.01, 0.01, 0.02, 0.02, 0.02, 0.02, 0.01, 0.01, // 4th row
  0.01, 0.01, 0.02, 0.03, 0.03, 0.02, 0.01, 0.01, // 5th row
  0.01, 0.01, 0.02, 0.03, 0.03, 0.02, 0.01, 0.01, // 6th row
  0.01, 0.01, 0.02, 0.02, 0.02, 0.02, 0.01, 0.01, // 7th row
  0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, // 8th row
  0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01,
];

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
  pub checks: u8,
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
      en_passant_square = string_to_square(fen_parts[3]);
    } else {
      en_passant_square = INVALID_SQUARE;
    }

    let ply: u8 = fen_parts[4].parse::<u8>().unwrap_or(0);
    let move_count: u8 = fen_parts[5].parse::<u8>().unwrap_or(0);

    let mut game_state = GameState {
      side_to_play: side_to_move,
      checks: 0,
      en_passant_square: en_passant_square,
      castling_rights: casting_rights,
      board: board,
      ply: ply,
      move_count: move_count,
    };
    // Determine if we're check / Checkmate
    game_state.update_checks();
    game_state
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
      fen += square_to_string(self.en_passant_square).as_str();
    } else {
      fen.push('-');
    }
    fen.push(' ');
    fen += self.ply.to_string().as_str();
    fen.push(' ');
    fen += self.move_count.to_string().as_str();

    fen
  }

  pub fn get_piece_destinations(&self, source_square: usize, op: u64, ssp: u64) -> (u64, bool) {
    let mut promotion: bool = false;
    let destinations = match self.board.squares[source_square] {
      WHITE_KING | BLACK_KING => get_king_moves(ssp, op, source_square),
      WHITE_QUEEN | BLACK_QUEEN => get_queen_moves(ssp, op, source_square),
      WHITE_ROOK | BLACK_ROOK => get_rook_moves(ssp, op, source_square),
      WHITE_BISHOP | BLACK_BISHOP => get_bishop_moves(ssp, op, source_square),
      WHITE_KNIGHT | BLACK_KNIGHT => get_knight_moves(ssp, op, source_square),
      WHITE_PAWN => {
        let pawn_targets;
        if self.en_passant_square != INVALID_SQUARE {
          pawn_targets = op | (1 << self.en_passant_square);
        } else {
          pawn_targets = op;
        }
        if (source_square / 8) == 6 {
          promotion = true;
        }
        get_white_pawn_moves(ssp, pawn_targets, source_square)
      },
      BLACK_PAWN => {
        let pawn_targets;
        if self.en_passant_square != INVALID_SQUARE {
          pawn_targets = op | (1 << self.en_passant_square);
        } else {
          pawn_targets = op;
        }
        if (source_square / 8) == 1 {
          promotion = true;
        }
        get_black_pawn_moves(ssp, pawn_targets, source_square)
      },
      _ => 0,
    };

    (destinations, promotion)
  }

  /// Computes the number of attackers on each square for a color.
  /// Compare with your own color to get a number of defenders.
  pub fn get_heatmap(&self, color: Color, with_x_rays: bool) -> [usize; 64] {
    let mut heatmap: [usize; 64] = [0; 64];
    let opposite_color = Color::opposite(color);

    let ssp = self.board.get_color_mask(color);
    let op = match with_x_rays {
      true => 0,
      false => self.board.get_color_mask(opposite_color),
    };

    for source_square in 0..64 as usize {
      if !self.board.has_piece_with_color(source_square as u8, color) {
        continue;
      }
      let (destinations, _) = self.get_piece_destinations(source_square, op, ssp);
      for i in 0..64 {
        if ((1 << i) & destinations) != 0 {
          heatmap[i] += 1;
        }
      }
    }

    heatmap
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
    let mut all_moves = Vec::new();

    let ssp = self.board.get_color_mask(Color::White);
    let op = self.board.get_color_mask(Color::Black);

    // Only generate moves if we have a piece on the square
    for source_square in 0..64 as usize {
      if !self
        .board
        .has_piece_with_color(source_square as u8, Color::White)
      {
        continue;
      }

      let (destinations, promotion) = self.get_piece_destinations(source_square, op, ssp);

      for i in 0..64 {
        if ((1 << i) & destinations) != 0 {
          if !promotion {
            all_moves.push(Move {
              src: source_square as u8,
              dest: i,
              promotion: NO_PIECE,
            });
          } else {
            for promotion_piece in WHITE_QUEEN..WHITE_PAWN {
              all_moves.push(Move {
                src: source_square as u8,
                dest: i,
                promotion: promotion_piece,
              });
            }
          }
        }
      }
    }

    let black_heatmap = self.get_heatmap(Color::Black, false);
    if self.castling_rights.K == true
      && self.checks == 0
      && !self.board.has_piece(5)
      && !self.board.has_piece(6)
      && black_heatmap[5] == 0
      && black_heatmap[6] == 0
    {
      all_moves.push(Move {
        src: 4u8,
        dest: 6u8,
        promotion: NO_PIECE,
      });
    }
    if self.castling_rights.Q == true
      && self.checks == 0
      && !self.board.has_piece(1)
      && !self.board.has_piece(2)
      && !self.board.has_piece(3)
      && black_heatmap[2] == 0
      && black_heatmap[3] == 0
    {
      all_moves.push(Move {
        src: 4u8,
        dest: 2u8,
        promotion: NO_PIECE,
      });
    }

    // Now we need to remove all the moves where the moving side king is still in check.
    let mut illegal_moves = Vec::new();
    for m in &all_moves {
      let mut new_game_state = self.clone();
      new_game_state.apply_move(*m);
      let new_black_heatmap = new_game_state.get_heatmap(Color::Black, false);

      let king_square = new_game_state.board.get_white_king_square();
      if king_square == INVALID_SQUARE {
        illegal_moves.push(m);
      } else if new_black_heatmap[king_square as usize] != 0 {
        // We're in check, illegal move
        illegal_moves.push(m);
      }
    }

    // Now remove all the illegal moves
    let mut legal_moves: Vec<Move> = Vec::new();
    for m in &all_moves {
      if !illegal_moves.contains(&m) {
        legal_moves.push(*m);
      }
    }

    legal_moves
  }

  // Get all the possible moves for black in a position
  pub fn get_black_moves(&self) -> Vec<Move> {
    let mut all_moves = Vec::new();

    let ssp = self.board.get_color_mask(Color::Black);
    let op = self.board.get_color_mask(Color::White);

    // Only generate moves if we have a piece on the square
    for source_square in 0..64 as usize {
      if !self
        .board
        .has_piece_with_color(source_square as u8, Color::Black)
      {
        continue;
      }

      let (destinations, promotion) = self.get_piece_destinations(source_square, op, ssp);
      for i in 0..64 {
        if ((1 << i) & destinations) != 0 {
          if !promotion {
            all_moves.push(Move {
              src: source_square as u8,
              dest: i,
              promotion: NO_PIECE,
            });
          } else {
            for promotion_piece in BLACK_QUEEN..BLACK_PAWN {
              all_moves.push(Move {
                src: source_square as u8,
                dest: i,
                promotion: promotion_piece,
              });
            }
          }
        }
      }
    }

    // Now check castling.
    let white_heatmap = self.get_heatmap(Color::White, false);

    if self.castling_rights.k == true
      && self.checks == 0
      && !self.board.has_piece(62)
      && !self.board.has_piece(61)
      && white_heatmap[61] == 0
      && white_heatmap[62] == 0
    {
      all_moves.push(Move {
        src: 60u8,
        dest: 62u8,
        promotion: NO_PIECE,
      });
    }
    if self.castling_rights.q == true
      && self.checks == 0
      && !self.board.has_piece(59)
      && !self.board.has_piece(58)
      && !self.board.has_piece(57)
      && white_heatmap[59] == 0
      && white_heatmap[58] == 0
    {
      all_moves.push(Move {
        src: 60u8,
        dest: 58u8,
        promotion: NO_PIECE,
      });
    }

    // Now we need to remove all the moves where the moving side king is still in check.
    let mut illegal_moves = Vec::new();
    for m in &all_moves {
      let mut new_game_state = self.clone();
      new_game_state.apply_move(*m);
      let new_white_heatmap = new_game_state.get_heatmap(Color::White, false);

      let king_square = new_game_state.board.get_black_king_square();
      if king_square == INVALID_SQUARE {
        illegal_moves.push(m);
      } else if new_white_heatmap[king_square as usize] != 0 {
        // We're in check, illegal move
        illegal_moves.push(m);
      }
    }

    // Now remove all the illegal moves
    let mut legal_moves: Vec<Move> = Vec::new();
    for m in &all_moves {
      if !illegal_moves.contains(&m) {
        legal_moves.push(*m);
      }
    }

    legal_moves
  }

  pub fn get_king_square(&self) -> u8 {
    match self.side_to_play {
      Color::White => self.board.get_white_king_square(),
      Color::Black => self.board.get_black_king_square(),
    }
  }

  // Returns a position score, for the side to play
  pub fn evaluate_position(&self) -> (f32, bool) {
    // Check if we are checkmated or stalemated
    if self.get_moves().len() == 0 {
      match (self.side_to_play, self.checks) {
        (_, 0) => return (0.0, true),
        (Color::Black, _) => return (200.0, true),
        (Color::White, _) => return (-200.0, true),
      }
    }

    // Basic material count
    let mut score: f32 = 0.0;
    for i in 0..64 {
      match self.board.squares[i] {
        WHITE_QUEEN => score += 9.5,
        WHITE_ROOK => score += 5.0,
        WHITE_BISHOP => score += 3.05,
        WHITE_KNIGHT => score += 3.0,
        WHITE_PAWN => score += 1.0,
        BLACK_QUEEN => score -= 9.5,
        BLACK_ROOK => score -= 5.0,
        BLACK_BISHOP => score -= 3.05,
        BLACK_KNIGHT => score -= 3.0,
        BLACK_PAWN => score -= 1.0,
        _ => {},
      }
    }

    // Compare the mobility of both sides. Give +1 if one side has 15 more available moves than the other.
    score +=
      (self.get_white_moves().len() as isize - self.get_black_moves().len() as isize) as f32 / 15.0;

    // Get a pressure score, if one side has more attackers than defenders on a square, they get bonus points
    let white_heatmap = self.get_heatmap(Color::White, false);
    let black_heatmap = self.get_heatmap(Color::Black, false);

    for i in 0..64 {
      match self.board.squares[i] {
        WHITE_KING => score -= black_heatmap[i] as f32, // This means checks.
        WHITE_QUEEN | BLACK_QUEEN => {
          score += (white_heatmap[i] as f32 - black_heatmap[i] as f32) * 9.0
        },
        WHITE_ROOK | BLACK_ROOK => {
          score += (white_heatmap[i] as f32 - black_heatmap[i] as f32) * 4.8
        },
        WHITE_BISHOP | BLACK_BISHOP => {
          score += (white_heatmap[i] as f32 - black_heatmap[i] as f32) * 3.05
        },
        WHITE_KNIGHT | BLACK_KNIGHT => {
          score += (white_heatmap[i] as f32 - black_heatmap[i] as f32) * 3.0
        },
        WHITE_PAWN | BLACK_PAWN => {
          score += (white_heatmap[i] as f32 - black_heatmap[i] as f32) * 1.0
        },
        BLACK_KING => score += white_heatmap[i] as f32, // This means checks.
        _ => score += (white_heatmap[i] as f32 - black_heatmap[i] as f32) * HEATMAP_SCORES[i],
      }
    }

    (score, false)
  }

  pub fn update_checks(&mut self) {
    let opponent_color = Color::opposite(self.side_to_play);
    let opponent_heatmap = self.get_heatmap(opponent_color, false);
    let king_square = self.get_king_square();

    if king_square == INVALID_SQUARE {
      error!("Can't get king square ? {}", self.to_string());
      self.checks = 0;
      return;
    }
    self.checks = opponent_heatmap[king_square as usize] as u8;
  }

  pub fn apply_move(&mut self, chess_move: Move) -> () {
    // Check if the right side is moving:
    match Piece::color(self.board.squares[chess_move.src as usize]) {
      Some(_) => {},
      None => {
        error!(
          "Input moves with empty source square? {} - board:\n{}",
          chess_move, self.board
        );
        return;
      },
    }

    // Check the ply count first:
    if self.board.squares[chess_move.dest as usize] != NO_PIECE
      || self.board.squares[chess_move.src as usize] == WHITE_PAWN
      || self.board.squares[chess_move.src as usize] == BLACK_PAWN
    {
      self.ply = 0;
    }

    // En passant needs to remove the captured pawn.
    if (self.board.squares[chess_move.src as usize] == WHITE_PAWN
      || self.board.squares[chess_move.src as usize] == BLACK_PAWN)
      && self.en_passant_square == chess_move.dest
    {
      match (chess_move.dest as isize - chess_move.src as isize) {
        7 => self.board.squares[(chess_move.src - 1) as usize] = NO_PIECE,
        9 => self.board.squares[(chess_move.src + 1) as usize] = NO_PIECE,
        -7 => self.board.squares[(chess_move.src + 1) as usize] = NO_PIECE,
        -9 => self.board.squares[(chess_move.src - 1) as usize] = NO_PIECE,
        _ => {
          error!("Something when wrong when trying to remove captured piece from En-passant move.");
        },
      }
    }

    self.board.apply_move(chess_move);
    if self.side_to_play == Color::White {
      self.side_to_play = Color::Black;
    } else {
      self.move_count += 1;
      self.side_to_play = Color::White;
    }

    // Update castling rights.(just look if something from the rook/king)
    match chess_move.src {
      0 => self.castling_rights.Q = false,
      4 => {
        self.castling_rights.K = false;
        self.castling_rights.Q = false
      },
      7 => self.castling_rights.K = false,
      56 => self.castling_rights.q = false,
      60 => {
        self.castling_rights.k = false;
        self.castling_rights.q = false
      },
      63 => self.castling_rights.k = false,
      _ => {},
    }

    // Check if we have a en passant square
    if (self.board.squares[chess_move.dest as usize] == WHITE_PAWN
      || self.board.squares[chess_move.dest as usize] == BLACK_PAWN)
      && (chess_move.dest as isize - chess_move.src as isize).abs() == 16
    {
      self.en_passant_square = (chess_move.dest + chess_move.src) / 2;
    } else {
      self.en_passant_square = INVALID_SQUARE;
    }

    // Check if we have checks
    self.update_checks();
  }

  // Applies all moves from a vector of moves
  pub fn apply_moves(&mut self, move_list: &Vec<Move>) -> () {
    for chess_move in move_list {
      self.apply_move(*chess_move);
    }
  }

  pub fn apply_move_list(&mut self, move_list: &str) -> () {
    if move_list.is_empty() {
      return;
    }

    let moves: Vec<&str> = move_list.split(' ').collect();
    for chess_move in moves {
      self.apply_move(Move::from_string(&chess_move));
    }
  }
}

// -----------------------------------------------------------------------------
// Display/Default implementations for our game state
impl std::fmt::Display for GameState {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut message = String::from("\n");
    message += format!(
      "Move: {}, Ply: {} Side to play {} - Checks {}\n",
      self.move_count, self.ply, self.side_to_play, self.checks
    )
    .as_str();
    message += format!(
      "En passant: {} - Castling rights: ({},{},{},{})\n",
      square_to_string(self.en_passant_square),
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
      checks: 0,
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

  #[test]
  fn test_get_list_of_moves() {
    let fen = "r2qk2r/p1pb1ppp/3bpn2/8/2BP4/2N2Q2/PP3PPP/R1B2RK1 b kq - 2 12";
    let game_state = GameState::from_string(fen);
    let move_list = game_state.get_moves();
    assert_eq!(38, move_list.len());
    println!("List of moves (should include castling!):\n");
    for m in move_list {
      println!("{m}");
    }

    let fen = "5k2/P7/2p5/1p6/3P2NR/1p2p3/1P4q1/1K6 w - - 0 53";
    let game_state = GameState::from_string(fen);
    let move_list = game_state.get_moves();
    assert_eq!(21, move_list.len());
    println!("List of moves (should include a promotion):\n");
    for m in move_list {
      println!("{m}");
    }

    let fen = "r2q1rk1/p2b1ppp/3bpn2/2pP4/2B5/2N2Q2/PP3PPP/R1B2RK1 w - c6 0 14";
    let mut game_state = GameState::from_string(fen);
    let move_list = game_state.get_moves();
    println!("List of moves (should include a en-passant capture):\n");
    for m in &move_list {
      println!("{m}");
    }
    assert_eq!(43, move_list.len());

    // Apply the en-passant move, check that the destination capture pawn is gone.
    let en_passant_move = Move::from_string("d5c6");
    game_state.apply_move(en_passant_move);
    let expected_fen = "r2q1rk1/p2b1ppp/2Pbpn2/8/2B5/2N2Q2/PP3PPP/R1B2RK1 b - - 0 14";
    assert_eq!(expected_fen, game_state.to_string());
  }

  #[test]
  fn test_apply_some_moves() {
    let fen = "r2qk2r/p1pb1ppp/3bpn2/8/2BP4/2N2Q2/PP3PPP/R1B2RK1 b kq - 2 12";
    let mut game_state = GameState::from_string(fen);
    game_state.apply_move(Move::from_string("a7a5"));

    let expected_fen = "r2qk2r/2pb1ppp/3bpn2/p7/2BP4/2N2Q2/PP3PPP/R1B2RK1 w kq a6 0 13";
    assert_eq!(expected_fen, game_state.to_string().as_str());
    println!("{game_state}");
    /*
    game_state = GameState::default();
    game_state.apply_move_list("d2d4 g8f6 c2c4 e7e6 g1f3 d7d5 b1c3 d5c4 e2e4 c8b4 f1c4 g7e5 e1g1 b8f8 d1a4 c7c6 g1e2 b4d6 e2c3 d6b6 a4c6 b7d7 d2f2 g8e8 c1g5 f8b8 a1b1 h8h7 g5h4 b8b6 d4e5 g6e5 e5e6 f7e6 f2f4 e8g8 b1e1 g8g7 d3e4 f6g4 h2h3 d8b8 e1c2 b6c4 b2b3 f8h8 f3e5 c4e5 f4e5 c6c5 h4g3 g7f6 a2a4 f6e5 g3g6 e8f8 a4a5 f8e7 f2e2 f7f5 c2c3 e7g5 g2g4 g5c5 c3c4 h7h3 e5f3 e6f4 d4d5 d7c6 e1g1");
    let expected_fen = "8/p1pk1r2/2Nb3p/8/2P2P2/2Q1n2p/P4qPP/6RK b - - 0 36";
    assert_eq!(expected_fen, game_state.to_string().as_str());
    println!("{game_state}");
    */
  }

  #[test]
  fn test_check_legal_moves() {
    let fen = "4B3/p5k1/1pp4p/8/8/P6P/5PP1/2R3K1 b - - 0 37";
    let game_state = GameState::from_string(fen);
    let move_list = game_state.get_moves();
    println!("List of moves (should not include moves going into a check square):\n");
    for m in move_list {
      println!("{m}");
    }
  }

  #[test]
  fn test_check_legal_moves_2() {
    let fen = "rnbqk1nr/ppp2ppp/8/3pp3/B2bP3/8/P1PP1PPP/R3K1NR b - - 0 1";
    let game_state = GameState::from_string(fen);
    let move_list = game_state.get_moves();
    assert_eq!(8, move_list.len());
  }

  #[test]
  fn test_evaluate_position() {
    // This is a forced checkmate in 2:
    let fen = "1n4nr/5ppp/1N6/1P2p3/1P1k4/5P2/1p1NP1PP/R1B1KB1R w KQ - 0 35";
    let game_state = GameState::from_string(fen);
    let (evaluation, game_over) = game_state.evaluate_position();
    assert_eq!(false, game_over);
    println!("Evaluation {evaluation}");
  }

  #[test]
  fn test_evaluate_position_checkmate_in_one() {
    // This is a forced checkmate in 1:
    let fen = "1n4nr/5ppp/1N6/1P2p3/1P6/4kP2/1B1NP1PP/R3KB1R w KQ - 1 36";
    let game_state = GameState::from_string(fen);
    let (evaluation, game_over) = game_state.evaluate_position();
    assert_eq!(false, game_over);
    println!("Evaluation {evaluation}");
  }

  #[test]
  fn test_evaluate_position_checkmate() {
    // This is a "game over" position
    let fen = "1n4nr/5ppp/8/1P1Np3/1P6/4kP2/1B1NP1PP/R3KB1R b KQ - 2 37";
    let game_state = GameState::from_string(fen);
    let (evaluation, game_over) = game_state.evaluate_position();
    assert_eq!(true, game_over);
    assert_eq!(200.0, evaluation);
  }
  #[test]
  fn test_evaluate_position_hanging_queen() {
    // This should obviously be very bad for black:
    let fen = "rnbqkb1r/ppp1pppQ/5n2/3p4/3P4/8/PPP1PPPP/RNB1KBNR b KQkq - 0 3";
    let game_state = GameState::from_string(fen);
    let (evaluation, game_over) = game_state.evaluate_position();
    assert_eq!(false, game_over);
    assert!(evaluation < 4.0);
  }

  #[test]
  fn test_evaluate_position_queen_standoff() {
    // This should obviously be okay because queen is defended and attacked by a queen.
    let fen = "rnb1kbnr/pppp1ppp/5q2/4p3/4P3/5Q2/PPPP1PPP/RNB1KBNR w KQkq - 2 3";
    let game_state = GameState::from_string(fen);
    let (evaluation, game_over) = game_state.evaluate_position();
    assert_eq!(false, game_over);
    assert!(evaluation < 1.0);
    assert!(evaluation > -1.0);
  }
}