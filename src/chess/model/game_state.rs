use log::*;

use crate::chess::model::board::*;
use crate::chess::model::piece::NO_PIECE;
use crate::chess::model::piece::*;
use crate::chess::model::piece_moves::*;

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GamePhase {
  Opening,
  Middlegame,
  Endgame,
}

#[derive(Clone)]
pub struct GameState {
  pub side_to_play: Color,
  pub checks: u8,
  pub en_passant_square: u8,
  pub castling_rights: CastlingRights,
  pub board: Board,
  pub ply: u8,
  pub move_count: u8,
  pub available_moves_computed: bool,
  pub move_list: Vec<Move>,
  pub game_phase: Option<GamePhase>,
}

// -----------------------------------------------------------------------------
// Helper functions (prints)
pub fn print_heatmap(heatmap: &[usize; 64]) {
  let mut representation = String::from("\n");
  for rank in (0..8 as u8).rev() {
    for file in 0..8 {
      representation += (heatmap[(rank * 8 + file) as usize]).to_string().as_str();
      representation.push(' ');
    }
    representation.push('\n');
  }

  println!("{representation}");
}

pub fn print_mask(mask: u64) {
  let mut representation = String::from("\n");
  for rank in (0..8 as u8).rev() {
    for file in 0..8 {
      if (mask >> (rank * 8 + file) & 1) == 1 {
        representation.push('1');
      } else {
        representation.push('0');
      }
      representation.push(' ');
    }
    representation.push('\n');
  }

  println!("{representation}");
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
      available_moves_computed: false,
      move_list: Vec::new(),
      game_phase: None,
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

    let (ssp, mut op) = match with_x_rays {
      true => (0, 0),
      false => (
        self.board.get_color_mask(color),
        self.board.get_color_mask(opposite_color),
      ),
    };

    // To get the heatmap, we assume that any other piece on the board
    // is opposite color, as if we could capture everything
    op = ssp | op;

    for source_square in 0..64 as usize {
      if !self.board.has_piece_with_color(source_square as u8, color) {
        continue;
      }
      let (destinations, _) = self.get_piece_destinations(source_square, op, 0);
      for i in 0..64 {
        if ((1 << i) & destinations) != 0 {
          heatmap[i] += 1;
        }
      }
    }

    heatmap
  }

  /// Returns the location of the attackers of a particular square
  /// Each square is encoded with u64 bitmask of the sources.
  pub fn get_heatmap_with_sources(
    &self,
    color: Color,
    with_x_rays: bool,
  ) -> ([usize; 64], [usize; 64]) {
    let mut heatmap: [usize; 64] = [0; 64];
    let mut heatmap_sources: [usize; 64] = [0; 64];
    let opposite_color = Color::opposite(color);

    let (ssp, mut op) = match with_x_rays {
      true => (0, 0),
      false => (
        self.board.get_color_mask(color),
        self.board.get_color_mask(opposite_color),
      ),
    };

    // To get the heatmap, we assume that any other piece on the board
    // is opposite color, as if we could capture everything
    op = ssp | op;

    for source_square in 0..64 as usize {
      if !self.board.has_piece_with_color(source_square as u8, color) {
        continue;
      }
      let (destinations, _) = self.get_piece_destinations(source_square, op, 0);
      for i in 0..64 {
        if ((1 << i) & destinations) != 0 {
          heatmap[i] += 1;
          heatmap_sources[i] |= 1 << source_square;
        }
      }
    }

    (heatmap, heatmap_sources)
  }

  // Get all the possible moves in a position
  pub fn get_moves(&mut self) -> &Vec<Move> {
    if self.available_moves_computed == true {
      return &self.move_list;
    }

    match self.side_to_play {
      Color::White => self.move_list = self.get_white_moves(),
      Color::Black => self.move_list = self.get_black_moves(),
    }
    self.available_moves_computed = true;
    return &self.move_list;
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
      new_game_state.apply_move(m, false);
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
      new_game_state.apply_move(m, false);
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

  pub fn apply_move(&mut self, chess_move: &Move, compute_legal_moves: bool) -> () {
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
    match chess_move.dest {
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

    // Compute the list of legal moves if we need it
    if compute_legal_moves == true {
      let _ = self.get_moves();
    }
  }

  // Applies all moves from a vector of moves
  #[allow(dead_code)]
  pub fn apply_moves(&mut self, move_list: &Vec<Move>) -> () {
    for chess_move in move_list {
      self.apply_move(chess_move, false);
    }
  }

  pub fn apply_move_list(&mut self, move_list: &str) -> () {
    if move_list.is_empty() {
      return;
    }

    let moves: Vec<&str> = move_list.split(' ').collect();
    for chess_move in moves {
      self.apply_move(&Move::from_string(&chess_move), false);
    }
  }

  // Determine the game phrase and update it.
  pub fn update_game_phase(&mut self) {
    // Do not recalculate when we calculated already
    if let Some(_) = self.game_phase {
      return;
    }

    // Basic material count, disregarding pawns.
    let mut material_count: usize = 0;
    let mut development_index: usize = 0;
    for i in 0..64 {
      match self.board.squares[i] {
        WHITE_QUEEN | BLACK_QUEEN => material_count += 9,
        WHITE_ROOK | BLACK_ROOK => material_count += 5,
        WHITE_BISHOP | BLACK_BISHOP => material_count += 3,
        WHITE_KNIGHT | BLACK_KNIGHT => material_count += 3,
        _ => {},
      }
    }
    for i in 0..8 {
      match self.board.squares[i] {
        WHITE_QUEEN | WHITE_BISHOP | WHITE_KNIGHT => development_index += 1,
        _ => {},
      }
    }
    for i in 56..64 {
      match self.board.squares[i] {
        BLACK_QUEEN | BLACK_BISHOP | BLACK_KNIGHT => development_index += 1,
        _ => {},
      }
    }

    if material_count < 17 {
      self.game_phase = Some(GamePhase::Endgame);
      return;
    } else if development_index > 2 {
      self.game_phase = Some(GamePhase::Opening);
    } else {
      self.game_phase = Some(GamePhase::Middlegame);
    }
  }
}

// -----------------------------------------------------------------------------
// Display/Default implementations for our game state
impl std::fmt::Debug for GameState {
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
      available_moves_computed: false,
      move_list: Vec::new(),
      game_phase: None,
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
    println!("{}", game_state.to_string());

    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    let game_state = GameState::from_string(fen);
    assert_eq!(fen, game_state.to_string().as_str());
    println!("{}", game_state.to_string());

    let fen = "5rk1/3b1p2/1r3p1p/p1pPp3/8/1P6/P3BPPP/R1R3K1 w - c6 0 23";
    let game_state = GameState::from_string(fen);
    assert_eq!(fen, game_state.to_string().as_str());
    println!("{}", game_state.to_string());

    let fen = "r2qk2r/p1pb1ppp/3bpn2/8/2BP4/2N2Q2/PP3PPP/R1B2RK1 b kq - 2 12";
    let game_state = GameState::from_string(fen);
    assert_eq!(fen, game_state.to_string().as_str());
    println!("{}", game_state.to_string());
  }

  #[test]
  fn test_get_list_of_moves() {
    let fen = "r2qk2r/p1pb1ppp/3bpn2/8/2BP4/2N2Q2/PP3PPP/R1B2RK1 b kq - 2 12";
    let mut game_state = GameState::from_string(fen);
    let move_list = game_state.get_moves();
    assert_eq!(37, move_list.len());
    println!("List of moves (should include castling!):\n");
    for m in move_list {
      println!("{m}");
    }

    let fen = "5k2/P7/2p5/1p6/3P2NR/1p2p3/1P4q1/1K6 w - - 0 53";
    let mut game_state = GameState::from_string(fen);
    let move_list = game_state.get_moves();
    assert_eq!(20, move_list.len());
    println!("List of moves (should include a promotion):\n");
    for m in move_list {
      println!("{m}");
    }

    let fen = "r2q1rk1/p2b1ppp/3bpn2/2pP4/2B5/2N2Q2/PP3PPP/R1B2RK1 w - c6 0 14";
    let mut game_state = GameState::from_string(fen);
    let move_list = game_state.get_moves();
    println!("List of moves (should include a en-passant capture):\n");
    for m in move_list {
      println!("{m}");
    }
    assert_eq!(42, move_list.len());

    // Apply the en-passant move, check that the destination capture pawn is gone.
    let en_passant_move = Move::from_string("d5c6");
    game_state.apply_move(&en_passant_move, false);
    let expected_fen = "r2q1rk1/p2b1ppp/2Pbpn2/8/2B5/2N2Q2/PP3PPP/R1B2RK1 b - - 0 14";
    assert_eq!(expected_fen, game_state.to_string());
  }

  #[test]
  fn test_apply_some_moves() {
    let fen = "r2qk2r/p1pb1ppp/3bpn2/8/2BP4/2N2Q2/PP3PPP/R1B2RK1 b kq - 2 12";
    let mut game_state = GameState::from_string(fen);
    game_state.apply_move(&Move::from_string("a7a5"), false);

    let expected_fen = "r2qk2r/2pb1ppp/3bpn2/p7/2BP4/2N2Q2/PP3PPP/R1B2RK1 w kq a6 0 13";
    assert_eq!(expected_fen, game_state.to_string().as_str());
    println!("{}", game_state.to_string());

    /*
    game_state = GameState::default();
    game_state.apply_move_list("d2d4 g8f6 c2c4 e7e6 g1f3 d7d5 b1c3 d5c4 e2e4 c8b4 f1c4 g7e5 e1g1 b8f8 d1a4 c7c6 g1e2 b4d6 e2c3 d6b6 a4c6 b7d7 d2f2 g8e8 c1g5 f8b8 a1b1 h8h7 g5h4 b8b6 d4e5 g6e5 e5e6 f7e6 f2f4 e8g8 b1e1 g8g7 d3e4 f6g4 h2h3 d8b8 e1c2 b6c4 b2b3 f8h8 f3e5 c4e5 f4e5 c6c5 h4g3 g7f6 a2a4 f6e5 g3g6 e8f8 a4a5 f8e7 f2e2 f7f5 c2c3 e7g5 g2g4 g5c5 c3c4 h7h3 e5f3 e6f4 d4d5 d7c6 e1g1");
    let expected_fen = "8/p1pk1r2/2Nb3p/8/2P2P2/2Q1n2p/P4qPP/6RK b - - 0 36";
    assert_eq!(expected_fen, game_state.to_string().as_str());
    println!("{}",game_state.to_string());

    */
  }

  #[test]
  fn test_check_legal_moves() {
    let fen = "4B3/p5k1/1pp4p/8/8/P6P/5PP1/2R3K1 b - - 0 37";
    let mut game_state = GameState::from_string(fen);
    let move_list = game_state.get_moves();
    println!("List of moves (should not include moves going into a check square):\n");
    for m in move_list {
      println!("{m}");
    }
  }

  #[test]
  fn test_check_legal_moves_2() {
    let fen = "rnbqk1nr/ppp2ppp/8/3pp3/B2bP3/8/P1PP1PPP/R3K1NR b - - 0 1";
    let mut game_state = GameState::from_string(fen);
    let move_list = game_state.get_moves();
    assert_eq!(8, move_list.len());
  }

  #[test]
  fn check_blocked_pawns() {
    let fen = "rn2k3/1bpp1p1p/p2bp3/6Q1/3PP3/2PB4/PP2NPPP/RN2K2R b KQq - 0 13";
    let mut game_state = GameState::from_string(fen);
    //println!("List of moves (should not include moves d7d5\n");
    for m in game_state.get_moves() {
      //println!("{m}");
      assert_ne!("d7d5", m.to_string());
    }
  }

  #[test]
  fn update_game_phase() {
    let fen = "rn1qkb1r/1bp1pppp/p2p1n2/1p6/3PP3/4B1P1/PPPN1PBP/R2QK1NR b KQkq - 5 6";
    let mut game_state = GameState::from_string(fen);
    game_state.update_game_phase();
    assert_eq!(Some(GamePhase::Opening), game_state.game_phase);

    let fen = "r3k2r/2qnbppp/p1p5/1p1np3/3PQ3/2P1B1PP/PP1NNP2/R3K2R w KQkq - 4 14";
    let mut game_state = GameState::from_string(fen);
    game_state.update_game_phase();
    assert_eq!(Some(GamePhase::Middlegame), game_state.game_phase);

    let fen = "4r1k1/4b1p1/p3p2p/2pR4/2p5/4B1PP/PP3P2/2K5 w - - 0 27";
    let mut game_state = GameState::from_string(fen);
    game_state.update_game_phase();
    assert_eq!(Some(GamePhase::Endgame), game_state.game_phase);
  }
}
