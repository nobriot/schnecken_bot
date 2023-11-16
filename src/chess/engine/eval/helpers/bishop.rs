use crate::model::board_mask::*;
use crate::model::game_state::*;
use crate::model::piece::*;
use crate::model::tables::bishop_destinations::*;

/// Computes the number of pieces attacked by defended bishops
/// It will count as if the bishop can go through enemy rooks and queens.
///
/// ### Arguments
///
/// * `game_state` :  Game to look at
/// * `color` :       Color of the bishops to look at
///  
/// ### Return value
///
/// Number of majors/kings attacked by bishops.
///
#[inline]
pub fn get_bishop_victims(game_state: &GameState, color: Color) -> u32 {
  let mut victims: u32 = 0;
  // Look for bishops attacking major pieces, either forking or skewering
  let (mut bishops, op) = match color {
    Color::White => (
      game_state.board.pieces.white.bishop,
      (game_state.board.pieces.black.all()
        & !game_state.board.pieces.black.majors()
        & !game_state.board.pieces.black.king),
    ),
    Color::Black => (
      game_state.board.pieces.black.bishop,
      (game_state.board.pieces.white.all()
        & !game_state.board.pieces.white.majors()
        & !game_state.board.pieces.white.king),
    ),
  };

  while bishops != 0 {
    let bishop = bishops.trailing_zeros() as u8;
    let defenders = game_state.board.get_attackers(bishop, color);
    let attackers = game_state.board.get_attackers(bishop, Color::opposite(color));

    // Check that the pawn cannot be taken out too easily before assigning a bonus for the bishop attack.
    if attackers.count_ones() <= defenders.count_ones() {
      let attacked_pieces = match color {
        Color::White => {
          let destinations =
            get_bishop_destinations(game_state.board.pieces.white.all(), op, bishop as usize);
          (destinations
            & (game_state.board.pieces.black.majors() | game_state.board.pieces.black.king))
            .count_few_ones()
        },
        Color::Black => {
          let destinations =
            get_bishop_destinations(game_state.board.pieces.black.all(), op, bishop as usize);
          (destinations
            & (game_state.board.pieces.white.majors() | game_state.board.pieces.white.king))
            .count_few_ones()
        },
      };

      victims += attacked_pieces;
    }

    bishops &= bishops - 1;
  }

  victims
}

#[cfg(test)]
mod tests {
  use super::*;

  #[ignore]
  #[test]
  fn test_bishop_attack() {
    let fen = "8/2q3r1/8/4B3/8/2k3n1/8/1K2R3 w - - 0 1";
    let game_state = GameState::from_fen(fen);
    assert_eq!(3, get_bishop_victims(&game_state, Color::White));
    assert_eq!(0, get_bishop_victims(&game_state, Color::Black));

    // We do not like it particularly if the bishop is under attack and un-defended
    let fen = "8/2q3r1/8/4B3/8/2k3n1/8/1K6 w - - 0 1";
    let game_state = GameState::from_fen(fen);
    assert_eq!(0, get_bishop_victims(&game_state, Color::White));
    assert_eq!(0, get_bishop_victims(&game_state, Color::Black));

    // Bishop can x-ray through majors
    let fen = "8/8/5B2/4r3/3q4/2k3n1/8/1K6 w - - 0 1";
    let game_state = GameState::from_fen(fen);
    assert_eq!(3, get_bishop_victims(&game_state, Color::White));
    assert_eq!(0, get_bishop_victims(&game_state, Color::Black));

    // Bishop cannot x-ray through minors
    let fen = "8/6B1/5n2/4r3/3q4/2k5/8/1K6 w - - 0 1";
    let game_state = GameState::from_fen(fen);
    assert_eq!(0, get_bishop_victims(&game_state, Color::White));
    assert_eq!(0, get_bishop_victims(&game_state, Color::Black));

    let fen = "4r3/6R1/8/4b3/3Q4/2K3N1/8/1k6 w - - 0 1";
    let game_state = GameState::from_fen(fen);
    assert_eq!(0, get_bishop_victims(&game_state, Color::White));
    assert_eq!(3, get_bishop_victims(&game_state, Color::Black));

    let fen = "8/6R1/5Q2/3K4/8/6N1/1b6/1k6 w - - 0 1";
    let game_state = GameState::from_fen(fen);
    assert_eq!(0, get_bishop_victims(&game_state, Color::White));
    assert_eq!(2, get_bishop_victims(&game_state, Color::Black));
  }
}
