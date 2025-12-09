use crate::model::board::*;
use crate::model::board_mask::*;
use crate::model::castling_rights::*;
use crate::model::moves::*;
use crate::model::piece::*;
use crate::model::piece_set::*;

#[test]
fn display_board() {
  let mut board = Board { pieces:            PieceSet::new(),
                          side_to_play:      Color::White,
                          castling_rights:   CastlingRights::default(),
                          en_passant_square: INVALID_SQUARE,
                          checkers:          0,
                          pins:              0,
                          hash:              0, };
  board.pieces.update(WHITE_ROOK, 0);
  board.pieces.update(WHITE_KNIGHT, 1);
  board.pieces.update(WHITE_BISHOP, 2);
  board.pieces.update(WHITE_QUEEN, 3);
  board.pieces.update(WHITE_KING, 4);
  board.pieces.update(WHITE_BISHOP, 5);
  board.pieces.update(WHITE_KNIGHT, 6);
  board.pieces.update(WHITE_ROOK, 7);
  board.pieces.update(WHITE_PAWN, 8);
  board.pieces.update(WHITE_PAWN, 9);
  board.pieces.update(WHITE_PAWN, 10);
  board.pieces.update(WHITE_PAWN, 11);
  board.pieces.update(WHITE_PAWN, 12);
  board.pieces.update(WHITE_PAWN, 13);
  board.pieces.update(WHITE_PAWN, 14);
  board.pieces.update(WHITE_PAWN, 15);

  board.pieces.update(BLACK_PAWN, 48);
  board.pieces.update(BLACK_PAWN, 49);
  board.pieces.update(BLACK_PAWN, 50);
  board.pieces.update(BLACK_PAWN, 51);
  board.pieces.update(BLACK_PAWN, 52);
  board.pieces.update(BLACK_PAWN, 53);
  board.pieces.update(BLACK_PAWN, 54);
  board.pieces.update(BLACK_PAWN, 55);
  board.pieces.update(BLACK_ROOK, 56);
  board.pieces.update(BLACK_KNIGHT, 57);
  board.pieces.update(BLACK_BISHOP, 58);
  board.pieces.update(BLACK_QUEEN, 59);
  board.pieces.update(BLACK_KING, 60);
  board.pieces.update(BLACK_BISHOP, 61);
  board.pieces.update(BLACK_KNIGHT, 62);
  board.pieces.update(BLACK_ROOK, 63);

  println!("Board: {}", board);
}

#[test]
fn from_string() {
  let mut board = Board::from_fen("rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR");
  println!("Board: {}", board);

  let test_fen = "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2";
  board = Board::from_fen(test_fen);
  println!("Board: {}", board);

  assert_eq!(test_fen.split(' ').collect::<Vec<_>>()[0], board.to_fen());

  let test_fen_2 = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
  board = Board::from_fen(test_fen_2);
  println!("Board: {}", board);

  assert_eq!(test_fen_2.split(' ').collect::<Vec<_>>()[0], board.to_fen())
}

#[test]
fn apply_move() {
  let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
  let mut board = Board::from_fen(fen);
  println!("Board: {}", board.to_fen());

  // Try and capture a piece
  board.apply_move(&Move::from_string("b3g3"));
  println!("Board: {}", board.to_fen());
  assert_eq!(board.to_fen(), "8/5pk1/5p1p/2R5/5K2/6r1/7P/8");

  // Try and promote a piece (super jump from h2 to h8)
  board.apply_move(&Move::from_string("h2h8N"));
  println!("Board: {}", board.to_fen());
  assert_eq!(board.to_fen(), "7N/5pk1/5p1p/2R5/5K2/6r1/8/8");

  // Same for black: promote to a black queen:
  board.apply_move(&Move::from_string("f6f1q"));
  println!("Board: {}", board.to_fen());
  assert_eq!(board.to_fen(), "7N/5pk1/7p/2R5/5K2/6r1/8/5q2");
}

#[test]
fn test_fr_to_index() {
  assert_eq!(0, Board::fr_to_index(1, 1));
  assert_eq!(1, Board::fr_to_index(2, 1));
  assert_eq!(3, Board::fr_to_index(4, 1));
  assert_eq!(6, Board::fr_to_index(7, 1));
  assert_eq!(7, Board::fr_to_index(8, 1));
  assert_eq!(8, Board::fr_to_index(1, 2));
  assert_eq!(9, Board::fr_to_index(2, 2));
  assert_eq!(62, Board::fr_to_index(7, 8));
  assert_eq!(63, Board::fr_to_index(8, 8));
}

