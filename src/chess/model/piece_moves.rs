use crate::chess::model::board::board_mask_to_string;
use crate::chess::model::board::string_to_square;

// Array of u64 (board bitmasks bitmasks indicating if a knight can move from a square to another)

// Knights can always go at the same "jumps", regardless of the board.
// So we store this as a const table.
/// Possible knight moves from a square.
pub const KNIGHT_MOVES: [u64; 64] = [
  0x0000000000020400,
  0x0000000000050800,
  0x00000000000A1100,
  0x0000000000142200,
  0x0000000000284400,
  0x0000000000508800,
  0x0000000000A01000,
  0x0000000000402000,
  0x0000000002040004,
  0x0000000005080008,
  0x000000000A110011,
  0x0000000014220022,
  0x0000000028440044,
  0x0000000050880088,
  0x00000000A0100010,
  0x0000000040200020,
  0x0000000204000402,
  0x0000000508000805,
  0x0000000A1100110A,
  0x0000001422002214,
  0x0000002844004428,
  0x0000005088008850,
  0x000000A0100010A0,
  0x0000004020002040,
  0x0000020400040200,
  0x0000050800080500,
  0x00000A1100110A00,
  0x0000142200221400,
  0x0000284400442800,
  0x0000508800885000,
  0x0000A0100010A000,
  0x0000402000204000,
  0x0002040004020000,
  0x0005080008050000,
  0x000A1100110A0000,
  0x0014220022140000,
  0x0028440044280000,
  0x0050880088500000,
  0x00A0100010A00000,
  0x0040200020400000,
  0x0204000402000000,
  0x0508000805000000,
  0x0A1100110A000000,
  0x1422002214000000,
  0x2844004428000000,
  0x5088008850000000,
  0xA0100010A0000000,
  0x4020002040000000,
  0x0400040200000000,
  0x0800080500000000,
  0x1100110A00000000,
  0x2200221400000000,
  0x4400442800000000,
  0x8800885000000000,
  0x100010A000000000,
  0x2000204000000000,
  0x0004020000000000,
  0x0008050000000000,
  0x00110A0000000000,
  0x0022140000000000,
  0x0044280000000000,
  0x0088500000000000,
  0x0010A00000000000,
  0x0020400000000000,
];

