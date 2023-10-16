use rand::Rng;
use std::fs::File;
use std::io::Write;

use chess::model::board::INVALID_SQUARE;
use chess::model::board_geometry::*;
use chess::model::board_mask::*;
use chess::model::piece_moves::*;

// This is where our definitions are exported at the end
use chess::model::tables::bishop_destinations::*;

fn main() {
  let filename = "bishop_table.rs";
  println!("-----------------------------------------------------------------");
  println!("Generating bishop tables in {}...", filename);
  let mut output_file = File::create(filename).unwrap();

  // First the Bishop span, with and without edges (constants)
  let mut bishop_span: [u64; 64] = [0; 64];
  let mut bishop_span_without_edges: [u64; 64] = [0; 64];
  for i in 0..64 {
    bishop_span[i] = get_moves_from_offsets(&BISHOP_MOVE_OFFSETS, true, 0, 0, i);
    bishop_span_without_edges[i] = BOARD_WITHOUT_EDGES & bishop_span[i];
  }

  let _ = write!(
    output_file,
    "/// Array of BoardMasks indicating where the bishop can reach\n/// if there were no other pieces on the board\n///\npub const BISHOP_SPAN:[u64; 64] = {:#018X?};\n\n",
    bishop_span
  );
  let _ = write!(
    output_file,
    "/// Array of BoardMasks indicating where the bishop can reach\n/// if there were no other pieces on the board, removing the edges\n///\npub const BISHOP_SPAN_WITHOUT_EDGES:[u64; 64] = {:#018X?};\n\n",
    bishop_span_without_edges
  );

  // Then we calculate indices of relevant bits went usingl as bishop spans without edges.
  // Then the indices of the bits on the bishop span without those edges
  let mut bishop_span_indexes: [[usize; 9]; 64] = [[INVALID_SQUARE.into(); 9]; 64];
  // How many bits are describing relevant blockers based on the position
  let mut bishop_blocker_numbers: [u8; 64] = [64; 64];

  for i in 0..64 {
    let mut index = 0;
    for j in 0..64 {
      if !square_in_mask!(j, bishop_span_without_edges[i]) {
        continue;
      }
      bishop_span_indexes[i][index] = j;
      index += 1;
    }
    bishop_blocker_numbers[i] = index as u8;
  }

  let _ = write!(
    output_file,"/// For a given position, this table indicate the BoardMasks indices of\n/// possible blockers for the BISHOP_SPAN.\n///\n///\n");
  let _ = write!(
    output_file,
    "pub const BISHOP_SPAN_INDEXES: [[usize; 9]; 64] = {:#?};\n\n",
    bishop_span_indexes
  );
  let _ = write!(
      output_file,"/// For a given position, this table indicate the Number of relevant blockers bits for a bishop\n///\n");
  let _ = write!(
    output_file,
    "pub const BISHOP_BLOCKER_NUMBERS: [u8; 64] = {:#?};\n\n",
    bishop_blocker_numbers
  );

  // Now we want to find these bishop magic constants
  let mut bishop_magic: [u64; 64] = [0; 64];

  for i in 0..64 {
    println!("Searching magic for square {i}");
    bishop_magic[i] = find_bishop_magic(i);
  }

  let _ = write!(output_file, "/// Bishop Magic Numbers\n///\n");
  let _ = write!(
    output_file,
    "pub const BISHOP_MAGIC:[u64; 64] = {:#018X?};\n\n",
    bishop_magic
  );

  // Now we pre-compute the list of destinations for all blocker masks
  let mut bishop_destination_table: [[u64; MAX_BISHOP_BLOCKERS_MASK_COMBINATIONS]; 64] =
    [[0; MAX_BISHOP_BLOCKERS_MASK_COMBINATIONS]; 64];

  for i in 0..64 {
    let mut blockers: [u64; MAX_BISHOP_BLOCKERS_MASK_COMBINATIONS] =
      [0; MAX_BISHOP_BLOCKERS_MASK_COMBINATIONS];
    let blocker_combinations = 1 << bishop_span_without_edges[i].count_ones();

    // Assemble the combinations of possible blockers for square `i`
    for b in 0..blocker_combinations {
      let mut blocker_mask: BoardMask = 0;
      for j in 0..bishop_span_without_edges[i].count_ones() as usize {
        assert!(bishop_span_indexes[i][j] != 255);
        if b & (1 << j) != 0 {
          blocker_mask |= 1 << bishop_span_indexes[i][j];
        }
      }
      blockers[b] = blocker_mask;
    }

    for b in 0..blocker_combinations {
      let j: usize =
        (blockers[b].wrapping_mul(bishop_magic[i]) >> (64 - bishop_blocker_numbers[i])) as usize;

      if bishop_destination_table[i][j] == 0 {
        bishop_destination_table[i][j] =
          get_moves_from_offsets(&BISHOP_MOVE_OFFSETS, true, 0, blockers[b], i);
      } else if bishop_destination_table[i][j]
        != get_moves_from_offsets(&BISHOP_MOVE_OFFSETS, true, 0, blockers[b], i)
      {
        println!("Oh oh... square: {i} blocker {b}, derived index is {j} for blocker mask:");
        print_board_mask(blockers[b]);
        println!("Look up table says:");
        print_board_mask(bishop_destination_table[i][j]);
        println!("while manual calculation says:");
        print_board_mask(get_moves_from_offsets(
          &BISHOP_MOVE_OFFSETS,
          true,
          0,
          blockers[b],
          i,
        ));
        println!(
          "Wrapping mul (shift is {}):",
          (64 - bishop_blocker_numbers[i])
        );
        print_board_mask(blockers[b].wrapping_mul(bishop_magic[i]));
        print_board_mask(
          (blockers[b].wrapping_mul(bishop_magic[i])) >> (64 - bishop_blocker_numbers[i]),
        );
        panic!("Do not use this result!");
      }
    }
  }

  // test sanity:
  let mut rng = rand::thread_rng();
  for _ in 0..1000 {
    let blockers = rand::random::<u64>();
    let square = rng.gen_range(0..64);

    let manual_calculation =
      get_moves_from_offsets(&BISHOP_MOVE_OFFSETS, true, 0, blockers, square);
    let index: usize = ((blockers & bishop_span_without_edges[square])
      .wrapping_mul(bishop_magic[square])
      >> (64 - bishop_blocker_numbers[square])) as usize;
    let look_up_table = bishop_destination_table[square][index];

    println!("Result for blockers from square {square} - looked up index {index}:");
    print_board_mask(blockers & bishop_span_without_edges[square]);
    print_board_mask(manual_calculation);
    print_board_mask(look_up_table);
    assert_eq!(manual_calculation, look_up_table);
  }

  /*
   */
  println!("");
  println!("Done! 🙂");
}

