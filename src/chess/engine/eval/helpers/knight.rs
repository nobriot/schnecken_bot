use crate::model::game_state::*;
use crate::model::piece::*;
use crate::model::piece_moves::*;

use super::generic::is_attacked;
use super::generic::is_hanging;

/// Computes the values of the pieces that a knight attacks.
///
/// ### Argument
/// * `game_state`: A GameState object representing a position, side to play, etc.
/// * `i`         : Index of the square on the board
///
/// ### Returns
///
/// Zero if there is no bishop on the square
/// The value of attacked enemy pieces if it attacks them.
/// Puts a value of 1.0 for a king check, in case we're forking king + something else.
///
pub fn knight_attack(game_state: &GameState, i: usize) -> f32 {
  let mut value: f32 = 0.0;

  // If we have no knight on the square, return immediately.
  let color = if game_state.board.squares[i] == WHITE_KNIGHT {
    Color::White
  } else if game_state.board.squares[i] == BLACK_KNIGHT {
    Color::Black
  } else {
    return value;
  };

  // If the knight is attacked by the opponent and not defended, we do not even
  // consider anything here:
  if is_hanging(game_state, i) && is_attacked(game_state, i) {
    return value;
  }

  let destinations = get_knight_moves(0, 0, i);

  // Get the knight destinations
  for s in 0..64 {
    if destinations & (1 << s) == 0 {
      continue;
    }
    if game_state
      .board
      .has_piece_with_color(s as u8, Color::opposite(color))
    {
      if !game_state.board.has_king(s) {
        value += Piece::material_value_from_u8(game_state.board.squares[s]);
      } else {
        value += 1.0 * Color::score_factor(Color::opposite(color));
      }
    }
  }

  value.abs()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_knight_attack() {
    let fen = "rq3b1r/pp1nkp2/2n1p2p/2pp3p/Q4P2/P1PPPb2/1P1N2P1/R1B1KBR1 w Q - 0 17";
    let game_state = GameState::from_fen(fen);
    assert_eq!(0.0, knight_attack(&game_state, 0));
    assert_eq!(0.0, knight_attack(&game_state, 1));
    assert_eq!(3.05, knight_attack(&game_state, 11));

    let fen = "1r3b2/ppqnkpr1/2n4p/2ppp2p/Q1P2P2/P1NPP3/1P4P1/R1B1KBR1 w Q - 0 22";
    let game_state = GameState::from_fen(fen);
    assert_eq!(0.0, knight_attack(&game_state, 0));
    assert_eq!(0.0, knight_attack(&game_state, 1));
    assert_eq!(1.0, knight_attack(&game_state, 18));

    let fen = "1r3b2/ppqnkpr1/2n4p/2pNp2p/Q1P2P2/P2PP3/1P4P1/R1B1KBR1 b Q - 0 22";
    let game_state = GameState::from_fen(fen);
    assert_eq!(0.0, knight_attack(&game_state, 0));
    assert_eq!(0.0, knight_attack(&game_state, 1));
    assert_eq!(10.0, knight_attack(&game_state, 35));

    let fen = "2kr1b1r/ppp2ppp/2nqp3/6P1/4B3/2nP1N1P/PPP1PP2/R1BQK2R w KQ - 2 13";
    let game_state = GameState::from_fen(fen);
    for i in 0..64 {
      assert_eq!(0.0, knight_attack(&game_state, i));
    }
  }
}
