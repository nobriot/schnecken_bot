use crate::model::board_mask::BoardMask;
use crate::model::piece_moves::{get_moves_from_offsets, BISHOP_MOVE_OFFSETS};

static mut BISHOP_TABLE_INITIALIZED: bool = false;
static mut BISHOP_DESTINATION_TABLE: [[u64; MAX_BISHOP_BLOCKERS_MASK_COMBINATIONS]; 64] =
  [[0; MAX_BISHOP_BLOCKERS_MASK_COMBINATIONS]; 64];

/// Number of combinations of blockers in a bishop span
pub const MAX_BISHOP_BLOCKERS: usize = 9;
pub const MAX_BISHOP_BLOCKERS_MASK_COMBINATIONS: usize = 1 << MAX_BISHOP_BLOCKERS;

/// Array of BoardMasks indicating where the bishop can reach
/// if there were no other pieces on the board
///
pub const BISHOP_SPAN: [u64; 64] = [
  0x8040201008040200,
  0x0080402010080500,
  0x0000804020110A00,
  0x0000008041221400,
  0x0000000182442800,
  0x0000010204885000,
  0x000102040810A000,
  0x0102040810204000,
  0x4020100804020002,
  0x8040201008050005,
  0x00804020110A000A,
  0x0000804122140014,
  0x0000018244280028,
  0x0001020488500050,
  0x0102040810A000A0,
  0x0204081020400040,
  0x2010080402000204,
  0x4020100805000508,
  0x804020110A000A11,
  0x0080412214001422,
  0x0001824428002844,
  0x0102048850005088,
  0x02040810A000A010,
  0x0408102040004020,
  0x1008040200020408,
  0x2010080500050810,
  0x4020110A000A1120,
  0x8041221400142241,
  0x0182442800284482,
  0x0204885000508804,
  0x040810A000A01008,
  0x0810204000402010,
  0x0804020002040810,
  0x1008050005081020,
  0x20110A000A112040,
  0x4122140014224180,
  0x8244280028448201,
  0x0488500050880402,
  0x0810A000A0100804,
  0x1020400040201008,
  0x0402000204081020,
  0x0805000508102040,
  0x110A000A11204080,
  0x2214001422418000,
  0x4428002844820100,
  0x8850005088040201,
  0x10A000A010080402,
  0x2040004020100804,
  0x0200020408102040,
  0x0500050810204080,
  0x0A000A1120408000,
  0x1400142241800000,
  0x2800284482010000,
  0x5000508804020100,
  0xA000A01008040201,
  0x4000402010080402,
  0x0002040810204080,
  0x0005081020408000,
  0x000A112040800000,
  0x0014224180000000,
  0x0028448201000000,
  0x0050880402010000,
  0x00A0100804020100,
  0x0040201008040201,
];

/// Array of BoardMasks indicating where the bishop can reach
/// if there were no other pieces on the board, removing the edges
///
pub const BISHOP_SPAN_WITHOUT_EDGES: [u64; 64] = [
  0x0040201008040200,
  0x0000402010080400,
  0x0000004020100A00,
  0x0000000040221400,
  0x0000000002442800,
  0x0000000204085000,
  0x0000020408102000,
  0x0002040810204000,
  0x0020100804020000,
  0x0040201008040000,
  0x00004020100A0000,
  0x0000004022140000,
  0x0000000244280000,
  0x0000020408500000,
  0x0002040810200000,
  0x0004081020400000,
  0x0010080402000200,
  0x0020100804000400,
  0x004020100A000A00,
  0x0000402214001400,
  0x0000024428002800,
  0x0002040850005000,
  0x0004081020002000,
  0x0008102040004000,
  0x0008040200020400,
  0x0010080400040800,
  0x0020100A000A1000,
  0x0040221400142200,
  0x0002442800284400,
  0x0004085000500800,
  0x0008102000201000,
  0x0010204000402000,
  0x0004020002040800,
  0x0008040004081000,
  0x00100A000A102000,
  0x0022140014224000,
  0x0044280028440200,
  0x0008500050080400,
  0x0010200020100800,
  0x0020400040201000,
  0x0002000204081000,
  0x0004000408102000,
  0x000A000A10204000,
  0x0014001422400000,
  0x0028002844020000,
  0x0050005008040200,
  0x0020002010080400,
  0x0040004020100800,
  0x0000020408102000,
  0x0000040810204000,
  0x00000A1020400000,
  0x0000142240000000,
  0x0000284402000000,
  0x0000500804020000,
  0x0000201008040200,
  0x0000402010080400,
  0x0002040810204000,
  0x0004081020400000,
  0x000A102040000000,
  0x0014224000000000,
  0x0028440200000000,
  0x0050080402000000,
  0x0020100804020000,
  0x0040201008040200,
];

