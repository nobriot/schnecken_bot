use lazy_static::lazy_static;
use log::*;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Mutex;
use std::vec::Vec;

use crate::model::board::Board;
use crate::model::game_state::GameState;
use crate::model::moves::Move;

// Parsing PGN data. We do not parse anotations here
const PGN_REGEX: &str = r#"(\d*\.{1,3}\s+)?(?P<mv>([BKQNR]?[abcdefgh]?[12345678]?x?[abcdefgh][12345678]=?[BQNRbqnr]?|O-O|O-O-O)[#\+]?)[\?!]*\s+"#;

// List of board configurations with an associated set of moves
type ChessBook = Mutex<HashMap<Board, Vec<Move>>>;

lazy_static! {
  static ref CHESS_BOOK: ChessBook = Mutex::new(HashMap::new());
}

#[rustfmt::skip]
pub fn initialize_chess_book() {
  // Do not do this sevaral times.
  if CHESS_BOOK.lock().unwrap().len() > 0 {
    return ;
  }

  // Sicilian:
  let sicilian = "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 a7a6 f2f3 e7e6 a2a3 b8c6 c1e3 d6d5 e4d5 f6d5 c3d5 e6d5 d1d2";
  add_line_to_book(sicilian);
  let sicilian = "e2e4 c7c5 b1c3 a7a6 g1e2 d7d6 g2g3 g8f6 f1g2 e7e5 d2d3 b7b5 e1g1 f8e7 d1e1 e8g8 c3d1 b8c6 d1e3 a8b8 h2h3";
  add_line_to_book(sicilian);
  let sicilian = "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 a7a6 c1e3 e7e5 d4b3 c8e6 f2f3 f8e7 d1d2 e8g8 e1c1 b8d7 g2g4 b7b5 g4g5 b5b4 c3e2 f6e8 f3f4 a6a5 f4f5 a5a4 b3d4 e5d4 e2d4 b4b3 c1b1 b3c2 d4c2 e6b3 a2b3 a4b3 c2a3 d7e5 h2h4 a8a4 e3d4 d8a8";
  add_line_to_book(sicilian);
  let sicilian = "1. e4 c5 2. Nf3 d6 3. d4 cxd4 4. Nxd4 Nf6 5. Nc3 a6 6. Be3 e5 7. Nb3 Be6 8. f3 Be7 9. Qd2 O-O 10. O-O-O Nbd7 11. g4 b5 12. g5 b4 13. Ne2 Ne8 14. f4 a5 15. f5 a4 16. Nbd4 exd4 17. Nxd4 b3 18. Kb1 bxa2+ 19. Ka1 Nc5 20. fxe6 fxe6 21. Nc6 Qc7 22. Nxe7+ Qxe7 23. Bxc5 dxc5 24. Bc4";
  add_pgn_to_book(sicilian);
  let pgn = "1. e4 c5 2. Nf3 d6 3. d4 cxd4 4. Nxd4 Nf6 5. f3 e5 6. Nb3 Be6 7. c4 Nbd7 8. Be3 Rc8 9. Nc3 Bxc4 10. Bxc4 Rxc4 11. Qd3 Rc6 ";
  add_pgn_to_book(pgn);

  // Ruy lopez
  let ruy_lopez_pgn = "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3 O-O 9. h3 Na5 10. Bc2 c5 11. d4 Qc7 12. Nbd2 cxd4 13. cxd4 Nc6 14. Nb3 a5";
  add_pgn_to_book(ruy_lopez_pgn);

  // Weirdo provocative line:
  let pgn = "1. e4 f6 2. Qh5+ g6 3. Be2 gxh5 4. Bxh5";
  add_pgn_to_book(pgn);
  add_single_move_to_book("rnbqk1nr/ppppp2p/5ppb/7Q/4P3/8/PPPPBPPP/RNB1K1NR w KQkq - 2 4", "h5c5");

  // Budapest:
  let pgn = "1. d4 Nf6 2. c4 e5 3. dxe5 Ng4 4. Bf4 Nc6 5. Nf3 Bb4+ 6. Nbd2 Qe7 7. e3 Ngxe5 8. Nxe5 Nxe5 9. Be2 O-O 10. O-O Bxd2 11. Qxd2 d6 12. b4";
  add_pgn_to_book(pgn);

  // King indian defence
  let pgn = "1. Nf3 Nf6 2. c4 g6 3. Nc3 Bg7 4. e4 d6 5. d4 O-O 6. Be2 e5 7. O-O Nc6 8. d5 Ne7 9. b4 Nh5 10. Re1 f5 11. Ng5 Nf6 12. Bf3 c6 13. Be3 h6 14. Ne6 Bxe6 15. dxe6 fxe4";
  add_pgn_to_book(pgn);
  let pgn = "1. c4 Nf6 2. Nc3 g6 3. e4 d6 4. d4 Bg7 5. Nf3 O-O 6. Be2 e5 7. O-O Nc6 8. d5 Ne7 9. b4 a5 10. Ba3 axb4 11. Bxb4 Nd7 12. a4 Bh6 13. a5 f5 14. Bd3 Kh8 15. Re1 Nf6 16. c5 fxe4 17. cxd6 cxd6 18. Nxe4 Nxe4 19. Bxe4 Bf5 20. Qd3 Qd7 21. Qa3 Bxe4 22. Rxe4";
  add_pgn_to_book(pgn);

  // Queen indian defence
  let pgn = "1. d4 Nf6 2. c4 e6 3. Nf3 b6 4. g3 Ba6 5. b3 Bb4+ 6. Bd2 Be7 7. Bg2 c6 8. Bc3 d5 9. Ne5 Nfd7 10. Nxd7 Nxd7 11. Nd2 O-O 12. O-O f5 13. Rc1 Nf6 14. Bb2 Bd6 15. Nf3 Qe7 16. Ne5 Rac8 17. Nd3 Rfd8 18. Re1 Qe8 19. e3 g5";
  add_pgn_to_book(pgn);

  // Indian defence
  let pgn = "1. d4 Nf6 2. c4 e5 3. dxe5 Ng4 4. Nf3 Bc5 5. e3 Nc6 6. Be2 O-O 7. O-O Re8 8. Nc3 Ngxe5 9. Nxe5 Nxe5 10. b3 a5 11. Bb2 Ra6 12. Ne4 Ba7 13. Qd5 Rh6 14. Bxe5 c6 15. Bf6 gxf6 16. Qd3 d5 17. Ng3 Re5 18. Rad1 f5 19. Qc3";
  add_pgn_to_book(pgn);

  // Sample WC games:
  let pgn = "1. d4 Nf6 2. c4 e6 3. Nf3 d5 4. Nc3 dxc4 5. e4 Bb4 6. Bxc4 Nxe4 7. O-O Nf6 8. Qa4+ Nc6 9. Ne5 Bd6 10. Nxc6 bxc6 11. Qxc6+ Bd7 12. Qf3 O-O 13. Bg5 h6 14. Bh4 Rb8 15. b3 Rb6 16. Ne4 Be7 17. Nxf6+ Bxf6 18. Bxf6 Qxf6 19. Qxf6 gxf6 20. d5 e5 21. Rfc1 a5 22. Be2 c6 23. dxc6 Rxc6 24. Rxc6 Bxc6 25. Rc1 Bd7 26. Rc5 Ra8 27. f4 exf4 28. Bf3 Ra6 29. Kf2 Be6 30. Be2 Ra8 31. Bf3 Ra6 32. Bb7 Ra7 33. Be4 Kg7 34. Kf3 a4 35. Bc2 axb3 36. Bxb3 Rb7 37. Kxf4 Bxb3 38. axb3 Rxb3 39. g3 Rb4+ 40. Kf3 Rb3+ 41. Kf4 Rb4+ 42. Kf3 Rb3+ 43. Kf4";
  add_pgn_to_book(pgn);
  let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. d3 d6 6. c3 g6 7. Bg5 Bg7 8. Nbd2 h6 9. Bh4 O-O 10. O-O b5 11. Bc2 Qe8 12. Re1 Nh5 13. Nf1 Nf4 14. Ne3 Bb7 15. a4 Nd8 16. d4 Nde6 17. b4 exd4 18. cxd4 a5 19. d5 Ng5 20. Nxg5 hxg5 21. Bxg5 Qe5 22. Qg4 Nh5 23. Rab1 Bc8 24. Nf5 bxa4 25. Qh4 Bxf5 26. exf5 Qd4 27. Qxd4 Bxd4 28. Re4 Bf6 29. Bxf6 Nxf6 30. Rc4 axb4 31. Rcxb4 Nxd5 32. Rc4 a3 33. fxg6 c5 34. gxf7+ Kxf7 35. Bb3 Kf6 36. Rd1 Rfb8 37. Ba2 Rb2 38. Rxd5 Rxa2 39. Rxd6+ Ke7 40. Rd1 Ra5 41. h4 Rb2 42. Re4+ Kf6 43. Rd6+ Kf7 44. Rd7+ Kf6 45. Rd6+ Kf7 46. Rd7+ Kf6 47. Rd6+";
  add_pgn_to_book(pgn);
  let pgn = "1. Nf3 Nf6 2. c4 e6 3. g3 d5 4. Bg2 Be7 5. d4 O-O 6. Qc2 c5 7. O-O cxd4 8. Nxd4 e5 9. Nf5 d4 10. Nxe7+ Qxe7 11. Bg5 h6 12. Bxf6 Qxf6 13. Nd2 Bf5 14. Qb3 Nd7 15. Qa3 Qb6 16. Rfc1 Rfc8 17. b4 a5 18. c5 Qa6 19. Nc4 Be6 20. Nd6 axb4";
  add_pgn_to_book(pgn);
  let pgn = "1. e4 c5 2. Nf3 d6 3. Bb5+ Nc6 4. O-O Bd7 5. Re1 Nf6 6. h3 a6 7. Bf1 g5 8. d4 g4 9. d5 gxf3 10. dxc6 Bxc6 11. Qxf3 Nd7";
  add_pgn_to_book(pgn);

  // More random games
  let pgn = "1. f4 d5 2. Nf3 g6 3. g3 Bg7 4. Bg2 Nf6 5. O-O O-O 6. d3 c5 7. c3 Nc6 8. Na3 Rb8 9. Ne5 Qc7 10. Qa4 a6 11. Nxc6 bxc6 12. e4 Bd7 13. Nc2 h5 14. Ne3 h4 15. Qd1 Nh5 16. Bf3 hxg3 17. Bxh5 gxh5 18. Qxh5 c4 19. Rf3 gxh2+ 20. Kxh2";
  add_pgn_to_book(pgn);

  // Aggressive games
  // Fried liver:
  let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bc4 Nf6 4. Ng5 d5 5. exd5 Nxd5 6. Nxf7 Kxf7 7. Qf3+ Ke6 8. Nc3 Nb4 9. O-O c6 10. d4 Kd7 11. a3";
  add_pgn_to_book(pgn);
  add_single_move_to_book("r1bq1b1r/ppp1k1pp/2n5/3np3/2B5/5Q2/PPPP1PPP/RNB1K2R w KQ - 2 8", "c4d5");
  add_single_move_to_book("r1b2b1r/ppp2kpp/2n2q2/3np3/2B5/5Q2/PPPP1PPP/RNB1K2R w KQ - 2 8", "c4d5");
  let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bc4 Nf6 4. Ng5 d5 5. exd5 Na5 6. Bb5+ c6 7. dxc6 bxc6 8. Qf3 Be7 9. Bd3";
  add_pgn_to_book(pgn);

  // Traps:
  add_single_move_to_book("rnbqkbnr/pppp2pp/5p2/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 3", "f3e5");
  add_single_move_to_book("rnbqkbnr/pppp2pp/8/4p3/4P3/8/PPPP1PPP/RNBQKB1R w KQkq - 0 4", "d1h5");
  add_single_move_to_book("rnbqkbnr/pppp3p/6p1/4p2Q/4P3/8/PPPP1PPP/RNB1KB1R w KQkq - 0 5", "h5e5");

  add_single_move_to_book("rnbqkb1r/pppp1ppp/5n2/4N3/8/8/PPPPQPPP/RNB1KB1R w KQkq - 2 5", "e5c6");
  add_single_move_to_book("r1bqk2r/pppp1ppp/2n5/8/2BP4/2b2N2/P4PPP/R1BQ1RK1 w kq - 0 10", "c1a3");
  add_single_move_to_book("r1bqk2r/pppp1ppp/2n5/8/2BP4/B4N2/P4PPP/b2Q1RK1 w kq - 0 11", "f1e1");
  add_single_move_to_book("r1bqk2r/ppppnppp/8/8/2BP4/B4N2/P4PPP/b2QR1K1 w kq - 2 12", "a3e7");

  add_single_move_to_book("rnbqk1nr/ppp2ppp/8/4P3/1BP5/8/PP2KpPP/RN1Q1BNR b kq - 1 7", "f2g1n");
  add_single_move_to_book("rnbqk1nr/ppp2ppp/8/4P3/1BP5/8/PP2K1PP/RN1Q1BR1 b kq - 0 8", "c8d4");
  

  add_single_move_to_book("r1b1kbnr/pp3ppp/1q2p3/3pP3/3n4/3B1N2/PP3PPP/RNBQK2R w KQkq - 0 8", "f3d4");
  add_single_move_to_book("r1b1kbnr/pp3ppp/4p3/3pP3/3q4/3B4/PP3PPP/RNBQK2R w KQkq - 0 9", "d3b5");
  add_single_move_to_book("r3kbnr/pp1b1ppp/4p3/1B1pP3/3q4/8/PP3PPP/RNBQK2R w KQkq - 2 10", "b5d7");
  add_single_move_to_book("r4bnr/pp1k1ppp/4p3/3pP3/3q4/8/PP3PPP/RNBQK2R w KQ - 0 11", "d1d4");

  add_single_move_to_book("rnbqkb1r/ppp1ppp1/5n1p/6N1/8/3B4/PPP2PPP/RNBQK2R w KQkq - 0 6", "g5f7");
  add_single_move_to_book("rnbq1b1r/ppp1pkp1/5n1p/8/8/3B4/PPP2PPP/RNBQK2R w KQ - 0 7", "d3g6");
  add_single_move_to_book("rnbq1b1r/ppp1p1p1/5nkp/8/8/8/PPP2PPP/RNBQK2R w KQ - 0 8", "d1d8");

  add_single_move_to_book("rnb1kbnr/pppp1ppp/8/8/3PPp1q/6P1/PPP4P/RNBQKBNR b KQkq - 0 4", "f4g3");
  add_single_move_to_book("rnb1kbnr/pppp1ppp/8/8/3PP2q/5Np1/PPP4P/RNBQKB1R b KQkq - 1 5", "g3g2");
  add_single_move_to_book("rnb1kbnr/pppp1ppp/8/8/3PP2N/8/PPP3pP/RNBQKB1R b KQkq - 0 6", "g2h1q");

  add_single_move_to_book("r1bqkb1r/pppp1ppp/2n5/1B2p3/4P1n1/5N1P/PPPP1PP1/RNBQ1RK1 b kq - 0 5", "h7h5");
  add_single_move_to_book("r1bqkb1r/pppp1pp1/2n5/1B2p2p/4P1P1/5N2/PPPP1PP1/RNBQ1RK1 b kq - 0 6", "h5g4");
  add_single_move_to_book("r1bqkb1r/pppp1pp1/2n5/1B2p3/4P1p1/8/PPPP1PPN/RNBQ1RK1 b kq - 1 7", "d8h4");

  add_pgn_from_position("r1b1kb1r/pppp1pp1/2n5/1B2p3/4P1pq/8/PPPP1PPN/RNBQ1RK1 w kq - 2 8","8. f4 g3 9. Rf3 Qxh2+ 10. Kf1 Nd4 11. Ke1 Qxg2 12. Rf1 Rh1 13. Qe2 Nxe2 14. Rxh1 Nd4 15. Bd3 Qxh1+ 16. Bf1 Qxe4+ 17. Kd1 Qxc2+ 18. Ke1 Qxc1#");


  add_single_move_to_book("rnbqkb1r/pppp1ppp/8/4P3/6n1/7P/PPPNPPP1/R1BQKBNR b KQkq - 0 4", "g4e3");
  add_single_move_to_book("rnbqkb1r/pppp1ppp/8/4P3/8/4P2P/PPPNP1P1/R1BQKBNR b KQkq - 0 5", "d8h4");


  add_single_move_to_book("r1b1k2r/ppppqppp/2n5/4P3/1bP2Bn1/P4N2/1P1NPPPP/R2QKB1R b KQkq - 0 7", "g4e5");
  add_single_move_to_book("rnbqk2r/ppp2ppp/3b4/8/2P3n1/5N1P/PP2PPP1/RNBQKB1R b KQkq - 0 6", "g4f2");
  add_single_move_to_book("rnbqk2r/ppp2ppp/3b4/8/2P5/5N1P/PP2PKP1/RNBQ1B1R b kq - 0 7", "d6g3");

  add_single_move_to_book("rnbqk1nr/ppp2p1p/3b4/6p1/4P3/5N2/PPPP2PP/RNBQKB1R b KQkq - 0 5", "g5g4");
  add_single_move_to_book("rnbqk1nr/ppp2p1p/3b4/4P3/6p1/5N2/PPPP2PP/RNBQKB1R b KQkq - 0 6", "g5f3");
  add_single_move_to_book("rnbqk1nr/ppp2p1p/3P4/8/8/5p2/PPPP2PP/RNBQKB1R b KQkq - 0 7", "d8e4");
  add_single_move_to_book("rnb1k1nr/ppp2p1p/3P4/8/7q/5pP1/PPPP3P/RNBQKB1R b KQkq - 0 8", "h4e4");

  add_single_move_to_book("r1bqkbnr/pppp1ppp/8/4N3/2BnP3/8/PPPP1PPP/RNBQK2R b KQkq - 0 4", "d8g5");
  add_single_move_to_book("r1b1kbnr/pppp1Npp/8/6q1/2BnP3/8/PPPP1PPP/RNBQK2R b KQkq - 0 5", "g5g2");
  add_single_move_to_book("r1b1kbnr/pppp1Npp/8/8/2BnP3/8/PPPP1PqP/RNBQKR2 b Qkq - 1 6", "g2e4");
  add_pgn_from_position("r1b1kbnr/pppp1ppp/8/4N1q1/2BnP3/8/PPPP1PPP/RNBQK2R w KQkq - 1 5", "5. Bxf7+ Kd8 6. O-O Qxe5 7. c3 Ne6 8. d3 g5 9. Nd2");

}

