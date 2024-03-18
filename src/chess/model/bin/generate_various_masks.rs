use std::fs::File;
use std::io::Write;

use chess::model::board::Board;
use chess::model::board_mask::*;

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

  // White king shelter pawns
  let mut white_king_shelter_pawns: [u64; 64] = [0; 64];
  for i in 0..64 {
    let (file, rank) = Board::index_to_fr(i);
    if rank == 8 {
      continue;
    }

    let min_file = std::cmp::max(file - 1, 1);
    let max_file = std::cmp::min(file + 1, 8);
    let min_rank = std::cmp::min(rank + 1, 8);
    let max_rank = 8;

    for f in min_file..=max_file {
      for r in min_rank..=max_rank {
        let j = Board::fr_to_index(f, r);
        set_square_in_mask!(j, white_king_shelter_pawns[i as usize]);
      }
    }
  }

  let _ = write!(
    output_file,
    "/// White King shelter pawns\n///\npub const WHITE_KING_SHELTER_PAWNS: [u64; 64] = {:#018X?};\n\n",
    white_king_shelter_pawns
  );

  // Black king shelter pawns
  let mut black_king_shelter_pawns: [u64; 64] = [0; 64];
  for i in 0..64 {
    let (file, rank) = Board::index_to_fr(i);
    if rank == 1 {
      continue;
    }

    let min_file = std::cmp::max(file - 1, 1);
    let max_file = std::cmp::min(file + 1, 8);
    let min_rank = std::cmp::min(rank - 1, 1);
    let max_rank = std::cmp::max(rank - 1, 1);

    for f in min_file..=max_file {
      for r in min_rank..=max_rank {
        let j = Board::fr_to_index(f, r);
        set_square_in_mask!(j, black_king_shelter_pawns[i as usize]);
      }
    }
  }

  let _ = write!(
    output_file,
    "/// White King shelter pawns\n///\npub const BLACK_KING_SHELTER_PAWNS: [u64; 64] = {:#018X?};\n\n",
    black_king_shelter_pawns
  );
  //print_board_mask(black_king_shelter_pawns[4]);
}