/// For a given position, this table indicate the BoardMasks indices of
/// possible blockers for the BISHOP_SPAN.
///
///
pub const BISHOP_SPAN_INDEXES: [[usize; 9]; 64] = [
  [9, 18, 27, 36, 45, 54, 255, 255, 255],
  [10, 19, 28, 37, 46, 255, 255, 255, 255],
  [9, 11, 20, 29, 38, 255, 255, 255, 255],
  [10, 12, 17, 21, 30, 255, 255, 255, 255],
  [11, 13, 18, 22, 25, 255, 255, 255, 255],
  [12, 14, 19, 26, 33, 255, 255, 255, 255],
  [13, 20, 27, 34, 41, 255, 255, 255, 255],
  [14, 21, 28, 35, 42, 49, 255, 255, 255],
  [17, 26, 35, 44, 53, 255, 255, 255, 255],
  [18, 27, 36, 45, 54, 255, 255, 255, 255],
  [17, 19, 28, 37, 46, 255, 255, 255, 255],
  [18, 20, 25, 29, 38, 255, 255, 255, 255],
  [19, 21, 26, 30, 33, 255, 255, 255, 255],
  [20, 22, 27, 34, 41, 255, 255, 255, 255],
  [21, 28, 35, 42, 49, 255, 255, 255, 255],
  [22, 29, 36, 43, 50, 255, 255, 255, 255],
  [9, 25, 34, 43, 52, 255, 255, 255, 255],
  [10, 26, 35, 44, 53, 255, 255, 255, 255],
  [9, 11, 25, 27, 36, 45, 54, 255, 255],
  [10, 12, 26, 28, 33, 37, 46, 255, 255],
  [11, 13, 27, 29, 34, 38, 41, 255, 255],
  [12, 14, 28, 30, 35, 42, 49, 255, 255],
  [13, 29, 36, 43, 50, 255, 255, 255, 255],
  [14, 30, 37, 44, 51, 255, 255, 255, 255],
  [10, 17, 33, 42, 51, 255, 255, 255, 255],
  [11, 18, 34, 43, 52, 255, 255, 255, 255],
  [12, 17, 19, 33, 35, 44, 53, 255, 255],
  [9, 13, 18, 20, 34, 36, 41, 45, 54],
  [10, 14, 19, 21, 35, 37, 42, 46, 49],
  [11, 20, 22, 36, 38, 43, 50, 255, 255],
  [12, 21, 37, 44, 51, 255, 255, 255, 255],
  [13, 22, 38, 45, 52, 255, 255, 255, 255],
  [11, 18, 25, 41, 50, 255, 255, 255, 255],
  [12, 19, 26, 42, 51, 255, 255, 255, 255],
  [13, 20, 25, 27, 41, 43, 52, 255, 255],
  [14, 17, 21, 26, 28, 42, 44, 49, 53],
  [9, 18, 22, 27, 29, 43, 45, 50, 54],
  [10, 19, 28, 30, 44, 46, 51, 255, 255],
  [11, 20, 29, 45, 52, 255, 255, 255, 255],
  [12, 21, 30, 46, 53, 255, 255, 255, 255],
  [12, 19, 26, 33, 49, 255, 255, 255, 255],
  [13, 20, 27, 34, 50, 255, 255, 255, 255],
  [14, 21, 28, 33, 35, 49, 51, 255, 255],
  [22, 25, 29, 34, 36, 50, 52, 255, 255],
  [17, 26, 30, 35, 37, 51, 53, 255, 255],
  [9, 18, 27, 36, 38, 52, 54, 255, 255],
  [10, 19, 28, 37, 53, 255, 255, 255, 255],
  [11, 20, 29, 38, 54, 255, 255, 255, 255],
  [13, 20, 27, 34, 41, 255, 255, 255, 255],
  [14, 21, 28, 35, 42, 255, 255, 255, 255],
  [22, 29, 36, 41, 43, 255, 255, 255, 255],
  [30, 33, 37, 42, 44, 255, 255, 255, 255],
  [25, 34, 38, 43, 45, 255, 255, 255, 255],
  [17, 26, 35, 44, 46, 255, 255, 255, 255],
  [9, 18, 27, 36, 45, 255, 255, 255, 255],
  [10, 19, 28, 37, 46, 255, 255, 255, 255],
  [14, 21, 28, 35, 42, 49, 255, 255, 255],
  [22, 29, 36, 43, 50, 255, 255, 255, 255],
  [30, 37, 44, 49, 51, 255, 255, 255, 255],
  [38, 41, 45, 50, 52, 255, 255, 255, 255],
  [33, 42, 46, 51, 53, 255, 255, 255, 255],
  [25, 34, 43, 52, 54, 255, 255, 255, 255],
  [17, 26, 35, 44, 53, 255, 255, 255, 255],
  [9, 18, 27, 36, 45, 54, 255, 255, 255],
];

