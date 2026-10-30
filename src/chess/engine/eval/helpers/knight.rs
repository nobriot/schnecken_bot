use super::generic::threatened_victims;
use crate::model::board_geometry::BOARD_EDGES;
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
    knights &= knights - 1;

    let attacked_pieces = match color {
      Color::White => (KNIGHT_MOVES[knight as usize]
                       & (game_state.board.pieces.black.majors()
                          | game_state.board.pieces.black.king))
                                                                .count_few_ones(),
      Color::Black => (KNIGHT_MOVES[knight as usize]
                       & (game_state.board.pieces.white.majors()
                          | game_state.board.pieces.white.king))
                                                                .count_few_ones(),
    };
    // Most knights attack nothing: skip the expensive attackers lookups.
    if attacked_pieces == 0 {
      continue;
    }

    // Check that the knight can be taken down too easily
    let defenders = game_state.board.get_attackers(knight, color);
    let attackers = game_state.board.get_attackers(knight, Color::opposite(color));
    if attackers.count_ones() <= defenders.count_ones() {
      victims += threatened_victims(attacked_pieces, game_state.board.side_to_play == color);
    }
  }

  victims
}

/// Counts the knights on the edge of the board that have no safe square to go
/// to, i.e. every destination is either occupied by our own pieces, attacked
/// by an enemy pawn, or attacked by the enemy and not defended by us.
/// Such knights are likely to be lost (e.g. a knight grabbing the a2 pawn).
///
/// ### Argument
/// * `game_state`: A GameState object representing a position, side to play, etc.
/// * `color`     : Color of the knights to look at
///
/// ### Returns
///
/// Number of trapped knights on the edge of the board.
pub fn get_trapped_knights(game_state: &GameState, color: Color) -> u32 {
  let board = &game_state.board;
  let (mut knights, own_pieces, enemy_pawns) = match color {
    Color::White => {
      (board.pieces.white.knight & BOARD_EDGES, board.pieces.white.all(), board.pieces.black.pawn)
    },
    Color::Black => {
      (board.pieces.black.knight & BOARD_EDGES, board.pieces.black.all(), board.pieces.white.pawn)
    },
  };

  let mut trapped: u32 = 0;
  while knights != 0 {
    let knight = knights.trailing_zeros() as u8;
    let mut destinations = KNIGHT_MOVES[knight as usize] & !own_pieces;
    // We only need to know if there is at least one safe square, and the
    // defenders are only needed when the attackers do not decide it already.
    let mut has_safe_square = false;
    while destinations != 0 {
      let destination = destinations.trailing_zeros() as u8;
      destinations &= destinations - 1;
      let attackers = board.get_attackers(destination, Color::opposite(color));
      if attackers & enemy_pawns != 0 {
        continue;
      }
      if attackers == 0 || board.get_attackers(destination, color) & !(1 << knight) != 0 {
        has_safe_square = true;
        break;
      }
    }
    if !has_safe_square {
      trapped += 1;
    }
    knights &= knights - 1;
  }

  trapped
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_trapped_knights() {
    // https://lichess.org/kzc4RftG : 16...Nxa2, the knight cannot come back
    let fen = "4rrk1/pp3pp1/7p/1N1p1n2/3P4/2P4P/nP3PP1/3R1KNR w - - 0 17";
    let game_state = GameState::from_fen(fen);
    assert_eq!(1, get_trapped_knights(&game_state, Color::Black));
    assert_eq!(0, get_trapped_knights(&game_state, Color::White));

    // Before taking on a2, the knight on b4 is fine
    let fen = "4rrk1/pp3pp1/7p/1N1p1n2/1n1P4/2P4P/PP3PP1/3R1KNR b - - 0 16";
    let game_state = GameState::from_fen(fen);
    assert_eq!(0, get_trapped_knights(&game_state, Color::Black));

    // Knights at home in the start position are not trapped
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let game_state = GameState::from_fen(fen);
    assert_eq!(0, get_trapped_knights(&game_state, Color::White));
    assert_eq!(0, get_trapped_knights(&game_state, Color::Black));
  }

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
