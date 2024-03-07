use std::fmt::Display;

// Chess model
use crate::model::moves::Move;
use crate::model::piece::Color;

// Chess Engine
use crate::engine::decrement_eval_if_mating_sequence;

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

#[derive(Debug, Clone)]
pub struct VariationWithEval {
  pub variation: Vec<Move>,
  pub eval: f32,
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

  /// Returns the best evaluation in the current result.
  pub fn get_best_eval(&self) -> f32 {
    if self.variations.is_empty() {
      return f32::NAN;
    }

    self.variations[0].eval
  }

  /// Returns the best move in the current result.
  pub fn get_best_move(&self) -> Move {
    if self.is_empty() {
      return Move::default();
    }

    self.variations[0].variation[0]
  }

  pub fn get_top_moves(&self) -> Vec<Move> {
    let mut move_list = Vec::new();
    for line in &self.variations {
      if !line.variation.is_empty() {
        move_list.push(line.variation[0]);
      }
    }

    move_list
  }

  /// Put the previous move in the variations
  /// TODO: Explain well how this works
  pub fn push_move_to_variations(&mut self, mv: Move) {
    for line in &mut self.variations {
      line.variation.insert(0, mv);
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
        Move::vec_to_string(&v.variation)
      )?;
      i += 1;
    }

    Ok(())
  }
}
