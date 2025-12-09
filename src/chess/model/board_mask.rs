use super::board::Board;

// -----------------------------------------------------------------------------
//  Type definitions

/// Unsigned integer representing a Chess-board with squares sets to 0 or 1.
///
/// - a1 is bit 0
/// - b1 is bit 1
/// - etc...
pub type BoardMask = u64;

pub trait CountFewOnes {
  /// Counts the number of binary ones in the representation when there are few
  /// ones.
  ///
  /// e.g. 001010110 -> 4
  /// This function is quicker than `count_ones()` if there are less than 4 ones
  /// in the binary representation.
  ///
  /// use `count_ones()` in other cases.
  fn count_few_ones(&self) -> u32;
}

impl CountFewOnes for BoardMask {
  fn count_few_ones(&self) -> u32 {
    let mut count: u32 = 0;
    let mut copy = *self;
    while copy != 0 {
      copy &= copy - 1;
      count += 1;
    }

    count
  }
}

// -----------------------------------------------------------------------------
//  Macros

/// Helper macro that checks if a square is set in a BoardMask
///
/// Use like this: `square_in_mask!(square, mask)`
///
/// ### Arguments
///
/// * `square` Square value to check.
/// * `mask`   board mask to use for the check
///
/// ### Returns
///
/// Evaluates to True if the square is set in the mask. False if not
#[macro_export]
macro_rules! square_in_mask {
  ($square:expr, $mask:expr) => {
    (((1 << $square) & $mask) != 0)
  };
}

/// Helper macro that sets a square in a BoardMask
///
/// Use like this: `set_square_in_mask!(square, mask)`
///
/// ### Arguments
///
/// * `square` Square value to add to the BoardMask
/// * `mask`   board mask to modify
#[macro_export]
macro_rules! set_square_in_mask {
  ($square:expr, $mask:expr) => {
    $mask |= 1 << $square
  };
}

/// Helper macro that removes a square from a BoardMask
///
/// Use like this: `unset_square_in_mask!(square, mask)`
///
/// ### Arguments
///
/// * `square` Square value to revove from the BoardMask
/// * `mask`   board mask to modify
#[macro_export]
macro_rules! unset_square_in_mask {
  ($square:expr, $mask:expr) => {
    $mask &= !(1 << $square)
  };
}

// Make the macros public
pub use {set_square_in_mask, square_in_mask, unset_square_in_mask};

// -----------------------------------------------------------------------------
//  Functions

/// Helper macro printing a board mask
///
/// Will print 1 when the BoardMask is set, 0 when it's not set.
///
/// ### Arguments
///
/// * `mask` board mask to print
pub fn print_board_mask(mask: BoardMask) {
  let mut representation = String::from("\n");
  for rank in (1..=8).rev() {
    for file in 1..=8 {
      if (mask >> Board::fr_to_index(file, rank) & 1) == 1 {
        representation.push('1');
      } else {
        representation.push('0');
      }
      representation.push(' ');
    }
    representation.push('\n');
  }

  println!("{representation}");
}

/// Helper function returning a string from a board mask
///
/// Will print x when the BoardMask is set, . when it's not set.
///
/// ### Arguments
///
/// * `mask` board mask to stringify
pub fn board_mask_to_string(mask: BoardMask) -> String {
  let mut string = String::new();
  for rank in (1..=8).rev() {
    for file in 1..=8 {
      let square_index = Board::fr_to_index(file, rank);
      if ((mask >> square_index) & 1) == 1 {
        string.push('x');
      } else {
        string.push('.');
      }
      string.push(' ');
    }
    string.push('\n');
  }
  string
}
