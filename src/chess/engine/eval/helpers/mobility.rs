use super::generic::mask_sum;
use crate::chess::model::board::*;
use crate::chess::model::game_state::*;
use crate::chess::model::piece::*;

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
pub fn get_mobility_area(game_state: &GameState, color: Color) -> u64 {
  // We start assuming all squares are available, and substract our pawns,
  // king and the squares attacked by enemy pawns.
  let mut mobility_area: u64 = u64::MAX;

  let pawn = match color {
    Color::White => WHITE_PAWN,
    Color::Black => BLACK_PAWN,
  };
  let op_pawn = match color {
    Color::White => BLACK_PAWN,
    Color::Black => WHITE_PAWN,
  };
  let king = match color {
    Color::White => WHITE_KING,
    Color::Black => BLACK_KING,
  };
  let queen = match color {
    Color::White => WHITE_QUEEN,
    Color::Black => BLACK_QUEEN,
  };

  for i in 0..64 {
    let value = game_state.board.squares[i];
    if pawn == value || king == value || queen == value {
      mobility_area &= !(1 << i);
      continue;
    }

    // Check if controlled by op_pawns:
    let (file, mut rank) = Board::index_to_fr(i);
    match color {
      Color::White => rank += 1,
      Color::Black => rank -= 1,
    }
    if rank > 8 || rank == 0 {
      continue;
    }

    // Check on the left side:
    if file > 1 {
      let s = Board::fr_to_index(file - 1, rank);
      if game_state.board.squares[s] == op_pawn {
        mobility_area &= !(1 << i);
        continue;
      }
    }

    // Check on the right side:
    if file < 8 {
      let s = Board::fr_to_index(file + 1, rank);
      if game_state.board.squares[s] == op_pawn {
        mobility_area &= !(1 << i);
        continue;
      }
    }
  }

  mobility_area
}

/// Determines the number of available safe squares for each piece
/// TODO: Implement handling pins
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

  let ssp = game_state.board.get_color_mask(color);
  let op = game_state.board.get_color_mask(Color::opposite(color));
  let mobility_area = get_mobility_area(game_state, color);

  for i in 0..64 {
    let value = game_state.board.squares[i];

    if Piece::color(value).is_some()
      && Piece::color(value).unwrap() == color
      && Piece::is_piece(value)
    {
      let (squares, _) = game_state.get_piece_destinations(i, op, ssp);
      mobility += mask_sum(squares & mobility_area);
    }
  }

  mobility
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_mobility_area() {
    let fen = "rnbqk1nr/pppppppp/8/8/1b6/3P4/PPPQPPPP/RNBK1BNR b KQkq - 0 1";
    let game_state = GameState::from_fen(fen);
    assert_eq!(46, mask_sum(get_mobility_area(&game_state, Color::White)));
    assert_eq!(44, mask_sum(get_mobility_area(&game_state, Color::Black)));

    let fen = "r1bq2nr/pppppppp/P7/2kn1P2/1b6/3P1N1P/1PPQP1P1/RNBK1B1R b KQkq - 0 1";
    let game_state = GameState::from_fen(fen);
    //print_mask(get_mobility_area(&game_state, Color::White));
    assert_eq!(47, mask_sum(get_mobility_area(&game_state, Color::White)));
    assert_eq!(43, mask_sum(get_mobility_area(&game_state, Color::Black)));
  }

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