#[test]
fn test_index_to_fr() {
  assert_eq!((1, 1), Board::index_to_fr(0));
  assert_eq!((2, 1), Board::index_to_fr(1));
  assert_eq!((4, 1), Board::index_to_fr(3));
  assert_eq!((7, 1), Board::index_to_fr(6));
  assert_eq!((8, 1), Board::index_to_fr(7));
  assert_eq!((1, 2), Board::index_to_fr(8));
  assert_eq!((2, 2), Board::index_to_fr(9));
  assert_eq!((7, 8), Board::index_to_fr(62));
  assert_eq!((8, 8), Board::index_to_fr(63));
}

#[test]
fn test_get_piece() {
  let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
  let board = Board::from_fen(fen);
  assert_eq!(BLACK_ROOK, board.get_piece(2, 3));
  assert_eq!(WHITE_KING, board.get_piece(6, 4));
  assert_eq!(BLACK_KING, board.get_piece(7, 7));
}

#[test]
fn apply_board_en_passant_move() {
  let fen = "r1b2rk1/p1Q2p1p/4p1p1/1p6/2pPP3/5N2/PPP2PPP/2KR1B1R b - d3 0 1";
  let mut board = Board::from_fen(fen);

  board.apply_move(&Move::from_string("c4d3"));

  let expected_board =
    Board::from_fen("r1b2rk1/p1Q2p1p/4p1p1/1p6/4P3/3p1N2/PPP2PPP/2KR1B1R w - - 0 2");

  println!("{}", board);
  print_board_mask(board.pieces.pawns());
  print_board_mask(board.pieces.all());
  assert_eq!(board, expected_board);
}

#[test]
fn board_update_en_passant_square_move() {
  let fen = "r3k2r/1ppnqpp1/p1n1p2p/4P3/3PN1b1/2PB4/PPQ3PP/R1B2RK1 b kq - 0 13";
  let mut board = Board::from_fen(fen);

  board.apply_move(&Move::from_string("f7f5"));

  let expected_board =
    Board::from_fen("r3k2r/1ppnq1p1/p1n1p2p/4Pp2/3PN1b1/2PB4/PPQ3PP/R1B2RK1 w kq f6 0 14");

  assert_eq!(board, expected_board);
  assert_eq!(board.en_passant_square, string_to_square("f6"));
}