/// Adds a line in the opnening to the book
///
/// ### Arguments
///
/// * `line`: list of moves separated with spaces.
///
/// e.g. `e2e4 c7c5 g1f3 d7d6 c5d4 f3d4 g8f6 b1c3 a7a6`
///
pub fn add_line_to_book(line: &str) {
  let mut game_state = GameState::default();
  let moves: Vec<&str> = line.split(' ').collect();
  let mut book = CHESS_BOOK.lock().unwrap();

  for chess_move in moves {
    if !book.contains_key(&game_state.board) {
      let _ = book.insert(game_state.board, Vec::new());
    }

    let move_list = book.get_mut(&game_state.board).unwrap();
    let m = Move::from_string(chess_move);
    if !move_list.contains(&m) {
      move_list.push(m);
    }

    game_state.apply_move(&Move::from_string(chess_move));
  }
}

/// Adds a line in the opnening to the book
///
/// ### Arguments
///
/// * `pgn`: PGN format str.
///
/// e.g. `1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3 O-O`
///
pub fn add_pgn_to_book(pgn: &str) {
  let mut game_state = GameState::default();
  let mut book = CHESS_BOOK.lock().unwrap();

  let pgn_re = Regex::new(PGN_REGEX).unwrap();

  // Use regex to extract move notations
  let captures = pgn_re.captures_iter(&pgn);
  for value in captures {
    // Find the mv (e.g. 'Kf7') and the annotation (e.g. '{ [%eval 0.36] [%clk 0:10:00] }')
    let mv = value.name("mv");
    if mv.is_none() {
      return;
    }
    let mv = mv.unwrap().as_str();
    //println!("Move: {mv}");
    let m_result = game_state.board.find_move_from_pgn_notation(mv);

    if m_result.is_err() {
      println!("Could not parse move: {}", mv);
      return;
    }
    let m = m_result.unwrap();

    if !book.contains_key(&game_state.board) {
      let _ = book.insert(game_state.board, Vec::new());
    }

    let move_list = book.get_mut(&game_state.board).unwrap();
    if !move_list.contains(&m) {
      move_list.push(m);
    }

    game_state.apply_move(&m);
  } // for value in captures
}

