pub mod diagonals;
pub mod holes;
pub mod lines;
pub mod passed_pawns_areas;
pub mod rays;

use crate::model::board::*;
use crate::model::board_mask::BoardMask;
use std::cmp::max;

// -----------------------------------------------------------------------------
//  Constants

pub static BOARD_EDGES: BoardMask = 0xFF818181818181FF;
pub static BOARD_WITHOUT_EDGES: BoardMask = 0x007E7E7E7E7E7E00;
pub static BOARD_RIGHT_EDGE: BoardMask = 0x8080808080808080;
pub static BOARD_LEFT_EDGE: BoardMask = 0x0101010101010101;
pub static BOARD_DOWN_EDGE: BoardMask = 0x00000000000000FF;
pub static BOARD_BOTTOM_2_RANKS: BoardMask = 0x000000000000FFFF;
pub static BOARD_UP_EDGE: BoardMask = 0xFF00000000000000;
pub static BOARD_UPPER_2_RANKS: BoardMask = 0xFFFF000000000000;
/// Queen Side mask
pub static QUEEN_SIDE_MASK: BoardMask = 0x0F0F0F0F0F0F0F0F;

/// King Side mask
pub static KING_SIDE_MASK: BoardMask = 0xF0F0F0F0F0F0F0F0;

/// Ranks boardmasks
pub static RANKS: [u64; 8] = [0x00000000000000FF,
                              0x000000000000FF00,
                              0x0000000000FF0000,
                              0x00000000FF000000,
                              0x000000FF00000000,
                              0x0000FF0000000000,
                              0x00FF000000000000,
                              0xFF00000000000000];
/// Files boardmasks
pub static FILES: [u64; 8] = [0x0101010101010101,
                              0x0202020202020202,
                              0x0404040404040404,
                              0x0808080808080808,
                              0x1010101010101010,
                              0x2020202020202020,
                              0x4040404040404040,
                              0x8080808080808080];

/// Finds how many moves/squares away is a king from a particular square,
/// regardless of "obstacle pieces" along the way
///
/// # Arguments
///
/// * `king_position` - Square index where the king is located.
/// * `destination`   - Square index where the king wants to go.
///
/// # Return value
///
/// The number of moves the king needs to do to reach the destination. With
/// valid square indexes, it will be in [0..7]
pub fn get_king_distance(king_position: u8, destination: u8) -> u8 {
  let (sf, sr) = Board::index_to_fr(king_position);
  let (df, dr) = Board::index_to_fr(destination);

  // basically the distance will be the max of file/rank difference.
  max(sf.abs_diff(df), sr.abs_diff(dr))
}

// -----------------------------------------------------------------------------
//  Tests

#[cfg(test)]
mod tests {
  use super::*;
  use crate::model::board_mask::print_board_mask;
  use crate::{set_square_in_mask, square_in_mask};
  #[test]
  fn check_king_distance() {
    let start = Board::fr_to_index(4, 3);
    let destination = Board::fr_to_index(6, 6);
    assert_eq!(get_king_distance(start, destination), 3);
    assert_eq!(get_king_distance(destination, start), 3);

    let start = Board::fr_to_index(1, 1);
    let destination = Board::fr_to_index(8, 8);
    assert_eq!(get_king_distance(start, destination), 7);
    assert_eq!(get_king_distance(destination, start), 7);

    let start = Board::fr_to_index(7, 1);
    let destination = Board::fr_to_index(8, 8);
    assert_eq!(get_king_distance(start, destination), 7);
    assert_eq!(get_king_distance(destination, start), 7);

    let start = Board::fr_to_index(7, 1);
    let destination = Board::fr_to_index(8, 8);
    assert_eq!(get_king_distance(start, destination), 7);
    assert_eq!(get_king_distance(destination, start), 7);

    let start = Board::fr_to_index(1, 2);
    let destination = Board::fr_to_index(1, 3);
    assert_eq!(get_king_distance(start, destination), 1);
    assert_eq!(get_king_distance(destination, start), 1);

    let start = Board::fr_to_index(3, 2);
    let destination = Board::fr_to_index(3, 2);
    assert_eq!(get_king_distance(start, destination), 0);
    assert_eq!(get_king_distance(destination, start), 0);
  }