#[test]
fn test_hash_values() {
  // Position 1 - regular move
  let fen = "8/5pk1/5p1p/2R5/5K2/1r4P1/7P/8 b - - 8 43";
  let mut board = Board::from_fen(fen);
  board.apply_move(&Move::from_string("b3b4"));
  let after_move = board.hash;

  let fen = "8/5pk1/5p1p/2R5/1r3K2/6P1/7P/8 w - - 9 44";
  let board = Board::from_fen(fen);
  println!("Board after move hash: {}", after_move);
  println!("Board computed hash: {}", board.hash);
  assert_eq!(board.hash, after_move);

  // Position 2 - start position
  let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
  let mut board = Board::from_fen(fen);
  board.apply_move(&Move::from_string("e2e4"));
  let after_move = board.hash;

  let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1";
  let board = Board::from_fen(fen);
  println!("Board after move hash: {}", after_move);
  println!("Board computed hash: {}", board.hash);
  assert_eq!(board.hash, after_move);

  // Position 3 - King side white castle
  let fen = "rn1qkb1r/pbpp2pp/1p2pn2/8/4p3/2NP2P1/PPP1NPBP/R1BQK2R w KQkq - 0 7";
  let mut board = Board::from_fen(fen);
  board.apply_move(&Move::from_string("e1g1"));
  let after_move = board.hash;

  let fen = "rn1qkb1r/pbpp2pp/1p2pn2/8/4p3/2NP2P1/PPP1NPBP/R1BQ1RK1 b kq - 1 7";
  let board = Board::from_fen(fen);
  println!("Board after move hash: {}", after_move);
  println!("Board computed hash: {}", board.hash);
  assert_eq!(board.hash, after_move);

  // Position 3 - King side black castle
  let fen = "r2qk2r/pbpp2pp/1pn1pn2/2b5/4PB2/2N3P1/PPP1NPBP/R2Q1RK1 b kq - 2 9";
  let mut board = Board::from_fen(fen);
  board.apply_move(&Move::from_string("e8g8"));
  let after_move = board.hash;

  let fen = "r2q1rk1/pbpp2pp/1pn1pn2/2b5/4PB2/2N3P1/PPP1NPBP/R2Q1RK1 w - - 3 10";
  let board = Board::from_fen(fen);
  println!("Board after move hash: {}", after_move);
  println!("Board computed hash: {}", board.hash);
  assert_eq!(board.hash, after_move);

  // Position - Regular capture
  println!("Checking Regular capture");
  let fen = "r2q1rk1/pbpp2p1/1pn2n1p/2b1p1B1/4P3/2N3P1/PPP2PBP/R1NQ1RK1 w - - 0 12";
  let mut board = Board::from_fen(fen);
  board.apply_move(&Move::from_string("g5f6"));
  let after_move = board.hash;

  let fen = "r2q1rk1/pbpp2p1/1pn2B1p/2b1p3/4P3/2N3P1/PPP2PBP/R1NQ1RK1 b - - 0 12";
  let board = Board::from_fen(fen);
  println!("Board after move hash: {}", after_move);
  println!("Board computed hash: {}", board.hash);
  assert_eq!(board.hash, after_move);

  // Position - Promotion
  println!("Checking promotion");
  let fen = "8/7k/6pb/7p/4P3/3n2P1/B1p1N1KP/8 b - - 0 52";
  let mut board = Board::from_fen(fen);
  board.apply_move(&Move::from_string("c2c1q"));
  let after_move = board.hash;

  let fen = "8/7k/6pb/7p/4P3/3n2P1/B3N1KP/2q5 w - - 0 53";
  let board = Board::from_fen(fen);
  println!("Board after move hash: {}", after_move);
  println!("Board computed hash: {}", board.hash);
  assert_eq!(board.hash, after_move);

  // Position - White Queen side castle
  println!("Checking promotions queen side");
  let fen = "r3kbnr/ppp2ppp/2npbq2/4p3/4P3/2NPBQ2/PPP2PPP/R3KBNR w KQkq - 4 6";
  let mut board = Board::from_fen(fen);
  board.apply_move(&Move::from_string("e1c1"));
  let after_move = board.hash;

  let fen = "r3kbnr/ppp2ppp/2npbq2/4p3/4P3/2NPBQ2/PPP2PPP/2KR1BNR b kq - 5 6";
  let board = Board::from_fen(fen);
  println!("Board after move hash: {}", after_move);
  println!("Board computed hash: {}", board.hash);
  assert_eq!(board.hash, after_move);

  // Position - Black Queen side castle
  let fen = "r3kbnr/ppp2ppp/2npbq2/4p3/4P3/2NPBQ2/PPP2PPP/2KR1BNR b kq - 5 6";
  let mut board = Board::from_fen(fen);
  board.apply_move(&Move::from_string("e8c8"));
  let after_move = board.hash;

  let fen = "2kr1bnr/ppp2ppp/2npbq2/4p3/4P3/2NPBQ2/PPP2PPP/2KR1BNR w - - 6 7";
  let board = Board::from_fen(fen);
  println!("Board after move hash: {}", after_move);
  println!("Board computed hash: {}", board.hash);
  assert_eq!(board.hash, after_move);

  // Position - Losing Black king side castle
  let fen = "rnb1kbnr/ppp1pNpp/4q3/3p4/8/8/PPPPPPPP/RNBQKB1R w KQkq - 1 4";
  let mut board = Board::from_fen(fen);
  board.apply_move(&Move::from_string("f7h8"));
  let after_move = board.hash;

  let fen = "rnb1kbnN/ppp1p1pp/4q3/3p4/8/8/PPPPPPPP/RNBQKB1R b KQq - 0 4";
  let board = Board::from_fen(fen);
  println!("Board after move hash: {}", after_move);
  println!("Board computed hash: {}", board.hash);
  assert_eq!(board.hash, after_move);
}