/// Adds a line in the from a start position
///
/// ### Arguments
///
/// * `fen`: Fen format str.
/// * `pgn`: PGN format str.
///
pub fn add_pgn_from_position(fen: &str, pgn: &str) {
  let mut game_state = GameState::from_fen(fen);
  let mut book = CHESS_BOOK.lock().unwrap();

  let pgn_re = Regex::new(PGN_REGEX).unwrap();

  // Use regex to extract move notations
  let captures = pgn_re.captures_iter(&pgn);
  for value in captures {
    // Find the mv (e.g. 'Kf7') and the annotation (e.g. '{ [%eval 0.36] [%clk 0:10:00] }')
    let mv = value.name("mv");
    if mv.is_none() {
      return;
    }
    let mv = mv.unwrap().as_str();
    //println!("Move: {mv}");
    let m_result = game_state.board.find_move_from_pgn_notation(mv);

    if m_result.is_err() {
      println!("Could not parse move: {}", mv);
      return;
    }
    let m = m_result.unwrap();

    if !book.contains_key(&game_state.board) {
      let _ = book.insert(game_state.board, Vec::new());
    }

    let move_list = book.get_mut(&game_state.board).unwrap();
    if !move_list.contains(&m) {
      move_list.push(m);
    }

    game_state.apply_move(&m);
  } // for value in captures
}