  #[test]
  fn generate_lines_between_squares() {
    use crate::model::board_mask::BoardMask;
    use crate::model::moves::*;
    use crate::model::piece_moves::*;
    use std::fs::File;
    use std::io::Write;

    // Here we generate look-up boardmasks that indicate a line between two squares.
    // Start from i, continue in a straight line like rooks.
    let mut lines: [[u64; 64]; 64] = [[0; 64]; 64];
    for i in 0..64 {
      for j in 0..64 {
        let inital_rank = (i / 8) as isize;
        let inital_file = (i % 8) as isize;
        let mut mask: BoardMask = 0;
        let mut destination_reached = false;
        for (file_offset, rank_offset) in ROOK_MOVE_OFFSETS {
          if destination_reached {
            break;
          }
          let mut rank = inital_rank;
          let mut file = inital_file;
          mask = 0;
          // Each move can be repeated until we meet a piece or fall of the board:
          loop {
            rank += rank_offset;
            file += file_offset;

            // Did we go too far ?
            fr_bounds_or_break!(file, rank);

            let current_square: u64 = 1 << (rank * 8 + file);
            mask |= current_square;
            if current_square & (1 << j) != 0 {
              destination_reached = true;
              break;
            }
          }
        }

        if destination_reached {
          lines[i][j] = mask;
        }
      }
    }

    println!("Generating line constants...",);
    let mut output_file = File::create("./model/board_geometry/lines.rs").unwrap();
    let _ = writeln!(output_file, "/// Represents boardmasks of lines between i and j squares");
    let _ = writeln!(output_file, "/// i not included, j included.");
    let _ = write!(output_file, "pub static LINES: [[u64; 64]; 64]  = {:#018X?};", lines);

    println!("a1 -> b2");
    print_board_mask(lines[string_to_square("a1") as usize][string_to_square("b2") as usize]);
    println!("c4 -> g4");
    print_board_mask(lines[string_to_square("c4") as usize][string_to_square("g4") as usize]);
    print_board_mask(lines[string_to_square("g4") as usize][string_to_square("c4") as usize]);
  }

  #[test]
  fn generate_diagonals_between_squares() {
    use crate::model::board_mask::BoardMask;
    use crate::model::moves::*;
    use crate::model::piece_moves::*;
    use std::fs::File;
    use std::io::Write;

    println!("Generating diagonal constants...",);
    let mut output_file = File::create("./model/board_geometry/diagonals.rs").unwrap();
    // Here we generate look-up boardmasks that indicate a diagonal between two
    // squares. Start from i, continue in a diagonal like bishops.
    let mut diagonals: [[u64; 64]; 64] = [[0; 64]; 64];
    for i in 0..64 {
      for j in 0..64 {
        let inital_rank = (i / 8) as isize;
        let inital_file = (i % 8) as isize;
        let mut mask: BoardMask = 0;
        let mut destination_reached = false;
        for (file_offset, rank_offset) in BISHOP_MOVE_OFFSETS {
          if destination_reached {
            break;
          }
          let mut rank = inital_rank;
          let mut file = inital_file;
          mask = 0;
          // Each move can be repeated until we meet a piece or fall of the board:
          loop {
            rank += rank_offset;
            file += file_offset;

            // Did we go too far ?
            fr_bounds_or_break!(file, rank);

            let current_square: u64 = 1 << (rank * 8 + file);
            mask |= current_square;
            if current_square & (1 << j) != 0 {
              destination_reached = true;
              break;
            }
          }
        }

        if destination_reached {
          diagonals[i][j] = mask;
        }
      }
    }

    let _ = writeln!(output_file, "/// Represents boardmasks of diagonals between i and j squares");
    let _ = writeln!(output_file, "/// i not included, j included.");

    let _ = write!(output_file, "pub static DIAGONALS: [[u64; 64]; 64]  = {:#018X?};", diagonals);

    println!("a1 -> b2");
    print_board_mask(diagonals[string_to_square("a1") as usize][string_to_square("b2") as usize]);
    println!("h7 -> d3");
    print_board_mask(diagonals[string_to_square("h7") as usize][string_to_square("d3") as usize]);

    println!("c4 -> g4");
    print_board_mask(diagonals[string_to_square("c4") as usize][string_to_square("g4") as usize]);
    print_board_mask(diagonals[string_to_square("g4") as usize][string_to_square("c4") as usize]);
  }

