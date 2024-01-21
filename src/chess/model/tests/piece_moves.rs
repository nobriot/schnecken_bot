use crate::model::board_mask::board_mask_to_string;
use crate::model::moves::string_to_square;
use crate::model::piece_moves::*;

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
  let opponent_control: u64 = 1 << string_to_square("a3");
  let expected_squares: u64 = (1 << string_to_square("a2"))
    | (1 << string_to_square("a4"))
    | (1 << string_to_square("b4"))
    | (1 << string_to_square("c3"))
    | (1 << string_to_square("c2"))
    | (1 << string_to_square("b2"));

  let calculated_squares = get_king_moves(
    same_side_pieces,
    opponent_control,
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
