// Chess Engine
use crate::engine::eval::position::{Score, decrement_mate_score, mate_distance_in_plies};
// Chess model
use crate::model::moves::Move;
use crate::model::piece::Color;
use std::fmt::Display;

const VARIATION_LENGTH: usize = 10;

#[derive(Debug, Clone)]
pub struct Variation {
  moves:  [Move; VARIATION_LENGTH],
  length: u8,
}

impl Variation {
  pub fn new() -> Self {
    Variation { moves:  [Move::null(); 10],
                length: 0, }
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
    Some(self.moves)
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

impl Default for Variation {
  fn default() -> Self {
    Variation::new()
  }
}

#[derive(Debug, Clone)]
pub struct VariationWithEval {
  pub variation:  Variation,
  pub eval:       Score,
  /// Number of plies searched to get the eval, from the first move of the
  /// variation (including search extensions and the plies of cached
  /// evaluations). It can be larger than the length of the variation.
  pub depth:      usize,
  /// The variation ends with a draw by repetition (or 50 moves): its eval
  /// depends on the moves that led to the position, not only on the board.
  pub repetition: bool,
}

impl VariationWithEval {
  /// Creates a new variation with eval from a single move
  pub fn new_from_move(eval: Score, mv: Move) -> Self {
    Self::new_from_move_with_depth(eval, 1, mv)
  }

  /// Creates a new variation with eval from a single move, for an eval that
  /// was obtained by searching `depth` plies (e.g. read from the eval cache)
  pub fn new_from_move_with_depth(eval: Score, depth: usize, mv: Move) -> Self {
    let mut variation = Variation::new();
    variation.add(mv);
    VariationWithEval { variation,
                        eval,
                        depth: depth.max(1),
                        repetition: false }
  }

  /// Creates a new variation from a single move leading to a draw by
  /// repetition (or 50 moves)
  pub fn new_repetition_draw(mv: Move) -> Self {
    VariationWithEval { repetition: true,
                        ..Self::new_from_move(0, mv) }
  }

  /// Formats the eval in pawns, followed by the mate notation for mate
  /// scores, e.g. `1.25` or `-299.94 (#-3)`
  pub fn eval_string(&self) -> String {
    let pawns = format!("{:.2}", self.eval as f32 / 100.0);
    match mate_string(self.eval) {
      Some(mate) => format!("{pawns} ({mate})"),
      None => pawns,
    }
  }
}

/// Mate notation (`#3`, or `#-3` when Black mates) for mate scores, in moves
fn mate_string(eval: Score) -> Option<String> {
  mate_distance_in_plies(eval).map(|plies| {
                                format!("#{}{}", if eval < 0 { "-" } else { "" }, plies.div_ceil(2))
                              })
}

/// Search result. Contains lines (vector of moves) associated with an
/// evaluation
#[derive(Debug, Clone)]
pub struct SearchResult {
  // How many lines/variations do we keep track of
  lines:          usize,
  // How do we sort evals (from white or black point of view)
  sort:           Color,
  // Top `lines` variations.
  // FIXME: Put that private at some point
  pub variations: Vec<VariationWithEval>,
}

impl SearchResult {
  /// Creates a new search result
  pub fn new(lines: usize, color: Color) -> Self {
    SearchResult { lines,
                   sort: color,
                   variations: Vec::with_capacity(lines) }
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

    // Else check if there is space left between lines (i.e. capacity) and
    // self.len()
    if self.len() < self.lines {
      self.variations.insert(self.len(), variation);
    }
  }

  pub fn clear(&mut self) {
    self.variations.clear();
  }

  /// Returns the best evaluation in the current position
  pub fn get_eval(&self) -> Option<Score> {
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
      if !line.variation.is_empty()
         && let Some(mv) = line.variation.get_first_move()
      {
        move_list.push(mv);
      }
    }

    move_list
  }

  /// Put the previous move in the variations
  /// TODO: Explain well how this works
  pub fn push_move_to_variations(&mut self, mv: Move) {
    for line in &mut self.variations {
      line.variation.push_front(mv);
      line.eval = decrement_mate_score(line.eval);
      line.depth += 1;
    }
    self.sort = Color::opposite(self.sort);
  }
}

impl Display for SearchResult {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "Search Result:")?;
    for (i, v) in self.variations.iter().enumerate() {
      let eval = mate_string(v.eval).unwrap_or_else(|| format!("{:.2}", v.eval as f32 / 100.0));
      writeln!(f, "Line {:<2}: Eval {:<7} @ depth {} - {}", i, eval, v.depth, v.variation)?;
    }

    Ok(())
  }
}
