use crate::model::board_mask::BoardMask;
use crate::model::piece_moves::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;
// -----------------------------------------------------------------------------
// Static variables used to store our calculations
static mut rook_table: [[u64; MAX_ROOK_BLOCKERS_MASK_COMBINATIONS]; 64] =
  [[0; MAX_ROOK_BLOCKERS_MASK_COMBINATIONS]; 64];
static mut rook_masks: [[u64; MAX_ROOK_BLOCKERS_MASK_COMBINATIONS]; 64] =
  [[0; MAX_ROOK_BLOCKERS_MASK_COMBINATIONS]; 64];
static mut key_map: Lazy<HashMap<(usize, u64), u64>> = Lazy::new(|| HashMap::new());
static mut tables_initialized: bool = false;

/// Number of combinations of blockers in a rook span
pub const MAX_ROOK_BLOCKERS: usize = 12;
pub const MAX_ROOK_BLOCKERS_MASK_COMBINATIONS: usize = 1 << MAX_ROOK_BLOCKERS;

/// Array of BoardMasks indicating where the rook can reach if there were
/// no other pieces on the board
/// Edges are included.
///
pub const ROOK_SPAN: [u64; 64] = [
  0x01010101010101FE,
  0x02020202020202FD,
  0x04040404040404FB,
  0x08080808080808F7,
  0x10101010101010EF,
  0x20202020202020DF,
  0x40404040404040BF,
  0x808080808080807F,
  0x010101010101FE01,
  0x020202020202FD02,
  0x040404040404FB04,
  0x080808080808F708,
  0x101010101010EF10,
  0x202020202020DF20,
  0x404040404040BF40,
  0x8080808080807F80,
  0x0101010101FE0101,
  0x0202020202FD0202,
  0x0404040404FB0404,
  0x0808080808F70808,
  0x1010101010EF1010,
  0x2020202020DF2020,
  0x4040404040BF4040,
  0x80808080807F8080,
  0x01010101FE010101,
  0x02020202FD020202,
  0x04040404FB040404,
  0x08080808F7080808,
  0x10101010EF101010,
  0x20202020DF202020,
  0x40404040BF404040,
  0x808080807F808080,
  0x010101FE01010101,
  0x020202FD02020202,
  0x040404FB04040404,
  0x080808F708080808,
  0x101010EF10101010,
  0x202020DF20202020,
  0x404040BF40404040,
  0x8080807F80808080,
  0x0101FE0101010101,
  0x0202FD0202020202,
  0x0404FB0404040404,
  0x0808F70808080808,
  0x1010EF1010101010,
  0x2020DF2020202020,
  0x4040BF4040404040,
  0x80807F8080808080,
  0x01FE010101010101,
  0x02FD020202020202,
  0x04FB040404040404,
  0x08F7080808080808,
  0x10EF101010101010,
  0x20DF202020202020,
  0x40BF404040404040,
  0x807F808080808080,
  0xFE01010101010101,
  0xFD02020202020202,
  0xFB04040404040404,
  0xF708080808080808,
  0xEF10101010101010,
  0xDF20202020202020,
  0xBF40404040404040,
  0x7F80808080808080,
];

/// Array of BoardMasks indicating where the rook can reach if there were
/// no other pieces on the board
/// Edges are not included.
///
pub const ROOK_SPAN_WITHOUT_EDGES: [u64; 64] = [
  0x000101010101017E,
  0x000202020202027C,
  0x000404040404047A,
  0x0008080808080876,
  0x001010101010106E,
  0x002020202020205E,
  0x004040404040403E,
  0x008080808080807E,
  0x0001010101017E00,
  0x0002020202027C00,
  0x0004040404047A00,
  0x0008080808087600,
  0x0010101010106E00,
  0x0020202020205E00,
  0x0040404040403E00,
  0x0080808080807E00,
  0x00010101017E0100,
  0x00020202027C0200,
  0x00040404047A0400,
  0x0008080808760800,
  0x00101010106E1000,
  0x00202020205E2000,
  0x00404040403E4000,
  0x00808080807E8000,
  0x000101017E010100,
  0x000202027C020200,
  0x000404047A040400,
  0x0008080876080800,
  0x001010106E101000,
  0x002020205E202000,
  0x004040403E404000,
  0x008080807E808000,
  0x0001017E01010100,
  0x0002027C02020200,
  0x0004047A04040400,
  0x0008087608080800,
  0x0010106E10101000,
  0x0020205E20202000,
  0x0040403E40404000,
  0x0080807E80808000,
  0x00017E0101010100,
  0x00027C0202020200,
  0x00047A0404040400,
  0x0008760808080800,
  0x00106E1010101000,
  0x00205E2020202000,
  0x00403E4040404000,
  0x00807E8080808000,
  0x007E010101010100,
  0x007C020202020200,
  0x007A040404040400,
  0x0076080808080800,
  0x006E101010101000,
  0x005E202020202000,
  0x003E404040404000,
  0x007E808080808000,
  0x7E01010101010100,
  0x7C02020202020200,
  0x7A04040404040400,
  0x7608080808080800,
  0x6E10101010101000,
  0x5E20202020202000,
  0x3E40404040404000,
  0x7E80808080808000,
];

