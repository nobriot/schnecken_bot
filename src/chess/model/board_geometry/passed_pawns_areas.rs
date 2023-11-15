/// Represents boardmask of area where there should be no pawns if
/// we want to consider the white pawn passed.
/// For a pawn on square i, check if WHITE_PASSED_PAWN_AREA[i] contains any opponent pawns
pub const WHITE_PASSED_PAWN_AREA: [u64; 64] = [
  0x0003030303030300,
  0x0007070707070700,
  0x000E0E0E0E0E0E00,
  0x001C1C1C1C1C1C00,
  0x0038383838383800,
  0x0070707070707000,
  0x00E0E0E0E0E0E000,
  0x00C0C0C0C0C0C000,
  0x0003030303030000,
  0x0007070707070000,
  0x000E0E0E0E0E0000,
  0x001C1C1C1C1C0000,
  0x0038383838380000,
  0x0070707070700000,
  0x00E0E0E0E0E00000,
  0x00C0C0C0C0C00000,
  0x0003030303000000,
  0x0007070707000000,
  0x000E0E0E0E000000,
  0x001C1C1C1C000000,
  0x0038383838000000,
  0x0070707070000000,
  0x00E0E0E0E0000000,
  0x00C0C0C0C0000000,
  0x0003030300000000,
  0x0007070700000000,
  0x000E0E0E00000000,
  0x001C1C1C00000000,
  0x0038383800000000,
  0x0070707000000000,
  0x00E0E0E000000000,
  0x00C0C0C000000000,
  0x0003030000000000,
  0x0007070000000000,
  0x000E0E0000000000,
  0x001C1C0000000000,
  0x0038380000000000,
  0x0070700000000000,
  0x00E0E00000000000,
  0x00C0C00000000000,
  0x0003000000000000,
  0x0007000000000000,
  0x000E000000000000,
  0x001C000000000000,
  0x0038000000000000,
  0x0070000000000000,
  0x00E0000000000000,
  0x00C0000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
];

/// Represents boardmask of area where there should be no pawns if
/// we want to consider the black pawn passed.
/// For a pawn on square i, check if BLACK_PASSED_PAWN_AREA[i] contains any opponent pawns
pub const BLACK_PASSED_PAWN_AREA: [u64; 64] = [
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000000,
  0x0000000000000300,
  0x0000000000000700,
  0x0000000000000E00,
  0x0000000000001C00,
  0x0000000000003800,
  0x0000000000007000,
  0x000000000000E000,
  0x000000000000C000,
  0x0000000000030300,
  0x0000000000070700,
  0x00000000000E0E00,
  0x00000000001C1C00,
  0x0000000000383800,
  0x0000000000707000,
  0x0000000000E0E000,
  0x0000000000C0C000,
  0x0000000003030300,
  0x0000000007070700,
  0x000000000E0E0E00,
  0x000000001C1C1C00,
  0x0000000038383800,
  0x0000000070707000,
  0x00000000E0E0E000,
  0x00000000C0C0C000,
  0x0000000303030300,
  0x0000000707070700,
  0x0000000E0E0E0E00,
  0x0000001C1C1C1C00,
  0x0000003838383800,
  0x0000007070707000,
  0x000000E0E0E0E000,
  0x000000C0C0C0C000,
  0x0000030303030300,
  0x0000070707070700,
  0x00000E0E0E0E0E00,
  0x00001C1C1C1C1C00,
  0x0000383838383800,
  0x0000707070707000,
  0x0000E0E0E0E0E000,
  0x0000C0C0C0C0C000,
  0x0003030303030300,
  0x0007070707070700,
  0x000E0E0E0E0E0E00,
  0x001C1C1C1C1C1C00,
  0x0038383838383800,
  0x0070707070707000,
  0x00E0E0E0E0E0E000,
  0x00C0C0C0C0C0C000,
];