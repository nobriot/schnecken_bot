/// Tables of squares that are good for each pieces in each game phase.
pub struct OpeningSquareTable;

#[rustfmt::skip]
impl OpeningSquareTable {
  pub const WHITE_KING: [isize; 64] = [
     20,  30,  20,   0,   0,   0,  30,  20, // 1st row
      0,   0,   0, -10, -10,   0,   0,   0, // 2nd row
    -10, -20, -20, -20, -20, -20, -20, -10, // 3rd row
    -20, -30, -30, -30, -30, -30, -30, -20, // 4th row
    -30, -40, -40, -40, -40, -40, -40, -30, // 5th row
    -40, -50, -50, -50, -50, -50, -50, -40, // 6th row
    -50, -50, -50, -50, -50, -50, -50, -50, // 7th row
    -50, -50, -50, -50, -50, -50, -50, -50, // 8th row
  ];

  pub const BLACK_KING: [isize; 64] = [
    -50, -50, -50, -50, -50, -50, -50, -50, // 1st row
    -50, -50, -50, -50, -50, -50, -50, -50, // 2nd row
    -40, -50, -50, -50, -50, -50, -50, -40, // 3th row
    -30, -40, -40, -40, -40, -40, -40, -30, // 4th row
    -20, -30, -30, -30, -30, -30, -30, -20, // 5th row
    -10, -20, -20, -20, -20, -20, -20, -10, // 6th row
      0,   0,   0, -10, -10,   0,   0,   0, // 7th row
     20,  30,  20,   0,   0,   0,  30,  20, // 8th row
 ];

  pub const QUEEN: [isize; 64] = [
    -20, -10, -10,  -5,  -5, -10, -10, -20, // 1st row
    -10,   0,   0,   0,   0,   0,   0, -10, // 2nd row
    -10,   5,   5,   5,   5,   5,   5, -10, // 3rd row
     -5,   5,   5,   5,   5,   5,   5,  -5, // 4th row
     -5,   5,   5,   5,   5,   5,   5,  -5, // 5th row
    -10,   5,   5,   5,   5,   5,   5, -10, // 6th row
    -10,   0,   0,   0,   0,   0,   0, -10, // 7th row
    -20, -10, -10,  -5,  -5, -10, -10, -20, // 8th row
  ];

  pub const WHITE_ROOK: [isize; 64] = [
      0,   0,  10,  15,  15,  10,   0,   0, // 1st row
    -20,  -5,   0,   0,   0,   0,  -5, -20, // 2nd row
    -10,  -5,  -5,  -5,  -5,  -5,  -5, -10, // 3rd row
    -10, -10, -10, -10, -10, -10, -10, -10, // 4th row
    -10, -10, -10, -10, -10, -10, -10, -10, // 5th row
    -10, -10, -10, -10, -10, -10, -10, -10, // 6th row
    -10, -10, -10, -10, -10, -10, -10, -10, // 7th row
    -10, -10, -10, -10, -10, -10, -10, -10, // 8th row
  ];

  pub const BLACK_ROOK: [isize; 64] = [
    -10, -10, -10, -10, -10, -10, -10, -10, // 8th row
    -10, -10, -10, -10, -10, -10, -10, -10, // 7th row
    -10, -10, -10, -10, -10, -10, -10, -10, // 6th row
    -10, -10, -10, -10, -10, -10, -10, -10, // 5th row
    -10, -10, -10, -10, -10, -10, -10, -10, // 4th row
    -10,  -5,  -5,  -5,  -5,  -5,  -5, -10, // 3rd row
    -20,  -5,   0,   0,   0,   0,  -5, -20, // 2nd row
      0,   0,  10,  15,  15,  10,   0,   0, // 1st row
  ];
}

pub struct MiddleGameSquareTable;

#[rustfmt::skip]
impl MiddleGameSquareTable {
  pub const WHITE_KING: [isize; 64] = [
     20,  30,  20,   0,   0,  10,  30,  20, // 1st row
      0,   0,   0,   0,   0,   0,   0,   0, // 2nd row
    -10, -20, -20, -20, -20, -20, -20, -10, // 3rd row
    -20, -30, -30, -30, -30, -30, -30, -20, // 4th row
    -30, -40, -40, -40, -40, -40, -40, -30, // 5th row
    -40, -50, -50, -50, -50, -50, -50, -40, // 6th row
    -50, -50, -50, -50, -50, -50, -50, -50, // 7th row
    -50, -50, -50, -50, -50, -50, -50, -50, // 8th row
  ];

  pub const BLACK_KING: [isize; 64] = [
    -50, -50, -50, -50, -50, -50, -50, -50, // 1st row
    -50, -50, -50, -50, -50, -50, -50, -50, // 2nd row
    -40, -50, -50, -50, -50, -50, -50, -40, // 3th row
    -30, -40, -40, -40, -40, -40, -40, -30, // 4th row
    -20, -30, -30, -30, -30, -30, -30, -20, // 5th row
    -10, -20, -20, -20, -20, -20, -20, -10, // 6th row
      0,   0,   0,   0,   0,   0,   0,   0, // 7th row
     20,  30,  20,   0,   0,  10,  30,  20, // 8th row
 ];

