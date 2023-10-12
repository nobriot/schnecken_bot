use log::*;
use std::collections::VecDeque;

use crate::model::board::*;
use crate::model::board_mask::*;
use crate::model::moves::*;
use crate::model::piece::*;
use crate::model::tables::zobrist::BoardHash;

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
  Stalemate,
  Draw,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct GameState {
  pub board: Board,
  pub ply: u8,
  pub move_count: usize,
  // Vector of position representing the last x positions, from the start
  pub last_positions: VecDeque<BoardHash>,
  pub last_moves: Vec<Move>,
}

// -----------------------------------------------------------------------------
// Game state implementation

impl GameState {
  /// Takes a board and makes it into a GameState, assuming default values for the rest.
  ///
  /// ### Arguments
  ///
  /// * `board`: Board object to use for the Game State
  ///
  /// ### Return Value
  ///
  /// GameState object with the board passed in argument
  ///
  pub fn from_board(board: &Board) -> Self {
    GameState {
      board: board.clone(),
      ply: 0,
      move_count: 0,
      last_positions: VecDeque::with_capacity(LAST_POSITIONS_SIZE),
      last_moves: Vec::new(),
    }
  }

  /// Takes a full FEN notation and converts it into a Game State
  ///
  /// ### Arguments
  ///
  /// * `fen`: String to use to create a GameState object
  ///
  /// ### Return Value
  ///
  /// GameState object
  ///
  pub fn from_fen(fen: &str) -> Self {
    let fen_parts: Vec<&str> = fen.split(' ').collect();
    if fen_parts.len() < 6 {
      error!("Fen too small to generate a game state");
      return GameState::default();
    }

    let board = Board::from_fen(fen);
    let ply: u8 = fen_parts[4].parse::<u8>().unwrap_or(0);
    let move_count: usize = fen_parts[5].parse::<usize>().unwrap_or(0);

    GameState {
      board: board,
      ply: ply,
      move_count: move_count,
      last_positions: VecDeque::with_capacity(LAST_POSITIONS_SIZE),
      last_moves: Vec::new(),
    }
  }

  /// Exports the game state to a FEN notation
  ///
  /// ### Arguments
  ///
  /// * `self`: Reference to a GameState object
  ///
  /// ### Return Value
  ///
  /// String representing the game state.
  ///
  pub fn to_fen(&self) -> String {
    let mut fen = String::new();

    fen += self.board.to_fen().as_str();
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
    self
      .last_positions
      .iter()
      .filter(|x| **x == self.board.hash)
      .count()
    // self.last_positions.iter().fold(0,|count, x| if *x == self.board.hash { count + 1 } else { count },)
  }

  /// Get all the possible moves in a position, for the side to play.
  pub fn get_moves(&self) -> Vec<Move> {
    self.board.get_moves()
  }

  pub fn get_king_square(&self) -> u8 {
    match self.board.side_to_play {
      Color::White => self.board.get_white_king_square(),
      Color::Black => self.board.get_black_king_square(),
    }
  }

  pub fn apply_move(&mut self, chess_move: &Move) -> () {
    if !square_in_mask!(chess_move.src(), self.board.pieces.all()) {
      error!(
        "Input moves with empty source square? {} - board:\n{}",
        chess_move, self.board
      );
      return;
    }

    // Save the last position:
    if square_in_mask!(chess_move.src(), self.board.pieces.pawns()) {
      // Cannot really repeat a position after a pawn moves, assume anything forward is a novel position
      self.last_positions.clear();
    } else {
      if self.last_positions.len() >= LAST_POSITIONS_SIZE {
        self.last_positions.pop_back();
      }
      self.last_positions.push_front(self.board.hash);
    }

    // Check the ply count first:
    if square_in_mask!(chess_move.dest(), self.board.pieces.all())
      || square_in_mask!(chess_move.src(), self.board.pieces.pawns())
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
      self.move_count,
      self.ply,
      self.board.side_to_play,
      self.board.checks()
    )
    .as_str();
    message += format!(
      "En passant: {} - Castling rights: {}\n",
      square_to_string(self.board.en_passant_square),
      self.board.castling_rights.to_fen(),
    )
    .as_str();

    message += format!("Board: {}\n", self.board.to_fen()).as_str();

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
    let game_state = GameState::from_fen(fen);
    let move_list = game_state.get_moves();
    assert_eq!(37, move_list.len());
    println!("List of moves (should include castling!):\n");
    for m in move_list {
      println!("{m}");
    }

