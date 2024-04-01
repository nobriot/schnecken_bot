use std::fmt::Display;

// Chess model
use crate::model::moves::Move;
use crate::model::piece::Color;

// Chess Engine
use crate::engine::decrement_eval_if_mating_sequence;

const VARIATION_LENGTH: usize = 10;

#[derive(Debug, Clone)]
pub struct Variation {
  moves: [Move; VARIATION_LENGTH],
  length: u8,
}

impl Variation {
  pub fn new() -> Self {
    Variation {
      moves: [Move::null(); 10],
      length: 0,
    }
  }

  pub fn add(&mut self, mv: Move) {
    if self.length as usize >= VARIATION_LENGTH {
      return;
    }
    self.moves[self.length as usize] = mv;
    self.length += 1;
  }

  pub fn push_front(&mut self, mv: Move) {
    if self.length as usize >= VARIATION_LENGTH {
      self.length = (VARIATION_LENGTH - 1) as u8;
    }
    self.moves.rotate_right(1);
    self.moves[0] = mv;
    self.length += 1;
  }

  pub fn clear(&mut self) {
    self.moves = [Move::null(); 10];
    self.length = 0;
  }

  pub fn pop(&mut self) -> Option<Move> {
    if self.length == 0 {
      return None;
    }

    self.length -= 1;
    Some(self.moves[self.length as usize])
  }

  pub fn len(&self) -> usize {
    self.length as usize
  }

  pub fn is_empty(&self) -> bool {
    self.length == 0
  }

  pub fn get_moves(&self) -> Option<[Move; VARIATION_LENGTH]> {
    if self.length == 0 {
      return None;
    }
    Some(self.moves.clone())
  }

  pub fn get_first_move(&self) -> Option<Move> {
    if self.length == 0 {
      return None;
    }
    Some(self.moves[0])
  }
}

impl Display for Variation {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for mv in &self.moves {
      if mv.is_null() {
        break;
      }
      write!(f, " {}", mv)?;
    }

    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct VariationWithEval {
  pub variation: Variation,
  pub eval: f32,
}

impl VariationWithEval {
  /// Creates a new variation with eval from a single move
  ///
  pub fn new_from_move(eval: f32, mv: Move) -> Self {
    let mut variation = Variation::new();
    variation.add(mv);
    VariationWithEval { variation, eval }
  }
}

/// Search result. Contains lines (vector of moves) associated with an
/// evaluation
///
#[derive(Debug, Clone)]
pub struct SearchResult {
  // How many lines/variations do we keep track of
  lines: usize,
  // How do we sort evals (from white or black point of view)
  sort: Color,
  // Top `lines` variations.
  //FIXME: Put that private at some point
  pub variations: Vec<VariationWithEval>,
}

impl SearchResult {
  /// Creates a new search result
  ///
  pub fn new(lines: usize, color: Color) -> Self {
    SearchResult {
      lines,
      sort: color,
      variations: Vec::with_capacity(lines),
    }
  }

  /// Gets the number of lines actually present in the analysis
  pub fn len(&self) -> usize {
    self.variations.len()
  }

  pub fn is_empty(&self) -> bool {
    self.variations.is_empty()
  }

  /// Gets the index'th variation with eval from the results.
  pub fn get(&self, index: usize) -> VariationWithEval {
    debug_assert!(index < self.lines);
    self.variations[index].clone()
  }

  /// Update our variations by inserting the new variation with eval
  /// based on the eval.
  pub fn update(&mut self, variation: VariationWithEval) {
    // Check if we want to insert in the middle of the results
    for position in 0..self.len() {
      let better = match self.sort {
        Color::White => variation.eval >= self.variations[position].eval,
        Color::Black => variation.eval <= self.variations[position].eval,
      };
      if better {
        if self.variations.len() == self.lines {
          self.variations.remove(self.variations.len() - 1);
        }
        self.variations.insert(position, variation);
        return;
      }
    }

    // Else check if there is space left between lines (i.e. capacity) and self.len()
    if self.len() < self.lines {
      self.variations.insert(self.len(), variation);
    }
  }

  pub fn clear(&mut self) {
    self.variations.clear();
  }

  /// Returns the best evaluation in the current position
  pub fn get_eval(&self) -> Option<f32> {
    if self.variations.is_empty() {
      return None;
    }

    Some(self.variations[0].eval)
  }

  /// Returns the best move in the current result.
  pub fn get_best_move(&self) -> Option<Move> {
    if self.is_empty() {
      return None;
    }

    self.variations[0].variation.get_first_move()
  }

  pub fn get_top_moves(&self) -> Vec<Move> {
    let mut move_list = Vec::with_capacity(self.variations.len());
    for line in &self.variations {
      if !line.variation.is_empty() {
        if let Some(mv) = line.variation.get_first_move() {
          move_list.push(mv);
        }
      }
    }

    move_list
  }

  /// Put the previous move in the variations
  /// TODO: Explain well how this works
  pub fn push_move_to_variations(&mut self, mv: Move) {
    for line in &mut self.variations {
      line.variation.push_front(mv);
      line.eval = decrement_eval_if_mating_sequence(line.eval);
    }
    self.sort = Color::opposite(self.sort);
  }
}

impl Display for SearchResult {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Search Result:\n")?;
    let mut i = 0;
    for v in &self.variations {
      write!(
        f,
        "Line {:<2}: Eval {:<7.2} @ depth {} - {}\n",
        i,
        v.eval,
        v.variation.len(),
        v.variation
      )?;
      i += 1;
    }

    Ok(())
  }
}