 pub const WHITE_ROOK: [isize; 64] = [
    0,   0,  10,  15,  15,  10,   0,   0, // 1st row
  -20,  -5,   0,   0,   0,   0,  -5, -20, // 2nd row
  -10,  -5,  -5,  -5,  -5,  -5,  -5, -10, // 3rd row
  -10, -10, -10, -10, -10, -10, -10, -10, // 4th row
  -10, -10, -10, -10, -10, -10, -10, -10, // 5th row
  -10, -10, -10, -10, -10, -10, -10, -10, // 6th row
  -10, -10, -10, -10, -10, -10, -10, -10, // 7th row
  -10, -10, -10, -10, -10, -10, -10, -10, // 8th row
];

pub const BLACK_ROOK: [isize; 64] = [
  -10, -10, -10, -10, -10, -10, -10, -10, // 8th row
  -10, -10, -10, -10, -10, -10, -10, -10, // 7th row
  -10, -10, -10, -10, -10, -10, -10, -10, // 6th row
  -10, -10, -10, -10, -10, -10, -10, -10, // 5th row
  -10, -10, -10, -10, -10, -10, -10, -10, // 4th row
  -10,  -5,  -5,  -5,  -5,  -5,  -5, -10, // 3rd row
  -20,  -5,   0,   0,   0,   0,  -5, -20, // 2nd row
    0,   0,  10,  15,  15,  10,   0,   0, // 1st row
];
}

pub struct EndgameSquareTable;

#[rustfmt::skip]
impl EndgameSquareTable {
  /// Symmetric here, we want the king to be attracted to the middle of the board
  /// in the endgame.
  pub const KING: [isize; 64] = [
    -50, -30, -30, -30, -30, -30, -30, -50, // 1st row
    -30, -30,   0,   0,   0,   0, -30, -30, // 2nd row
    -30, -10,  20,  30,  30,  20, -10, -30, // 3rd row.
    -30, -10,  30,  40,  40,  30, -10, -30, // 4th row
    -30, -10,  30,  40,  40,  30, -10, -30, // 5th row
    -30, -10,  20,  30,  30,  20, -10, -30, // 6th row
    -30, -30,   0,   0,   0,   0, -30, -30, // 7th row
    -50, -30, -30, -30, -30, -30, -30, -50, // 8th row
  ];

  pub const QUEEN: [isize; 64] = [
    -20, -10, -10,  -5,  -5, -10, -10, -20, // 1st row
    -10,   0,   0,   0,   0,   0,   0, -10, // 2nd row
    -10,   5,   5,   5,   5,   5,   5, -10, // 3rd row
     -5,   5,   5,   5,   5,   5,   5,  -5, // 4th row
     -5,   5,   5,   5,   5,   5,   5,  -5, // 5th row
    -10,   5,   5,   5,   5,   5,   5, -10, // 6th row
    -10,   0,   0,   0,   0,   0,   0, -10, // 7th row
    -20, -10, -10,  -5,  -5, -10, -10, -20, // 8th row
  ];

  pub const WHITE_ROOK: [isize; 64] = [
      0,   0,   0,   5,   5,   0,   0,   0, // 1st row
     -5,   0,   0,   0,   0,   0,   0,  -5, // 2nd row
     -5,   0,   0,   0,   0,   0,   0,  -5, // 3rd row
     -5,   0,   0,   0,   0,   0,   0,  -5, // 4th row
     -5,   0,   0,   0,   0,   0,   0,  -5, // 5th row
     -5,   0,   0,   0,   0,   0,   0,  -5, // 6th row
      5,  10,  10,  10,  10,  10,  10,   5, // 7th row, me likey!
      0,   0,   0,   0,   0,   0,   0,   0, // 8th row
  ];
  pub const BLACK_ROOK: [isize; 64] = [
     0,   0,   0,   0,   0,   0,   0,   0, // 1st row
     5,  10,  10,  10,  10,  10,  10,   5, // 2nd row, me likey!
    -5,   0,   0,   0,   0,   0,   0,  -5, // 3rd row
    -5,   0,   0,   0,   0,   0,   0,  -5, // 4th row
    -5,   0,   0,   0,   0,   0,   0,  -5, // 5th row.
    -5,   0,   0,   0,   0,   0,   0,  -5, // 6th row
     5,  10,  10,  10,  10,  10,  10,   5, // 7th row
     0,   0,   0,   0,   0,   0,   0,   0, // 8th row 
  ];
}

pub struct SquareTable;

