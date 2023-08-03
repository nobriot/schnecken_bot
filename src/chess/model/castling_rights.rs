#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[allow(non_snake_case)]
pub struct CastlingRights {
  pub K: bool,
  pub Q: bool,
  pub k: bool,
  pub q: bool,
}

impl Default for CastlingRights {
  fn default() -> Self {
    CastlingRights {
      K: true,
      Q: true,
      k: true,
      q: true,
    }
  }
}

impl std::fmt::Display for CastlingRights {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut representation = String::new();
    if self.K {
      representation.push('K');
    }
    if self.Q {
      representation.push('Q');
    }
    if self.k {
      representation.push('k');
    }
    if self.q {
      representation.push('q');
    }

    if representation.len() == 0 {
      representation.push('-');
    }

    f.write_str(representation.as_str())
  }
}