// Same as for the rooks
fn find_bishop_magic(square: usize) -> BoardMask {
  let mut used: [BoardMask; MAX_BISHOP_BLOCKERS_MASK_COMBINATIONS] =
    [0; MAX_BISHOP_BLOCKERS_MASK_COMBINATIONS];
  let mut blockers: [BoardMask; MAX_BISHOP_BLOCKERS_MASK_COMBINATIONS] =
    [0; MAX_BISHOP_BLOCKERS_MASK_COMBINATIONS];

  let relevant_squares = BISHOP_SPAN_WITHOUT_EDGES[square];
  let n = relevant_squares.count_ones();
  assert_eq!(BISHOP_BLOCKER_NUMBERS[square], n as u8);
  let blocker_combinations = 1 << n;
  println!("{n} relevant squares, {blocker_combinations} blocker combinations");

  // Assemble the combinations of possible blockers for square `square`
  for b in 0..blocker_combinations {
    let mut blocker_mask: BoardMask = 0;
    for j in 0..n as usize {
      assert!(BISHOP_SPAN_INDEXES[square][j] != 255);
      if b & (1 << j) != 0 {
        blocker_mask |= 1 << BISHOP_SPAN_INDEXES[square][j];
      }
    }
    blockers[b] = blocker_mask;
  }

  for _k in 0..100_000_000 {
    // bitwise AND on 3 times random to get a random number with few bits set to 1.
    let magic: BoardMask = rand::random::<u64>() & rand::random::<u64>() & rand::random::<u64>();

    if ((relevant_squares.wrapping_mul(magic)) & 0xFF00000000000000).count_ones() < 6 {
      continue;
    }

    // Now test if our magic number works well with all combinations
    for i in 0..MAX_BISHOP_BLOCKERS_MASK_COMBINATIONS {
      used[i] = 0;
    }

    let mut fail = false;
    for b in 0..blocker_combinations {
      let j: usize =
        (blockers[b].wrapping_mul(magic) >> (64 - BISHOP_BLOCKER_NUMBERS[square])) as usize;

      let bishop_destinations =
        get_moves_from_offsets(&BISHOP_MOVE_OFFSETS, true, 0, blockers[b], square);
      assert_ne!(bishop_destinations, 0);
      if used[j] == 0 {
        used[j] = bishop_destinations;
      } else if used[j] != bishop_destinations {
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