/// Adds a position in the opnening to the book
///
/// ### Arguments
///
/// * `fen`: Fen of the position to reach
/// * `mv`:  Notation of the move to play
///
pub fn add_single_move_to_book(fen: &str, mv: &str) {
  let game_state = GameState::from_fen(fen);
  let mut book = CHESS_BOOK.lock().unwrap();
  let m = Move::from_string(mv);

  if !book.contains_key(&game_state.board) {
    let _ = book.insert(game_state.board, Vec::new());
  }

  let move_list = book.get_mut(&game_state.board).unwrap();
  if !move_list.contains(&m) {
    move_list.push(m);
  }
}

// Check our known book moves, known positions that have been computed with an
// evaluation before, so that we do not need to find moves ourselves.
pub fn get_book_moves(board: &Board) -> Option<Vec<Move>> {
  let book = CHESS_BOOK.lock().unwrap();
  if book.contains_key(board) {
    Some(book.get(board).unwrap().clone())
  } else {
    None
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  #[test]
  fn test_book_lines() {
    initialize_chess_book();
    assert!(get_book_moves(&GameState::default().board).is_some());

    let fen = "rnbqkb1r/1p2pppp/p2p1n2/8/3NP3/2N5/PPP2PPP/R1BQKB1R w KQkq - 0 6";
    let game_state = GameState::from_fen(fen);
    assert!(get_book_moves(&game_state.board).is_some());

    let fen = "r4b1r/ppkbpppp/1qnp1n2/1B2N3/P2pP3/3K4/1PPB1PPP/RN1Q3R w - - 5 10";
    let game_state = GameState::from_fen(fen);
    assert_eq!(get_book_moves(&game_state.board), None);
  }
}