#[rustfmt::skip]
impl SquareTable {
  pub const WHITE_BISHOP: [isize; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20, // 1st row
    -10,   5,   0,   0,   0,   0,   5, -10, // 2nd row
    -10,  10,  10,  10,  10,  10,  10, -10, // 3rd row
    -10,   0,  10,  10,  10,  10,   0, -10, // 4th row
    -10,   0,   5,  10,  10,   5,   0, -10, // 5th row
    -10,   5,   5,  10,  10,   5,   5, -10, // 6th row
    -10,   0,   0,   0,   0,   0,   0, -10, // 7th row
    -20, -10, -10, -10, -10, -10, -10, -20, // 8th row
  ];
  
  pub const BLACK_BISHOP: [isize; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20, // 1st row
    -10,   0,   0,   0,   0,   0,   0, -10, // 2nd row
    -10,   5,   5,  10,  10,   5,   5, -10, // 3rd row
    -10,   0,   5,  10,  10,   5,   0, -10, // 4th row
    -10,   0,  10,  10,  10,  10,   0, -10, // 5th row
    -10,  10,  10,  10,  10,  10,  10, -10, // 6th row
    -10,   5,   0,   0,   0,   0,   5, -10, // 7th row
    -20, -10, -10, -10, -10, -10, -10, -20, // 8th row
  ];
  
  pub const KNIGHT: [isize; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50, // 1st row
    -40, -20,   0,   0,   0,   0, -20, -40, // 2nd row
    -30,   0,  10,  15,  15,  10,   0, -30, // 3rd row
    -30,   5,  15,  20,  20,  15,   5, -30, // 4th row
    -30,   0,  15,  20,  20,  15,   0, -30, // 5th row
    -30,   5,  10,  15,  15,  10,   5, -30, // 6th row
    -40, -20,   0,   5,   5,   0, -20, -40, // 7th row
    -50, -40, -30, -30, -30, -30, -40, -50, // 8th row
  ];
  
  pub const WHITE_PAWN: [isize; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0, // 1st row
      5,  10,  10, -20, -20,  10,  10,   5, // 2nd row
      5,  -5, -10,   0,   0, -10,  -5,   5, // 3rd row
      0,   0,   0,  20,  20,   0,   0,   0, // 4th row
      5,   5,  10,  25,  25,  10,   5,   5, // 5th row
      10, 10,  20,  30,  30,  20,  10,  10, // 6th row
      50, 50,  50,  50,  50,  50,  50,  50, // 7th row
      0,   0,   0,   0,   0,   0,   0,   0, // 8th row
  ];

  pub const WHITE_PASSED_PAWN: [isize; 64] = [
     0,   0,   0,   0,   0,   0,   0,   0, // 1st row
     0,   0,   0,   0,   0,   0,   0,   0, // 2nd row
    10,  10,  10,  10,  10,  10,  10,  10, // 3rd row
    20,  20,  20,  20,  20,  20,  20,  20, // 4th row
    30,  30,  30,  30,  30,  30,  30,  30, // 5th row
    40,  40,  40,  40,  40,  40,  40,  40, // 6th row
    50,  50,  50,  50,  50,  50,  50,  50, // 7th row
     0,   0,   0,   0,   0,   0,   0,   0, // 8th row
  ];


  pub const BLACK_PAWN: [isize; 64] = [
    0,   0,   0,   0,   0,   0,   0,   0, // 1st row
    50,  50,  50,  50,  50,  50,  50,  50, // 2nd row
    10,  10,  20,  30,  30,  20,  10,  10, // 3rd row
    5,   5,  10,  25,  25,  10,   5,   5, // 4th row
    0,   0,   0,  20,  20,   0,   0,   0, // 5th row
    5,  -5, -10,   0,   0, -10,  -5,   5, // 6th row
    5,  10,  10, -20, -20,  10,  10,   5, // 7th row
    0,   0,   0,   0,   0,   0,   0,   0, // 8th row
  ];

  pub const BLACK_PASSED_PAWN: [isize; 64] = [
    0,   0,   0,   0,   0,   0,   0,   0, // 1st row
    50,  50,  50,  50,  50,  50,  50,  50, // 2nd row
    40,  40,  40,  40,  40,  40,  40,  40, // 3rd row
    30,  30,  30,  30,  30,  30,  30,  30, // 4th row
    20,  20,  20,  20,  20,  20,  20,  20, // 5th row
    10,  10,  10,  10,  10,  10,  10,  10, // 6th row
     0,   0,   0,   0,   0,   0,   0,   0, // 7th row
     0,   0,   0,   0,   0,   0,   0,   0, // 8th row
  ];

  pub const QUEEN: [isize; 64] = [
    -20, -10, -10,  -5,  -5, -10, -10, -20, // 1st row
    -10,   0,   0,   0,   0,   0,   0, -10, // 2nd row
    -10,   5,   5,   5,   5,   5,   5, -10, // 3rd row
    -5,   5,   5,   5,   5,   5,   5,  -5, // 4th row
    -5,   5,   5,   5,   5,   5,   5,  -5, // 5th row
    -10,   5,   5,   5,   5,   5,   5, -10, // 6th row
    -10,   0,   0,   0,   0,   0,   0, -10, // 7th row
    -20, -10, -10,  -5,  -5, -10, -10, -20, // 8th row
  ];
}
