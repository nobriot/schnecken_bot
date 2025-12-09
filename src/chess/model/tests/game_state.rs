use crate::model::board::*;
use crate::model::game_state::*;
use crate::model::moves::*;

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
  game_state.apply_move_from_notation("d5c6");
  let expected_fen = "r2q1rk1/p2b1ppp/2Pbpn2/8/2B5/2N2Q2/PP3PPP/R1B2RK1 b - - 0 14";
  assert_eq!(expected_fen, game_state.to_fen());
}

#[test]
fn test_apply_some_moves() {
  let fen = "r2qk2r/p1pb1ppp/3bpn2/8/2BP4/2N2Q2/PP3PPP/R1B2RK1 b kq - 2 12";
  let mut game_state = GameState::from_fen(fen);
  game_state.apply_move_from_notation("a7a5");

  let expected_fen = "r2qk2r/2pb1ppp/3bpn2/p7/2BP4/2N2Q2/PP3PPP/R1B2RK1 w kq - 0 13";
  assert_eq!(expected_fen, game_state.to_fen().as_str());
  println!("{}", game_state.to_fen());

  // game_state = GameState::default();
  // game_state.apply_move_list("d2d4 g8f6 c2c4 e7e6 g1f3 d7d5 b1c3 d5c4 e2e4
  // c8b4 f1c4 g7e5 e1g1 b8f8 d1a4 c7c6 g1e2 b4d6 e2c3 d6b6 a4c6 b7d7 d2f2 g8e8
  // c1g5 f8b8 a1b1 h8h7 g5h4 b8b6 d4e5 g6e5 e5e6 f7e6 f2f4 e8g8 b1e1 g8g7 d3e4
  // f6g4 h2h3 d8b8 e1c2 b6c4 b2b3 f8h8 f3e5 c4e5 f4e5 c6c5 h4g3 g7f6 a2a4 f6e5
  // g3g6 e8f8 a4a5 f8e7 f2e2 f7f5 c2c3 e7g5 g2g4 g5c5 c3c4 h7h3 e5f3 e6f4 d4d5
  // d7c6 e1g1"); let expected_fen = "8/p1pk1r2/2Nb3p/8/2P2P2/2Q1n2p/P4qPP/6RK
  // b - - 0 36"; assert_eq!(expected_fen, game_state.to_fen().as_str());
  // println!("{}",game_state.to_fen());
  //
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
  // print_board_mask(game_state.white_bitmap.unwrap());
  assert_eq!(8, move_list.len());
}

#[test]
fn test_legal_moves_3() {
  let fen = "8/8/2K5/8/R2Q4/8/8/2k5 b - - 28 97";
  let game_state = GameState::from_fen(fen);
  let move_list = game_state.get_moves();
  let legal_moves = ["c1c2", "c1b1"];
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
  let legal_moves = ["e8e7", "e8f7", "e8f8", "e8d7", "e8d8", "d4d3", "d4c3"];
  for m in &move_list {
    println!("{m}");
    assert!(legal_moves.contains(&m.to_string().as_str()));
  }
  assert_eq!(legal_moves.len(), move_list.len());
}

#[test]
fn check_blocked_pawns() {
  let fen = "rn2k3/1bpp1p1p/p2bp3/6Q1/3PP3/2PB4/PP2NPPP/RN2K2R b KQq - 0 13";
  let game_state = GameState::from_fen(fen);
  // println!("List of moves (should not include moves d7d5\n");
  for m in game_state.get_moves() {
    // println!("{m}");
    assert_ne!("d7d5", m.to_string());
  }
}

#[test]
fn test_copying() {
  let fen = "rn1qkb1r/1bp1pppp/p2p1n2/1p6/3PP3/4B1P1/PPPN1PBP/R2QK1NR b KQkq - 5 6";
  let mut game_state = GameState::from_fen(fen);
  let last_position =
    Board::from_fen("rn1qkb1r/1bp1pppp/p2p1n2/1p6/3PP3/4B1P1/PPPN1PBP/R2QK1N w KQkq - 5 6");

  game_state.last_positions.add(last_position.hash);
  let mut game_state_copy = game_state.clone();
  game_state.get_moves();

  assert_eq!(1, game_state_copy.last_positions.len());
  game_state_copy.last_positions.clear();
  assert_eq!(1, game_state.last_positions.len());
  assert_eq!(0, game_state_copy.last_positions.len());
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
  let legal_moves: Vec<&str> = vec!["b4c6", "g4d7", "f6d7", "c7c6", "e8e7", "e8d8"];
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
  let legal_moves: Vec<&str> = vec!["f8e7", "f7g6", "f7g8"];
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
  let legal_moves: Vec<&str> = vec!["f7e7", "f7e6", "f7f6"];
  let computed_moves = game_state.get_moves();

  for m in &computed_moves {
    println!("Move: {m}");
    assert!(legal_moves.contains(&m.to_string().as_str()));
  }

  assert_eq!(3, computed_moves.len());

  // Another test: take out the checking piece:
  println!("---------------------------");
  let fen = "5k1r/p1pp2N1/p4n1p/4p3/1P6/4P2P/2P2P1R/3K4 w - - 4 35";
  let mut game_state = GameState::from_fen(fen);
  game_state.apply_move(&Move::from_string("g7e6"));

  // List of legal moves are:
  let legal_moves: Vec<&str> = vec!["f8e8", "f8e7", "f8f7", "f8g8", "d7e6"];
  let computed_moves = game_state.get_moves();

  for m in &computed_moves {
    println!("Move: {m}");
    assert!(legal_moves.contains(&m.to_string().as_str()));
  }

  assert_eq!(5, computed_moves.len());
}
