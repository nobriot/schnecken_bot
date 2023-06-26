use log::*;
use rand::Rng;
use std::cmp::Ordering;
use std::thread;
use std::time::{Duration, Instant};

// From our module
use crate::chess::engine::cache::get_engine_cache;
use crate::chess::engine::position_evaluation::*;
use crate::chess::engine::theory::*;
use crate::chess::model::board::Move;
use crate::chess::model::game_state::GameState;
use crate::chess::model::piece::*;

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

  // Sort moves based on their potential/interesting-ness
  pub fn sort_moves(&mut self) {
    //println!("sort_moves");
    let fen = self.game_state.to_string();
    self
      .game_state
      .move_list
      .sort_by(|a, b| best_move_potential(&fen, a, b));
  }

  // Checks the engine cache if we know the move list, else derives the
  // list of moves from the game state
  fn get_moves_with_cache(&mut self) {
    let fen = self.game_state.to_string();
    let fen_str = fen.as_str();

    // Check if we computed the same position before
    if let Some(cached_moves) = get_engine_cache().get_move_list(fen_str) {
      //debug!("Known position. Using the cache");
      self.game_state.move_list = Move::string_to_vec(cached_moves.as_str());
      self.game_state.available_moves_computed = true;
    } else {
      //debug!("New position. Computing manually");
      self.game_state.get_moves();
      get_engine_cache().set_move_list(fen_str, Move::vec_to_string(&self.game_state.move_list));
    }
  }

  fn get_game_state_with_cache(&mut self) {
    let fen = self.game_state.to_string();
    let fen_str = fen.as_str();

    if let Some(game_phase) = get_engine_cache().get_game_phase(fen_str) {
      self.game_state.game_phase = Some(game_phase);
    } else {
      self.game_state.update_game_phase();
      if let Some(phase) = self.game_state.game_phase {
        get_engine_cache().set_game_phase(fen_str, phase);
      }
    }
  }

  fn get_eval_with_cache(&mut self) {
    let fen = self.game_state.to_string();
    let fen_str = fen.as_str();

    // TODO: Check if we just did a 3-fold repetition
    if let Some(evaluation) = get_engine_cache().get_eval(fen_str) {
      self.eval = Some(evaluation);
      (_, self.game_over) = is_game_over(&self.game_state);
    } else {
      let (eval, game_over) = evaluate_position(&self.game_state);
      self.eval = Some(eval);
      self.game_over = game_over;
      get_engine_cache().set_eval(fen_str, eval);
    }
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
      // Check if we computed the same position before
      self.get_moves_with_cache();
      self.get_game_state_with_cache();
      self.get_eval_with_cache();
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
    if self.variations[0].eval.is_some() {
      if self.variations[0].eval.unwrap() > 100.0 {
        self.eval = Some(self.variations[0].eval.unwrap() - 2.0);
      } else if self.variations[0].eval.unwrap() < -100.0 {
        self.eval = Some(self.variations[0].eval.unwrap() + 2.0);
      } else {
        self.eval = self.variations[0].eval;
      }
    }
  }

  // Returns true if moves are added, false otherwise
  pub fn add_next_moves(&mut self) -> bool {
    //println!("add_next_moves");
    if self.game_over == true || self.eval.is_none() {
      return false;
    }

    if self.game_state.move_list.len() == 0 {
      self.get_moves_with_cache();

      self.sort_moves();
    }

    if self.eval.is_some() && self.variations.len() < self.game_state.move_list.len() {
      for m in &self.game_state.move_list {
        let mut move_found = false;
        for i in 0..self.variations.len() {
          if self.variations[i].chess_move == *m {
            move_found = true;
            break;
          }
        }
        if move_found == false {
          let mut new_game_state = GameState::from_string(self.game_state.to_string().as_str());
          new_game_state.apply_move(m, false);
          let chess_line = ChessLine {
            game_state: new_game_state,
            chess_move: m.clone(),
            variations: Vec::new(),
            eval: None,
            game_over: false,
          };
          self.variations.push(chess_line);
        }
      }
      return true;
    }

    if self.eval.is_some() && self.variations.len() == self.game_state.move_list.len() {
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
  pub fn prune_lines(&mut self) {
    if self.variations.len() < 5 || self.variations.len() < self.game_state.get_moves().len() {
      return;
    }

    for i in 0..self.variations.len() {
      self.variations[i].prune_lines();
      break;
    }

    // We want all the variations evaluated for that depth before we prune
    for i in 0..self.variations.len() {
      if let None = self.variations[i].eval {
        return;
      }
    }

    // Now we can compare evaluations.
    let best_evaluation = self.variations[0].eval.unwrap();

    let mut snip_index = self.variations.len();
    for i in 1..self.variations.len() {
      if (best_evaluation - self.variations[i].eval.unwrap()).abs() > 5.0 {
        snip_index = i;
        break;
      }
    }

    // Always keep at least 5 lines and max 20
    if snip_index < 5 {
      snip_index = 5;
    } else if snip_index > 20 {
      snip_index = 20;
    }

    if snip_index < self.variations.len() - 1 {
      //println!("Keeping only {snip_index} lines from");
      //display_lines(0, &self.variations);
      self.variations.truncate(snip_index)
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
    (None, None) => return Ordering::Equal,
    (None, _) => return Ordering::Greater,
    (_, None) => return Ordering::Less,
    (_, _) => {
      let a_value = a.eval.unwrap();
      let b_value = b.eval.unwrap();
      let a_depth = a.get_depth();
      let b_depth = b.get_depth();

      if a_value.abs() > 100.0 || b_value.abs() > 100.0 {
        if a_value > b_value {
          return less;
        } else if a_value < b_value {
          return greater;
        } else if a_depth > b_depth {
          return Ordering::Greater;
        } else if a_depth < b_depth {
          return Ordering::Less;
        }
        return Ordering::Equal;
      }

      // Not checkmates, we trust longer lines for relatively close evals.
      let value_diff = a_value - b_value;

      if value_diff.abs() < 0.5 {
        if a_depth > (b_depth + value_diff.abs() as usize) {
          return Ordering::Less;
        } else if (a_depth + value_diff.abs() as usize) < b_depth {
          return Ordering::Greater;
        }
      }

      if a_value > b_value {
        return less;
      } else if a_value < b_value {
        return greater;
      }
      return Ordering::Equal;
    },
  }
}

// Returns which moves seems most interesting, based the game state.
// 1. Double checks
// 2. Queen promotions
// 3. Captures (ordered by captured material)
// 4. Checks
// 5. Tempos
// All the rest ?
pub fn best_move_potential(fen: &String, a: &Move, b: &Move) -> Ordering {
  let game_state = GameState::from_string(fen.as_str());
  let mut game_state_a = GameState::from_string(fen.as_str());
  game_state_a.apply_move(a, false);
  let mut game_state_b = GameState::from_string(fen.as_str());
  game_state_b.apply_move(b, false);

  match (game_state_a.checks, game_state_b.checks) {
    (2, 2) => {
      // Continue the comparison to tie-break.
    },
    (2, _) => return Ordering::Less,
    (_, 2) => return Ordering::Greater,
    (_, _) => {},
  }

  match (a.promotion, b.promotion) {
    (BLACK_QUEEN | WHITE_QUEEN, BLACK_QUEEN | WHITE_QUEEN) => return Ordering::Equal,
    (BLACK_QUEEN | WHITE_QUEEN, _) => return Ordering::Less,
    (_, BLACK_QUEEN | WHITE_QUEEN) => return Ordering::Greater,
    (_, _) => {},
  }

  let a_captured_value =
    Piece::material_value_from_u8(game_state.board.squares[a.dest as usize]).abs();
  let b_captured_value =
    Piece::material_value_from_u8(game_state.board.squares[b.dest as usize]).abs();

  if a_captured_value > b_captured_value {
    return Ordering::Less;
  } else if a_captured_value < b_captured_value {
    return Ordering::Greater;
  }

  // We like castling in general:
  let a_castle = game_state.board.is_castle(a);
  let b_castle = game_state.board.is_castle(b);
  match (a_castle, b_castle) {
    (true, true) => {},
    (true, false) => return Ordering::Less,
    (false, true) => return Ordering::Greater,
    (_, _) => {},
  }

  match (game_state_a.checks, game_state_b.checks) {
    (1, 1) => {},
    (1, _) => return Ordering::Less,
    (_, 1) => return Ordering::Greater,
    (_, _) => {},
  }

  return Ordering::Equal;
}

// Sorts the chess lines based on what side would play in each variation
pub fn sort_chess_lines(side_to_move: Color, lines: &mut Vec<ChessLine>) {
  //println!("sort_chess_lines {}", side_to_move);
  lines.sort_by(|a, b| best_evaluation(a, b, side_to_move));
  //println!("end of sort_chess_lines {}", side_to_move);
}

pub fn evaluate_next_nodes(chess_line: &mut ChessLine, deadline: Instant) {
  // Add the next layer of moves.
  if false == chess_line.add_next_moves() {
    return;
  }

  // Process all the moves:
  chess_line.evaluate(deadline);

  // Rank the moves by eval
  chess_line.sort_variations();
  chess_line.back_propagate_evaluations();
}

// For now we just apply the entire line and evaluate the end result
pub fn select_best_move(fen: &str, deadline: Instant) -> Result<Vec<ChessLine>, ()> {
  let mut chess_lines: Vec<ChessLine> = Vec::new();
  let mut game_state = GameState::from_string(fen);

  // Get the list of moves to assess:
  let _ = game_state.get_moves();
  if game_state.move_list.len() == 0 {
    return Err(());
  }

  // Add all the moves to the chess lines:
  for m in &game_state.move_list {
    let mut new_game_state = GameState::from_string(game_state.to_string().as_str());
    new_game_state.apply_move(m, false);
    let chess_line = ChessLine {
      game_state: new_game_state,
      chess_move: m.clone(),
      variations: Vec::new(),
      eval: None,
      game_over: false,
    };
    chess_lines.push(chess_line);
  }

  // Process all the moves, all ratings
  for i in 0..chess_lines.len() {
    chess_lines[i].get_moves_with_cache();
    chess_lines[i].sort_moves();
    chess_lines[i].evaluate(deadline);
  }

  if chess_lines.len() <= 1 {
    // Only 1 legal move, no need to think too much about that
    return Ok(chess_lines);
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
        sort_chess_lines(game_state.side_to_play, &mut chess_lines);
        return Ok(chess_lines);
      }
    }

    // Go one level deeper in the tree. Spawn one thread per line.
    let mut handles = Vec::new();
    while chess_lines.len() > 0 {
      let mut line = chess_lines.pop().unwrap();
      let deadline_copy = deadline;
      handles.push(thread::spawn(move || {
        evaluate_next_nodes(&mut line, deadline_copy);
        line
      }));
    }

    // Wait that all the threads are done:
    chess_lines = handles.into_iter().map(|t| t.join().unwrap()).collect();

    // Rank the moves by eval
    for i in 0..chess_lines.len() {
      chess_lines[i].sort_variations();
      chess_lines[i].back_propagate_evaluations();
    }

    sort_chess_lines(game_state.side_to_play, &mut chess_lines);

    // Prune branches with low evaluations
    for i in 0..chess_lines.len() {
      chess_lines[i].prune_lines();
    }
  } // loop
}