    let fen = "5k2/P7/2p5/1p6/3P2NR/1p2p3/1P4q1/1K6 w - - 0 53";
    let game_state = GameState::from_fen(fen);
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
    let game_state = GameState::from_fen(fen);
    let move_list = game_state.get_moves();
    println!("List of moves (should not include moves going into a check square):\n");
    for m in move_list {
      println!("{m}");
    }
  }

  #[test]
  fn test_check_legal_moves_2() {
    let fen = "rnbqk1nr/ppp2ppp/8/3pp3/B2bP3/8/P1PP1PPP/R3K1NR b - - 0 1";
    let game_state = GameState::from_fen(fen);
    let move_list = game_state.get_moves();
    for m in &move_list {
      println!("{m}");
    }
    //print_board_mask(game_state.white_bitmap.unwrap());
    assert_eq!(8, move_list.len());
  }

  #[test]
  fn test_legal_moves_3() {
    let fen = "8/8/2K5/8/R2Q4/8/8/2k5 b - - 28 97";
    let game_state = GameState::from_fen(fen);
    let move_list = game_state.get_moves();
    let legal_moves = vec!["c1c2", "c1b1"];
    for m in &move_list {
      println!("{m}");
      assert!(legal_moves.contains(&m.to_string().as_str()));
    }
    assert_eq!(legal_moves.len(), move_list.len());
  }

  #[test]
  fn test_legal_moves_en_passant() {
    let fen = "4k3/8/8/8/2PpP3/8/8/4K3 b - c3 0 3";
    let game_state = GameState::from_fen(fen);
    let move_list = game_state.get_moves();
    let legal_moves = vec!["e8e7", "e8f7", "e8f8", "e8d7", "e8d8", "d4d3", "d4c3"];
    for m in &move_list {
      println!("{m}");
      assert!(legal_moves.contains(&m.to_string().as_str()));
    }
    assert_eq!(legal_moves.len(), move_list.len());
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

    // Spin at it for 1 second
    let start_time = Instant::now();
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
    let fen = "rn1qkb1r/1bp1pppp/p2p1n2/1p6/3PP3/4B1P1/PPPN1PBP/R2QK1NR w KQkq - 5 6";

    let mut positions_computed = 0;
    let game_state = GameState::from_fen(fen);

    // Spin at it for 1 second
    let start_time = Instant::now();
    while Instant::now() < (start_time + Duration::from_millis(1000)) {
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

  #[test]
  fn test_pawn_double_jump_blocked() {
    let fen = "5r1k/1P5p/5p1N/4p3/2NpPnp1/3P4/2PB1PPP/R5K1 b - - 0 36";
    let game_state = GameState::from_fen(fen);
    let moves = game_state.get_moves();
    for m in &moves {
      println!("{m}");
      assert_ne!("h5h7", m.to_string());
    }
    assert_eq!(18, moves.len());
  }

  #[test]
  fn test_get_moves_while_in_check() {
    let fen = "r3kb1r/ppp2ppp/3p1n2/1Q1p4/1n1P2bP/2N2N2/PPP1PPP1/R3KB1R b - - 0 1";
    let game_state = GameState::from_fen(fen);

    // List of legal moves are: 6:
    let mut legal_moves: Vec<&str> = Vec::new();
    legal_moves.push("b4c6");
    legal_moves.push("g4d7");
    legal_moves.push("f6d7");
    legal_moves.push("c7c6");
    legal_moves.push("e8e7");
    legal_moves.push("e8d8");

    let computed_moves = game_state.get_moves();

    for m in &computed_moves {
      println!("Move: {m}");
      assert!(legal_moves.contains(&m.to_string().as_str()));
    }

    assert_eq!(6, computed_moves.len());

    // Second test:
    println!("---------------------------");
    let fen = "5b1r/3Q1k1p/3p1p2/2pN2p1/3P3P/5N2/PPP1PPP1/R3KB1R b KQ - 0 18";
    let game_state = GameState::from_fen(fen);

    // List of legal moves are: 3:
    let mut legal_moves: Vec<&str> = Vec::new();
    legal_moves.push("f8e7");
    legal_moves.push("f7g6");
    legal_moves.push("f7g8");

    let computed_moves = game_state.get_moves();

    for m in &computed_moves {
      println!("Move: {m}");
      assert!(legal_moves.contains(&m.to_string().as_str()));
    }

    assert_eq!(3, computed_moves.len());

    // Third test: double check:
    println!("---------------------------");
    let fen = "rnbq1bn1/pppp1kp1/7r/4Np1Q/4P3/8/PPPP1PPP/RNB1KB1R b KQ - 0 6";
    let game_state = GameState::from_fen(fen);

    // List of legal moves are: 3:
    let mut legal_moves: Vec<&str> = Vec::new();
    legal_moves.push("f7e7");
    legal_moves.push("f7e6");
    legal_moves.push("f7f6");

    let computed_moves = game_state.get_moves();

    for m in &computed_moves {
      println!("Move: {m}");
      assert!(legal_moves.contains(&m.to_string().as_str()));
    }

    assert_eq!(3, computed_moves.len());

    // Another test: take out the checking piece:'
    println!("---------------------------");
    let fen = "5k1r/p1pp2N1/p4n1p/4p3/1P6/4P2P/2P2P1R/3K4 w - - 4 35";
    let mut game_state = GameState::from_fen(fen);
    game_state.apply_move(&Move::from_string("g7e6"));

    // List of legal moves are:
    let mut legal_moves: Vec<&str> = Vec::new();
    legal_moves.push("f8e8");
    legal_moves.push("f8e7");
    legal_moves.push("f8f7");
    legal_moves.push("f8g8");
    legal_moves.push("d7e6");

    let computed_moves = game_state.get_moves();

    for m in &computed_moves {
      println!("Move: {m}");
      assert!(legal_moves.contains(&m.to_string().as_str()));
    }

    assert_eq!(5, computed_moves.len());
  }
}
