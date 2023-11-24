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

  // Bongloud with other responses than e5:
  let pgn = "1. e4 f5 2. Ke2 fxe4 3. d3 exd3+ 4. Qxd3 Nf6 5. Nf3 d5 6. Nc3 e6 7. Bg5 Be7 8. Re1 c5 9. Kd1 Nc6 10. Qd2 c4 11. Bxf6 gxf6 12. Kc1 Qa5 13. g3 b5 ";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, pgn);
  let pgn = "1. e4 c5 2. Ke2 Nf6 3. Nc3 Nc6 4. Ke1 g6 5. f4 d6 6. Nf3 Bg7 7. Bb5 O-O 8. Bxc6 bxc6 9. d3 Rb8 10. h3 c4 11. Kf2 Rxb2 12. Bxb2 Qb6+ 13. Nd4 Qxd4+ 14. Kf3 e5 15. g3";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, pgn);
  let pgn = "1. e4 e6 2. Ke2 d5 3. d3 Nc6 4. Nf3 Nf6 5. e5 Nd7 6. d4 f6 7. exf6 Qxf6 8. Be3 e5 9. Nc3 e4 10. Ne5 Ne7 11. Ng4 Qg6 12. f3 h5 13. Nf2 Nb6 14. Qe1 exf3+ 15. gxf3 Qxc2+ 16. Qd2";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, pgn);
  let pgn = "1. e4 g6 2. Ke2 Bg7 3. Nc3 e5 4. d3 Nf6 5. Bg5 c6 6. g3 d5 7. Bg2 Bg4+ 8. f3 Bd7 9. Ke1 d4 10. Nce2 Qb6 11. b3 a5 12. h4 h5 13. Bh3 a4 14. Kf2 O-O";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, pgn);
  let pgn = "1. e4 b6 2. Ke2 e5 3. d3 d5 4. Nf3 Nc6 5. Nc3 d4 6. Nd5 Nge7 7. Nxe7 Bxe7 8. a3 h6 9. Ke1 Be6 10. Be2 Qd7 11. h4 a5 12. Bd2 O-O";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, pgn);

  // Double bongcloud
  let pgn = "1. e4 e5 2. Ke2 Ke7 3. Nf3 d5 4. d4 dxe4 5. Nxe5 Nd7 6. Nc4 Nb6 7. Ne3 f5 8. c4 Nf6 9. Nc3 Kf7 10. g3 c5 11. d5 g5 12. Qc2 f4 13. h4 fxe3 14. hxg5 Ng4 15. Nxe4 exf2 16. Bf4";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, pgn);

  // Wayward Queen attack:
  let wayward = "1. e4 e5 2. Qh5 d6 3. Bc4 g6 4. Qd1 Nf6 5. d3 c6 6. Nf3 Bg7 7. O-O b5 8. Bb3 O-O 9. a4 b4 10. Be3 a5 11. Nbd2 Nbd7 12. c3 bxc3 13. bxc3 Qc7 14. Re1 d5 15. Qc2 Re8";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, wayward);
  let wayward = "1. e4 e5 2. Qh5 Nf6 3. Qxe5+ Be7 4. Qf4 Nc6 5. d3 O-O 6. Nf3 d5 7. Be2 Bb4+ 8. Bd2 Re8 9. Nc3 dxe4 10. dxe4 Bxc3 11. Bxc3 Nxe4 12. Rd1 Qe7 13. O-O Nxc3 14. bxc3";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, wayward);
  let wayward = "1. e4 e5 2. Qh5 Nf6 3. Qxe5+ Be7 4. Qf4 Nc6 5. d3 O-O 6. Nf3 d5 7. Be2 Bb4+ 8. Bd2 Re8 9. Nc3 Bd6 10. Qh4 Be7 11. exd5 Nxd5 12. Qa4 Nxc3 13. Bxc3 Bd6 14. Kf1 ";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, wayward);
  let wayward = "1. e4 e5 2. Qh5 Nc6 3. Bc4 g6 4. Qf3 Nf6 5. Ne2 Bg7 6. d3 O-O 7. a3 d6 8. Nbc3 Bg4 9. Qg3 Be6 10. Bg5 Nh5 11. Qe3 Nd4 12. Qd2 Qe8";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, wayward);
  let pgn = "1. e4 e5 2. Qh5 Nc6 3. Bc4 g6 4. Qf3 Nf6 5. Ne2 Bg7 6. d3 O-O 7. a3 d6 8. Nbc3 h6 9. O-O Kh7 10. Bd2";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, pgn);
  let pgn = "1. e4 e5 2. Qh5 d6 3. Bc4 g6 4. Qd1 Nf6 5. d3 c6 6. Nf3 Bg7 7. O-O b5 8. Bb3 O-O 9. Be3";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, pgn);
  

  // Weirdo provocative line:
  let pgn = "1. e4 f6 2. Qh5+ g6 3. Be2";
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

  // Wing gambit
  let pgn = "1. e4 c5 2. b4 cxb4 3. a3 e5 4. Bb2 Nc6 5. Nf3 Qb6 6. Nxe5 bxa3 7. Bc3 a2 8. Nxf7 Kxf7 9. Bc4+ Ke7 10. Rxa2 Nf6 11. O-O";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, pgn);
  let pgn = "1. e4 c5 2. b4 d5 3. exd5 Qxd5 4. Nf3 cxb4 5. a3 e6 6. Bb2 Nf6 7. Be2 Qd8 8. O-O a5 9. Ne5 Nbd7 10. Bf3 Qc7 11. Qe2 Ra6 12. axb4 axb4 13. Nxd7 Nxd7 14. d3 Bd6 15. Bxg7 Rg8 16. Bb2 Bxh2+ 17. Kh1 Bf4 18. Re1 Rg5 19. Qe4 Rc5 20. Rxa6 bxa6 21. Qxb4 Bd6";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, pgn);
  let pgn = "1. e4 c5 2. b4 cxb4 3. d4 d5 4. exd5 Nf6 5. a3 Nxd5 6. axb4 Nxb4 7. Nf3 e6 8. Be2 Be7 9. c3 N4c6 10. Bf4 Nd7 11. O-O O-O 12. Na3 Nf6 13. c4 a6 14. Nb5 Bd7 15. Nc3 Nb4 16. Ne5 Bc6";
  add_pgn_to_book(&PROVOCATIVE_CHESS_BOOK, pgn);

  // Vienna game, black bongcloud
  let pgn = "1. e4 e5 2. Nc3 Ke7 3. f4 d6 4. d4 Ke8 5. Nf3 exd4 6. Qxd4 Nc6 7. Qf2 g6 8. Be3 Bg7 9. e5 Nh6 10. h3 Nf5 11. O-O-O Nxe3 12. Qxe3 Kf8 13. Bc4 Na5 14. Be2 Be6 15. Rhe1";
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