#[test]
fn test_game_over_insufficient_material() {
  let fen = "8/4nk2/8/8/8/2K5/8/8 w - - 0 1";
  let board = Board::from_fen(fen);
  assert!(board.is_game_over_by_insufficient_material());

  let fen = "8/5k2/8/8/8/2KB4/8/8 w - - 0 1";
  let board = Board::from_fen(fen);
  assert!(board.is_game_over_by_insufficient_material());

  let fen = "8/4nk2/8/8/8/2KB4/8/8 w - - 0 1";
  let board = Board::from_fen(fen);
  assert!(!board.is_game_over_by_insufficient_material());

  let fen = "8/4nk2/8/8/8/2KP4/8/8 w - - 0 1";
  let board = Board::from_fen(fen);
  assert!(!board.is_game_over_by_insufficient_material());
}

#[ignore]
#[test]
fn generate_ranks_files() {
  let mut ranks: [u64; 8] = [0; 8];
  let mut files: [u64; 8] = [0; 8];

  for i in 0..64 {
    let (file, rank) = Board::index_to_fr(i);
    set_square_in_mask!(i, ranks[(rank - 1) as usize]);
    set_square_in_mask!(i, files[(file - 1) as usize]);
  }
  println!("pub const RANKS:[u64; 8] = {:#018X?};", ranks);
  println!("pub const FILES:[u64; 8] = {:#018X?};", files);
}

#[test]
fn test_pins_mask_calculations() {
  // Here we have a queen pinning a pawn
  let board = Board::from_fen("rnbqkbnr/pppp1ppp/8/4p2Q/4P3/8/PPPP1PPP/RNB1KBNR b KQkq - 1 2");
  print_board_mask(board.get_pins_rays(Color::Black));
  assert_eq!(9078117754732544, board.get_pins_rays(Color::Black));

  // Here basically all white pieces are pinned.
  let board = Board::from_fen("4r3/3k4/4r2b/8/4RP2/q2PKN1q/8/8 w - - 0 1");
  println!("Board: {}", board);

  // Pawn 19 is pinned to the rook direction
  // knight 21 is pinned to the queen direction
  // Rook 28 is pinned to the rooks direction
  // Pawn 29 is pinned to the bishop direction
  print_board_mask(board.get_pins_rays(Color::White));
  assert_eq!(158674092752896, board.get_pins_rays(Color::White));

  // Try another board without any pin
  let board =
    Board::from_fen("r2qr3/p2n1pkp/1p1p1np1/3bp3/2P1P2P/3B1N2/PP1Q1PP1/R3K2R w KQ - 0 15");
  println!("Board: {}", board);
  print_board_mask(board.get_pins_rays(Color::White));
  assert_eq!(0, board.get_pins_rays(Color::White));

  // Same again, a black pawn is in the way of the pin
  let board = Board::from_fen("r2qr3/p2n1pkp/1p1p1np1/3bp3/2P4P/3B1NP1/PP1Q1PP1/R3K2R w KQ - 0 15");
  println!("Board: {}", board);
  print_board_mask(board.get_pins_rays(Color::White));
  assert_eq!(0, board.get_pins_rays(Color::White));
}

#[test]
fn test_get_attackers() {
  let board = Board::from_fen("4r3/2k5/4r2b/6P1/3RRP2/q2PKN1q/8/3B4 w - - 0 1");
  println!("Board: {}", board);

  // A3 attackers:
  assert_eq!(0, board.get_attackers(string_to_square("a3"), Color::White));
  assert_eq!(0, board.get_attackers(string_to_square("a3"), Color::Black));

  // D3 attackers:
  let e = (1 << string_to_square("d4")) | (1 << string_to_square("e3"));
  let a = board.get_attackers(string_to_square("d3"), Color::White);
  assert_eq!(e, a);

  let e = 1 << string_to_square("a3");
  let a = board.get_attackers(string_to_square("d3"), Color::Black);
  assert_eq!(e, a);

  // F3 attackers:
  let e = (1 << string_to_square("d1")) | (1 << string_to_square("e3"));
  let a = board.get_attackers(string_to_square("f3"), Color::White);
  assert_eq!(e, a);

  let e = 1 << string_to_square("h3");
  let a = board.get_attackers(string_to_square("f3"), Color::Black);
  assert_eq!(e, a);

  // G5 attackers:
  let e = (1 << string_to_square("f3")) | (1 << string_to_square("f4"));
  let a = board.get_attackers(string_to_square("g5"), Color::White);
  assert_eq!(e, a);

  let e = 1 << string_to_square("h6");
  let a = board.get_attackers(string_to_square("g5"), Color::Black);
  assert_eq!(e, a);

  // F6 attackers:
  let e = 1 << string_to_square("g5");
  let a = board.get_attackers(string_to_square("f6"), Color::White);
  assert_eq!(e, a);

  let e = 1 << string_to_square("e6");
  let a = board.get_attackers(string_to_square("f6"), Color::Black);
  assert_eq!(e, a);

  // Test with board edges/pawns
  let board = Board::from_fen("6k1/5P2/4P3/8/4K3/8/8/8 w - - 0 1  ");
  println!("---------------------------------------------");
  println!("Board: {}", board);
  // G8 attackers:
  let e = 1 << string_to_square("f7");
  let a = board.get_attackers(string_to_square("g8"), Color::White);
  assert_eq!(e, a);

  let e = 0;
  let a = board.get_attackers(string_to_square("g8"), Color::Black);
  assert_eq!(e, a);

  let board = Board::from_fen("8/8/8/3k4/8/5p2/4p3/3K4 w - - 0 1");
  println!("---------------------------------------------");
  println!("Board: {}", board);
  // D1 attackers:
  let e = 0;
  let a = board.get_attackers(string_to_square("d1"), Color::White);
  assert_eq!(e, a);

  let e = 1 << string_to_square("e2");
  let a = board.get_attackers(string_to_square("d1"), Color::Black);
  assert_eq!(e, a);
}

