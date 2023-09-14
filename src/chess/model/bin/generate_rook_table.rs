use std::fs::File;
use std::io::{Error, Write};

fn main() {
  println!("FIXME: Get that to work");

  /*
  let mut rook_moves: [u64; 64] = [0; 64];
  for i in 0..64 {
    rook_moves[i] = get_moves_from_offsets(&ROOK_MOVE_OFFSETS, true, 0, 0, i);
  }
  println!("pub const ROOK_SPAN:[u64; 64] = {:#018X?};", rook_moves);

  let mut rook_span_indexes: [[usize; 12]; 64] = [[INVALID_SQUARE.into(); 12]; 64];

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
      if !square_in_mask!(j, ROOK_SPAN[i] & piece_edges) {
        continue;
      }
      println!("ROOK_SPAN_INDEXES[i={}][index={}] = {};", i, index, j);
      rook_span_indexes[i][index] = j;
      index += 1;
    }
  }
  println!(
    "pub const ROOK_SPAN_INDEXES: [[usize; 12]; 64] = {:#?};",
    rook_span_indexes
  );

  let mut bishop_moves: [u64; 64] = [0; 64];
  for i in 0..64 {
    bishop_moves[i] = get_moves_from_offsets(&BISHOP_MOVE_OFFSETS, true, 0, 0, i);
  }
  println!("pub const BISHOP_SPAN:[u64; 64] = {:#018X?};", bishop_moves);

  let mut board_edges: BoardMask = 0;
  for i in 0..64 {
    let (file, rank) = Board::index_to_fr(i);
    if file == 1 || rank == 1 || file == 8 || rank == 8 {
      set_square_in_mask!(i, board_edges);
    }
  }
  let mut edges_right: BoardMask = 0;
  let mut edges_left: BoardMask = 0;
  let mut edges_up: BoardMask = 0;
  let mut edges_down: BoardMask = 0;
  for i in 0..64 {
    let (file, rank) = Board::index_to_fr(i);
    if file == 1 {
      set_square_in_mask!(i, board_edges);
      set_square_in_mask!(i, edges_left);
    }
    if file == 8 {
      set_square_in_mask!(i, board_edges);
      set_square_in_mask!(i, edges_right);
    }
    if rank == 1 {
      set_square_in_mask!(i, board_edges);
      set_square_in_mask!(i, edges_down);
    }
    if rank == 8 {
      set_square_in_mask!(i, board_edges);
      set_square_in_mask!(i, edges_up);
    }
  }
  //print_board_mask(board_edges);
  println!("pub const BOARD_EDGES:BoardMask = {:#018X?};", board_edges);
  println!(
    "pub const BOARD_RIGHT_EDGE:BoardMask = {:#018X?};",
    edges_right
  );
  println!(
    "pub const BOARD_LEFT_EDGE:BoardMask = {:#018X?};",
    edges_left
  );
  println!(
    "pub const BOARD_DOWN_EDGE:BoardMask = {:#018X?};",
    edges_down
  );
  println!("pub const BOARD_UP_EDGE:BoardMask = {:#018X?};", edges_up);
  print_board_mask(edges_down);
  print_board_mask(edges_up);
  print_board_mask(edges_right);
  print_board_mask(edges_left);

  println!("pub const BOARD_EDGES:BoardMask = {:#018X?};", board_edges);
  println!(
    "pub const BOARD_WITHOUT_EDGES:BoardMask = {:#018X?};",
    !board_edges
  );

  // Calculate all possible rook destinations based on the list of blockers
  static mut rook_table: [[u64; MAX_ROOK_BLOCKERS]; 64] = [[0; MAX_ROOK_BLOCKERS]; 64];
  for i in 0..64 {
    let mut blocker_mask: BoardMask = 0;
    for blockers in 0..MAX_ROOK_BLOCKERS {
      for j in 0..12 {
        if (1 << j) & blockers != 0 && ROOK_SPAN_INDEXES[i][j] != 255 {
          blocker_mask |= (1 << ROOK_SPAN_INDEXES[i][j]);
        }
      }
      unsafe {
        rook_table[i][blockers] =
          get_moves_from_offsets(&ROOK_MOVE_OFFSETS, true, 0, blocker_mask, i);
      }
      /*
      println!(
        "pub const ROOK_DESTINATIONS[{}][{}] =  {:#?};",
        i,
        blockers,
        get_moves_from_offsets(&ROOK_MOVE_OFFSETS, true, 0, blocker_mask, i)
      );*/
    }
  }
  let path = "rook_table.rs";
  let mut output = File::create(path).unwrap();
  unsafe {
    write!(
      output,
      "pub const ROOK_TABLE: [[u64; MAX_ROOK_BLOCKERS]; 64] = {:#?};",
      rook_table
    );
  }
   */
}