  #[ignore]
  #[test]
  fn generate_rays_between_squares() {
    use crate::model::board_mask::BoardMask;
    use crate::model::moves::*;
    use crate::model::piece_moves::*;
    use std::fs::File;
    use std::io::Write;

    println!("Generating lines and diagonal constants...",);
    let mut output_file = File::create("./model/board_geometry/rays.rs").unwrap();
    // Here we generate look-up boardmasks that indicate a diagonal between two
    // squares. Start from i, continue in a diagonal like bishops.
    let mut rays: [[u64; 64]; 64] = [[0; 64]; 64];
    for i in 0..64 {
      for j in 0..64 {
        let inital_rank = (i / 8) as isize;
        let inital_file = (i % 8) as isize;
        let mut mask: BoardMask = 0;
        let mut destination_reached = false;
        for (file_offset, rank_offset) in BISHOP_MOVE_OFFSETS {
          if destination_reached {
            break;
          }
          let mut rank = inital_rank;
          let mut file = inital_file;
          mask = 0;
          // Each move can be repeated until we meet a piece or fall of the board:
          loop {
            rank += rank_offset;
            file += file_offset;

            // Did we go too far ?
            fr_bounds_or_break!(file, rank);

            let current_square: u64 = 1 << (rank * 8 + file);
            mask |= current_square;
            if current_square & (1 << j) != 0 {
              destination_reached = true;
              break;
            }
          }
        }

        if destination_reached {
          rays[i][j] = mask;
          continue;
        }

        mask = 0;
        for (file_offset, rank_offset) in ROOK_MOVE_OFFSETS {
          if destination_reached {
            break;
          }
          let mut rank = inital_rank;
          let mut file = inital_file;
          mask = 0;
          // Each move can be repeated until we meet a piece or fall of the board:
          loop {
            rank += rank_offset;
            file += file_offset;

            // Did we go too far ?
            fr_bounds_or_break!(file, rank);

            let current_square: u64 = 1 << (rank * 8 + file);
            mask |= current_square;
            if current_square & (1 << j) != 0 {
              destination_reached = true;
              break;
            }
          }
        }

        if destination_reached {
          rays[i][j] = mask;
        }
      }
    }

    let _ = writeln!(output_file,
                     "/// Represents boardmasks of lines and diagonals between i and j squares");
    let _ = writeln!(output_file, "/// i not included, j included.");
    let _ = writeln!(output_file, "/// use like this: `RAYS[i][j]`.");

    let _ = write!(output_file, "pub static RAYS: [[u64; 64]; 64]  = {:#018X?};", rays);

    println!("a1 -> b2");
    print_board_mask(rays[string_to_square("a1") as usize][string_to_square("b2") as usize]);
    println!("h7 -> d3");
    print_board_mask(rays[string_to_square("h7") as usize][string_to_square("d3") as usize]);

    println!("c4 -> g4");
    print_board_mask(rays[string_to_square("c4") as usize][string_to_square("g4") as usize]);
    print_board_mask(rays[string_to_square("g4") as usize][string_to_square("c4") as usize]);
  }

  #[ignore]
  #[test]
  fn generale_holes_board_area() {
    use crate::model::board_mask::BoardMask;
    use crate::model::moves::*;
    use std::fs::File;
    use std::io::Write;

    println!("Generating holes constants...",);
    let mut output_file = File::create("./model/board_geometry/holes_temp.rs").unwrap();

    let mut holes_area: BoardMask = 0;
    for i in 0..64 {
      let (_, rank) = Board::index_to_fr(i);
      if rank == 1 || rank == 2 || rank == 7 || rank == 8 {
        continue;
      }

      set_square_in_mask!(i, holes_area);
    }

    let _ = writeln!(output_file, "/// Represents boardmask of area where there can be holes");

    let _ = write!(output_file, "pub static HOLES_BOARD_AREA: BoardMask = {:#018X?};", holes_area);

    let mut hole_white_pawns: [u64; 64] = [0; 64];
    for i in 0..64 {
      if !square_in_mask!(i, holes_area) {
        continue;
      };

      let (file, mut rank) = Board::index_to_fr(i);

      while rank != 1 {
        rank -= 1;

        // Check on the left side:
        if file > 1 {
          let s = Board::fr_to_index(file - 1, rank);
          set_square_in_mask!(s, hole_white_pawns[i as usize]);
        }
        if file < 8 {
          let s = Board::fr_to_index(file + 1, rank);
          set_square_in_mask!(s, hole_white_pawns[i as usize]);
        }
      }
    }

    print_board_mask(hole_white_pawns[string_to_square("e4") as usize]);
    print_board_mask(hole_white_pawns[string_to_square("h5") as usize]);

    let _ = write!(output_file,
                   "\n\npub static HOLES_WHITE_PAWN_PLACEMENT: [u64; 64] = {:#018X?};",
                   hole_white_pawns);

    let mut hole_black_pawns: [u64; 64] = [0; 64];
    for i in 0..64 {
      if !square_in_mask!(i, holes_area) {
        continue;
      };

      let (file, mut rank) = Board::index_to_fr(i);

      while rank != 8 {
        rank += 1;

        // Check on the left side:
        if file > 1 {
          let s = Board::fr_to_index(file - 1, rank);
          set_square_in_mask!(s, hole_black_pawns[i as usize]);
        }
        if file < 8 {
          let s = Board::fr_to_index(file + 1, rank);
          set_square_in_mask!(s, hole_black_pawns[i as usize]);
        }
      }
    }

    print_board_mask(hole_black_pawns[string_to_square("e4") as usize]);
    print_board_mask(hole_black_pawns[string_to_square("h5") as usize]);

    let _ = write!(output_file,
                   "\n\npub static HOLES_BLACK_PAWN_PLACEMENT: [u64; 64] = {:#018X?};",
                   hole_black_pawns);
  }
}
