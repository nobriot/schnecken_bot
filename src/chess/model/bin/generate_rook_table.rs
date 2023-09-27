use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

use chess::model::board::INVALID_SQUARE;
use chess::model::board_geometry::*;
use chess::model::board_mask::*;
use chess::model::piece_moves::*;

// This is where our definitions are exported at the end
use chess::model::tables::rook_destinations::*;

// How many bits are describing relevant blockers based on the position
pub const ROOK_BLOCKER_NUMBERS: [u8; 64] = [
  12, 11, 11, 11, 11, 11, 11, 12, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
  11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
  11, 10, 10, 10, 10, 10, 10, 11, 12, 11, 11, 11, 11, 11, 11, 12,
];

fn main() {
  let filename = "rook_table.rs";
  println!("-----------------------------------------------------------------");
  println!("Generating rook tables in {}...", filename);
  let mut output_file = File::create(filename).unwrap();

  // First the Rook span (constant)
  let mut rook_moves: [u64; 64] = [0; 64];
  for i in 0..64 {
    rook_moves[i] = get_moves_from_offsets(&ROOK_MOVE_OFFSETS, true, 0, 0, i);
  }

  let _ = write!(
    output_file,
    "/// Array of BoardMasks indicating where the rook can reach\n/// if there were no other pieces on the board\n///\npub const ROOK_SPAN:[u64; 64] = {:#018X?};",
    rook_moves
  );

  // Then the indices of the bits on the Rook span
  let mut rook_span_indexes: [[usize; 12]; 64] = [[INVALID_SQUARE.into(); 12]; 64];
  let mut rook_span_without_edges: [u64; 64] = [0; 64];

  for i in 0..64 {
    let mut index = 0;
    // Remove edges if the rook is on it.
    let mut egdes_to_keep: Vec<BoardMask> = Vec::new();
    let mut piece_edges = BOARD_WITHOUT_EDGES;

    if square_in_mask!(i, BOARD_RIGHT_EDGE) {
      egdes_to_keep.push(BOARD_RIGHT_EDGE);
    }
    if square_in_mask!(i, BOARD_LEFT_EDGE) {
      egdes_to_keep.push(BOARD_LEFT_EDGE);
    }
    if square_in_mask!(i, BOARD_UP_EDGE) {
      egdes_to_keep.push(BOARD_UP_EDGE);
    }
    if square_in_mask!(i, BOARD_DOWN_EDGE) {
      egdes_to_keep.push(BOARD_DOWN_EDGE);
    }

    for edge in &egdes_to_keep {
      piece_edges |= edge;
    }
    if !egdes_to_keep.contains(&BOARD_RIGHT_EDGE) {
      piece_edges &= !BOARD_RIGHT_EDGE;
    }
    if !egdes_to_keep.contains(&BOARD_LEFT_EDGE) {
      piece_edges &= !BOARD_LEFT_EDGE;
    }
    if !egdes_to_keep.contains(&BOARD_UP_EDGE) {
      piece_edges &= !BOARD_UP_EDGE;
    }
    if !egdes_to_keep.contains(&BOARD_DOWN_EDGE) {
      piece_edges &= !BOARD_DOWN_EDGE;
    }

    //println!("i = {};", i);
    //print_board_mask(piece_edges);
    for j in 0..64 {
      if !square_in_mask!(j, rook_moves[i] & piece_edges) {
        continue;
      }
      println!("ROOK_SPAN_INDEXES[i={}][index={}] = {};", i, index, j);
      rook_span_indexes[i][index] = j;
      index += 1;
    }

    let mut span_mask = 0;
    for j in 0..12 {
      if rook_span_indexes[i][j] != 255 {
        span_mask |= 1 << rook_span_indexes[i][j];
      }
    }
    unsafe {
      rook_span_without_edges[i] = span_mask;
    }
  }

  let _ = write!(
    output_file,
    "/// Array of BoardMasks indicating where the rook can reach\n/// if there were no other pieces on the board\n///\npub const ROOK_SPAN_WITHOUT_EDGES:[u64; 64] = {:#018X?};",
    rook_span_without_edges
  );

  let _ = write!(
    output_file,"/// For a given position, this table indicate the BoardMasks indices of\n/// possible blockers for the ROOK_SPAN.\n///\n///\n");
  let _ = write!(
    output_file,
    "pub const ROOK_SPAN_INDEXES: [[usize; 12]; 64] = {:#?};",
    rook_span_indexes
  );

  return;

  // Calculate all possible rook destinations based on the list of blockers
  static mut rook_table: [[u64; MAX_ROOK_BLOCKERS_MASK_COMBINATIONS]; 64] =
    [[0; MAX_ROOK_BLOCKERS_MASK_COMBINATIONS]; 64];
  static mut rook_masks: [[u64; MAX_ROOK_BLOCKERS_MASK_COMBINATIONS]; 64] =
    [[0; MAX_ROOK_BLOCKERS_MASK_COMBINATIONS]; 64];
  for i in 0..64 {
    for blockers in 0..MAX_ROOK_BLOCKERS_MASK_COMBINATIONS {
      let mut blocker_mask: BoardMask = 0;
      for j in 0..12 {
        if ROOK_SPAN_INDEXES[i][j] != 255 && (blockers & (1 << j) != 0) {
          blocker_mask |= 1 << ROOK_SPAN_INDEXES[i][j];
        }
      }
      unsafe {
        rook_table[i][blockers] =
          get_moves_from_offsets(&ROOK_MOVE_OFFSETS, true, 0, blocker_mask, i);
        rook_masks[i][blockers] = blocker_mask;
      }
    }
  }

  // Test our table a little bit
  unsafe { print_board_mask(rook_table[0][0]) };
  unsafe { print_board_mask(rook_masks[0][0]) };
  unsafe { print_board_mask(rook_table[34][3]) };
  unsafe { print_board_mask(rook_masks[34][3]) };
  unsafe { print_board_mask(rook_table[34][5]) };
  unsafe { print_board_mask(rook_masks[34][5]) };

  println!("TODO: Magic number lookup reduction");

  let mut key_map: HashMap<u64, u64> = HashMap::new();

  // Sorts blockers that results in the same final squares:

  for i in 0..64 {
    for blockers in 0..MAX_ROOK_BLOCKERS_MASK_COMBINATIONS {
      key_map.insert(unsafe { rook_masks[i][blockers] }, blockers as u64);
    }
  }

  // Determine magic bits
  static mut rook_magic: [u64; 64] = [0; 64];
  static mut rook_magic_table: [[u64; MAX_ROOK_BLOCKERS_MASK_COMBINATIONS]; 64] =
    [[0; MAX_ROOK_BLOCKERS_MASK_COMBINATIONS]; 64];

  unsafe {
    for i in 0..64 {
      for j in 0..1_000_000_000 {
        let magic = rand::random::<u64>();

        if rook_magic[i] != 0 {
          break;
        }

        for m in 0..MAX_ROOK_BLOCKERS_MASK_COMBINATIONS {
          let index: usize = (rook_masks[i][m].wrapping_mul(magic)
            >> (64 - ROOK_BLOCKER_NUMBERS[i])
            & 0xFFFF) as usize;

          if rook_magic_table[i][index] == 0 {
            rook_magic_table[i][index] = rook_table[i][m]
          } else if rook_magic_table[i][index] != rook_table[i][m] {
            for ii in 0..MAX_ROOK_BLOCKERS_MASK_COMBINATIONS {
              rook_magic_table[i][ii] == 0;
            }
            println!("failed at m: {m}");
            break;
          }

          if m == MAX_ROOK_BLOCKERS_MASK_COMBINATIONS {
            println!("Found magic number for i {i}: {magic}");
            rook_magic[i] = magic;
          }
        }
      }
    }

    println!("pub const ROOK_MAGIC: [u64; 64] = {:#?}", rook_magic);
  }
  /*let _ = write!(
    output_file,
    "pub const ROOK_TABLE: [[usize; MAX_ROOK_BLOCKERS_MASK_COMBINATIONS]; 64] = {:#?};",
    unsafe { rook_table }
  );
  */

  /*
   */
  println!("");
  println!("Done! 🙂");
}
