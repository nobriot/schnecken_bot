use log::*;
use rand::Rng;
use std::cmp;
use std::cmp::Ordering;
use std::time::{Duration, Instant};

// From our module
use crate::chess::model::board::Move;
use crate::chess::model::game_state::GameState;
use crate::chess::model::piece::*;
use crate::chess::theory::*;

#[derive(Debug, Clone)]
pub struct ChessLine {
  pub game_state: GameState,
  pub chess_move: Move,
  pub variations: Vec<ChessLine>,
  pub eval: Option<f32>,
  pub game_over: bool,
}

impl ChessLine {
  pub fn get_depth(&self) -> usize {
    //println!("get_depth");
    if self.variations.len() == 0 {
      return 1;
    }
    let mut depth: usize = 2;
    let mut line = &self.variations[0];
    loop {
      if line.variations.len() > 0 {
        line = &line.variations[0];
        depth += 1;
      } else {
        break;
      }
    }

    //println!("Returning {}", depth);
    return depth;
  }

  // Sort based on what side is supposed to play first for the line
  pub fn sort_variations(&mut self) {
    //println!("sort_variations");
    for i in 0..self.variations.len() {
      self.variations[i].sort_variations();
    }
    self
      .variations
      .sort_by(|a, b| best_evaluation(a, b, self.game_state.side_to_play));
  }

  // Evaluate all variations
  pub fn evaluate(&mut self, deadline: Instant) {
    //println!("Evaluate");
    let current_time = Instant::now();
    if current_time > deadline {}
    if self.game_over == true {
      return;
    }

    if self.variations.len() == 0 {
      let (eval, game_over) = self.game_state.evaluate_position();
      self.eval = Some(eval);
      self.game_over = game_over;
      return;
    }

    let mut game_over = false;
    let mut variation_index = 0;
    for i in 0..self.variations.len() {
      self.variations[i].evaluate(deadline);
      if self.variations[i].game_over == true {
        game_over = true;
        variation_index = i;
        break;
      }
    }

    // Keep only 1 variation since it is game over (ideally it should be checkmates, not stalemates though)
    if game_over == true {
      let mut new_variations: Vec<ChessLine> = Vec::new();
      new_variations.push(self.variations[variation_index].to_owned());
      self.variations = new_variations;
    }
  }

  // Evaluate all variations
  pub fn back_propagate_evaluations(&mut self) {
    //println!("back_propagate_evaluations");
    // Assuming here that everything is evaluated and that it is sorted.
    if self.variations.len() == 0 {
      return;
    }
    self.variations[0].back_propagate_evaluations();
    self.eval = self.variations[0].eval;
  }

  // Returns true if moves are added, false otherwise
  pub fn add_next_moves(&mut self) -> bool {
    //println!("add_next_moves");
    if self.game_over == true {
      return false;
    }
    if self.variations.len() == 0 {
      let candidates = ranked_moves(self.game_state);
      if candidates.len() == 0 {
        error!("We just computed moves for a game that is over. This is a waste of CPU.");
        self.game_over = true;
        return false;
      }

      // Add all the moves to the chess lines:
      for m in candidates {
        let mut new_game_state = self.game_state.clone();
        new_game_state.apply_move(m);
        let chess_line = ChessLine {
          game_state: new_game_state,
          chess_move: m,
          variations: Vec::new(),
          eval: None,
          game_over: false,
        };
        self.variations.push(chess_line);
      }
      return true;
    } else {
      for i in 0..self.variations.len() {
        if true == self.variations[i].add_next_moves() {
          return true;
        }
      }
    }
    return false;
  }

  /// Keeps only the top "number_of_lines" in the tree.
  /// You should call this on a sorted set of lines
  pub fn trim_lines(&mut self, number_of_lines: usize) {
    //println!("trim_lines");
    if self.variations.len() == 0 {
      return;
    }
    self.variations.truncate(number_of_lines);

    for i in 0..self.variations.len() {
      if self.variations[i].get_depth() == 2 {
        self.variations[i].trim_lines(number_of_lines);
        break;
      }
    }
  }
}

pub fn best_evaluation(a: &ChessLine, b: &ChessLine, s: Color) -> Ordering {
  // Adjust what is better for what side:
  let greater;
  let less;
  match s {
    Color::White => {
      greater = Ordering::Greater;
      less = Ordering::Less;
    },
    Color::Black => {
      greater = Ordering::Less;
      less = Ordering::Greater;
    },
  }

  // Now compare the evaluations
  match (a.eval, b.eval) {
    (None, _) => return greater,
    (_, None) => return less,
    (_, _) => {
      let a_value = a.eval.unwrap();
      let b_value = b.eval.unwrap();
      if a_value > b_value {
        return less;
      } else if a_value < b_value {
        return greater;
      } else {
        // Shortest line with same eval is better: FIXME: This is true only for stuff like checkmate.
        let a_depth = a.get_depth();
        let b_depth = b.get_depth();
        if a_depth > b_depth {
          return Ordering::Greater;
        } else if a_depth < b_depth {
          return Ordering::Less;
        } else {
          return Ordering::Equal;
        }
      }
    },
  }
}