#[test]
fn apply_under_promotion() {
  let fen = "8/8/6k1/8/8/4K3/5pq1/8 b - - 3 72";
  let mut board = Board::from_fen(fen);

  board.apply_move(&mv!(string_to_square("f2"), string_to_square("f1"), Promotion::BlackKnight));

  let expected_board = Board::from_fen("8/8/6k1/8/8/4K3/6q1/5n2 w - - 0 73");
  assert_eq!(board, expected_board);
}

#[test]
fn get_move_remove_checker_by_capturing_en_passant() {
  let fen = "8/p7/1pR5/6pk/6Pp/7P/P6K/3rr3 b - g3 0 34";
  let board = Board::from_fen(fen);

  let moves = board.get_moves();
  for m in &moves {
    println!("Move : {}", m);
  }
  assert_eq!(1, moves.len());
}

#[test]
fn check_legal_moves_king_capture_undefended_pieces() {
  let fen = "1K6/8/8/8/8/3k4/2Q5/8 b - - 1 1";
  let board = Board::from_fen(fen);

  let moves = board.get_moves();
  for m in &moves {
    println!("Move : {}", m);
  }
  assert_eq!(3, moves.len());
}

#[test]
fn check_legal_moves_en_passant_discovery() {
  let fen = "2b1R3/2p2p2/1p3N2/4P3/kp2Q1P1/4B3/2P1PK2/5B2 w - - 2 35";
  let mut board = Board::from_fen(fen);
  board.apply_move(&Move::from_string("c2c4"));

  // Here b4c3 is forbidden, it creates a bad-ass discovered check by removing 2
  // pawns from the pin ray.
  let moves = board.get_moves();
  for m in &moves {
    println!("Move : {}", m);
    assert_ne!(*m, en_passant_mv!(string_to_square("b4"), string_to_square("c3")));
  }
  assert_eq!(13, moves.len());

  // Another example
  let fen = "8/1r3p2/1p2p1p1/1b2P3/k2p1R1P/5P2/2P5/2K1Q3 w - - 4 60";
  let mut board = Board::from_fen(fen);
  board.apply_move(&Move::from_string("c2c4"));
  // Same story here with d4c3
  let moves = board.get_moves();
  for m in &moves {
    println!("Move : {} / {}", m, string_to_square("c3"));
  }
  assert_eq!(16, moves.len());
}

#[test]
fn check_with_discovery_no_en_passant() {
  let fen = "r1b3k1/2Bp1ppp/p1p5/2P5/3b2K1/P7/1P3rPP/4q3 b - - 3 24";
  let mut board = Board::from_fen(fen);
  board.apply_move(&Move::from_string("d7d5"));

  // Here c5d6 is forbidden
  let moves = board.get_moves();
  for m in &moves {
    println!("Move : {}", m);
    assert_ne!(*m, en_passant_mv!(string_to_square("c5"), string_to_square("d6")));
  }
  // assert_eq!(13, moves.len());
}