/// For a given position, this table indicate the Number of relevant blockers bits for a bishop
///
pub const BISHOP_BLOCKER_NUMBERS: [u8; 64] = [
  6, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 7, 9, 9, 7, 5, 5,
  5, 5, 7, 9, 9, 7, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 6,
];

/// Bishop Magic Numbers
///
pub const BISHOP_MAGIC: [u64; 64] = [
  0x4820021228210011,
  0x0010920801202000,
  0x0044110602000082,
  0x0054441080000281,
  0x0801104080000200,
  0x0823822120202806,
  0x088400C410080440,
  0x8001004202014000,
  0x00A0203484108400,
  0x0024200200811100,
  0x0082284481020100,
  0x0800342502020804,
  0x2000011040004010,
  0x2010210108400002,
  0x0081020210040428,
  0x8240A30410C40410,
  0x0806001044104400,
  0x0008024210212200,
  0x4008000908010212,
  0x200C01084010A000,
  0x0023002890400920,
  0x2001000200410400,
  0x0402400414124800,
  0x1001870602011100,
  0x0008200040050100,
  0x810424000210AC00,
  0x0044100008404040,
  0x0001004024040002,
  0x2241001041004000,
  0x2000910000806000,
  0x0068004402020208,
  0x0060A4202086080C,
  0x8404108800408200,
  0x000A109040020208,
  0x0184004804071200,
  0x0000C00808008200,
  0x0148020400801100,
  0x2001004102020100,
  0x4402080100286410,
  0x0085040030638A01,
  0x2008112808002040,
  0x0228841048004242,
  0x0580402410000100,
  0x000002A011004800,
  0x0100404810424200,
  0x0201200085002480,
  0x1004880248402400,
  0x4104040042000044,
  0x400401410820C888,
  0x0000804828248410,
  0x985C110245108004,
  0x301000004202000A,
  0x00024030212A0000,
  0x0080B16250010400,
  0x0210040148420802,
  0x4014C46808490000,
  0x6A00110801042000,
  0x001C020100880421,
  0x000101A032111008,
  0x0080842111842400,
  0x00000000C0082220,
  0x02AAD44010622080,
  0x0800841090420081,
  0x0140010801010020,
];

#[cold]
unsafe fn initialize_bishop_table() {
  for i in 0..64 {
    let mut blockers: [u64; MAX_BISHOP_BLOCKERS_MASK_COMBINATIONS] =
      [0; MAX_BISHOP_BLOCKERS_MASK_COMBINATIONS];
    let blocker_combinations = 1 << BISHOP_SPAN_WITHOUT_EDGES[i].count_ones();

    // Assemble the combinations of possible blockers for square `i`
    for b in 0..blocker_combinations {
      let mut blocker_mask: BoardMask = 0;
      for j in 0..BISHOP_SPAN_WITHOUT_EDGES[i].count_ones() as usize {
        assert!(BISHOP_SPAN_INDEXES[i][j] != 255);
        if b & (1 << j) != 0 {
          blocker_mask |= 1 << BISHOP_SPAN_INDEXES[i][j];
        }
      }
      blockers[b] = blocker_mask;
    }

    for b in 0..blocker_combinations {
      let j: usize =
        (blockers[b].wrapping_mul(BISHOP_MAGIC[i]) >> (64 - BISHOP_BLOCKER_NUMBERS[i])) as usize;

      if BISHOP_DESTINATION_TABLE[i][j] == 0 {
        BISHOP_DESTINATION_TABLE[i][j] =
          get_moves_from_offsets(&BISHOP_MOVE_OFFSETS, true, 0, blockers[b], i);
      } else if BISHOP_DESTINATION_TABLE[i][j]
        != get_moves_from_offsets(&BISHOP_MOVE_OFFSETS, true, 0, blockers[b], i)
      {
        panic!("Bishop table initialization went wrong! =(");
      }
    }
  }
  BISHOP_TABLE_INITIALIZED = true;
}

pub fn get_bishop_destinations(
  same_side_pieces: BoardMask,
  opponent_pieces: BoardMask,
  square: usize,
) -> BoardMask {
  // Tried to save the BISHOP_DESTINATION_TABLE as a constant but then the stack
  // overflows. I could not find a nice way to store it on the heap
  // so I just make a static variable here, that get initialized once.
  unsafe {
    if !BISHOP_TABLE_INITIALIZED {
      initialize_bishop_table();
    }
  }

  let blockers = (same_side_pieces | opponent_pieces) & BISHOP_SPAN_WITHOUT_EDGES[square];
  let blockers_key =
    (blockers.wrapping_mul(BISHOP_MAGIC[square]) >> (64 - BISHOP_BLOCKER_NUMBERS[square])) as usize;

  unsafe {
    BISHOP_DESTINATION_TABLE
      .get_unchecked(square)
      .get_unchecked(blockers_key)
      & !same_side_pieces
  }
}