// Sort the moves based on "interesting-ness"
// 1. Double checks
// 2. Checks
// 3. Promotions
// 4. Captures
// 5. Tempos - pins ?
// 6. All the rest
pub fn ranked_moves(game_state: GameState) -> Vec<Move> {
  let moves = game_state.get_moves();

  // Try to apply all the moves and quickly look at the resulting position:
  let mut moves_arrays: [Vec<Move>; 5] = Default::default();
  for m in &moves {
    let capture: bool = match game_state.board.squares[m.dest as usize] {
      NO_PIECE => false,
      _ => true,
    };
    let mut new_game_state = game_state.clone();
    new_game_state.apply_move(*m);

    match (new_game_state.checks, capture, m.promotion) {
      (2, _, _) => moves_arrays[0].push(*m),
      (_, _, WHITE_QUEEN | BLACK_QUEEN) => moves_arrays[1].push(*m),
      (1, _, _) => moves_arrays[2].push(*m),
      (0, true, _) => moves_arrays[3].push(*m),
      (_, _, _) => moves_arrays[4].push(*m),
    }
  }

  let mut moves = Vec::new();
  for index in 0..moves_arrays.len() {
    moves.append(&mut moves_arrays[index]);
  }

  moves
}

// Sorts the chess lines based on what side would play in each variation
pub fn sort_chess_lines(side_to_move: Color, lines: &mut Vec<ChessLine>) {
  //println!("sort_chess_lines {}", side_to_move);
  lines.sort_by(|a, b| best_evaluation(a, b, side_to_move));
  //println!("end of sort_chess_lines {}", side_to_move);
}

// For now we just apply the entire line and evaluate the end result
pub fn select_best_move(fen: &str, deadline: Instant) -> Result<Vec<ChessLine>, ()> {
  let mut chess_lines: Vec<ChessLine> = Vec::new();
  let game_state = GameState::from_string(fen);

  // Get the list of moves to assess:
  let candidates = ranked_moves(game_state);
  if candidates.len() == 0 {
    return Err(());
    //return Ok((Move::default(), game_state.evaluate_position()));
  }

  // Add all the moves to the chess lines:
  for m in candidates {
    let mut new_game_state = game_state.clone();
    new_game_state.apply_move(m);
    let chess_line = ChessLine {
      game_state: new_game_state,
      chess_move: m,
      variations: Vec::new(),
      eval: None,
      game_over: false,
    };
    chess_lines.push(chess_line);
  }

  // Process all the moves:
  for i in 0..chess_lines.len() {
    chess_lines[i].evaluate(deadline);
  }

  // Rank the moves by eval
  sort_chess_lines(game_state.side_to_play, &mut chess_lines);

  // Now loop the process:
  //display_lines(0, &chess_lines);
  loop {
    // Check if we have been thinking too much:
    let current_time = Instant::now();
    if current_time > deadline {
      if chess_lines.len() == 0 {
        return Err(());
      } else {
        return Ok(chess_lines);
      }
    }

    // Find the shortest non-game over line
    let mut index = 0;
    let mut depth = usize::MAX;
    let mut best_eval = 0.0;
    for i in 0..chess_lines.len() {
      if (i == 0)
        || (best_eval < chess_lines[i].eval.unwrap()
          && chess_lines[i].game_state.side_to_play == Color::White)
        || (best_eval > chess_lines[i].eval.unwrap()
          && chess_lines[i].game_state.side_to_play == Color::Black)
      {
        best_eval = chess_lines[i].eval.unwrap();
      }
    }
    for i in 0..cmp::min(chess_lines.len(), 10) {
      if chess_lines[i].get_depth() < depth
        && chess_lines[i].game_over == false
        && chess_lines[i].game_over == false
      {
        depth = chess_lines[i].get_depth();
        index = i;
      }
    }
    if depth == usize::MAX {
      // We have nothing to look at ? (all the lines are Game Over ??)
      return Ok(chess_lines);
    }

    if false == chess_lines[index].add_next_moves() {
      return Ok(chess_lines);
    };

    // Process all the moves:
    for i in 0..chess_lines.len() {
      chess_lines[i].evaluate(deadline);
    }

    // Rank the moves by eval
    for i in 0..chess_lines.len() {
      chess_lines[i].sort_variations();
    }
    for i in 0..chess_lines.len() {
      chess_lines[i].back_propagate_evaluations();
    }
    sort_chess_lines(game_state.side_to_play, &mut chess_lines);

    // Trim branches with low evaluations
    //for i in 0..chess_lines.len() {
    //  chess_lines[i].trim_lines(10);
    // }
  } // loop
}

