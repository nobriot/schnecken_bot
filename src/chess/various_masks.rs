/// Queen Side mask
///
pub const QUEEN_SIDE_MASK:BoardMask = 0x0F0F0F0F0F0F0F0F;

/// King Side mask
///
pub const KING_SIDE_MASK:BoardMask = 0xF0F0F0F0F0F0F0F0;

/// White King shelter pawns
///
pub const WHITE_KING_SHELTER_PAWNS: [u64; 64] = [
    0x0303030303030300,
    0x0707070707070700,
    0x0E0E0E0E0E0E0E00,
    0x1C1C1C1C1C1C1C00,
    0x3838383838383800,
    0x7070707070707000,
    0xE0E0E0E0E0E0E000,
    0xC0C0C0C0C0C0C000,
    0x0303030303030000,
    0x0707070707070000,
    0x0E0E0E0E0E0E0000,
    0x1C1C1C1C1C1C0000,
    0x3838383838380000,
    0x7070707070700000,
    0xE0E0E0E0E0E00000,
    0xC0C0C0C0C0C00000,
    0x0303030303000000,
    0x0707070707000000,
    0x0E0E0E0E0E000000,
    0x1C1C1C1C1C000000,
    0x3838383838000000,
    0x7070707070000000,
    0xE0E0E0E0E0000000,
    0xC0C0C0C0C0000000,
    0x0303030300000000,
    0x0707070700000000,
    0x0E0E0E0E00000000,
    0x1C1C1C1C00000000,
    0x3838383800000000,
    0x7070707000000000,
    0xE0E0E0E000000000,
    0xC0C0C0C000000000,
    0x0303030000000000,
    0x0707070000000000,
    0x0E0E0E0000000000,
    0x1C1C1C0000000000,
    0x3838380000000000,
    0x7070700000000000,
    0xE0E0E00000000000,
    0xC0C0C00000000000,
    0x0303000000000000,
    0x0707000000000000,
    0x0E0E000000000000,
    0x1C1C000000000000,
    0x3838000000000000,
    0x7070000000000000,
    0xE0E0000000000000,
    0xC0C0000000000000,
    0x0300000000000000,
    0x0700000000000000,
    0x0E00000000000000,
    0x1C00000000000000,
    0x3800000000000000,
    0x7000000000000000,
    0xE000000000000000,
    0xC000000000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000000,
];

/// White King shelter pawns
///
pub const BLACK_KING_SHELTER_PAWNS: [u64; 64] = [
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000003,
    0x0000000000000007,
    0x000000000000000E,
    0x000000000000001C,
    0x0000000000000038,
    0x0000000000000070,
    0x00000000000000E0,
    0x00000000000000C0,
    0x0000000000000303,
    0x0000000000000707,
    0x0000000000000E0E,
    0x0000000000001C1C,
    0x0000000000003838,
    0x0000000000007070,
    0x000000000000E0E0,
    0x000000000000C0C0,
    0x0000000000030303,
    0x0000000000070707,
    0x00000000000E0E0E,
    0x00000000001C1C1C,
    0x0000000000383838,
    0x0000000000707070,
    0x0000000000E0E0E0,
    0x0000000000C0C0C0,
    0x0000000003030303,
    0x0000000007070707,
    0x000000000E0E0E0E,
    0x000000001C1C1C1C,
    0x0000000038383838,
    0x0000000070707070,
    0x00000000E0E0E0E0,
    0x00000000C0C0C0C0,
    0x0000000303030303,
    0x0000000707070707,
    0x0000000E0E0E0E0E,
    0x0000001C1C1C1C1C,
    0x0000003838383838,
    0x0000007070707070,
    0x000000E0E0E0E0E0,
    0x000000C0C0C0C0C0,
    0x0000030303030303,
    0x0000070707070707,
    0x00000E0E0E0E0E0E,
    0x00001C1C1C1C1C1C,
    0x0000383838383838,
    0x0000707070707070,
    0x0000E0E0E0E0E0E0,
    0x0000C0C0C0C0C0C0,
    0x0003030303030303,
    0x0007070707070707,
    0x000E0E0E0E0E0E0E,
    0x001C1C1C1C1C1C1C,
    0x0038383838383838,
    0x0070707070707070,
    0x00E0E0E0E0E0E0E0,
    0x00C0C0C0C0C0C0C0,
];

