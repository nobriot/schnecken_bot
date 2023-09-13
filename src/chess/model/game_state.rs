use log::*;
use std::collections::VecDeque;

use crate::chess::model::board::*;
use crate::chess::model::board_mask::*;
use crate::chess::model::moves::*;
use crate::chess::model::piece::NO_PIECE;
use crate::chess::model::piece::*;
use crate::chess::model::tables::zobrist::BoardHash;

/// Start game state for a standard chess game.
pub const START_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
/// How many positions do we memorize in the GameState to check for 3-fold repetitions
pub const LAST_POSITIONS_SIZE: usize = 30;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GamePhase {
  Opening,
  Middlegame,
  Endgame,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum GameStatus {
  #[default]
  Ongoing,
  WhiteWon,
  BlackWon,
  Draw,
}

#[derive(Clone)]
pub struct GameState {
  pub checks: u8,
  pub board: Board,
  pub ply: u8,
  pub move_count: usize,
  // Vector of position representing the last x positions, from the start
  pub last_positions: VecDeque<BoardHash>,
  pub last_moves: Vec<Move>,
}

// -----------------------------------------------------------------------------
// Helper functions (prints)
#[allow(dead_code)]
pub fn print_heatmap(heatmap: &[usize; 64]) {
  let mut representation = String::from("\n");
  for rank in (1..=8).rev() {
    for file in 1..=8 {
      representation += heatmap[Board::fr_to_index(file, rank)].to_string().as_str();
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
  pub fn from_fen(fen: &str) -> Self {
    let fen_parts: Vec<&str> = fen.split(' ').collect();
    if fen_parts.len() < 6 {
      error!("Fen too small to generate a game state");
      return GameState::default();
    }

    let board = Board::from_fen(fen);
    let ply: u8 = fen_parts[4].parse::<u8>().unwrap_or(0);
    let move_count: usize = fen_parts[5].parse::<usize>().unwrap_or(0);

    let mut game_state = GameState {
      checks: 0,
      board: board,
      ply: ply,
      move_count: move_count,
      last_positions: VecDeque::with_capacity(LAST_POSITIONS_SIZE),
      last_moves: Vec::new(),
    };
    // Determine if we're check / Checkmate
    game_state.update_checks();
    game_state
  }

  pub fn to_fen(&self) -> String {
    let mut fen = String::new();

    fen += self.board.to_string().as_str();
    fen.push(' ');
    match self.board.side_to_play {
      Color::White => fen.push('w'),
      Color::Black => fen.push('b'),
    }
    fen.push(' ');

    fen += self.board.castling_rights.to_fen().as_str();
    fen.push(' ');

    if self.board.en_passant_square != INVALID_SQUARE {
      fen += square_to_string(self.board.en_passant_square).as_str();
    } else {
      fen.push('-');
    }
    fen.push(' ');
    fen += self.ply.to_string().as_str();
    fen.push(' ');
    fen += self.move_count.to_string().as_str();

    fen
  }

  /// Computes if a square is under attack by the color for a given game state.
  /// X-Rays are included.
  ///
  /// # Arguments
  ///
  /// * `self` -   A GameState object representing a position, side to play, etc.
  /// * `color` -  The color for which we want to determine the bitmap
  ///
  /// # Return value
  ///
  /// A bitmask indicating squares under control by the color for that game state.
  pub fn get_color_bitmap_with_xrays(&self, color: Color) -> BoardMask {
    let mut bitmap: BoardMask = 0;

    for source_square in 0..64_usize {
      if !self.board.has_piece_with_color(source_square as u8, color) {
        continue;
      }
      let (destinations, _) = self.board.get_piece_destinations(source_square, 0, 0);
      for i in 0..64 {
        if ((1 << i) & destinations) != 0 {
          bitmap |= 1 << i;
        }
      }
    }

    bitmap
  }

  /// Computes the number of attackers on each square for a color.
  /// Compare with your own color to get a number of defenders.
  pub fn get_heatmap(&self, color: Color, with_x_rays: bool) -> [usize; 64] {
    let mut heatmap: [usize; 64] = [0; 64];
    let opposite_color = Color::opposite(color);

    let (ssp, mut op) = match with_x_rays {
      true => (0, 0),
      false => (
        self.board.get_piece_color_mask(color),
        self.board.get_piece_color_mask(opposite_color),
      ),
    };

    // To get the heatmap, we assume that any other piece on the board
    // is opposite color, as if we could capture everything
    op |= ssp;

    for source_square in 0..64_usize {
      if !self.board.has_piece_with_color(source_square as u8, color) {
        continue;
      }
      let (destinations, _) = self.board.get_piece_destinations(source_square, op, 0);
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
        self.board.get_piece_color_mask(color),
        self.board.get_piece_color_mask(opposite_color),
      ),
    };

    // To get the heatmap, we assume that any other piece on the board
    // is opposite color, as if we could capture everything
    op |= ssp;

    for source_square in 0..64_usize {
      if !self.board.has_piece_with_color(source_square as u8, color) {
        continue;
      }
      let (destinations, _) = self.board.get_piece_destinations(source_square, op, 0);
      for i in 0..64 {
        if ((1 << i) & destinations) != 0 {
          heatmap[i] += 1;
          heatmap_sources[i] |= 1 << source_square;
        }
      }
    }

    (heatmap, heatmap_sources)
  }

  /// Checks the previous board configuration and checks if we repeated the position
  ///
  /// ### Arguments
  ///
  /// * `self`: GameState reference
  ///
  /// ### Return value
  ///
  /// Number of repetitions that occurred for the current position.
  /// Current position is not counted.
  /// i.e. 0 the position just occured for the first time. 2 means a threefold repetition
  ///
  pub fn get_board_repetitions(&self) -> usize {
    let mut repetition_count = 0;
    for hash in &self.last_positions {
      if self.board.hash == *hash {
        repetition_count += 1;
      }
    }

    repetition_count
  }

  // Get all the possible moves in a position
  pub fn get_moves(&self) -> Vec<Move> {
    match self.board.side_to_play {
      Color::White => return self.get_white_moves(),
      Color::Black => return self.get_black_moves(),
    }
  }

  // Get all the possible moves for white in a position
  pub fn get_white_moves(&self) -> Vec<Move> {
    let mut all_moves = Vec::with_capacity(60);

    let ssp = self.board.get_piece_color_mask(Color::White);
    let op = self.board.get_piece_color_mask(Color::Black);

    // Only generate moves if we have a piece on the square
    for source_square in 0..64_usize {
      if !self
        .board
        .has_piece_with_color(source_square as u8, Color::White)
      {
        continue;
      }

      let (destinations, promotion) = self.board.get_piece_destinations(source_square, op, ssp);

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

    if self.board.castling_rights.K()
      && self.checks == 0
      && !self.board.has_piece(5)
      && !self.board.has_piece(6)
      && !square_in_mask!(5, self.board.black_masks.control)
      && !square_in_mask!(6, self.board.black_masks.control)
    {
      all_moves.push(Move {
        src: 4u8,
        dest: 6u8,
        promotion: NO_PIECE,
      });
    }
    if self.board.castling_rights.Q()
      && self.checks == 0
      && !self.board.has_piece(1)
      && !self.board.has_piece(2)
      && !self.board.has_piece(3)
      && !square_in_mask!(2, self.board.black_masks.control)
      && !square_in_mask!(3, self.board.black_masks.control)
    {
      all_moves.push(Move {
        src: 4u8,
        dest: 2u8,
        promotion: NO_PIECE,
      });
    }

    // Now we need to remove all the moves where the moving side king is still in check.
    // FIXME: This is very slow.
    let mut illegal_moves = Vec::new();
    for m in &all_moves {
      let mut new_board = self.board.clone();
      new_board.apply_move(m);

      let king_square = new_board.get_white_king_square();
      if king_square == INVALID_SQUARE
        || square_in_mask!(king_square, new_board.black_masks.control)
      {
        // We're in check or king disappeared, illegal move
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

    let ssp = self.board.get_piece_color_mask(Color::Black);
    let op = self.board.get_piece_color_mask(Color::White);

    // Only generate moves if we have a piece on the square
    for source_square in 0..64_usize {
      if !self
        .board
        .has_piece_with_color(source_square as u8, Color::Black)
      {
        continue;
      }

      let (destinations, promotion) = self.board.get_piece_destinations(source_square, op, ssp);
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
    if self.board.castling_rights.k()
      && self.checks == 0
      && !self.board.has_piece(62)
      && !self.board.has_piece(61)
      && !square_in_mask!(61, self.board.white_masks.control)
      && !square_in_mask!(62, self.board.white_masks.control)
    {
      all_moves.push(Move {
        src: 60u8,
        dest: 62u8,
        promotion: NO_PIECE,
      });
    }
    if self.board.castling_rights.q()
      && self.checks == 0
      && !self.board.has_piece(59)
      && !self.board.has_piece(58)
      && !self.board.has_piece(57)
      && !square_in_mask!(59, self.board.white_masks.control)
      && !square_in_mask!(58, self.board.white_masks.control)
    {
      all_moves.push(Move {
        src: 60u8,
        dest: 58u8,
        promotion: NO_PIECE,
      });
    }

    // Now we need to remove all the moves where the moving side king is still in check.
    // FIXME: This is very slow.
    let mut illegal_moves = Vec::new();
    for m in &all_moves {
      let mut new_board = self.board.clone();
      new_board.apply_move(m);

      let king_square = new_board.get_black_king_square();
      if king_square == INVALID_SQUARE
        || square_in_mask!(king_square, new_board.white_masks.control)
      {
        // We're in check or disappeared, illegal move
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
    match self.board.side_to_play {
      Color::White => self.board.get_white_king_square(),
      Color::Black => self.board.get_black_king_square(),
    }
  }

  pub fn update_checks(&mut self) {
    let opponent_color = Color::opposite(self.board.side_to_play);
    let opponent_heatmap = self.get_heatmap(opponent_color, false);
    let king_square = self.get_king_square();

    if king_square == INVALID_SQUARE {
      error!("Can't get king square ? {}", self.to_fen());
      self.checks = 0;
      return;
    }
    self.checks = opponent_heatmap[king_square as usize] as u8;
  }

  pub fn apply_move(&mut self, chess_move: &Move) -> () {
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

    // Save the last position:
    if self.last_positions.len() >= LAST_POSITIONS_SIZE {
      self.last_positions.pop_back();
    }
    self.last_positions.push_front(self.board.hash);

    // Check the ply count first:
    if self.board.squares[chess_move.dest as usize] != NO_PIECE
      || self.board.squares[chess_move.src as usize] == WHITE_PAWN
      || self.board.squares[chess_move.src as usize] == BLACK_PAWN
    {
      self.ply = 0;
    } else if self.ply < 255 {
      self.ply += 1;
    }

    // Move count (keep in that order, as the side to play is updated in the board.apply_move())
    if self.board.side_to_play == Color::Black {
      self.move_count += 1;
    }
    // Move the pieces on the board
    self.board.apply_move(chess_move);

    // Save the move we applied.
    self.last_moves.push(chess_move.clone());

    // Check if we have checks
    self.update_checks();
  }

  // Applies all moves from a vector of moves
  pub fn apply_moves(&mut self, move_list: &Vec<Move>) -> () {
    for chess_move in move_list {
      self.apply_move(chess_move);
    }
  }

  pub fn apply_move_list(&mut self, move_list: &str) -> () {
    if move_list.is_empty() {
      return;
    }

    let moves: Vec<&str> = move_list.split(' ').collect();
    for chess_move in moves {
      self.apply_move(&Move::from_string(chess_move));
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
      self.move_count, self.ply, self.board.side_to_play, self.checks
    )
    .as_str();
    message += format!(
      "En passant: {} - Castling rights: {}\n",
      square_to_string(self.board.en_passant_square),
      self.board.castling_rights.to_fen(),
    )
    .as_str();

    message += format!("Board: {}\n", self.board.to_string()).as_str();

    message += "last positions:\n";
    for i in 0..self.last_positions.len() {
      message += format!("- {}\n", self.last_positions[i]).as_str();
    }

    f.write_str(message.as_str())
  }
}

impl Default for GameState {
  fn default() -> Self {
    GameState {
      checks: 0,
      board: Board::from_fen(START_POSITION_FEN),
      ply: 0,
      move_count: 1,
      last_positions: VecDeque::with_capacity(LAST_POSITIONS_SIZE),
      last_moves: Vec::new(),
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
    let game_state = GameState::from_fen(START_POSITION_FEN);
    assert_eq!(START_POSITION_FEN, game_state.to_fen().as_str());
    println!("{}", game_state.to_fen());

    let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
    let game_state = GameState::from_fen(fen);
    assert_eq!(fen, game_state.to_fen().as_str());
    println!("{}", game_state.to_fen());

    let fen = "5rk1/3b1p2/1r3p1p/p1pPp3/8/1P6/P3BPPP/R1R3K1 w - c6 0 23";
    let game_state = GameState::from_fen(fen);
    assert_eq!(fen, game_state.to_fen().as_str());
    println!("{}", game_state.to_fen());

    let fen = "r2qk2r/p1pb1ppp/3bpn2/8/2BP4/2N2Q2/PP3PPP/R1B2RK1 b kq - 2 12";
    let game_state = GameState::from_fen(fen);
    assert_eq!(fen, game_state.to_fen().as_str());
    println!("{}", game_state.to_fen());
  }

  #[test]
  fn test_get_list_of_moves() {
    let fen = "r2qk2r/p1pb1ppp/3bpn2/8/2BP4/2N2Q2/PP3PPP/R1B2RK1 b kq - 2 12";
    let mut game_state = GameState::from_fen(fen);
    let move_list = game_state.get_moves();
    assert_eq!(37, move_list.len());
    println!("List of moves (should include castling!):\n");
    for m in move_list {
      println!("{m}");
    }

    let fen = "5k2/P7/2p5/1p6/3P2NR/1p2p3/1P4q1/1K6 w - - 0 53";
    let mut game_state = GameState::from_fen(fen);
    let move_list = game_state.get_moves();
    println!("List of moves (should include a promotion):\n");
    assert_eq!(20, move_list.len());
    for m in move_list {
      println!("{m}");
    }

    let fen = "r2q1rk1/p2b1ppp/3bpn2/2pP4/2B5/2N2Q2/PP3PPP/R1B2RK1 w - c6 0 14";
    let mut game_state = GameState::from_fen(fen);
    let move_list = game_state.get_moves();
    println!("List of moves (should include a en-passant capture):\n");
    assert_eq!(42, move_list.len());
    for m in move_list {
      println!("{m}");
    }

    // Apply the en-passant move, check that the destination capture pawn is gone.
    let en_passant_move = Move::from_string("d5c6");
    game_state.apply_move(&en_passant_move);
    let expected_fen = "r2q1rk1/p2b1ppp/2Pbpn2/8/2B5/2N2Q2/PP3PPP/R1B2RK1 b - - 0 14";
    assert_eq!(expected_fen, game_state.to_fen());
  }

  #[test]
  fn test_apply_some_moves() {
    let fen = "r2qk2r/p1pb1ppp/3bpn2/8/2BP4/2N2Q2/PP3PPP/R1B2RK1 b kq - 2 12";
    let mut game_state = GameState::from_fen(fen);
    game_state.apply_move(&Move::from_string("a7a5"));

    let expected_fen = "r2qk2r/2pb1ppp/3bpn2/p7/2BP4/2N2Q2/PP3PPP/R1B2RK1 w kq - 0 13";
    assert_eq!(expected_fen, game_state.to_fen().as_str());
    println!("{}", game_state.to_fen());

    /*
    game_state = GameState::default();
    game_state.apply_move_list("d2d4 g8f6 c2c4 e7e6 g1f3 d7d5 b1c3 d5c4 e2e4 c8b4 f1c4 g7e5 e1g1 b8f8 d1a4 c7c6 g1e2 b4d6 e2c3 d6b6 a4c6 b7d7 d2f2 g8e8 c1g5 f8b8 a1b1 h8h7 g5h4 b8b6 d4e5 g6e5 e5e6 f7e6 f2f4 e8g8 b1e1 g8g7 d3e4 f6g4 h2h3 d8b8 e1c2 b6c4 b2b3 f8h8 f3e5 c4e5 f4e5 c6c5 h4g3 g7f6 a2a4 f6e5 g3g6 e8f8 a4a5 f8e7 f2e2 f7f5 c2c3 e7g5 g2g4 g5c5 c3c4 h7h3 e5f3 e6f4 d4d5 d7c6 e1g1");
    let expected_fen = "8/p1pk1r2/2Nb3p/8/2P2P2/2Q1n2p/P4qPP/6RK b - - 0 36";
    assert_eq!(expected_fen, game_state.to_fen().as_str());
    println!("{}",game_state.to_fen());

    */
  }

  #[test]
  fn test_check_legal_moves() {
    let fen = "4B3/p5k1/1pp4p/8/8/P6P/5PP1/2R3K1 b - - 0 37";
    let mut game_state = GameState::from_fen(fen);
    let move_list = game_state.get_moves();
    println!("List of moves (should not include moves going into a check square):\n");
    for m in move_list {
      println!("{m}");
    }
  }

  #[test]
  fn test_check_legal_moves_2() {
    let fen = "rnbqk1nr/ppp2ppp/8/3pp3/B2bP3/8/P1PP1PPP/R3K1NR b - - 0 1";
    let mut game_state = GameState::from_fen(fen);
    let move_list = game_state.get_moves().clone();
    for m in &move_list {
      println!("{m}");
    }
    //print_board_mask(game_state.white_bitmap.unwrap());
    assert_eq!(8, move_list.len());
  }

  #[test]
  fn check_blocked_pawns() {
    let fen = "rn2k3/1bpp1p1p/p2bp3/6Q1/3PP3/2PB4/PP2NPPP/RN2K2R b KQq - 0 13";
    let mut game_state = GameState::from_fen(fen);
    //println!("List of moves (should not include moves d7d5\n");
    for m in game_state.get_moves() {
      //println!("{m}");
      assert_ne!("d7d5", m.to_string());
    }
  }

  #[test]
  fn test_copying() {
    let fen = "rn1qkb1r/1bp1pppp/p2p1n2/1p6/3PP3/4B1P1/PPPN1PBP/R2QK1NR b KQkq - 5 6";
    let mut game_state = GameState::from_fen(fen);
    let last_position =
      Board::from_fen("rn1qkb1r/1bp1pppp/p2p1n2/1p6/3PP3/4B1P1/PPPN1PBP/R2QK1N w");
    game_state.last_positions.push_front(last_position.hash);

    let mut game_state_copy = game_state.clone();
    game_state.get_moves();

    assert!(1 == game_state_copy.last_positions.len());
    game_state_copy.last_positions.pop_front();
    assert!(1 == game_state.last_positions.len());
    assert!(0 == game_state_copy.last_positions.len());
  }

  #[test]
  fn game_state_bench_move_applications_per_second() {
    use std::time::{Duration, Instant};
    let fen = "rn1qkb1r/1bp1pppp/p2p1n2/1p6/3PP3/4B1P1/PPPN1PBP/R2QK1NR b KQkq - 5 6";
    let mut game_state = GameState::from_fen(fen);

    let mut positions_computed = 0;
    let start_time = Instant::now();

    // Spin at it for 1 second
    while Instant::now() < (start_time + Duration::from_millis(1000)) {
      let moves = game_state.get_moves();
      if !moves.is_empty() {
        let m = moves[0];
        game_state.apply_move(&m);
        positions_computed += 1;
      } else {
        game_state = GameState::from_fen(fen);
      }
    }

    // 1000 kNPS would be nice. Right now we are at a very low number LOL
    assert!(
      positions_computed > 1_000_000,
      "Number of NPS for computing and applying moves: {}",
      positions_computed
    );
  }

  #[test]
  fn game_state_bench_compute_legal_moves_per_second() {
    use std::time::{Duration, Instant};
    let fen = "rn1qkb1r/1bp1pppp/p2p1n2/1p6/3PP3/4B1P1/PPPN1PBP/R2QK1NR b KQkq - 5 6";

    let mut positions_computed = 0;
    let start_time = Instant::now();

    // Spin at it for 1 second
    while Instant::now() < (start_time + Duration::from_millis(1000)) {
      let game_state = GameState::from_fen(fen);
      let _ = game_state.get_moves();
      positions_computed += 1;
    }

    // 1000 kNPS would be nice. Right now we are at a very low number LOL
    assert!(
      positions_computed > 1_000_000,
      "Number of NPS for computing legal moves: {}",
      positions_computed
    );
  }
}