pub fn play_move(game_state: &GameState, suggested_time_ms: u64) -> Result<String, ()> {
  let fen = game_state.to_string();
  // Check if it is a known position
  if let Some(moves) = get_theory_moves(&fen) {
    info!("We are in theory! Easy");
    let mut rng = rand::thread_rng();
    let random_good_move = rng.gen_range(0..moves.len());
    return Ok(moves[random_good_move].to_string());
  }
  info!("We're out of theory for {fen}");

  // Try to evaluate ourselves.
  info!("Using {suggested_time_ms} ms to find a move");
  let deadline = Instant::now()
    + Duration::new(
      suggested_time_ms / 1000,
      (suggested_time_ms % 1000) as u32 * 1_000_000,
    );

  if let Ok(chess_lines) = select_best_move(&fen, deadline) {
    display_lines(3, &chess_lines);
    return Ok(chess_lines[0].chess_move.to_string());
  }

  // Fallback on playing a random move:
  warn!("Eval went wrong. Playing a random move!");
  let move_list = game_state.get_moves();
  if move_list.len() == 0 {
    warn!("Cannot compute any move from fen: {fen}");
    return Err(());
  }

  let mut rng = rand::thread_rng();
  let random_legal_move = rng.gen_range(0..move_list.len());
  return Ok(move_list[random_legal_move].to_string());
}

pub fn display_lines(mut number_of_lines: usize, chess_lines: &Vec<ChessLine>) {
  if number_of_lines == 0 || number_of_lines > chess_lines.len() {
    number_of_lines = chess_lines.len();
  }
  for i in 0..number_of_lines {
    let mut moves: String = String::new();
    let mut current_line = &chess_lines[i];
    moves += current_line.chess_move.to_string().as_str();
    moves += " ";

    while current_line.variations.len() != 0 {
      current_line = &current_line.variations[0];
      moves += current_line.chess_move.to_string().as_str();
      moves += " ";
    }

    if current_line.game_over {
      moves += "/ Game Over";
    }
    debug!(
      "Line {} Eval: {} - {}",
      i,
      chess_lines[i].eval.unwrap_or(f32::NAN),
      moves
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_select_best_move_checkmate_in_one() {
    // This is a forced checkmate in 1:
    let fen = "1n4nr/5ppp/1N6/1P2p3/1P6/4kP2/1B1NP1PP/R3KB1R w KQ - 1 36";
    let deadline = Instant::now() + Duration::new(1, 0);
    let chess_lines = select_best_move(fen, deadline).expect("This should not be an error");
    display_lines(10, &chess_lines);
    let expected_move = Move::from_string("b6d5");
    assert_eq!(chess_lines[0].eval.unwrap(), 200.0);
    assert_eq!(expected_move, chess_lines[0].chess_move);
  }

  #[test]
  fn test_select_best_move_checkmate_in_one_for_black() {
    // This is a forced checkmate in 1:
    let fen = "8/8/2p1pkp1/p3p3/P1P1P1P1/6q1/7q/3K4 b - - 2 55";
    let deadline = Instant::now() + Duration::new(1, 0);
    let chess_lines = select_best_move(fen, deadline).expect("This should not be an error");
    display_lines(10, &chess_lines);
    let expected_move = Move::from_string("g3g1");
    assert_eq!(chess_lines[0].eval.unwrap(), -200.0);
    assert_eq!(expected_move, chess_lines[0].chess_move);
  }

  #[test]
  fn test_select_best_move_checkmate_in_two() {
    // This is a forced checkmate in 2: c1b2 d4e3 b6d5
    let fen = "1n4nr/5ppp/1N6/1P2p3/1P1k4/5P2/1p1NP1PP/R1B1KB1R w KQ - 0 35";
    let deadline = Instant::now() + Duration::new(5, 0);
    let chess_lines = select_best_move(fen, deadline).expect("This should not be an error");
    display_lines(10, &chess_lines);
    display_lines(5, &chess_lines[0].variations);

    let expected_move = Move::from_string("c1b2");
    //assert_eq!(chess_lines[0].eval.unwrap(), 200.0);
    assert_eq!(expected_move, chess_lines[0].chess_move);
  }

  #[test]
  fn test_select_best_defensive_move() {
    // Only good defense is : h8f8
    let fen = "r1bqk2r/ppppbp1p/2n5/3Bp1pQ/4P3/3P4/PPPN1PPP/R3K1NR b KQq - 0 7";
    let deadline = Instant::now() + Duration::new(5, 0);
    let chess_lines = select_best_move(fen, deadline).expect("This should not be an error");
    display_lines(10, &chess_lines);
    println!("Line 0 sublines: ");
    display_lines(0, &chess_lines[1].variations);

    let expected_move = Move::from_string("h8f8");
    //assert_eq!(chess_lines[0].eval.unwrap(), 200.0);
    assert_eq!(expected_move, chess_lines[0].chess_move);
  }
}
