// -----------------------------------------------------------------------------
// Constants
const K_MASK: u8 = 0b00001000;
const Q_MASK: u8 = 0b00000100;
#[allow(non_upper_case_globals)]
const k_MASK: u8 = 0b00000010;
#[allow(non_upper_case_globals)]
const q_MASK: u8 = 0b00000001;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[allow(non_snake_case)]
pub struct CastlingRights {
  // Will be using the 4 LSB: 0000KQkq
  pub rights: u8,
}

impl CastlingRights {
  /// Indicates if the White Kingside castle is allowed.
  ///
  /// ### Return Value
  ///
  /// true if the white kingside castle is allowed, false otherwise
  ///
  #[allow(non_snake_case)]
  pub fn K(&self) -> bool {
    return (self.rights & K_MASK) != 0;
  }
  /// Sets the bit for the if the White Kingside castle right.
  ///
  /// ### Arguments
  ///
  /// * `right_value`: Set to true if the white kingside castle right is allowed, false otherwise
  ///
  #[allow(non_snake_case)]
  pub fn set_K(&mut self, right_value: bool) {
    if right_value {
      self.rights |= K_MASK;
    } else {
      self.rights &= !K_MASK;
    }
  }

  /// Indicates if the White Queenside castle is allowed.
  ///
  /// ### Return Value
  ///
  /// true if the white kingside castle is allowed, false otherwise
  ///
  #[allow(non_snake_case)]
  pub fn Q(&self) -> bool {
    return (self.rights & Q_MASK) != 0;
  }
  /// Sets the bit for the if the White Kingside castle right.
  ///
  /// ### Arguments
  ///
  /// * `right_value`: Set to true if the white kingside castle right is allowed, false otherwise
  ///
  #[allow(non_snake_case)]
  pub fn set_Q(&mut self, right_value: bool) {
    if right_value {
      self.rights |= Q_MASK;
    } else {
      self.rights &= !Q_MASK;
    }
  }

  /// Remove white's castling rights
  ///
  pub fn clear_white_rights(&mut self) {
    self.rights &= !(K_MASK | Q_MASK);
  }

  /// Indicates if the Black Kingside castle is allowed.
  ///
  /// ### Return Value
  ///
  /// true if the black kingside castle is allowed, false otherwise
  ///
  #[allow(non_snake_case)]
  pub fn k(&self) -> bool {
    return (self.rights & k_MASK) != 0;
  }
  /// Sets the bit for the if the White Kingside castle right.
  ///
  /// ### Arguments
  ///
  /// * `right_value`: Set to true if the white kingside castle right is allowed, false otherwise
  ///
  #[allow(non_snake_case)]
  pub fn set_k(&mut self, right_value: bool) {
    if right_value {
      self.rights |= k_MASK;
    } else {
      self.rights &= !k_MASK;
    }
  }

  /// Indicates if the Black Queenside castle is allowed.
  ///
  /// ### Return Value
  ///
  /// true if the black queenside castle is allowed, false otherwise
  ///
  #[allow(non_snake_case)]
  pub fn q(&self) -> bool {
    return (self.rights & q_MASK) != 0;
  }

  /// Sets the black queenside castle right
  ///
  /// ### Arguments
  ///
  /// * `right_value`: Set to true if the black queenside castle right is allowed, false otherwise
  ///
  #[allow(non_snake_case)]
  pub fn set_q(&mut self, right_value: bool) {
    if right_value {
      self.rights |= q_MASK;
    } else {
      self.rights &= !q_MASK;
    }
  }

  /// Remove white's castling rights
  ///
  pub fn clear_black_rights(&mut self) {
    self.rights &= !(k_MASK | q_MASK);
  }

  /// Returns a string/fen representation of the castling rights
  ///
  /// ### Return Value
  ///
  /// * String representation that can be used in a FEN.
  ///
  pub fn to_fen(&self) -> String {
    let mut fen = String::new();

    if self.K() {
      fen.push('K');
    }
    if self.Q() {
      fen.push('Q');
    }
    if self.k() {
      fen.push('k');
    }
    if self.q() {
      fen.push('q');
    }

    if fen.len() == 0 {
      fen.push('-');
    }
    return fen;
  }
}

impl Default for CastlingRights {
  fn default() -> Self {
    CastlingRights {
      rights: K_MASK | Q_MASK | k_MASK | q_MASK,
    }
  }
}

impl std::fmt::Display for CastlingRights {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut representation = String::new();
    if self.K() {
      representation.push('K');
    }
    if self.Q() {
      representation.push('Q');
    }
    if self.k() {
      representation.push('k');
    }
    if self.q() {
      representation.push('q');
    }

    if representation.len() == 0 {
      representation.push('-');
    }

    f.write_str(representation.as_str())
  }
}
