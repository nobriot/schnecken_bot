use crate::model::board_mask::*;
use crate::model::game_state::*;
use crate::model::piece::*;
use crate::model::piece_moves::*;

/// Computes the number of major / king pieces that a knight attacks.
///
/// ### Argument
/// * `game_state`: A GameState object representing a position, side to play, etc.
/// * `i`         : Index of the square on the board
///
/// ### Returns
///
/// Number of majors/king pieces attacked by a knight.
///
#[inline]
pub fn get_knight_victims(game_state: &GameState, color: Color) -> u32 {
  let mut victims: u32 = 0;
  // Look for knights attacking pieces, or forking
  let mut knights = match color {
    Color::White => game_state.board.pieces.white.knight,
    Color::Black => game_state.board.pieces.black.knight,
  };
  while knights != 0 {
    let knight = knights.trailing_zeros() as u8;
    let defenders = game_state.board.get_attackers(knight, color);
    let attackers = game_state.board.get_attackers(knight, Color::opposite(color));

    // Check that the knight can be taken down too easily
    if attackers.count_ones() <= defenders.count_ones() {
      let attacked_pieces = match color {
        Color::White => (KNIGHT_MOVES[knight as usize]
          & (game_state.board.pieces.black.majors() | game_state.board.pieces.black.king))
          .count_few_ones(),
        Color::Black => (KNIGHT_MOVES[knight as usize]
          & (game_state.board.pieces.white.majors() | game_state.board.pieces.white.king))
          .count_few_ones(),
      };

      victims += attacked_pieces;
    }

    knights &= knights - 1;
  }

  victims
}

#[cfg(test)]
mod tests {
  use super::*;

  #[ignore]
  #[test]
  fn test_knight_attack() {
    let fen = "rq3b1r/pp1nkp2/2n1p2p/2pp3p/Q4P2/P1PPPb2/1P1N2P1/R1B1KBR1 w Q - 0 17";
    let game_state = GameState::from_fen(fen);
    assert_eq!(1, get_knight_victims(&game_state, Color::White));
    assert_eq!(0, get_knight_victims(&game_state, Color::Black));

    let fen = "1r3b2/ppqnkpr1/2n4p/2ppp2p/Q1P2P2/P1NPP3/1P4P1/R1B1KBR1 w Q - 0 22";
    let game_state = GameState::from_fen(fen);
    assert_eq!(0, get_knight_victims(&game_state, Color::White));
    assert_eq!(0, get_knight_victims(&game_state, Color::Black));

    let fen = "1r3b2/ppqnkpr1/2n4p/2pNp2p/Q1P2P2/P2PP3/1P4P1/R1B1KBR1 b Q - 0 22";
    let game_state = GameState::from_fen(fen);
    assert_eq!(2, get_knight_victims(&game_state, Color::White));
    assert_eq!(0, get_knight_victims(&game_state, Color::Black));

    let fen = "2kr1b1r/ppp2ppp/2nqp3/6P1/4B3/2nP1N1P/PPP1PP2/R1BQK2R w KQ - 2 13";
    let game_state = GameState::from_fen(fen);
    assert_eq!(0, get_knight_victims(&game_state, Color::White));
    assert_eq!(2, get_knight_victims(&game_state, Color::Black));
  }
}
