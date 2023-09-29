use crate::model::board_mask::*;
use crate::model::game_state::*;
use crate::model::piece::*;
use crate::model::tables::pawn_destinations::*;

/// Mobility area
///
/// This is the squares where we can navigate to with out pieces "safely"
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine piece mobility
///
/// # Return value
///
/// Board mask with squares that can be used by a color for their pieces.
///
pub fn get_mobility_area(game_state: &GameState, color: Color) -> BoardMask {
  // We start assuming all squares are available, and substract our pawns,
  // king and the squares attacked by enemy pawns.
  let mut mobility_area: BoardMask = u64::MAX;

  let mut op_pawns = match color {
    Color::White => game_state.board.pieces.black.pawn,
    Color::Black => game_state.board.pieces.white.pawn,
  };
  let pieces = match color {
    Color::White => game_state.board.pieces.white,
    Color::Black => game_state.board.pieces.black,
  };

  let pawn_control = match color {
    Color::White => &WHITE_PAWN_CONTROL,
    Color::Black => &BLACK_PAWN_CONTROL,
  };

  while op_pawns != 0 {
    let pawn = op_pawns.trailing_zeros() as usize;
    mobility_area &= !(pawn_control[pawn]);
    op_pawns &= op_pawns - 1;
  }

  mobility_area &= !(pieces.king | pieces.queen | pieces.pawn);

  mobility_area
}

/// Determines the number of available safe squares for each piece
/// FIXME: Implement handling pins
///
/// # Arguments
///
/// * `game_state` - A GameState object representing a position, side to play, etc.
/// * `color` -      The color for which we want to determine piece mobility
///
/// # Return value
///
/// Number of square that a color's piece can occupy.
pub fn get_piece_mobility(game_state: &GameState, color: Color) -> usize {
  let mut mobility: usize = 0;

  let mobility_area = get_mobility_area(game_state, color);
  let pieces = match color {
    Color::White => game_state.board.pieces.white,
    Color::Black => game_state.board.pieces.black,
  };

  mobility
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  #[ignore]
  fn test_mobility_area() {
    let fen = "rnbqk1nr/pppppppp/8/8/1b6/3P4/PPPQPPPP/RNBK1BNR b KQkq - 0 1";
    let game_state = GameState::from_fen(fen);
    assert_eq!(
      46,
      get_mobility_area(&game_state, Color::White).count_ones()
    );
    assert_eq!(
      44,
      get_mobility_area(&game_state, Color::Black).count_ones()
    );

    let fen = "r1bq2nr/pppppppp/P7/2kn1P2/1b6/3P1N1P/1PPQP1P1/RNBK1B1R b KQkq - 0 1";
    let game_state = GameState::from_fen(fen);
    //print_board_mask(get_mobility_area(&game_state, Color::White));
    assert_eq!(
      47,
      get_mobility_area(&game_state, Color::White).count_ones()
    );
    assert_eq!(
      43,
      get_mobility_area(&game_state, Color::Black).count_ones()
    );
  }

  #[ignore]
  #[test]
  fn test_piece_mobility() {
    let fen = "rnbqk1nr/pppppppp/8/8/1b6/3P4/PPPQPPPP/RNBK1BNR b KQkq - 0 1";
    let game_state = GameState::from_fen(fen);
    assert_eq!(10, get_piece_mobility(&game_state, Color::White));
    assert_eq!(8, get_piece_mobility(&game_state, Color::Black));

    let fen = "r1bq2nr/pppppppp/P7/2kn1P2/1b6/3P1N1P/1PPQP1P1/RNBK1B1R b KQkq - 0 1";
    let game_state = GameState::from_fen(fen);
    assert_eq!(21, get_piece_mobility(&game_state, Color::White));
    assert_eq!(11, get_piece_mobility(&game_state, Color::Black));
  }
}
