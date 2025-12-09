use chess::model::board::INVALID_SQUARE;
use chess::model::board_geometry::*;
use chess::model::board_mask::*;
use chess::model::piece_moves::*;
// This is where our definitions are exported at the end
use chess::model::tables::rook_destinations::*;
use rand::Rng;
use std::fs::File;
use std::io::Write;

// How many bits are describing relevant blockers based on the position
pub const ROOK_BLOCKER_NUMBERS: [u8; 64] =
  [12, 11, 11, 11, 11, 11, 11, 12, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
   11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
   11, 10, 10, 10, 10, 10, 10, 11, 12, 11, 11, 11, 11, 11, 11, 12];

fn main() {
  let filename = "rook_table.rs";
  println!("-----------------------------------------------------------------");
  println!("Generating rook tables in {}...", filename);
  let mut output_file = File::create(filename).unwrap();

  // First the Rook span (constant)
  let mut rook_span: [u64; 64] = [0; 64];
  for i in 0..64 {
    rook_span[i] = get_moves_from_offsets(&ROOK_MOVE_OFFSETS, true, 0, 0, i);
  }

  let _ = write!(output_file,
                 "/// Array of BoardMasks indicating where the rook can reach\n/// if there were no other pieces on the board\n///\npub const ROOK_SPAN:[u64; 64] = {:#018X?};\n\n",
                 rook_span);

  // Then we calculate indices of relevant bits as well as rook spans without
  // edges. Then the indices of the bits on the Rook span
  let mut rook_span_indexes: [[usize; 12]; 64] = [[INVALID_SQUARE.into(); 12]; 64];
  let mut rook_span_without_edges: [u64; 64] = [0; 64];
  // How many bits are describing relevant blockers based on the position
  let mut rook_blocker_numbers: [u8; 64] = [64; 64];

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

    // println!("i = {};", i);
    // print_board_mask(piece_edges);
    for j in 0..64 {
      if !square_in_mask!(j, rook_span[i] & piece_edges) {
        continue;
      }
      // println!("ROOK_SPAN_INDEXES[i={}][index={}] = {};", i, index, j);
      rook_span_indexes[i][index] = j;
      index += 1;
    }
    rook_blocker_numbers[i] = index as u8;

    let mut span_mask = 0;
    for j in 0..12 {
      if rook_span_indexes[i][j] != 255 {
        span_mask |= 1 << rook_span_indexes[i][j];
      }
    }
    rook_span_without_edges[i] = span_mask;
  }

  let _ = write!(output_file,
                 "/// Array of BoardMasks indicating where the rook can reach\n/// if there were no other pieces on the board\n///\npub const ROOK_SPAN_WITHOUT_EDGES:[u64; 64] = {:#018X?};\n\n",
                 rook_span_without_edges);

  let _ = write!(output_file,
                 "/// For a given position, this table indicate the BoardMasks indices of\n/// possible blockers for the ROOK_SPAN.\n///\n///\n");
  let _ = write!(output_file,
                 "pub const ROOK_SPAN_INDEXES: [[usize; 12]; 64] = {:#?};\n\n",
                 rook_span_indexes);
  let _ = write!(output_file,
                 "/// For a given position, this table indicate the Number of relevant blockers bits for a rook\n///\n");
  let _ = write!(output_file,
                 "pub const ROOK_BLOCKER_NUMBERS: [u8; 64] = {:#?};\n\n",
                 rook_blocker_numbers);

  // Now we want to find these rook magic constants
  let mut rook_magic: [u64; 64] = [0; 64];

  for i in 0..64 {
    println!("Searching magic for square {i}");
    rook_magic[i] = find_rook_magic(i);
  }

  let _ = write!(output_file, "/// Rook Magic Numbers\n///\n");
  let _ = write!(output_file, "pub const ROOK_MAGIC:[u64; 64] = {:#?};\n\n", rook_magic);

  // Now we pre-compute the list of destinations for all blocker masks
  let mut rook_destination_table: [[u64; MAX_ROOK_BLOCKERS_MASK_COMBINATIONS]; 64] =
    [[0; MAX_ROOK_BLOCKERS_MASK_COMBINATIONS]; 64];

  for i in 0..64 {
    let mut blockers: [u64; MAX_ROOK_BLOCKERS_MASK_COMBINATIONS] =
      [0; MAX_ROOK_BLOCKERS_MASK_COMBINATIONS];
    let blocker_combinations = 1 << ROOK_SPAN_WITHOUT_EDGES[i].count_ones();

    // Assemble the combinations of possible blockers for square `i`
    for b in 0..blocker_combinations {
      let mut blocker_mask: BoardMask = 0;
      for j in 0..ROOK_SPAN_WITHOUT_EDGES[i].count_ones() as usize {
        assert!(ROOK_SPAN_INDEXES[i][j] != 255);
        if b & (1 << j) != 0 {
          blocker_mask |= 1 << ROOK_SPAN_INDEXES[i][j];
        }
      }
      blockers[b] = blocker_mask;
    }

    for b in 0..blocker_combinations {
      let j: usize =
        (blockers[b].wrapping_mul(rook_magic[i]) >> (64 - ROOK_BLOCKER_NUMBERS[i])) as usize;

      if rook_destination_table[i][j] == 0 {
        rook_destination_table[i][j] =
          get_moves_from_offsets(&ROOK_MOVE_OFFSETS, true, 0, blockers[b], i);
      } else if rook_destination_table[i][j]
                != get_moves_from_offsets(&ROOK_MOVE_OFFSETS, true, 0, blockers[b], i)
      {
        println!("Oh oh... square: {i} blocker {b}, derived index is {j} for blocker mask:");
        print_board_mask(blockers[b]);
        println!("Look up table says:");
        print_board_mask(rook_destination_table[i][j]);
        println!("while manual calculation says:");
        print_board_mask(get_moves_from_offsets(&ROOK_MOVE_OFFSETS, true, 0, blockers[b], i));
        println!("Wrapping mul (shift is {}):", (64 - ROOK_BLOCKER_NUMBERS[i]));
        print_board_mask(blockers[b].wrapping_mul(rook_magic[i]));
        print_board_mask((blockers[b].wrapping_mul(rook_magic[i]))
                         >> (64 - ROOK_BLOCKER_NUMBERS[i]));
        panic!("Do not use this result!");
      }
    }
  }

  let _ = write!(output_file, "/// Rook Destination Table\n///\n");
  let _ = write!(output_file,
                 "pub const ROOK_DESTINATION_TABLE: [[u64; MAX_ROOK_BLOCKERS_MASK_COMBINATIONS]; 64] = {:#?};",
                 rook_destination_table);

  // test sanity:
  let mut rng = rand::thread_rng();
  for _ in 0..1000 {
    let blockers = rand::random::<u64>();
    let square = rng.gen_range(0..64);

    let manual_calculation = get_moves_from_offsets(&ROOK_MOVE_OFFSETS, true, 0, blockers, square);
    let index: usize = ((blockers & ROOK_SPAN_WITHOUT_EDGES[square]).wrapping_mul(rook_magic
                                                                                    [square])
                        >> (64 - ROOK_BLOCKER_NUMBERS[square])) as usize;
    let look_up_table = rook_destination_table[square][index];

    println!("Result for blockers from square {square} - looked up index {index}:");
    print_board_mask(blockers & ROOK_SPAN_WITHOUT_EDGES[square]);
    print_board_mask(manual_calculation);
    print_board_mask(look_up_table);
    assert_eq!(manual_calculation, look_up_table);
  }

  //
  println!("");
  println!("Done! 🙂");
}

