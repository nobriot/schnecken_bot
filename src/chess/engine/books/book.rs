use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;
use std::vec::Vec;

use super::*;
use crate::model::board::Board;
use crate::model::moves::Move;

lazy_static! {
  static ref CHESS_BOOK: ChessBook = Mutex::new(HashMap::new());
}

#[rustfmt::skip]
pub fn initialize_chess_book() {
  // Do not do this several times.
  if CHESS_BOOK.lock().unwrap().len() > 0 {
    return ;
  }

  // Sicilian:
  let sicilian = "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 a7a6 f2f3 e7e6 a2a3 b8c6 c1e3 d6d5 e4d5 f6d5 c3d5 e6d5 d1d2";
  add_line_to_book(&CHESS_BOOK,sicilian);
  let sicilian = "e2e4 c7c5 b1c3 a7a6 g1e2 d7d6 g2g3 g8f6 f1g2 e7e5 d2d3 b7b5 e1g1 f8e7 d1e1 e8g8 c3d1 b8c6 d1e3 a8b8 h2h3";
  add_line_to_book(&CHESS_BOOK,sicilian);
  let sicilian = "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 a7a6 c1e3 e7e5 d4b3 c8e6 f2f3 f8e7 d1d2 e8g8 e1c1 b8d7 g2g4 b7b5 g4g5 b5b4 c3e2 f6e8 f3f4 a6a5 f4f5 a5a4 b3d4 e5d4 e2d4 b4b3 c1b1 b3c2 d4c2 e6b3 a2b3 a4b3 c2a3 d7e5 h2h4 a8a4 e3d4 d8a8";
  add_line_to_book(&CHESS_BOOK, sicilian);
  let sicilian = "1. e4 c5 2. Nf3 d6 3. d4 cxd4 4. Nxd4 Nf6 5. Nc3 a6 6. Be3 e5 7. Nb3 Be6 8. f3 Be7 9. Qd2 O-O 10. O-O-O Nbd7 11. g4 b5 12. g5 b4 13. Ne2 Ne8 14. f4 a5 15. f5 a4 16. Nbd4 exd4 17. Nxd4 b3 18. Kb1 bxa2+ 19. Ka1 Nc5 20. fxe6 fxe6 21. Nc6 Qc7 22. Nxe7+ Qxe7 23. Bxc5 dxc5 24. Bc4";
  add_pgn_to_book(&CHESS_BOOK,sicilian);
  let pgn = "1. e4 c5 2. Nf3 d6 3. d4 cxd4 4. Nxd4 Nf6 5. f3 e5 6. Nb3 Be6 7. c4 Nbd7 8. Be3 Rc8 9. Nc3 Bxc4 10. Bxc4 Rxc4 11. Qd3 Rc6 ";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. e4 c5 2. Nc3 a6 3. Nge2 Nf6 4. e5 Ng4 5. f4 e6 6. d4 d5 7. exd6 Bxd6 8. h3 Nf6 9. dxc5 Bxc5 10. Qxd8+ Kxd8 11. g4 Bd7 12. Bd2 Bc6 13. Rh2 Kc7 14. O-O-O Nbd7 15. Bg2 Nb6 16. Bxc6 bxc6 17. Ng3";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. e4 c5 2. c3 Nf6 3. e5 Nd5 4. d4 cxd4 5. Nf3 d6 6. Bc4 dxe5 7. Nxe5 e6 8. O-O Bd6 9. cxd4 Nc6 10. Bxd5 exd5 11. Re1 Bxe5 12. Bf4 O-O 13. Bxe5 Bf5 14. Nc3 Qd7";
  add_pgn_to_book(&CHESS_BOOK,pgn);

  // Ruy lopez
  let ruy_lopez_pgn = "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3 O-O 9. h3 Na5 10. Bc2 c5 11. d4 Qc7 12. Nbd2 cxd4 13. cxd4 Nc6 14. Nb3 a5";
  add_pgn_to_book(&CHESS_BOOK,ruy_lopez_pgn);
  let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bb5 Nf6 4. O-O Nxe4 5. d4 Nd6 6. Bxc6 dxc6 7. dxe5 Nf5 8. Qxd8+ Kxd8 9. Nc3 Ke8 10. h3 h5 11. Bf4 Be7 12. Rad1 Be6 13. Ng5 Rh6 14. Rfe1 Bb4 15. g4 hxg4 16. hxg4 Ne7 17. Nxe6 Rxe6 18. Re3 Ng6 19. Bg3 Bxc3 20. bxc3 Nf8 21. f4 Rg6 22. g5";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bb5 Nf6 4. O-O Nxe4 5. Re1 Nd6 6. Nxe5 Be7 7. Bf1 Nxe5 8. Rxe5 O-O 9. d4 Bf6 10. Re1 Re8 11. c3 Rxe1 12. Qxe1 Ne8 13. Bf4 d5 14. Bd3 g6 15. Nd2 Ng7 16. Nf3 Bf5 17. Bxf5 Nxf5 18. Qd2 a5 19. Re1 a4";
  add_pgn_to_book(&CHESS_BOOK, pgn);

  // Berlin draw
  let pgn = "1.e4 e5 2. Nf3 Nc6 3. Bb5 Nf6 4. O-O Nxe4 5. d4 Nd6 6. dxe5 Nxb5 7. a4 Nbd4 8. Nxd4 Nxd4 9. Qxd4 d5 10. exd6 Qxd6 11. Qe4+ Qe6 12. Qd4 Qd6 13. Qe4+";
  add_pgn_to_book(&CHESS_BOOK, pgn);

  // Stafford gambit
  let stafford = "1. e4 e5 2. Nf3 Nf6 3. Nxe5 Nc6 4. Nxc6 dxc6 5. d3 Bc5 6. Be2 Ng4 7. Bxg4 Qh4 8. Qf3 Bxg4 9. Qg3 Qxg3 10. hxg3 Be6 11. Nc3 a5 12. Ne2 a4 13. f3 h5 14. d4 Be7 15. Nf4 Bc4 16. Rxh5 Rxh5 17. Nxh5 g6 18. Nf4";
  add_pgn_to_book(&CHESS_BOOK, stafford);
  add_pgn_from_position(&CHESS_BOOK,"r1bqk2r/ppp2ppp/2p5/2b5/4P1n1/3P4/PPP1BPPP/RNBQ1RK1 b kq - 4 7", "7... h5 8. Bxg4 hxg4 9. Bf4 Be6 10. Nd2 Qd7 11. Re1 Bd6 12. e5 Be7 13. Ne4 O-O-O 14. Ng5 Bxg5 15. Bxg5 Rdf8");
  add_pgn_from_position(&CHESS_BOOK,"r1bqk2r/ppp2ppp/2p5/2b5/4P1n1/3P4/PPP1BPPP/RNBQ1RK1 b kq - 4 7", "7... h5 8. Nc3 Qf6 9. Bxg4 hxg4 10. e5 Qf5 11. Qe1 Be6 12. Qe4 Qh5 13. Bf4 O-O-O 14. Bg3");
  let stafford = "1. e4 e5 2. Nf3 Nf6 3. Nxe5 Nc6 4. Nxc6 dxc6 5. d3 Qe7 6. Be2 Be6 7. f4 O-O-O 8. Nc3 Qb4 9. a3 Qb6 10. f5 Bd7 11. b4 c5 12. Be3 Qc6 13. b5 Qd6 14. a4 Qe5 15. Qd2 Bd6 ";
  add_pgn_to_book(&CHESS_BOOK, stafford);

  // Scotch
  let pgn = "1. e4 e5 2. Nf3 Nc6 3. d4 exd4 4. Bc4 Bb4+ 5. c3 dxc3 6. O-O cxb2 7. Bxb2 Nf6 8. e5 Nh5 9. Nbd2 b5 10. Bxb5 Rb8 11. Qa4 a6 12. Bxc6 dxc6 13. Rad1 Qd5 14. Qc2 Be6";
  add_pgn_to_book(&CHESS_BOOK, pgn);
  let pgn = "1. e4 e5 2. Nf3 Nc6 3. d4 exd4 4. c3 dxc3 5. Bc4 cxb2 6. Bxb2 Bb4+ 7. Nc3 Bxc3+ 8. Bxc3 Nf6 9. Ng5 O-O 10. O-O h6 11. Nxf7 Rxf7 12. Bxf7+ Kxf7 13. Qb3+ d5 14. Rfd1 Ne7 15. Bxf6 Kxf6 16. exd5 Nf5 17. Qc3+ Kg6 18. Qc2 Kf7 19. Rac1 Nd6 20. Qxc7+ Qxc7 21. Rxc7+ Kf6";
  add_pgn_to_book(&CHESS_BOOK, pgn);

  // Halloween gambit:
  let pgn = "1. e4 e5 2. Nf3 Nc6 3. Nc3 Nf6 4. Nxe5 Nxe5 5. d4 Ng6 6. e5 Ng8 7. Bc4 c6 8. Qe2 Bb4 9. O-O d5 10. exd6+ Kf8 11. Bxf7 Bxc3 12. Bxg8 Kxg8 13. Qc4+ Kf8 14. Qxc3";
  add_pgn_to_book(&CHESS_BOOK, pgn);
  let pgn = "1. e4 e5 2. Nf3 Nc6 3. Nc3 Nf6 4. Nxe5 Nxe5 5. d4 Ng6 6. e5 Ng8 7. Bc4 d6 8. Qf3 Qd7 9. O-O Kd8 10. Bxf7 Qf5 11. Qd5 Nh6 12. Bxh6 gxh6 13. f4 Nxf4 14. Qb3 Qg4 15. g3 Nh3+ 16. Kg2 Ng5 17. Rae1 Qxd4 18. Kh1 a5 19. Bg8 Be7 20. Rf4";
  add_pgn_to_book(&CHESS_BOOK, pgn);
  let pgn = "1. e4 e5 2. Nf3 Nc6 3. Nc3 Nf6 4. Nxe5 Nxe5 5. d4 Nc6 6. d5 Ne5 7. f4 Ng6 8. e5 Bc5 9. exf6 O-O 10. Be2 Qxf6 11. Ne4 Qb6 12. Nxc5 Qxc5 13. a4 Re8 14. Ra3 d6 15. Rc3 Qb6 16. Rb3 Qxb3 17. cxb3 Bg4 18. h3 Rxe2+ 19. Qxe2 Bxe2 20. Kxe2";
  add_pgn_to_book(&CHESS_BOOK, pgn);
  let pgn = "1. e4 e5 2. Nc3 Nf6 3. Nf3 Nc6 4. Nxe5 Nxe5 5. d4 Nc6 6. d5 Ne5 7. f4 Ng6 8. e5 Ng8 9. Qe2 Bb4 10. f5 N6e7 11. Bg5 Kf8 12. Qf3 f6 13. Bh4 Bxc3+ 14. bxc3 h5 15. O-O-O d6 16. e6 Nh6 17. Bd3 Ng4 18. Bg3 Ne5 19. Bxe5 dxe5 20. g4 hxg4 21. Qxg4";
  add_pgn_to_book(&CHESS_BOOK, pgn);

  // Budapest:
  let pgn = "1. d4 Nf6 2. c4 e5 3. dxe5 Ng4 4. Bf4 Nc6 5. Nf3 Bb4+ 6. Nbd2 Qe7 7. e3 Ngxe5 8. Nxe5 Nxe5 9. Be2 O-O 10. O-O Bxd2 11. Qxd2 d6 12. b4";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. d4 Nf6 2. c4 e5 3. dxe5 Ng4 4. Nc3 Nxe5 5. e4 Bb4 6. Bd2 d6 7. f4 Bg4 8. Qc2 Ned7 9. a3 Bxc3 10. Bxc3 O-O 11. Kf2 a5 12. h3 Be6 13. Nf3 f6 14. Re1 Nc6 15. g4 a4 16. Rg1 Nc5 17. Rg3 Qe7";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. d4 Nf6 2. c4 e5 3. Nf3 e4 4. Nfd2 c6 5. f3 d5 6. fxe4 dxe4 7. e3 c5 8. Nb3 a5 9. Nc3 cxd4 10. Qxd4 Nbd7";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. d4 Nf6 2. c4 e5 3. dxe5 Ng4 4. Nf3 Nc6 5. Bf4 Bb4+ 6. Nc3 Bxc3+ 7. bxc3 Qe7 8. Qd5 f6 9. exf6 Nxf6 10. Qd3 d6 11. g3 Ne5 12. Nxe5 dxe5 13. Bg5 c6";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. d4 Nf6 2. c4 e5 3. dxe5 Ng4 4. Nf3 Nc6 5. h3 Ngxe5 6. Nxe5 Nxe5 7. Nc3 Bb4 8. Qb3 Qe7 9. e3 O-O 10. Be2 b6 11. O-O Bxc3 12. Qxc3 Bb7 13. b4";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. e4 c5 2. c3 Nf6 3. d3 g6 4. f4 Bg7 5. g3 O-O 6. Bg2 Nc6 7. Ne2 e5 8. fxe5 Nxe5 9. O-O d6 10. h3 h6 11. d4 Nc6 12. Nd2 Re8 13. a3 Qc7 14. Rb1 cxd4 15. cxd4 d5 16. e5 Nxe5 17. dxe5 Qxe5 18. Rf2 Bf5 19. Ra1 Qc7 20. Nf1 Bc2";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. e4 c5 2. c3 Nc6 3. d4 d5 4. exd5 Qxd5 5. Nf3 Nf6 6. dxc5 Qxd1+ 7. Kxd1 e6 8. Be3 Ng4 9. b4 a5 10. b5 Nxe3+ 11. fxe3 Nd8 12. c6 bxc6 13. a4 Bc5 14. Nbd2 Bxe3 15. Ne5 cxb5 16. axb5";
  add_pgn_to_book(&CHESS_BOOK,pgn);

  // Vienna Game
  let pgn = "1. e4 e5 2. Nc3 Nf6 3. f4 d5 4. fxe5 Nxe4 5. Nf3 Be7 6. Qe2 Nxc3 7. dxc3 c5 8. Bf4 Nc6 9. O-O-O Be6 10. h4 h6 11. g3 Qd7 12. Bg2 O-O-O ";
  add_pgn_to_book(&CHESS_BOOK,pgn);

  // Danish gambit
  let pgn = "1. e4 e5 2. d4 exd4 3. c3 dxc3 4. Nxc3 Bc5 5. Nf3 d6 6. Bc4 Nf6 7. e5 dxe5 8. Qxd8+ Kxd8 9. Nxe5 Re8 10. Bf4 Be6 11. O-O-O+ Kc8 12. Bxe6+ fxe6 13. Nd3 Bf8 14. Rhe1 Nc6 15. Bg3 b6";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. e4 e5 2. d4 exd4 3. c3 Qe7 4. Bd3 d5 5. cxd4 dxe4 6. Bc2 Nc6 7. Nc3 Bd7 8. Nge2 f5 9. Bf4 Nf6 10. O-O O-O-O";
  add_pgn_to_book(&CHESS_BOOK,pgn);

  // Alekine defence
  let pgn = "1. e4 Nf6 2. e5 Nd5 3. d4 d6 4. c4 Nb6 5. f4 Bf5 6. Nc3 e6 7. Nf3 dxe5 8. fxe5 Nc6 9. Be3 Bg4 10. Be2 Bxf3 11. gxf3 Qh4+ 12. Bf2 Qf4 13. c5 Nd7 14. Qc1 Qxc1+ 15. Rxc1 O-O-O";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. e4 Nf6 2. e5 Nd5 3. d4 d6 4. c4 Nb6 5. f4 g6 6. Nc3 Bg7 7. Be3 Be6 8. Rc1 Nxc4 9. Bxc4 Bxc4 10. Qa4+ b5 11. Nxb5 Bxb5 12. Qxb5+ Nd7 13. Nf3 Rb8 14. Qe2 Nb6 15. Bd2 Qd7 16. h4 Nd5 17. b3 h5";
  add_pgn_to_book(&CHESS_BOOK,pgn);

  // Evans gambit
  let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bc4 Bc5 4. b4 Bxb4 5. c3 Ba5 6. d4 d6 7. Qb3 Qd7 8. dxe5 Nxe5 9. Nxe5 dxe5 10. O-O c6 11. Rd1 Qc7 12. Ba3 Bg4 13. f3 b5 14. fxg4 bxc4 15. Qxc4 Bb6+ 16. Kf1 Nf6 17. h3 Rd8 18. Rxd8+ Qxd8 19. Ke2 Qd7 20. Nd2 c5 21. Rd1";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bc4 Bc5 4. b4 Bxb4 5. c3 Ba5 6. d4 exd4 7. Qb3 Qf6 8. O-O Bb6 9. e5 Qg6 10. cxd4 Nge7";
  add_pgn_to_book(&CHESS_BOOK,pgn);

  // Center game
  let pgn = "1. e4 e5 2. d4 exd4 3. Qxd4 Nc6 4. Qe3 Nf6 5. Nc3 Bb4 6. Bd2 O-O 7. O-O-O Re8 8. Qg3 Rxe4 9. a3 Bd6 10. f4 Re8 11. Bd3 Bf8 12. Nf3 d5";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. e4 e5 2. d4 exd4 3. Nf3 Bb4+ 4. Nbd2 Nc6 5. Bd3 Bc5 6. Nb3 Bb6 7. O-O d6 8. a4 a6 9. Re1 h6 10. a5 Ba7 11. e5 Nge7 12. Bf4 O-O";
  add_pgn_to_book(&CHESS_BOOK,pgn);

  // King's gambit
  let pgn = "1. e4 e5 2. f4 exf4 3. Nf3 g5 4. h4 g4 5. Ne5 Nf6 6. Bc4 d5 7. exd5 Bd6 8. d4 Nh5 9. O-O O-O 10. Rxf4 Nxf4 11. Bxf4 Qxh4 12. g3 Qh5 13. Nc3 Bf5 14. Qd2 f6 15. Nd3 Nd7 16. Bxd6 cxd6 17. Nf4 Qh6";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. e4 e5 2. f4 d5 3. exd5 exf4 4. Nf3 Nf6 5. Bc4 Nxd5 6. O-O Be6 7. Bb3 Be7 8. c4 Nb6 9. d4 Nxc4 10. Nc3 Nb6 11. d5";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. e4 e5 2. f4 Bc5 3. Nf3 d6 4. c3 Nf6 5. d4 exd4 6. cxd4 Bb6 7. Nc3 O-O 8. e5 dxe5 9. fxe5 Nd5 10. Bg5 Nxc3 11. bxc3 Qd5 12. Bd3 Bg4 13. h3 Bh5 14. Qe2 Nc6 15. Be4 Qd7 16. O-O";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. e4 e5 2. f4 Qh4+ 3. g3 Qe7 4. fxe5 d6 5. exd6 Qxe4+ 6. Qe2 Qxe2+ 7. Nxe2 Bxd6 8. Bg2 Nc6 9. d4 Bg4 10. c3 O-O-O 11. h3 Bxe2 12. Kxe2 Bxg3 13. Bxc6";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. e4 e5 2. f4 d5 3. Nf3 dxe4 4. Nxe5 Bd6 5. Nc3 Nf6 6. d4 exd3 7. Bxd3 O-O 8. Be3 Nc6 9. Nxc6 bxc6 10. O-O Re8 11. Qf3 Ng4 12. Bf2 Nxf2 13. Qxf2";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  add_pgn_from_position(&CHESS_BOOK, "rnbqkbnr/pppp1p1p/8/6p1/2B1Pp2/5N2/PPPP2PP/RNBQK2R b KQkq - 1 4", "4... g4 5. O-O gxf3 6. Qxf3 Qf6 7. d3 d6 8. Nc3 Ne7");

  // Russian game
  let pgn = "1. e4 e5 2. Nf3 Nf6 3. Nxe5 d6 4. Nf3 Nxe4 5. Bd3 Nf6 6. O-O Be7 7. c3 Bg4 8. h3 Bh5 9. Re1 Nbd7 10. Bf1 O-O 11. d4 d5 12. Bf4 Ne4 13. Bd3 f5";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. e4 e5 2. Nf3 Nf6 3. Nxe5 d6 4. Nf3 Nxe4 5. Nc3 Nxc3 6. dxc3 Be7 7. Be3 Nc6 8. Qd2 Be6 9. O-O-O Qd7 10. Kb1 Bf6 11. h4 O-O-O 12. Nd4 Nxd4 13. Bxd4 Be5 14. Qe3 Qa4 15. b3 Bxd4 16. Rxd4 Qc6 17. Kb2 Kb8 18. Bd3 d5";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. e4 e5 2. Nf3 Nf6 3. d4 Nxe4 4. Nxe5 d5 5. Nd2 Bd6 6. Bd3 Bxe5 7. dxe5 Nc5 8. Nf3 Bg4 9. Be2 Nc6 10. O-O O-O 11. h3 Bh5 12. Be3 Ne4 13. Bf4 Re8 14. Re1 Qd7 15. Qd3 Nc5 16. Qd2 Ne4 17. Qe3 Bxf3 18. Bxf3 Nxe5 19. Bxe4 dxe4 20. Rad1 Qb5 21. Qe2 Qxb2 22. Qxe4 Qxa2";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. e4 e5 2. Nf3 Nf6 3. Nxe5 Nxe4 4. Qe2 Qe7 5. Qxe4 d6 6. d4 dxe5 7. dxe5 Nc6 8. Bb5 Bd7 9. Nc3 O-O-O 10. Bf4 g5 11. Bg3 a6 12. Bc4 Be6 13. Bxa6 Qc5 14. Bd3 h5 15. h3 Rd4 16. Qe2 h4 17. Bh2 g4 18. hxg4 h3 19. f4 Qa5 20. Kf1 Qb6 21. Ne4 Qxb2 22. Rd1 Rxd3 23. Rxd3 Nd4 24. Qd2 Bb4 25. c3 Qb1+ 26. Qd1 Qxa2 27. Qd2";
  add_pgn_to_book(&CHESS_BOOK,pgn);

  // Scandinavian defence
  let pgn = "1. e4 d5 2. exd5 Qxd5 3. Nc3 Qa5 4. d4 Nf6 5. Nf3 c6 6. Bc4 Bf5 7. Bd2 e6 8. Nd5 Qd8 9. Nxf6+ gxf6 10. Bb3 Nd7 11. Qe2 Qc7 12. Nh4 Bg6 13. O-O-O O-O-O 14. g3 Bd6 15. Nxg6 hxg6 16. h4 f5 17. Bg5 Rde8 18. h5 gxh5 19. Rxh5 Rhg8 20. Bh4";
  add_pgn_to_book(&CHESS_BOOK, pgn);
  let pgn = "1. e4 d5 2. exd5 Qxd5 3. Nc3 Qd6 4. d4 Nf6 5. Nf3 Nc6 6. d5 Nb4 7. Nb5 Qd8 8. c4 e6 9. a3 Na6 10. Qa4 c6 11. dxc6 bxc6 12. Nc3 Nc5 13. Qc2 Qc7 14. g3 a5 15. Bf4 Bd6 16. Bxd6 Qxd6 17. Rd1 Qc7 18. Bg2 O-O 19. O-O";
  add_pgn_to_book(&CHESS_BOOK, pgn);
  let pgn = "1. e4 d5 2. e5 c5 3. c3 Nc6 4. d4 a6 5. Be2 h6 6. Be3 cxd4 7. cxd4 Bf5 8. a3 e6 9. Nf3 Nge7 10. O-O Bh7 11. Nc3 Nf5 12. b4 Nxe3 13. fxe3 Be7 14. Na4 O-O 15. Rc1 Nb8 16. Qb3";
  add_pgn_to_book(&CHESS_BOOK, pgn);

  // Queen gambit
  let pgn = "1. d4 d5 2. c4 c6 3. Nf3 Nf6 4. Nc3 e6 5. Bg5 h6 6. Bh4 dxc4 7. e4 g5 8. Bg3 b5 9. Be2 Bb7 10. O-O Nbd7 11. Ne5 Bg7 12. Nxd7 Nxd7 13. Bd6 a6 14. a4 b4 15. Bxb4 Qb6 16. Ba3 Qxd4 17. Qc2 c5";
  add_pgn_to_book(&CHESS_BOOK,pgn);

  // King indian defence
  let pgn = "1. Nf3 Nf6 2. c4 g6 3. Nc3 Bg7 4. e4 d6 5. d4 O-O 6. Be2 e5 7. O-O Nc6 8. d5 Ne7 9. b4 Nh5 10. Re1 f5 11. Ng5 Nf6 12. Bf3 c6 13. Be3 h6 14. Ne6 Bxe6 15. dxe6 fxe4";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. c4 Nf6 2. Nc3 g6 3. e4 d6 4. d4 Bg7 5. Nf3 O-O 6. Be2 e5 7. O-O Nc6 8. d5 Ne7 9. b4 a5 10. Ba3 axb4 11. Bxb4 Nd7 12. a4 Bh6 13. a5 f5 14. Bd3 Kh8 15. Re1 Nf6 16. c5 fxe4 17. cxd6 cxd6 18. Nxe4 Nxe4 19. Bxe4 Bf5 20. Qd3 Qd7 21. Qa3 Bxe4 22. Rxe4";
  add_pgn_to_book(&CHESS_BOOK,pgn);

  // Queen indian defence
  let pgn = "1. d4 Nf6 2. c4 e6 3. Nf3 b6 4. g3 Ba6 5. b3 Bb4+ 6. Bd2 Be7 7. Bg2 c6 8. Bc3 d5 9. Ne5 Nfd7 10. Nxd7 Nxd7 11. Nd2 O-O 12. O-O f5 13. Rc1 Nf6 14. Bb2 Bd6 15. Nf3 Qe7 16. Ne5 Rac8 17. Nd3 Rfd8 18. Re1 Qe8 19. e3 g5";
  add_pgn_to_book(&CHESS_BOOK,pgn);

  // Indian defence
  let pgn = "1. d4 Nf6 2. c4 e5 3. dxe5 Ng4 4. Nf3 Bc5 5. e3 Nc6 6. Be2 O-O 7. O-O Re8 8. Nc3 Ngxe5 9. Nxe5 Nxe5 10. b3 a5 11. Bb2 Ra6 12. Ne4 Ba7 13. Qd5 Rh6 14. Bxe5 c6 15. Bf6 gxf6 16. Qd3 d5 17. Ng3 Re5 18. Rad1 f5 19. Qc3";
  add_pgn_to_book(&CHESS_BOOK,pgn);

  // Sample WC games:
  let pgn = "1. d4 Nf6 2. c4 e6 3. Nf3 d5 4. Nc3 dxc4 5. e4 Bb4 6. Bxc4 Nxe4 7. O-O Nf6 8. Qa4+ Nc6 9. Ne5 Bd6 10. Nxc6 bxc6 11. Qxc6+ Bd7 12. Qf3 O-O 13. Bg5 h6 14. Bh4 Rb8 15. b3 Rb6 16. Ne4 Be7 17. Nxf6+ Bxf6 18. Bxf6 Qxf6 19. Qxf6 gxf6 20. d5 e5 21. Rfc1 a5 22. Be2 c6 23. dxc6 Rxc6 24. Rxc6 Bxc6 25. Rc1 Bd7 26. Rc5 Ra8 27. f4 exf4 28. Bf3 Ra6 29. Kf2 Be6 30. Be2 Ra8 31. Bf3 Ra6 32. Bb7 Ra7 33. Be4 Kg7 34. Kf3 a4 35. Bc2 axb3 36. Bxb3 Rb7 37. Kxf4 Bxb3 38. axb3 Rxb3 39. g3 Rb4+ 40. Kf3 Rb3+ 41. Kf4 Rb4+ 42. Kf3 Rb3+ 43. Kf4";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. d3 d6 6. c3 g6 7. Bg5 Bg7 8. Nbd2 h6 9. Bh4 O-O 10. O-O b5 11. Bc2 Qe8 12. Re1 Nh5 13. Nf1 Nf4 14. Ne3 Bb7 15. a4 Nd8 16. d4 Nde6 17. b4 exd4 18. cxd4 a5 19. d5 Ng5 20. Nxg5 hxg5 21. Bxg5 Qe5 22. Qg4 Nh5 23. Rab1 Bc8 24. Nf5 bxa4 25. Qh4 Bxf5 26. exf5 Qd4 27. Qxd4 Bxd4 28. Re4 Bf6 29. Bxf6 Nxf6 30. Rc4 axb4 31. Rcxb4 Nxd5 32. Rc4 a3 33. fxg6 c5 34. gxf7+ Kxf7 35. Bb3 Kf6 36. Rd1 Rfb8 37. Ba2 Rb2 38. Rxd5 Rxa2 39. Rxd6+ Ke7 40. Rd1 Ra5 41. h4 Rb2 42. Re4+ Kf6 43. Rd6+ Kf7 44. Rd7+ Kf6 45. Rd6+ Kf7 46. Rd7+ Kf6 47. Rd6+";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. Nf3 Nf6 2. c4 e6 3. g3 d5 4. Bg2 Be7 5. d4 O-O 6. Qc2 c5 7. O-O cxd4 8. Nxd4 e5 9. Nf5 d4 10. Nxe7+ Qxe7 11. Bg5 h6 12. Bxf6 Qxf6 13. Nd2 Bf5 14. Qb3 Nd7 15. Qa3 Qb6 16. Rfc1 Rfc8 17. b4 a5 18. c5 Qa6 19. Nc4 Be6 20. Nd6 axb4";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. e4 c5 2. Nf3 d6 3. Bb5+ Nc6 4. O-O Bd7 5. Re1 Nf6 6. h3 a6 7. Bf1 g5 8. d4 g4 9. d5 gxf3 10. dxc6 Bxc6 11. Qxf3 Nd7";
  add_pgn_to_book(&CHESS_BOOK,pgn);

  // Desprez
  let desprez = "1. h4 d5 2. e3 c5 3. Nf3 Nc6 4. Bb5 Qc7 5. O-O a6 6. Bxc6+ Qxc6 7. Ne5 Qc7 8. d4 e6 9. c4 dxc4 10. a4 b6 11. Nd2 Bb7 12. Ndxc4 Rd8 13. b3 Nf6";
  add_pgn_to_book(&CHESS_BOOK, desprez);
  let desprez = "1. h4 d5 2. d4 c5 3. e3 Nc6 4. c4 e6 5. Nf3 Nf6 6. a3 a6 7. dxc5 Bxc5 8. b4 Be7 9. Bb2 dxc4 10. Qxd8+ Bxd8 11. Bxc4 b5 12. Bd3 O-O 13. Nbd2 Be7 14. Ke2 Bb7 15. Rhc1 Rfc8 16. Nb3 Nd7 17. Rc2 Nd8 18. Rac1 Rxc2+ 19. Rxc2 Rc8 20. Rxc8 Bxc8 21. g4";
  add_pgn_to_book(&CHESS_BOOK, desprez);
  let desprez = "1. h4 e5 2. c4 Nf6 3. Nc3 d5 4. cxd5 Nxd5 5. e3 Nxc3 6. bxc3 Bd6 7. Nf3 O-O 8. d4 Nc6 9. Rb1 Qe7 10. Bc4 Bf5 11. Rb2 h6 12. Be2 b6 13. Nd2";
  add_pgn_to_book(&CHESS_BOOK, desprez);

  // More random games
  let pgn = "1. f4 d5 2. Nf3 g6 3. g3 Bg7 4. Bg2 Nf6 5. O-O O-O 6. d3 c5 7. c3 Nc6 8. Na3 Rb8 9. Ne5 Qc7 10. Qa4 a6 11. Nxc6 bxc6 12. e4 Bd7 13. Nc2 h5 14. Ne3 h4 15. Qd1 Nh5 16. Bf3 hxg3 17. Bxh5 gxh5 18. Qxh5 c4 19. Rf3 gxh2+ 20. Kxh2";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. e4 c5 2. Nc3 a6 3. Nge2 Nf6 4. e5 Ng4 5. f4 e6 6. d4 d5 7. exd6 Bxd6 8. h3 Nf6 9. dxc5 Bxc5 10. Qxd8+ Kxd8 11. g4 Bd7 12. Bd2 Bc6 13. Rh2 Kc7 14. O-O-O Nbd7 15. Bg2 Nb6 16. Bxc6 bxc6 17. Ng3";
  add_pgn_to_book(&CHESS_BOOK,pgn);

  // Aggressive games
  // Fried liver:
  let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bc4 Nf6 4. Ng5 d5 5. exd5 Nxd5 6. Nxf7 Kxf7 7. Qf3+ Ke6 8. Nc3 Nb4 9. O-O c6 10. d4 Kd7 11. a3";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  add_single_move_to_book(&CHESS_BOOK, "r1bq1b1r/ppp1k1pp/2n5/3np3/2B5/5Q2/PPPP1PPP/RNB1K2R w KQ - 2 8", "c4d5");
  add_single_move_to_book(&CHESS_BOOK, "r1b2b1r/ppp2kpp/2n2q2/3np3/2B5/5Q2/PPPP1PPP/RNB1K2R w KQ - 2 8", "c4d5");
  let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bc4 Nf6 4. Ng5 d5 5. exd5 Na5 6. Bb5+ c6 7. dxc6 bxc6 8. Qf3 Be7 9. Bd3";
  add_pgn_to_book(&CHESS_BOOK,pgn);

  // Vienna game
  let pgn = "1. e4 e5 2. Nc3 Nf6 3. Bc4 Nxe4 4. Qh5 Nd6 5. Bb3 Nc6 6. Nb5 g6 7. Qf3 f5 8. Qd5 Qf6 9. Nxc7+ Kd8 10. Nxa8";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. e4 e5 2. Nc3 Nf6 3. Bc4 Nxe4 4. Qh5 Nd6 5. Bb3 Be7 6. Qxe5 O-O 7. Nf3 Nc6 8. Qf4 Na5 9. O-O";
  add_pgn_to_book(&CHESS_BOOK,pgn);
  let pgn = "1. e4 e5 2. Nc3 Nf6 3. Bc4 Nxe4 4. Nxe4 d5 5. Bd3 dxe4 6. Bxe4 f5 7. Bf3 Bd6 8. d3 O-O 9. Ne2 c6 10. O-O";
  add_pgn_to_book(&CHESS_BOOK,pgn);

  // Greek gift:
  add_pgn_from_position(&CHESS_BOOK, "rnbq1rk1/pppn1ppp/4p3/3pP3/1b1P4/2NB1N2/PPP2PPP/R1BQK2R w KQ - 5 7", "7. Bxh7+ Kxh7 8. Ng5+ Kg8 9. Qd3 Qxg5 10. Bxg5 Nc6 11. a3 Be7 12. h4");
  add_pgn_from_position(&CHESS_BOOK, "rnbq1rk1/pppn1ppp/4p3/3pP3/1b1P4/2NB1N2/PPP2PPP/R1BQK2R w KQ - 5 7", "7. Bxh7+ Kh8 8. Ng5 g6 9. Qd3 Nc6 10. Bxg6 fxg6 11. Qh3+ Kg7 12. Qh7#");
  add_pgn_from_position(&CHESS_BOOK, "rnbq1rk1/pppn1ppp/4p3/3pP3/1b1P4/2NB1N2/PPP2PPP/R1BQK2R w KQ - 5 7", "7. Bxh7+ Kxh7 8. Ng5+ Kg6 9. h4 f5 10. h5+ Kh6 11. Nxe6+ Kh7 12. Nxd8");
  add_pgn_from_position(&CHESS_BOOK, "rnbq1rk1/pppn1ppp/4p3/3pP3/1b1P4/2NB1N2/PPP2PPP/R1BQK2R w KQ - 5 7", "1. Bxh7+ Kh8 2. Ng5 g6 3. Bxg6 Kg7 4. Nxf7 Rxf7 5. Bxf7 Kxf7 6. Qh5+");
  

  // Traps:
  add_single_move_to_book(&CHESS_BOOK, "rnbqkbnr/pppp2pp/5p2/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 3", "f3e5");
  add_single_move_to_book(&CHESS_BOOK, "rnbqkbnr/pppp2pp/8/4p3/4P3/8/PPPP1PPP/RNBQKB1R w KQkq - 0 4", "d1h5");
  add_single_move_to_book(&CHESS_BOOK, "rnbqkbnr/pppp3p/6p1/4p2Q/4P3/8/PPPP1PPP/RNB1KB1R w KQkq - 0 5", "h5e5");

  add_single_move_to_book(&CHESS_BOOK, "rnbqkb1r/pppp1ppp/5n2/4N3/8/8/PPPPQPPP/RNB1KB1R w KQkq - 2 5", "e5c6");
  add_single_move_to_book(&CHESS_BOOK, "r1bqk2r/pppp1ppp/2n5/8/2BP4/2b2N2/P4PPP/R1BQ1RK1 w kq - 0 10", "c1a3");
  add_single_move_to_book(&CHESS_BOOK, "r1bqk2r/pppp1ppp/2n5/8/2BP4/B4N2/P4PPP/b2Q1RK1 w kq - 0 11", "f1e1");
  add_single_move_to_book(&CHESS_BOOK, "r1bqk2r/ppppnppp/8/8/2BP4/B4N2/P4PPP/b2QR1K1 w kq - 2 12", "a3e7");

  add_single_move_to_book(&CHESS_BOOK, "rnbqk1nr/ppp2ppp/8/4P3/1BP5/8/PP2KpPP/RN1Q1BNR b kq - 1 7", "f2g1n");
  add_single_move_to_book(&CHESS_BOOK, "rnbqk1nr/ppp2ppp/8/4P3/1BP5/8/PP2K1PP/RN1Q1BR1 b kq - 0 8", "c8d4");
  

  add_single_move_to_book(&CHESS_BOOK, "r1b1kbnr/pp3ppp/1q2p3/3pP3/3n4/3B1N2/PP3PPP/RNBQK2R w KQkq - 0 8", "f3d4");
  add_single_move_to_book(&CHESS_BOOK, "r1b1kbnr/pp3ppp/4p3/3pP3/3q4/3B4/PP3PPP/RNBQK2R w KQkq - 0 9", "d3b5");
  add_single_move_to_book(&CHESS_BOOK, "r3kbnr/pp1b1ppp/4p3/1B1pP3/3q4/8/PP3PPP/RNBQK2R w KQkq - 2 10", "b5d7");
  add_single_move_to_book(&CHESS_BOOK, "r4bnr/pp1k1ppp/4p3/3pP3/3q4/8/PP3PPP/RNBQK2R w KQ - 0 11", "d1d4");

  add_single_move_to_book(&CHESS_BOOK, "rnbqkb1r/ppp1ppp1/5n1p/6N1/8/3B4/PPP2PPP/RNBQK2R w KQkq - 0 6", "g5f7");
  add_single_move_to_book(&CHESS_BOOK, "rnbq1b1r/ppp1pkp1/5n1p/8/8/3B4/PPP2PPP/RNBQK2R w KQ - 0 7", "d3g6");
  add_single_move_to_book(&CHESS_BOOK, "rnbq1b1r/ppp1p1p1/5nkp/8/8/8/PPP2PPP/RNBQK2R w KQ - 0 8", "d1d8");

  add_single_move_to_book(&CHESS_BOOK, "rnb1kbnr/pppp1ppp/8/8/3PPp1q/6P1/PPP4P/RNBQKBNR b KQkq - 0 4", "f4g3");
  add_single_move_to_book(&CHESS_BOOK, "rnb1kbnr/pppp1ppp/8/8/3PP2q/5Np1/PPP4P/RNBQKB1R b KQkq - 1 5", "g3g2");
  add_single_move_to_book(&CHESS_BOOK, "rnb1kbnr/pppp1ppp/8/8/3PP2N/8/PPP3pP/RNBQKB1R b KQkq - 0 6", "g2h1q");

  add_single_move_to_book(&CHESS_BOOK, "r1bqkb1r/pppp1ppp/2n5/1B2p3/4P1n1/5N1P/PPPP1PP1/RNBQ1RK1 b kq - 0 5", "h7h5");
  add_single_move_to_book(&CHESS_BOOK, "r1bqkb1r/pppp1pp1/2n5/1B2p2p/4P1P1/5N2/PPPP1PP1/RNBQ1RK1 b kq - 0 6", "h5g4");
  add_single_move_to_book(&CHESS_BOOK, "r1bqkb1r/pppp1pp1/2n5/1B2p3/4P1p1/8/PPPP1PPN/RNBQ1RK1 b kq - 1 7", "d8h4");

  add_pgn_from_position(&CHESS_BOOK, "r1b1kb1r/pppp1pp1/2n5/1B2p3/4P1pq/8/PPPP1PPN/RNBQ1RK1 w kq - 2 8","8. f4 g3 9. Rf3 Qxh2+ 10. Kf1 Nd4 11. Ke1 Qxg2 12. Rf1 Rh1 13. Qe2 Nxe2 14. Rxh1 Nd4 15. Bd3 Qxh1+ 16. Bf1 Qxe4+ 17. Kd1 Qxc2+ 18. Ke1 Qxc1#");


  add_single_move_to_book(&CHESS_BOOK, "rnbqkb1r/pppp1ppp/8/4P3/6n1/7P/PPPNPPP1/R1BQKBNR b KQkq - 0 4", "g4e3");
  add_single_move_to_book(&CHESS_BOOK, "rnbqkb1r/pppp1ppp/8/4P3/8/4P2P/PPPNP1P1/R1BQKBNR b KQkq - 0 5", "d8h4");


  add_single_move_to_book(&CHESS_BOOK, "r1b1k2r/ppppqppp/2n5/4P3/1bP2Bn1/P4N2/1P1NPPPP/R2QKB1R b KQkq - 0 7", "g4e5");
  add_single_move_to_book(&CHESS_BOOK, "rnbqk2r/ppp2ppp/3b4/8/2P3n1/5N1P/PP2PPP1/RNBQKB1R b KQkq - 0 6", "g4f2");
  add_single_move_to_book(&CHESS_BOOK, "rnbqk2r/ppp2ppp/3b4/8/2P5/5N1P/PP2PKP1/RNBQ1B1R b kq - 0 7", "d6g3");

  add_single_move_to_book(&CHESS_BOOK, "rnbqk1nr/ppp2p1p/3b4/6p1/4P3/5N2/PPPP2PP/RNBQKB1R b KQkq - 0 5", "g5g4");
  add_single_move_to_book(&CHESS_BOOK, "rnbqk1nr/ppp2p1p/3b4/4P3/6p1/5N2/PPPP2PP/RNBQKB1R b KQkq - 0 6", "g5f3");
  add_single_move_to_book(&CHESS_BOOK, "rnbqk1nr/ppp2p1p/3P4/8/8/5p2/PPPP2PP/RNBQKB1R b KQkq - 0 7", "d8e4");
  add_single_move_to_book(&CHESS_BOOK, "rnb1k1nr/ppp2p1p/3P4/8/7q/5pP1/PPPP3P/RNBQKB1R b KQkq - 0 8", "h4e4");

  add_single_move_to_book(&CHESS_BOOK, "r1bqkbnr/pppp1ppp/8/4N3/2BnP3/8/PPPP1PPP/RNBQK2R b KQkq - 0 4", "d8g5");
  add_single_move_to_book(&CHESS_BOOK, "r1b1kbnr/pppp1Npp/8/6q1/2BnP3/8/PPPP1PPP/RNBQK2R b KQkq - 0 5", "g5g2");
  add_single_move_to_book(&CHESS_BOOK, "r1b1kbnr/pppp1Npp/8/8/2BnP3/8/PPPP1PqP/RNBQKR2 b Qkq - 1 6", "g2e4");
  add_pgn_from_position(&CHESS_BOOK, "r1b1kbnr/pppp1ppp/8/4N1q1/2BnP3/8/PPPP1PPP/RNBQK2R w KQkq - 1 5", "5. Bxf7+ Kd8 6. O-O Qxe5 7. c3 Ne6 8. d3 g5 9. Nd2");

}

/// Check our known book moves, known positions that have been computed with an
/// evaluation before, so that we do not need to find moves ourselves.
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
