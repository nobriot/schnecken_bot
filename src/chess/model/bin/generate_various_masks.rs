use rand::Rng;
use std::fs::File;
use std::io::Write;

use chess::model::board_geometry::*;
use chess::model::board_mask::*;
use chess::model::piece_moves::*;

fn main() {
  let filename = "various_masks.rs";
  println!("-----------------------------------------------------------------");
  println!("Generating various masks in {}...", filename);
  let mut output_file = File::create(filename).unwrap();

  // Create Queen side/ king side masks
  let mut queenside: BoardMask = 0;
  let mut kingside: BoardMask = 0;
  for i in 0..64 {
    if i % 8 < 4 {
      set_square_in_mask!(i, queenside);
    }
    if i % 8 >= 4 {
      set_square_in_mask!(i, kingside);
    }
  }

  let _ = write!(
    output_file,
    "/// Queen Side mask\n///\npub const QUEEN_SIDE_MASK:BoardMask = {:#018X?};\n\n",
    queenside
  );
  let _ = write!(
    output_file,
    "/// King Side mask\n///\npub const KING_SIDE_MASK:BoardMask = {:#018X?};\n\n",
    kingside
  );
}