// Got inspired by https://www.chessprogramming.org/Looking_for_Magics
// Could not really come up with all of it myself.
fn find_rook_magic(square: usize) -> BoardMask {
  let mut used: [BoardMask; MAX_ROOK_BLOCKERS_MASK_COMBINATIONS] =
    [0; MAX_ROOK_BLOCKERS_MASK_COMBINATIONS];
  let mut blockers: [BoardMask; MAX_ROOK_BLOCKERS_MASK_COMBINATIONS] =
    [0; MAX_ROOK_BLOCKERS_MASK_COMBINATIONS];

  let relevant_squares = ROOK_SPAN_WITHOUT_EDGES[square];
  let n = relevant_squares.count_ones();
  assert_eq!(ROOK_BLOCKER_NUMBERS[square], n as u8);
  let blocker_combinations = 1 << n;
  println!("{n} relevant squares, {blocker_combinations} blocker combinations");

  // Assemble the combinations of possible blockers for square `square`
  for b in 0..blocker_combinations {
    let mut blocker_mask: BoardMask = 0;
    for j in 0..n as usize {
      assert!(ROOK_SPAN_INDEXES[square][j] != 255);
      if b & (1 << j) != 0 {
        blocker_mask |= 1 << ROOK_SPAN_INDEXES[square][j];
      }
    }
    blockers[b] = blocker_mask;
  }

  for _k in 0..100_000_000 {
    // bitwise AND on 3 times random to get a random number with few bits set to 1.
    let magic = rand::random::<u64>() & rand::random::<u64>() & rand::random::<u64>();

    if ((relevant_squares.wrapping_mul(magic)) & 0xFF00000000000000).count_ones() < 6 {
      continue;
    }

    // Now test if our magic number works well with all combinations
    for i in 0..MAX_ROOK_BLOCKERS_MASK_COMBINATIONS {
      used[i] = 0;
    }

    let mut fail = false;
    for b in 0..blocker_combinations {
      let j: usize =
        (blockers[b].wrapping_mul(magic) >> (64 - ROOK_BLOCKER_NUMBERS[square])) as usize;

      let rook_destinations =
        get_moves_from_offsets(&ROOK_MOVE_OFFSETS, true, 0, blockers[b], square);
      assert_ne!(rook_destinations, 0);
      if used[j] == 0 {
        used[j] = rook_destinations;
      } else if used[j] != rook_destinations {
        fail = true;
        break;
      }
    }

    if !fail {
      println!("Success for square: {square} - magic value {magic}\n");
      return magic;
    }
  }

  println!("Failed for square: {square}\n");
  panic!("Do not use this result!!");
}