pub fn play_move(game_state: &mut GameState, suggested_time_ms: u64) -> Result<String, ()> {
  let fen = game_state.to_string();
  // Check if it is a known position
  if let Some(moves) = get_theory_moves(&fen) {
    info!("We are in theory! Easy");
    let mut rng = rand::thread_rng();
    let random_good_move = rng.gen_range(0..moves.len());
    return Ok(moves[random_good_move].to_string());
  }

  // Try to evaluate ourselves.
  info!("Using {suggested_time_ms} ms to find a move");
  let deadline = Instant::now()
    + Duration::new(
      suggested_time_ms / 1000,
      (suggested_time_ms % 1000) as u32 * 1_000_000,
    );

  if let Ok(chess_lines) = select_best_move(&fen, deadline) {
    display_lines(5, &chess_lines);

    // Check how tight is the evaluation between the best lines
    let mut move_cutoff = 0;
    let best_eval = chess_lines[0].eval.unwrap();
    loop {
      if (move_cutoff + 1) < chess_lines.len()
        && (best_eval - chess_lines[move_cutoff + 1].eval.unwrap()).abs() < 0.2
      {
        move_cutoff += 1;
      } else {
        break;
      }
    }

    // Select a move amongs the best moves:
    let index;
    if move_cutoff > 1 {
      let mut rng = rand::thread_rng();
      index = rng.gen_range(0..move_cutoff);
    } else {
      index = move_cutoff;
    }
    debug!("Playing {index}th best move");

    return Ok(chess_lines[index].chess_move.to_string());
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
    println!(
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
    display_lines(0, &chess_lines);
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
    assert_eq!(chess_lines[0].eval.unwrap(), 196.0);
    assert_eq!(expected_move, chess_lines[0].chess_move);
  }

  #[test]
  fn test_select_best_defensive_move() {
    // Only good defense is : h8f8
    let fen = "r1bqk2r/ppppbp1p/2n5/3Bp1pQ/4P3/3P4/PPPN1PPP/R3K1NR b KQq - 0 7";
    let deadline = Instant::now() + Duration::new(5, 0);
    let chess_lines = select_best_move(fen, deadline).expect("This should not be an error");
    display_lines(10, &chess_lines);

    let expected_move = Move::from_string("h8f8");
    //assert_eq!(chess_lines[0].eval.unwrap(), 200.0);
    assert_eq!(expected_move, chess_lines[0].chess_move);
  }

  // Game https://lichess.org/Xjgkf4pp seemed really off. Testing some of the positions here
  #[test]
  fn test_select_pawn_capture() {
    let fen = "r2q1rk1/1pp1ppbp/p2p1np1/P7/6bP/R1N1Pn2/1PPP1PP1/2BQKB1R w K - 0 11";
    let deadline = Instant::now() + Duration::new(2, 10000000);
    let chess_lines = select_best_move(fen, deadline).expect("This should not be an error");
    display_lines(3, &chess_lines);
    display_lines(3, &chess_lines[2].variations[0].variations);

    let expected_move = Move::from_string("g2f3");
    //assert_eq!(chess_lines[0].eval.unwrap(), 200.0);
    assert_eq!(expected_move, chess_lines[0].chess_move);
  }

  #[test]
  fn evaluate_checkmate_with_castle() {
    let fen = "8/8/8/8/2nN4/1q6/ppP1NPPP/1k2K2R w K - 0 1";
    let deadline = Instant::now() + Duration::new(0, 10000000);
    let chess_lines = select_best_move(fen, deadline).expect("This should not be an error");
    assert_eq!("e1g1", chess_lines[0].chess_move.to_string());
    assert_eq!(chess_lines[0].eval.unwrap(), 200.0);
  }

  // From game : https://lichess.org/47V8eE5x -
  // Did not capture the knight, it was very obvious to capture.
  // Spent 2900 ms to come up with this crap: d7d5
  #[test]
  fn capture_the_damn_knight() {
    let fen = "rnb2r1k/pppp2pp/5N2/8/1bB5/8/PPPPQPPP/RNB1K2R b KQ - 0 9";
    let deadline = Instant::now() + Duration::new(3, 0);
    let chess_lines = select_best_move(fen, deadline).expect("This should not be an error");
    display_lines(0, &chess_lines);
    let best_move = chess_lines[0].chess_move.to_string();
    if "f8f6" != best_move && "g7f6" != best_move {
      assert!(
        false,
        "Should have been either f8f6 or g7f6, instead we have: {best_move}"
      );
    }
  }

  // From game : https://lichess.org/SKF7qgMu -
  // Did not capture the knight, it was very obvious to capture.
  // Spent 2450 ms to come up with this crap: e5f5
  #[test]
  fn save_the_queen() {
    let fen = "rnbqk2r/pp3ppp/2pbpn2/3pQ3/B3P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 6";
    let deadline = Instant::now() + Duration::new(3, 0);
    let chess_lines = select_best_move(fen, deadline).expect("This should not be an error");
    display_lines(0, &chess_lines);

    let best_move = chess_lines[0].chess_move.to_string();
    if "e5g5" != best_move && "e5d4" != best_move && "e5c3" != best_move {
      assert!(
        false,
        "Should have been either e5g5, e5d4 or e5c3, instead we have: {best_move}"
      );
    }
  }

  #[test]
  fn sort_moves() {
    let fen = "rnbqk2r/pp3ppp/2pbpn2/3pQ3/B3P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 6";

    let mut chess_line = ChessLine {
      game_state: GameState::from_string(fen),
      chess_move: Move::default(),
      variations: Vec::new(),
      eval: None,
      game_over: false,
    };

    let _ = chess_line.game_state.get_moves();
    chess_line.sort_moves();

    for m in chess_line.game_state.get_moves() {
      println!("{m}");
    }
  }

  #[test]
  fn test_dont_hang_pieces() {
    /* Got this in a game, hanging a knight, after thinking for 16_000 ms :
     Line 0 Eval: 0.79999995 - f8h6 d5e4 d7d5 e4d3
     Line 1 Eval: -0.30000085 - e4f6 d5d3
     Line 2 Eval: 2.3999996 - b7b5 d5e4 d7d5 e4d3 e7e5 b1c3
     Line 3 Eval: 2.5499997 - b7b6 d5e4 d7d5 e4d3 e7e5 b1c3
     Line 4 Eval: 3.2999995 - c6b8 d5e4 d7d5 e4d3 b8c6 b1c3
    */
    let fen = "r1bqkb1r/1ppppp1p/p1n5/3Q4/4n3/5N2/PPPP1PPP/RNB1KB1R b KQkq - 0 7";
    let deadline = Instant::now() + Duration::new(3, 0);
    let chess_lines = select_best_move(fen, deadline).expect("This should not be an error");
    display_lines(0, &chess_lines);
    let best_move = chess_lines[0].chess_move.to_string();
    if "e4f6" != best_move && "e4d6" != best_move {
      assert!(
        false,
        "Should have been either e4f6 or e4d6, instead we have: {best_move}"
      );
    }
  }

  #[test]
  fn test_dont_hang_pieces_2() {
    /*
      https://lichess.org/zcQesp7F#69
      Here we blunded a rook playing e2f2
      2k5/pp5p/2p3p1/8/1PpP4/P5KP/4r2P/8 b - - 1 35
      Using 1355 ms to find a move
      Line 0 Eval: -9.860003 - e2f2 g3f2 c8b8 f2g1 c4c3 g1g2 c3c2 g2g1 c2c1Q
      Line 1 Eval: -9.250003 - e2e5 d4e5 c8b8 g3g2 c4c3 e5e6 c3c2 e6e7 c2c1Q
      Line 2 Eval: -7.820003 - e2a2 g3f3 a2a3 f3g2
      Line 3 Eval: -8.105003 - e2h2 g3g4 h2e2
      Line 4 Eval: -7.9150023 - e2d2 b4b5 d2d4
      [2023-05-12T06:06:18Z INFO  schnecken_bot] Playing move e2f2 for game id zcQesp7F

    */
    let fen = "2k5/pp5p/2p3p1/8/1PpP4/P5KP/4r2P/8 b - - 1 35";
    let deadline = Instant::now() + Duration::new(1, 0);
    let chess_lines = select_best_move(fen, deadline).expect("This should not be an error");
    display_lines(0, &chess_lines);
    // Hanging the piece should not be in the top 10
    for i in 0..10 {
      if "e2f2" == chess_lines[i].chess_move.to_string() {
        assert!(
          false,
          "Top move {i} is e2f2, which is almost the worst move"
        );
      }
    }
  }
}
