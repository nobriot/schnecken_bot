use chess::model::board::*;
use chess::model::board_mask::*;
use std::fs::File;
use std::io::Write;

fn main() {
  let filename = "passed_pawns_areas.rs";
  println!("-----------------------------------------------------------------");
  println!("Generating passed pawns tables in {}...", filename);
  let mut output_file = File::create(filename).unwrap();

  // Where do we look for other pawns to determine if a white pawn is passed:
  let mut white_passed_pawn_area: [BoardMask; 64] = [0; 64];

  for i in 0..64usize {
    // Determine the rank / File:
    let (file, mut rank) = Board::index_to_fr(i as u8);
    if rank == 8 {
      continue;
    }

    loop {
      rank += 1;
      if rank == 8 {
        break;
      }
      let s = Board::fr_to_index(file, rank);
      // Check straight ahead:
      set_square_in_mask!(s, white_passed_pawn_area[i]);
      // Check on the left side:
      if file > 1 {
        let s = Board::fr_to_index(file - 1, rank);
        set_square_in_mask!(s, white_passed_pawn_area[i]);
      }
      // Check on the right side:
      if file < 8 {
        let s = Board::fr_to_index(file + 1, rank);
        set_square_in_mask!(s, white_passed_pawn_area[i]);
      }
    }
  }

  let _ = write!(output_file,
                 "/// Represents boardmask of area where there should be no pawns if\n");
  let _ = write!(output_file, "/// we want to consider the white pawn passed.\n");
  let _ = write!(output_file,
                 "/// For a pawn on square i, check if WHITE_PASSED_PAWN_AREA[i] contains any opponent pawns\n");

  let _ = write!(output_file,
                 "pub const WHITE_PASSED_PAWN_AREA: [BoardMask; 64] = {:#018X?};",
                 white_passed_pawn_area);

  // ---------------------------------------------------------------------------
  // Black pawns now
  // ---------------------------------------------------------------------------

  let _ = write!(output_file, "\n\n");

  // Where do we look for other pawns to determine if a white pawn is passed:
  let mut black_passed_pawn_area: [BoardMask; 64] = [0; 64];

  for i in 0..64usize {
    // Determine the rank / File:
    let (file, mut rank) = Board::index_to_fr(i as u8);
    if rank == 1 {
      continue;
    }

    loop {
      rank -= 1;
      if rank == 1 {
        break;
      }
      let s = Board::fr_to_index(file, rank);
      // Check straight ahead:
      set_square_in_mask!(s, black_passed_pawn_area[i]);
      // Check on the left side:
      if file > 1 {
        let s = Board::fr_to_index(file - 1, rank);
        set_square_in_mask!(s, black_passed_pawn_area[i]);
      }
      // Check on the right side:
      if file < 8 {
        let s = Board::fr_to_index(file + 1, rank);
        set_square_in_mask!(s, black_passed_pawn_area[i]);
      }
    }
  }

  let _ = write!(output_file,
                 "/// Represents boardmask of area where there should be no pawns if\n");
  let _ = write!(output_file, "/// we want to consider the black pawn passed.\n");
  let _ = write!(output_file,
                 "/// For a pawn on square i, check if BLACK_PASSED_PAWN_AREA[i] contains any opponent pawns\n");

  let _ = write!(output_file,
                 "pub const BLACK_PASSED_PAWN_AREA: [BoardMask; 64] = {:#018X?};",
                 black_passed_pawn_area);

  println!("");
  println!("Done! 🙂");
}