// Constant indicating how the file/rank are offset when a bishop moves
pub const BISHOP_MOVE_OFFSETS: [(isize, isize); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
// Constant indicating how the file/rank are offset when a rook moves
pub const ROOK_MOVE_OFFSETS: [(isize, isize); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// Constant indicating which squares a king can reach from its current square
pub const KING_MOVES: [u64; 64] = [
  0x0000000000000302,
  0x0000000000000705,
  0x0000000000000E0A,
  0x0000000000001C14,
  0x0000000000003828,
  0x0000000000007050,
  0x000000000000E0A0,
  0x000000000000C040,
  0x0000000000030203,
  0x0000000000070507,
  0x00000000000E0A0E,
  0x00000000001C141C,
  0x0000000000382838,
  0x0000000000705070,
  0x0000000000E0A0E0,
  0x0000000000C040C0,
  0x0000000003020300,
  0x0000000007050700,
  0x000000000E0A0E00,
  0x000000001C141C00,
  0x0000000038283800,
  0x0000000070507000,
  0x00000000E0A0E000,
  0x00000000C040C000,
  0x0000000302030000,
  0x0000000705070000,
  0x0000000E0A0E0000,
  0x0000001C141C0000,
  0x0000003828380000,
  0x0000007050700000,
  0x000000E0A0E00000,
  0x000000C040C00000,
  0x0000030203000000,
  0x0000070507000000,
  0x00000E0A0E000000,
  0x00001C141C000000,
  0x0000382838000000,
  0x0000705070000000,
  0x0000E0A0E0000000,
  0x0000C040C0000000,
  0x0003020300000000,
  0x0007050700000000,
  0x000E0A0E00000000,
  0x001C141C00000000,
  0x0038283800000000,
  0x0070507000000000,
  0x00E0A0E000000000,
  0x00C040C000000000,
  0x0302030000000000,
  0x0705070000000000,
  0x0E0A0E0000000000,
  0x1C141C0000000000,
  0x3828380000000000,
  0x7050700000000000,
  0xE0A0E00000000000,
  0xC040C00000000000,
  0x0203000000000000,
  0x0507000000000000,
  0x0A0E000000000000,
  0x141C000000000000,
  0x2838000000000000,
  0x5070000000000000,
  0xA0E0000000000000,
  0x40C0000000000000,
];

/// Returns a bitmask of the knight possible destination squares.
///
/// # Arguments
///
/// * `same_side_pieces` - boardmask of the same side pieces
/// * `opponent_pieces` - boardmask of the opponent pieces
/// * `square` - Start square for the knight
///
pub fn get_knight_moves(same_side_pieces: u64, _opponent_pieces: u64, square: usize) -> u64 {
  // Knight just cannot go where we have same side pieces
  return KNIGHT_MOVES[square] & (!same_side_pieces);
}

/// Computes the list of possible destinations when a piece can "repeat" a
/// move offset until the end of the board, capturing another piece or meeting
/// a same side piece.
///
/// Returns a bitmask of the possible destination squares.
///
/// # Arguments
///
/// * `move_offsets` - possible offsets to apply First the file, then the rank
/// * `recursion` - can the piece continue going in one direction ? true or is it only 1 jump ? false
/// * `same_side_pieces` - boardmask of the same side pieces
/// * `opponent_pieces` - boardmask of the opponent pieces
/// * `square` - Start square for the knight
/// * `square` - Start square for the knight
///
fn get_moves_from_offsets(
  move_offsets: &[(isize, isize)],
  recursion: bool,
  same_side_pieces: u64,
  opponent_pieces: u64,
  square: usize,
) -> u64 {
  let inital_rank = (square / 8) as isize;
  let inital_file = (square % 8) as isize;
  let mut destinations: u64 = 0;
  for (file_offset, rank_offset) in move_offsets {
    let mut rank = inital_rank;
    let mut file = inital_file;
    // Each move can be repeated until we meet a piece or fall of the board:
    loop {
      rank += rank_offset;
      file += file_offset;

      // Did we go too far ?
      if rank < 0 || rank > 7 || file < 0 || file > 7 {
        break;
      }

      let destination_bitmask: u64 = 1 << (rank * 8 + file);
      if destination_bitmask & same_side_pieces != 0 {
        break;
      }
      // Now we know it is a valid destination, add it to the list:
      destinations |= destination_bitmask;

      // If we just captured, we cannot go further
      if (destination_bitmask & opponent_pieces) != 0 {
        break;
      }
      if recursion == false {
        break;
      }
    }
  }
  destinations
}

/// Returns the list of possible bishop moves
///
/// # Arguments
///
/// * `same_side_pieces` - boardmask of the same side pieces
/// * `opponent_pieces` - boardmask of the opponent pieces
/// * `square` - Start square for the knight
///
pub fn get_bishop_moves(same_side_pieces: u64, opponent_pieces: u64, square: usize) -> u64 {
  return get_moves_from_offsets(
    &BISHOP_MOVE_OFFSETS,
    true,
    same_side_pieces,
    opponent_pieces,
    square,
  );
}

/// Returns the list of possible Rook moves
///
/// # Arguments
///
/// * `same_side_pieces` - boardmask of the same side pieces
/// * `opponent_pieces` - boardmask of the opponent pieces
/// * `square` - Start square for the knight
///
pub fn get_rook_moves(same_side_pieces: u64, opponent_pieces: u64, square: usize) -> u64 {
  return get_moves_from_offsets(
    &ROOK_MOVE_OFFSETS,
    true,
    same_side_pieces,
    opponent_pieces,
    square,
  );
}

/// Returns the list of possible Queen moves
///
/// # Arguments
///
/// * `same_side_pieces` - boardmask of the same side pieces
/// * `opponent_pieces` - boardmask of the opponent pieces
/// * `square` - Start square for the knight
///
pub fn get_queen_moves(same_side_pieces: u64, opponent_pieces: u64, square: usize) -> u64 {
  // A queen can do what bishops and rooks can do.
  return get_rook_moves(same_side_pieces, opponent_pieces, square)
    | get_bishop_moves(same_side_pieces, opponent_pieces, square);
}

/// Returns a bitmask of the king possible destination squares.
///
/// # Arguments
///
/// * `same_side_pieces` - boardmask of the same side pieces
/// * `opponent_pieces` - boardmask of the opponent pieces
/// * `square` - Start square for the knight
///
pub fn get_king_moves(same_side_pieces: u64, _opponent_pieces: u64, square: usize) -> u64 {
  // Knight just cannot go where we have same side pieces
  return KING_MOVES[square] & (!same_side_pieces);
}

/// Returns a bitmask of the white pawn possible destination squares.
///
/// # Arguments
///
/// * `same_side_pieces` - boardmask of the same side pieces
/// * `opponent_pieces` - boardmask of the opponent pieces
/// * `square` - Start square for the knight
///
pub fn get_white_pawn_moves(same_side_pieces: u64, opponent_pieces: u64, square: usize) -> u64 {
  let inital_rank = (square / 8) as isize;
  let inital_file = (square % 8) as isize;
  let mut destinations: u64 = 0;

  // Check if it can go by 1 :
  if inital_rank < 7 {
    let destination_bitmask: u64 = 1 << ((inital_rank + 1) * 8 + inital_file);
    if (destination_bitmask & same_side_pieces) == 0 && (destination_bitmask & opponent_pieces) == 0
    {
      destinations |= destination_bitmask;
    }
  }

  // Check if it can go by 2:
  if inital_rank == 1 {
    let destination_bitmask: u64 = 1 << ((inital_rank + 2) * 8 + inital_file);
    let blocking_bitmask: u64 = 1 << ((inital_rank + 1) * 8 + inital_file);
    if (destination_bitmask & same_side_pieces) == 0
      && (destination_bitmask & opponent_pieces) == 0
      && (blocking_bitmask & same_side_pieces) == 0
      && (blocking_bitmask & opponent_pieces) == 0
    {
      destinations |= destination_bitmask;
    }
  }

  // Check if it can capture
  // en passant can be handled by adding a phantom piece on the en passant square
  let capture_offsets: [(isize, isize); 2] = [(1, 1), (-1, 1)];
  for (file_offset, rank_offset) in capture_offsets {
    let rank = inital_rank + rank_offset;
    let file = inital_file + file_offset;

    // Did we go too far ?
    if rank < 0 || rank > 7 || file < 0 || file > 7 {
      continue;
    }

    // Now we know it is a valid destination, add it to the list:
    let destination_bitmask: u64 = 1 << (rank * 8 + file);
    if destination_bitmask & opponent_pieces != 0 {
      destinations |= destination_bitmask;
    }
  }

  destinations
}

/// Returns a bitmask of the white pawn possible destination squares.
///
/// # Arguments
///
/// * `same_side_pieces` - boardmask of the same side pieces
/// * `opponent_pieces` - boardmask of the opponent pieces
/// * `square` - Start square for the knight
///
pub fn get_black_pawn_moves(same_side_pieces: u64, opponent_pieces: u64, square: usize) -> u64 {
  let inital_rank = (square / 8) as isize;
  let inital_file = (square % 8) as isize;
  let mut destinations: u64 = 0;

  // Check if it can go by 1 :
  if inital_rank > 0 {
    let destination_bitmask: u64 = 1 << ((inital_rank - 1) * 8 + inital_file);
    if (destination_bitmask & same_side_pieces) == 0 && (destination_bitmask & opponent_pieces) == 0
    {
      destinations |= destination_bitmask;
    }
  }

  // Check if it can go by 2:
  if inital_rank == 6 {
    let destination_bitmask: u64 = 1 << ((inital_rank - 2) * 8 + inital_file);
    let blocking_bitmask: u64 = 1 << ((inital_rank - 1) * 8 + inital_file);
    if (destination_bitmask & same_side_pieces) == 0
      && (destination_bitmask & opponent_pieces) == 0
      && (blocking_bitmask & same_side_pieces) == 0
      && (blocking_bitmask & opponent_pieces) == 0
    {
      destinations |= destination_bitmask;
    }
  }

  // Check if it can capture
  // en passant can be handled by adding a phantom piece on the en passant square
  let capture_offsets: [(isize, isize); 2] = [(1, -1), (-1, -1)];
  for (file_offset, rank_offset) in capture_offsets {
    let rank = inital_rank + rank_offset;
    let file = inital_file + file_offset;

    // Did we go too far ?
    if rank < 0 || rank > 7 || file < 0 || file > 7 {
      continue;
    }

    // Now we know it is a valid destination, add it to the list:
    let destination_bitmask: u64 = 1 << (rank * 8 + file);
    if destination_bitmask & opponent_pieces != 0 {
      destinations |= destination_bitmask;
    }
  }

  destinations
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn check_knight_moves() {
    let expected_squares: u64 = (1 << string_to_square("a5"))
      | (1 << string_to_square("c5"))
      | (1 << string_to_square("d4"))
      | (1 << string_to_square("d2"))
      | (1 << string_to_square("c1"))
      | (1 << string_to_square("a1"));
    assert_eq!(
      expected_squares,
      get_knight_moves(0, 0, string_to_square("b3") as usize)
    );

    // Now block some of the destination squares with same side pieces.
    let expected_squares: u64 =
      (1 << string_to_square("a5")) | (1 << string_to_square("c1")) | (1 << string_to_square("a1"));
    let same_side_pieces: u64 =
      (1 << string_to_square("c5")) | (1 << string_to_square("d4")) | (1 << string_to_square("d2"));
    assert_eq!(
      expected_squares,
      get_knight_moves(same_side_pieces, 0, string_to_square("b3") as usize)
    );
  }

  #[test]
  fn check_bishop_moves() {
    // Let's take a bishop on b3, no other pieces
    let expected_squares: u64 = (1 << string_to_square("a2"))
      | (1 << string_to_square("a4"))
      | (1 << string_to_square("c2"))
      | (1 << string_to_square("c4"))
      | (1 << string_to_square("d1"))
      | (1 << string_to_square("d5"))
      | (1 << string_to_square("e6"))
      | (1 << string_to_square("f7"))
      | (1 << string_to_square("g8"));
    println!("Expected: \n{}", board_mask_to_string(expected_squares));
    println!(
      "Received: \n{}",
      board_mask_to_string(get_bishop_moves(0, 0, string_to_square("b3") as usize))
    );
    assert_eq!(
      expected_squares,
      get_bishop_moves(0, 0, string_to_square("b3") as usize)
    );

    // Same with captures and blocking pieces
    let same_side_pieces: u64 = 1 << string_to_square("e6");
    let opponent_pieces: u64 = 1 << string_to_square("c2");

    // Now we expect the bishop not to reach e6, and stop as c2 included
    let expected_squares: u64 = (1 << string_to_square("a2"))
      | (1 << string_to_square("a4"))
      | (1 << string_to_square("c2"))
      | (1 << string_to_square("c4"))
      | (1 << string_to_square("d5"));

    assert_eq!(
      expected_squares,
      get_bishop_moves(
        same_side_pieces,
        opponent_pieces,
        string_to_square("b3") as usize
      )
    );
  }

  #[test]
  fn check_rook_moves() {
    // Let's take a rook on b3, no other pieces
    let expected_squares: u64 = (1 << string_to_square("b8"))
      | (1 << string_to_square("b7"))
      | (1 << string_to_square("b6"))
      | (1 << string_to_square("b5"))
      | (1 << string_to_square("b4"))
      | (1 << string_to_square("b2"))
      | (1 << string_to_square("b1"))
      | (1 << string_to_square("a3"))
      | (1 << string_to_square("c3"))
      | (1 << string_to_square("d3"))
      | (1 << string_to_square("e3"))
      | (1 << string_to_square("f3"))
      | (1 << string_to_square("g3"))
      | (1 << string_to_square("h3"));

    let calculated_squares = get_rook_moves(0, 0, string_to_square("b3") as usize);
    println!("Expected: \n{}", board_mask_to_string(expected_squares));
    println!("Calculated: \n{}", board_mask_to_string(calculated_squares));
    assert_eq!(expected_squares, calculated_squares);

    // Same with captures and blocking pieces
    let same_side_pieces: u64 = 1 << string_to_square("b6");
    let opponent_pieces: u64 = 1 << string_to_square("d3");
    let expected_squares: u64 = (1 << string_to_square("b5"))
      | (1 << string_to_square("b4"))
      | (1 << string_to_square("b2"))
      | (1 << string_to_square("b1"))
      | (1 << string_to_square("a3"))
      | (1 << string_to_square("c3"))
      | (1 << string_to_square("d3"));
    let calculated_squares = get_rook_moves(
      same_side_pieces,
      opponent_pieces,
      string_to_square("b3") as usize,
    );

    println!("Expected: \n{}", board_mask_to_string(expected_squares));
    println!("Calculated: \n{}", board_mask_to_string(calculated_squares));
    assert_eq!(expected_squares, calculated_squares);
  }

  #[ignore]
  #[test]
  fn generate_king_moves() {
    let mut king_moves: [u64; 64] = [0; 64];
    let move_offsets: [(isize, isize); 8] = [
      (1, 0),
      (-1, 0),
      (0, 1),
      (0, -1),
      (1, 1),
      (1, -1),
      (-1, 1),
      (-1, -1),
    ];
    for i in 0..64 {
      king_moves[i] = get_moves_from_offsets(&move_offsets, false, 0, 0, i);
    }
    println!("pub const KING_MOVES:[u64; 64] = {:#018X?};", king_moves);
  }

  #[test]
  fn check_king_moves() {
    // Let's take a king on b3, no other pieces
    let expected_squares: u64 = (1 << string_to_square("a2"))
      | (1 << string_to_square("a3"))
      | (1 << string_to_square("a4"))
      | (1 << string_to_square("b4"))
      | (1 << string_to_square("c4"))
      | (1 << string_to_square("c3"))
      | (1 << string_to_square("c2"))
      | (1 << string_to_square("b2"));

    let calculated_squares = get_king_moves(0, 0, string_to_square("b3") as usize);
    println!("Expected: \n{}", board_mask_to_string(expected_squares));
    println!("Calculated: \n{}", board_mask_to_string(calculated_squares));
    assert_eq!(expected_squares, calculated_squares);

    // Same with captures and blocking pieces
    let same_side_pieces: u64 = 1 << string_to_square("c4");
    let opponent_pieces: u64 = 1 << string_to_square("a3");
    let expected_squares: u64 = (1 << string_to_square("a2"))
      | (1 << string_to_square("a3"))
      | (1 << string_to_square("a4"))
      | (1 << string_to_square("b4"))
      | (1 << string_to_square("c3"))
      | (1 << string_to_square("c2"))
      | (1 << string_to_square("b2"));

    let calculated_squares = get_king_moves(
      same_side_pieces,
      opponent_pieces,
      string_to_square("b3") as usize,
    );

    println!("Expected: \n{}", board_mask_to_string(expected_squares));
    println!("Calculated: \n{}", board_mask_to_string(calculated_squares));
    assert_eq!(expected_squares, calculated_squares);
  }

  #[test]
  fn check_white_pawn_moves() {
    // Let's take a pawn on a2, no other pieces
    let expected_squares: u64 = (1 << string_to_square("a3")) | (1 << string_to_square("a4"));

    let calculated_squares = get_white_pawn_moves(0, 0, string_to_square("a2") as usize);
    println!("Expected: \n{}", board_mask_to_string(expected_squares));
    println!("Calculated: \n{}", board_mask_to_string(calculated_squares));
    assert_eq!(expected_squares, calculated_squares);

    // put pieces around
    let same_side_pieces: u64 = 1 << string_to_square("b3");
    let opponent_pieces: u64 = 1 << string_to_square("a4");
    let expected_squares: u64 = 1 << string_to_square("a3");

    let calculated_squares = get_white_pawn_moves(
      same_side_pieces,
      opponent_pieces,
      string_to_square("a2") as usize,
    );

    println!("Expected: \n{}", board_mask_to_string(expected_squares));
    println!("Calculated: \n{}", board_mask_to_string(calculated_squares));
    assert_eq!(expected_squares, calculated_squares);

    // Try captures pieces around
    let same_side_pieces: u64 = 1 << string_to_square("b3");
    let opponent_pieces: u64 = (1 << string_to_square("c5")) | (1 << string_to_square("e5"));
    let expected_squares: u64 =
      (1 << string_to_square("c5")) | (1 << string_to_square("d5")) | (1 << string_to_square("e5"));

    let calculated_squares = get_white_pawn_moves(
      same_side_pieces,
      opponent_pieces,
      string_to_square("d4") as usize,
    );

    println!("Expected: \n{}", board_mask_to_string(expected_squares));
    println!("Calculated: \n{}", board_mask_to_string(calculated_squares));
    assert_eq!(expected_squares, calculated_squares);
  }

  #[test]
  fn check_black_pawn_moves() {
    // Let's take a pawn on a2, no other pieces
    let expected_squares: u64 = (1 << string_to_square("a6")) | (1 << string_to_square("a5"));

    let calculated_squares = get_black_pawn_moves(0, 0, string_to_square("a7") as usize);
    println!("Expected: \n{}", board_mask_to_string(expected_squares));
    println!("Calculated: \n{}", board_mask_to_string(calculated_squares));
    assert_eq!(expected_squares, calculated_squares);

    // put pieces around
    let same_side_pieces: u64 = 1 << string_to_square("b3");
    let opponent_pieces: u64 = 1 << string_to_square("a5");
    let expected_squares: u64 = 1 << string_to_square("a6");

    let calculated_squares = get_black_pawn_moves(
      same_side_pieces,
      opponent_pieces,
      string_to_square("a7") as usize,
    );

    println!("Expected: \n{}", board_mask_to_string(expected_squares));
    println!("Calculated: \n{}", board_mask_to_string(calculated_squares));
    assert_eq!(expected_squares, calculated_squares);

    // Try captures pieces around
    let same_side_pieces: u64 = 1 << string_to_square("b3");
    let opponent_pieces: u64 = (1 << string_to_square("c5")) | (1 << string_to_square("e5"));
    let expected_squares: u64 =
      (1 << string_to_square("c5")) | (1 << string_to_square("d5")) | (1 << string_to_square("e5"));

    let calculated_squares = get_black_pawn_moves(
      same_side_pieces,
      opponent_pieces,
      string_to_square("d6") as usize,
    );

    println!("Expected: \n{}", board_mask_to_string(expected_squares));
    println!("Calculated: \n{}", board_mask_to_string(calculated_squares));
    assert_eq!(expected_squares, calculated_squares);
  }
}