/// For a given position, this table indicate the BoardMasks indices of
/// possible blockers for the ROOK_SPAN.
///
/// Blockers on the edge of the board don't matter, because we capture them and
/// have to stop anyway.
///
///
pub const ROOK_SPAN_INDEXES: [[usize; 12]; 64] = [
  [1, 2, 3, 4, 5, 6, 8, 16, 24, 32, 40, 48],
  [2, 3, 4, 5, 6, 9, 17, 25, 33, 41, 49, 255],
  [1, 3, 4, 5, 6, 10, 18, 26, 34, 42, 50, 255],
  [1, 2, 4, 5, 6, 11, 19, 27, 35, 43, 51, 255],
  [1, 2, 3, 5, 6, 12, 20, 28, 36, 44, 52, 255],
  [1, 2, 3, 4, 6, 13, 21, 29, 37, 45, 53, 255],
  [1, 2, 3, 4, 5, 14, 22, 30, 38, 46, 54, 255],
  [1, 2, 3, 4, 5, 6, 15, 23, 31, 39, 47, 55],
  [9, 10, 11, 12, 13, 14, 16, 24, 32, 40, 48, 255],
  [10, 11, 12, 13, 14, 17, 25, 33, 41, 49, 255, 255],
  [9, 11, 12, 13, 14, 18, 26, 34, 42, 50, 255, 255],
  [9, 10, 12, 13, 14, 19, 27, 35, 43, 51, 255, 255],
  [9, 10, 11, 13, 14, 20, 28, 36, 44, 52, 255, 255],
  [9, 10, 11, 12, 14, 21, 29, 37, 45, 53, 255, 255],
  [9, 10, 11, 12, 13, 22, 30, 38, 46, 54, 255, 255],
  [9, 10, 11, 12, 13, 14, 23, 31, 39, 47, 55, 255],
  [8, 17, 18, 19, 20, 21, 22, 24, 32, 40, 48, 255],
  [9, 18, 19, 20, 21, 22, 25, 33, 41, 49, 255, 255],
  [10, 17, 19, 20, 21, 22, 26, 34, 42, 50, 255, 255],
  [11, 17, 18, 20, 21, 22, 27, 35, 43, 51, 255, 255],
  [12, 17, 18, 19, 21, 22, 28, 36, 44, 52, 255, 255],
  [13, 17, 18, 19, 20, 22, 29, 37, 45, 53, 255, 255],
  [14, 17, 18, 19, 20, 21, 30, 38, 46, 54, 255, 255],
  [15, 17, 18, 19, 20, 21, 22, 31, 39, 47, 55, 255],
  [8, 16, 25, 26, 27, 28, 29, 30, 32, 40, 48, 255],
  [9, 17, 26, 27, 28, 29, 30, 33, 41, 49, 255, 255],
  [10, 18, 25, 27, 28, 29, 30, 34, 42, 50, 255, 255],
  [11, 19, 25, 26, 28, 29, 30, 35, 43, 51, 255, 255],
  [12, 20, 25, 26, 27, 29, 30, 36, 44, 52, 255, 255],
  [13, 21, 25, 26, 27, 28, 30, 37, 45, 53, 255, 255],
  [14, 22, 25, 26, 27, 28, 29, 38, 46, 54, 255, 255],
  [15, 23, 25, 26, 27, 28, 29, 30, 39, 47, 55, 255],
  [8, 16, 24, 33, 34, 35, 36, 37, 38, 40, 48, 255],
  [9, 17, 25, 34, 35, 36, 37, 38, 41, 49, 255, 255],
  [10, 18, 26, 33, 35, 36, 37, 38, 42, 50, 255, 255],
  [11, 19, 27, 33, 34, 36, 37, 38, 43, 51, 255, 255],
  [12, 20, 28, 33, 34, 35, 37, 38, 44, 52, 255, 255],
  [13, 21, 29, 33, 34, 35, 36, 38, 45, 53, 255, 255],
  [14, 22, 30, 33, 34, 35, 36, 37, 46, 54, 255, 255],
  [15, 23, 31, 33, 34, 35, 36, 37, 38, 47, 55, 255],
  [8, 16, 24, 32, 41, 42, 43, 44, 45, 46, 48, 255],
  [9, 17, 25, 33, 42, 43, 44, 45, 46, 49, 255, 255],
  [10, 18, 26, 34, 41, 43, 44, 45, 46, 50, 255, 255],
  [11, 19, 27, 35, 41, 42, 44, 45, 46, 51, 255, 255],
  [12, 20, 28, 36, 41, 42, 43, 45, 46, 52, 255, 255],
  [13, 21, 29, 37, 41, 42, 43, 44, 46, 53, 255, 255],
  [14, 22, 30, 38, 41, 42, 43, 44, 45, 54, 255, 255],
  [15, 23, 31, 39, 41, 42, 43, 44, 45, 46, 55, 255],
  [8, 16, 24, 32, 40, 49, 50, 51, 52, 53, 54, 255],
  [9, 17, 25, 33, 41, 50, 51, 52, 53, 54, 255, 255],
  [10, 18, 26, 34, 42, 49, 51, 52, 53, 54, 255, 255],
  [11, 19, 27, 35, 43, 49, 50, 52, 53, 54, 255, 255],
  [12, 20, 28, 36, 44, 49, 50, 51, 53, 54, 255, 255],
  [13, 21, 29, 37, 45, 49, 50, 51, 52, 54, 255, 255],
  [14, 22, 30, 38, 46, 49, 50, 51, 52, 53, 255, 255],
  [15, 23, 31, 39, 47, 49, 50, 51, 52, 53, 54, 255],
  [8, 16, 24, 32, 40, 48, 57, 58, 59, 60, 61, 62],
  [9, 17, 25, 33, 41, 49, 58, 59, 60, 61, 62, 255],
  [10, 18, 26, 34, 42, 50, 57, 59, 60, 61, 62, 255],
  [11, 19, 27, 35, 43, 51, 57, 58, 60, 61, 62, 255],
  [12, 20, 28, 36, 44, 52, 57, 58, 59, 61, 62, 255],
  [13, 21, 29, 37, 45, 53, 57, 58, 59, 60, 62, 255],
  [14, 22, 30, 38, 46, 54, 57, 58, 59, 60, 61, 255],
  [15, 23, 31, 39, 47, 55, 57, 58, 59, 60, 61, 62],
];

pub fn init_rook_table() {
  if unsafe { tables_initialized } {
    return;
  }

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
        key_map.insert((i, blocker_mask), blockers as u64);
      }
    }
  }

  unsafe {
    tables_initialized = true;
  }
}

pub fn get_rook_destinations(
  same_side_pieces: BoardMask,
  opponent_pieces: BoardMask,
  square: usize,
) -> BoardMask {
  init_rook_table();
  // FIXME: This just does not work... and is slower than the manual search.
  //println!("-----------------------------------------------------------------");
  //println!("Get gook destinations for square {square}");
  //print_board_mask(opponent_pieces);
  let blockers = opponent_pieces & ROOK_SPAN[square];
  //println!("Blockers:");
  //print_board_mask(blockers);

  let blockers_key = unsafe { key_map[&(square, blockers)] } as usize;
  //println!("Blockers key: {blockers_key}");

  unsafe { rook_table[square][blockers_key] & !same_side_pieces }
}
