use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;
use std::vec::Vec;

use super::*;
use crate::model::board::Board;
use crate::model::moves::Move;

lazy_static! {
  static ref PROVOCATIVE_CHESS_BOOK: ChessBook = Mutex::new(HashMap::new());
}

#[rustfmt::skip]
pub fn initialize_chess_book() {
  // Do not do this several times.
  if PROVOCATIVE_CHESS_BOOK.lock().unwrap().len() > 0 {
    return ;
  }

  // Bongcloud:
  let bongcloud = "1. e4 e5 2. Ke2 d5 3. d3 c6 4. Nf3 Nf6 5. Nbd2 Bc5 6. h3 O-O 7. g3 a5 8. Bg2 Qb6 9. Qe1 dxe4 10. dxe4 Qa6+ 11. Kd1 Nbd7 12. Qe2 Qa7 13. Ke1 b5 14. Nb3 Bb6"; 
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, bongcloud);
  let bongcloud = "1. e4 e5 2. Ke2 Nf6 3. d3 Bc5 4. Bg5 d5 5. exd5 Qxd5 6. Nc3 Qe6 7. g3 Ng4 8. Bh3 Qc6 9. Ne4 f5 10. Nxc5 Qxh1 11. Nf3 Qxd1+ 12. Rxd1 Nc6 13. Bg2 e4 14. dxe4 b6 15. h3 bxc5 16. hxg4 fxg4 17. Ne1 Ba6+ 18. Nd3 "; 
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, bongcloud);
  let bongcloud = "1. e4 e5 2. Ke2 d5 3. d3 Nc6 4. c3 Bc5 5. exd5 Qxd5 6. Qb3 Qd6 7. Ke1 f5 8. Nf3 Nf6 9. Na3 Qe7 10. Be3 Bb6 11. Nc4 Be6 12. Qa3 Bxc4 13. Qxe7+ Kxe7 14. dxc4 Bxe3 15. fxe3"; 
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, bongcloud);
  let bongcloud = "1. e4 e5 2. Ke2 d5 3. d3 Nc6 4. c3 Nf6 5. Qc2 Bc5 6. Nf3 Qd6 7. b4 Bb6 8. a4 a6 9. h3 Be6 10. Nbd2 Ba7 11. Ke1 O-O 12. Be2 b5 13. exd5 Nxd5 14. Ne4 Qd7 15. Nfg5 Bf5 16. Qb3 h6 17. g4 Bg6 18. Nf3"; 
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, bongcloud);

  // Wayward Queen attack:
  let wayward = "1. e4 e5 2. Qh5 d6 3. Bc4 g6 4. Qd1 Nf6 5. d3 c6 6. Nf3 Bg7 7. O-O b5 8. Bb3 O-O 9. a4 b4 10. Be3 a5 11. Nbd2 Nbd7 12. c3 bxc3 13. bxc3 Qc7 14. Re1 d5 15. Qc2 Re8";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, wayward);
  let wayward = "1. e4 e5 2. Qh5 Nf6 3. Qxe5+ Be7 4. Qf4 Nc6 5. d3 O-O 6. Nf3 d5 7. Be2 Bb4+ 8. Bd2 Re8 9. Nc3 dxe4 10. dxe4 Bxc3 11. Bxc3 Nxe4 12. Rd1 Qe7 13. O-O Nxc3 14. bxc3";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, wayward);
  let wayward = "1. e4 e5 2. Qh5 Nf6 3. Qxe5+ Be7 4. Qf4 Nc6 5. d3 O-O 6. Nf3 d5 7. Be2 Bb4+ 8. Bd2 Re8 9. Nc3 Bd6 10. Qh4 Be7 11. exd5 Nxd5 12. Qa4 Nxc3 13. Bxc3 Bd6 14. Kf1 ";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, wayward);

  // Desprez
  let desprez = "1. h4 d5 2. e3 c5 3. Nf3 Nc6 4. Bb5 Qc7 5. O-O a6 6. Bxc6+ Qxc6 7. Ne5 Qc7 8. d4 e6 9. c4 dxc4 10. a4 b6 11. Nd2 Bb7 12. Ndxc4 Rd8 13. b3 Nf6";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, desprez);
  let desprez = "1. h4 d5 2. d4 c5 3. e3 Nc6 4. c4 e6 5. Nf3 Nf6 6. a3 a6 7. dxc5 Bxc5 8. b4 Be7 9. Bb2 dxc4 10. Qxd8+ Bxd8 11. Bxc4 b5 12. Bd3 O-O 13. Nbd2 Be7 14. Ke2 Bb7 15. Rhc1 Rfc8 16. Nb3 Nd7 17. Rc2 Nd8 18. Rac1 Rxc2+ 19. Rxc2 Rc8 20. Rxc8 Bxc8 21. g4";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, desprez);
  let desprez = "1. h4 e5 2. c4 Nf6 3. Nc3 d5 4. cxd5 Nxd5 5. e3 Nxc3 6. bxc3 Bd6 7. Nf3 O-O 8. d4 Nc6 9. Rb1 Qe7 10. Bc4 Bf5 11. Rb2 h6 12. Be2 b6 13. Nd2";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, desprez);

  // Stafford gambit
  let stafford = "1. e4 e5 2. Nf3 Nf6 3. Nxe5 Nc6 4. Nxc6 dxc6 5. d3 Bc5 6. Be2 Ng4 7. Bxg4 Qh4 8. Qf3 Bxg4 9. Qg3 Qxg3 10. hxg3 Be6 11. Nc3 a5 12. Ne2 a4 13. f3 h5 14. d4 Be7 15. Nf4 Bc4 16. Rxh5 Rxh5 17. Nxh5 g6 18. Nf4";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, stafford);
  add_pgn_from_position(&PROVOCATIVE_CHESS_BOOK,"r1bqk2r/ppp2ppp/2p5/2b5/4P1n1/3P4/PPP1BPPP/RNBQ1RK1 b kq - 4 7", "7... h5 8. Bxg4 hxg4 9. Bf4 Be6 10. Nd2 Qd7 11. Re1 Bd6 12. e5 Be7 13. Ne4 O-O-O 14. Ng5 Bxg5 15. Bxg5 Rdf8");
  add_pgn_from_position(&PROVOCATIVE_CHESS_BOOK,"r1bqk2r/ppp2ppp/2p5/2b5/4P1n1/3P4/PPP1BPPP/RNBQ1RK1 b kq - 4 7", "7... h5 8. Nc3 Qf6 9. Bxg4 hxg4 10. e5 Qf5 11. Qe1 Be6 12. Qe4 Qh5 13. Bf4 O-O-O 14. Bg3");
  let stafford = "1. e4 e5 2. Nf3 Nf6 3. Nxe5 Nc6 4. Nxc6 dxc6 5. d3 Qe7 6. Be2 Be6 7. f4 O-O-O 8. Nc3 Qb4 9. a3 Qb6 10. f5 Bd7 11. b4 c5 12. Be3 Qc6 13. b5 Qd6 14. a4 Qe5 15. Qd2 Bd6 ";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, stafford);

  // Weirdo provocative line:
  let pgn = "1. e4 f6 2. Qh5+ g6 3. Be2";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, pgn);

  // Halloween gambit:
  let pgn = "1. e4 e5 2. Nf3 Nc6 3. Nc3 Nf6 4. Nxe5 Nxe5 5. d4 Ng6 6. e5 Ng8 7. Bc4 c6 8. Qe2 Bb4 9. O-O d5 10. exd6+ Kf8 11. Bxf7 Bxc3 12. Bxg8 Kxg8 13. Qc4+ Kf8 14. Qxc3";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, pgn);
  let pgn = "1. e4 e5 2. Nf3 Nc6 3. Nc3 Nf6 4. Nxe5 Nxe5 5. d4 Ng6 6. e5 Ng8 7. Bc4 d6 8. Qf3 Qd7 9. O-O Kd8 10. Bxf7 Qf5 11. Qd5 Nh6 12. Bxh6 gxh6 13. f4 Nxf4 14. Qb3 Qg4 15. g3 Nh3+ 16. Kg2 Ng5 17. Rae1 Qxd4 18. Kh1 a5 19. Bg8 Be7 20. Rf4";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, pgn);
  let pgn = "1. e4 e5 2. Nf3 Nc6 3. Nc3 Nf6 4. Nxe5 Nxe5 5. d4 Nc6 6. d5 Ne5 7. f4 Ng6 8. e5 Bc5 9. exf6 O-O 10. Be2 Qxf6 11. Ne4 Qb6 12. Nxc5 Qxc5 13. a4 Re8 14. Ra3 d6 15. Rc3 Qb6 16. Rb3 Qxb3 17. cxb3 Bg4 18. h3 Rxe2+ 19. Qxe2 Bxe2 20. Kxe2";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, pgn);
  let pgn = "1. e4 e5 2. Nc3 Nf6 3. Nf3 Nc6 4. Nxe5 Nxe5 5. d4 Nc6 6. d5 Ne5 7. f4 Ng6 8. e5 Ng8 9. Qe2 Bb4 10. f5 N6e7 11. Bg5 Kf8 12. Qf3 f6 13. Bh4 Bxc3+ 14. bxc3 h5 15. O-O-O d6 16. e6 Nh6 17. Bd3 Ng4 18. Bg3 Ne5 19. Bxe5 dxe5 20. g4 hxg4 21. Qxg4";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, pgn);

  // Cochrane gambit
  let pgn = "1. e4 e5 2. Nf3 Nf6 3. Nxe5 d6 4. Nxf7 Kxf7 5. d4 g6 6. Nc3 Kg7 7. f3 c5 8. dxc5 Nc6 9. Bf4 Be6 10. Bxd6 Bxd6 11. cxd6 Ne8 12. Qd2 Qxd6 13. O-O-O Qc5 14. Na4 Qe7 15. Qc3+ Nf6";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, pgn);
  let pgn = "1. e4 e5 2. Nf3 Nf6 3. Nxe5 d6 4. Nxf7 Kxf7 5. Nc3 g6 6. d4 Kg7 7. f3 c5 8. dxc5 Be7 9. cxd6 Bxd6 10. Nb5 Be5 11. Qxd8 Rxd8 12. f4 Bd4 13. e5 Ng4 14. Nxd4 Rxd4 15. Bd3 Bf5 16. Bxf5 gxf5 17. h3 Nh6 18. Ke2 Rd7 19. Be3 Nc6 20. Rhd1 Rad8 21. Rxd7+ Rxd7 22. Rg1 a6 23. g4 Kg6 24. c4 a5";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, pgn);
  let pgn = "1. e4 e5 2. Nf3 Nf6 3. Nxe5 d6 4. Nxf7 Kxf7 5. Bc4+ d5 6. Bb3 a5 7. Nc3 a4 8. Bxd5+ Nxd5 9. Nxd5 c6 10. Ne3 Qh4 11. d3 Bb4+ 12. c3 Bd6 13. g3 Qh3 14. Qf3+ Ke8 15. Qe2 Rf8 16. Bd2 Be6 17. Rf1 Kf7 18. d4 Kg8 19. f4 Nd7 20. e5 Be7 21. c4 Kh8 22. g4 b5 23. d5 Bg8 24. O-O-O a3 25. b4";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, pgn);
  let pgn = "1. e4 e5 2. Nf3 Nf6 3. Nxe5 d6 4. Nxf7 Kxf7 5. Bc4+ Be6 6. Bxe6+ Kxe6 7. d4 Kf7 8. Nc3 Be7 9. g4 Rf8 10. g5 Nfd7 11. Qg4 Kg8 12. Be3 Nc6 13. h4 Nb6 14. O-O-O";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, pgn);

  // Scotch
  let pgn = "1. e4 e5 2. Nf3 Nc6 3. d4 exd4 4. Bc4 Bb4+ 5. c3 dxc3 6. O-O cxb2 7. Bxb2 Nf6 8. e5 Nh5 9. Nbd2 b5 10. Bxb5 Rb8 11. Qa4 a6 12. Bxc6 dxc6 13. Rad1 Qd5 14. Qc2 Be6";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, pgn);
  let pgn = "1. e4 e5 2. Nf3 Nc6 3. d4 exd4 4. c3 dxc3 5. Bc4 cxb2 6. Bxb2 Bb4+ 7. Nc3 Bxc3+ 8. Bxc3 Nf6 9. Ng5 O-O 10. O-O h6 11. Nxf7 Rxf7 12. Bxf7+ Kxf7 13. Qb3+ d5 14. Rfd1 Ne7 15. Bxf6 Kxf6 16. exd5 Nf5 17. Qc3+ Kg6 18. Qc2 Kf7 19. Rac1 Nd6 20. Qxc7+ Qxc7 21. Rxc7+ Kf6";

  // Wing gambit
  let pgn = "1. e4 c5 2. b4 cxb4 3. a3 e5 4. Bb2 Nc6 5. Nf3 Qb6 6. Nxe5 bxa3 7. Bc3 a2 8. Nxf7 Kxf7 9. Bc4+ Ke7 10. Rxa2 Nf6 11. O-O";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, pgn);
  let pgn = "1. e4 c5 2. b4 d5 3. exd5 Qxd5 4. Nf3 cxb4 5. a3 e6 6. Bb2 Nf6 7. Be2 Qd8 8. O-O a5 9. Ne5 Nbd7 10. Bf3 Qc7 11. Qe2 Ra6 12. axb4 axb4 13. Nxd7 Nxd7 14. d3 Bd6 15. Bxg7 Rg8 16. Bb2 Bxh2+ 17. Kh1 Bf4 18. Re1 Rg5 19. Qe4 Rc5 20. Rxa6 bxa6 21. Qxb4 Bd6";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, pgn);
  let pgn = "1. e4 c5 2. b4 cxb4 3. d4 d5 4. exd5 Nf6 5. a3 Nxd5 6. axb4 Nxb4 7. Nf3 e6 8. Be2 Be7 9. c3 N4c6 10. Bf4 Nd7 11. O-O O-O 12. Na3 Nf6 13. c4 a6 14. Nb5 Bd7 15. Nc3 Nb4 16. Ne5 Bc6";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, pgn);

  // Some other random openings:
  let pgn = "1. e4 h5 2. Nf3 c5 3. c3 d5 4. exd5 Qxd5 5. d4 Nf6 6. Na3 cxd4 7. Bc4 Qe4+ 8. Be2 d3 9. Qxd3 Qxd3 10. Bxd3";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, pgn);
  
}

/// Check our known book moves, known positions that have been computed with an
/// evaluation before, so that we do not need to find moves ourselves.
pub fn get_book_moves(board: &Board) -> Option<Vec<Move>> {
  let book = PROVOCATIVE_CHESS_BOOK.lock().unwrap();
  if book.contains_key(board) {
    Some(book.get(board).unwrap().clone())
  } else {
    None
  }
}
