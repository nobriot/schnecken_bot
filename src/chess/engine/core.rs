use log::*;
use rand::Rng;
use std::cmp::Ordering;
use std::time::{Duration, Instant};

// From our module
use super::cache::get_engine_cache;
use super::eval::position::*;
use super::theory::*;

// From other modules
use crate::chess::model::board::Move;
use crate::chess::model::game_state::GameState;
use crate::chess::model::piece::*;

const PRUNE_CUTOFF: f32 = 6.0;

#[derive(Debug, Clone)]
pub struct ChessLine {
  pub game_state: GameState,
  pub chess_move: Move,
  pub variations: Vec<ChessLine>,
  //pub move_hints: Vec<Move>,
  pub eval: Option<f32>,
  pub game_over: bool,
  pub permutation: bool, // Stop evaluating or calculating stuff for permutations
}

impl ChessLine {
  pub fn get_depth(&self) -> usize {
    //println!("get_depth");
    if self.variations.is_empty() {
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
      //println!("Known position {fen}. Using the cache");
      self.game_state.move_list = cached_moves;
      self.game_state.available_moves_computed = true;
    } else {
      //println!("New position {fen}. Computing manually");
      get_engine_cache().set_move_list(fen_str, self.game_state.get_moves());
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

  /// Assigns an evaluation to a position, using the engine cache.
  ///
  fn get_eval_with_cache(&mut self) {
    // Check if we reach a game over state first.
    let (eval, game_over) = is_game_over(&self.game_state);
    if game_over {
      self.eval = Some(eval);
      self.game_over = true;
      return;
    }

    // Never evaluated before
    if self.eval.is_none() {
      let fen = self.game_state.to_string();
      let fen_str = fen.as_str();
      if let Some(evaluation) = get_engine_cache().get_eval(fen_str) {
        self.eval = Some(evaluation);
        self.permutation = true;
      } else {
        let (eval, game_over) = evaluate_position(&self.game_state);
        self.eval = Some(eval);
        self.game_over = game_over;
        if !game_over {
          get_engine_cache().set_eval(fen_str, eval);
        }
      }
    }
  }

  /// Evaluate all variations in the ChessLine
  ///
  /// ### Arguments
  ///
  /// * `evaluate_all`:
  ///   If true, evaluate all moves.
  ///   If false, evaluates only "active" moves (checks, captures, promotions)
  /// * `deadline`: Timestamp at which, we have to return and stop evaluating
  ///
  pub fn evaluate(&mut self, evaluate_all: bool, deadline: Instant) {
    //println!("Evaluate");
    if Instant::now() > deadline || self.game_over {
      return;
    }

    if self.eval.is_none() || (self.variations.is_empty() && !self.permutation) {
      // Check if we computed the same position before
      self.get_moves_with_cache();
      self.get_game_state_with_cache();
      self.get_eval_with_cache();
      return;
    }

    for i in 0..self.variations.len() {
      if evaluate_all {
        self.variations[i].evaluate(evaluate_all, deadline);
      } else {
        // Check if the variation is "active", else skip for now
        if self.game_state.board.squares[self.variations[i].chess_move.dest as usize] != NO_PIECE
          || self.game_state.checks > 0
          || self.variations[i].game_state.checks > 0
          || self.variations[i].chess_move.promotion == WHITE_QUEEN
          || self.variations[i].chess_move.promotion == BLACK_QUEEN
        {
          self.variations[i].evaluate(evaluate_all, deadline);
        }
      }
    }

    // Check if we just evaluated some bad capture
    self
      .variations
      .sort_by(|a, b| best_evaluation(a, b, self.game_state.side_to_play));

    if !evaluate_all {
      // TODO: Check if we just had a bad capture going up the variations. If that's the case, evaluate all instead.
    }

    // When we just did a evaluate all, we want to already back-propagate / prune at depth n-1
    if evaluate_all {
      for i in 0..self.variations.len() {
        if self.variations[i].eval.is_none() {
          return;
        }
      }
      self.prune_lines();
    }
  }

  /// Sorts the end of the branch and back propagate upwards, recursively.
  pub fn back_propagate_evaluations(&mut self) {
    //println!("back_propagate_evaluations");
    // Assuming here that everything is evaluated and that it is sorted.
    if self.variations.is_empty() {
      return;
    }

    // Go deeper first
    for i in 0..self.variations.len() {
      self.variations[i].back_propagate_evaluations();
    }

    // Make sure to sort before back-propagating.
    self
      .variations
      .sort_by(|a, b| best_evaluation(a, b, self.game_state.side_to_play));

    if self.variations[0].eval.is_some() {
      if self.variations[0].eval.unwrap() > 100.0 {
        self.eval = Some(self.variations[0].eval.unwrap() - 2.0);
      } else if self.variations[0].eval.unwrap() < -100.0 {
        self.eval = Some(self.variations[0].eval.unwrap() + 2.0);
      } else {
        self.eval = self.variations[0].eval;
      }
      if self.eval.is_some() && !self.game_over {
        let fen = self.game_state.to_string();
        let fen_str = fen.as_str();
        get_engine_cache().set_eval(fen_str, self.eval.unwrap());
      }
    }
  }

  /// Go around along variations and checks the latest cache data to update
  /// permutation evaluations.
  pub fn update_permutations_eval(&mut self) {
    //println!("update_permutations_eval");

    for i in 0..self.variations.len() {
      self.variations[i].update_permutations_eval();
    }

    // Do not touch non-permutations!
    if !self.permutation {
      return;
    }

    let fen = self.game_state.to_string();
    let fen_str = fen.as_str();
    if get_engine_cache().has_fen(fen_str) {
      self.eval = get_engine_cache().get_eval(fen_str);
    } else {
      warn!("Permutation not found in the cache. Eval update will be skipped");
    }
  }

  /// Navigates at the end of the line and add more variations at the depth were
  /// variations stopped.
  pub fn add_next_moves(&mut self, deadline: Instant) -> bool {
    //println!("add_next_moves");
    if self.game_over || self.permutation || self.eval.is_none() || Instant::now() > deadline {
      return false;
    }

    if self.game_state.move_list.is_empty() {
      self.get_moves_with_cache();
      self.sort_moves();
    }

    let mut moves_added = false;
    if self.variations.is_empty() {
      for m in &self.game_state.move_list {
        let mut new_game_state = self.game_state.clone();
        new_game_state.apply_move(m, false);
        let chess_line = ChessLine {
          game_state: new_game_state,
          chess_move: m.clone(),
          variations: Vec::new(),
          eval: None,
          game_over: false,
          permutation: false,
        };
        self.variations.push(chess_line);
        moves_added = true;
      }
    } else {
      for i in 0..self.variations.len() {
        moves_added |= self.variations[i].add_next_moves(deadline);
      }
    }
    return moves_added;
  }

  /// Keeps only the top "number_of_lines" in the tree.
  /// You should call this on a sorted set of lines
  pub fn prune_lines(&mut self) {
    // Go in depth first
    for i in 0..self.variations.len() {
      self.variations[i].prune_lines();
      break;
    }

    if self.variations.is_empty() || self.variations[0].eval.is_none() {
      return;
    }

    // Do not remove what is not evaluated yet
    let mut stop_prune_index = self.variations.len();
    for i in 0..self.variations.len() {
      if self.variations[i].eval.is_none() {
        stop_prune_index = i;
        break;
      }
    }

    // Now we can compare evaluations.
    let best_evaluation = self.variations[0].eval.unwrap();
    let mut start_prune_index = self.variations.len();
    for i in 1..stop_prune_index {
      if (best_evaluation - self.variations[i].eval.unwrap()).abs() > PRUNE_CUTOFF {
        start_prune_index = i;
        break;
      }
    }
    if start_prune_index == self.variations.len() {
      return;
    }

    // Remove from start to stop:
    self.variations.drain(start_prune_index..stop_prune_index);
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

      if a_value > b_value {
        return less;
      } else if a_value < b_value {
        return greater;
      }
      return Ordering::Equal;
    },
  }
}

/// Returns which moves seems most interesting, based the game state.
///
/// It will sort using the following ordering:
/// 1. Double checks
/// 2. Queen promotions
/// 3. Captures (ordered by captured material)
/// 4. Checks
/// 5. Tempos
/// All the rest ?
pub fn best_move_potential(fen: &String, a: &Move, b: &Move) -> Ordering {
  let game_state = GameState::from_string(fen.as_str());
  let mut game_state_a = GameState::from_string(fen.as_str());
  game_state_a.apply_move(a, false);
  let mut game_state_b = GameState::from_string(fen.as_str());
  game_state_b.apply_move(b, false);

  match (game_state_a.checks, game_state_b.checks) {
    (2, 2) => {},
    (2, _) => return Ordering::Less,
    (_, 2) => return Ordering::Greater,
    (_, _) => {},
  }

  match (a.promotion, b.promotion) {
    (BLACK_QUEEN | WHITE_QUEEN, BLACK_QUEEN | WHITE_QUEEN) => {},
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

  // Single checks
  match (game_state_a.checks, game_state_b.checks) {
    (1, _) => return Ordering::Less,
    (_, 1) => return Ordering::Greater,
    (_, _) => {},
  }

  // We like castling in general:
  let a_castle = game_state.board.is_castle(a);
  let b_castle = game_state.board.is_castle(b);
  match (a_castle, b_castle) {
    (true, false) => return Ordering::Less,
    (false, true) => return Ordering::Greater,
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

// For now we just apply the entire line and evaluate the end result
pub fn select_best_move(
  game_state: &mut GameState,
  deadline: Instant,
) -> Result<Vec<ChessLine>, ()> {
  // Reset the engine cache:
  // FIXME: We should be able to call this function in parallel
  get_engine_cache().clear();

  let mut chess_lines: Vec<ChessLine> = Vec::new();

  // Get the list of moves to assess:
  let _ = game_state.get_moves();
  if game_state.move_list.is_empty() {
    return Err(());
  }
  let fen = game_state.to_string();
  game_state
    .move_list
    .sort_by(|a, b| best_move_potential(&fen, a, b));

  // Add all the moves to the chess lines:
  for m in &game_state.move_list {
    let mut new_game_state = game_state.clone();
    new_game_state.apply_move(m, false);
    let chess_line = ChessLine {
      game_state: new_game_state,
      chess_move: m.clone(),
      variations: Vec::new(),
      eval: None,
      game_over: false,
      permutation: false,
    };
    chess_lines.push(chess_line);
  }

  // Process all the moves, all ratings
  // Ignore the initial deadline, we do not want to return without evaluating anything
  let initial_deadline = Instant::now() + Duration::new(1, 0);
  for i in 0..chess_lines.len() {
    chess_lines[i].sort_moves();
    chess_lines[i].evaluate(true, initial_deadline);
  }

  if chess_lines.len() <= 1 {
    // Only 1 legal move, no need to think too much about that
    return Ok(chess_lines);
  }

  // Rank the moves by eval
  sort_chess_lines(game_state.side_to_play, &mut chess_lines);

  // Now loop the process:
  //display_lines(0, &chess_lines);
  let mut search_complete = false;
  let mut evaluate_all = false;
  let mut top_moves = chess_lines.len();
  loop {
    // Check if we have been thinking too much:
    if Instant::now() > deadline || search_complete {
      if chess_lines.is_empty() {
        return Err(());
      } else {
        sort_chess_lines(game_state.side_to_play, &mut chess_lines);
        return Ok(chess_lines);
      }
    }

    // Add depth to the tree. If we are stuck, switch into evaluate all.
    let mut moves_added = false;
    for i in 0..top_moves {
      moves_added |= chess_lines[i].add_next_moves(deadline);
    }
    if !moves_added && !evaluate_all {
      evaluate_all = true;
      continue;
    } else if !moves_added && evaluate_all {
      top_moves += 1;
      if top_moves > chess_lines.len() {
        top_moves = chess_lines.len()
      }
      // println!("We are stuck! ");
      // This condition should happen only 1 time in a row.
    }

    // Evaluate lines
    for i in 0..top_moves {
      chess_lines[i].evaluate(evaluate_all, deadline);
    }

    // Go around once more to find permutations and update them too
    for i in 0..top_moves {
      chess_lines[i].update_permutations_eval();
    }
    // Rank the moves by eval
    for i in 0..top_moves {
      chess_lines[i].back_propagate_evaluations();
    }

    sort_chess_lines(game_state.side_to_play, &mut chess_lines);

    // Prune branches with low evaluations
    for i in 0..top_moves {
      chess_lines[i].prune_lines();
    }

    // Keep analyzing branches based on their current eval:
    let best_eval = chess_lines[0].eval.unwrap();
    for i in 0..chess_lines.len() {
      if chess_lines[i].eval.is_none() {
        break;
      }
      if (best_eval - chess_lines[i].eval.unwrap()).abs() < PRUNE_CUTOFF {
        top_moves = i;
      }
    }

    // Check if we found a winning sequence:
    let mut current_line = &chess_lines[0];
    while !current_line.variations.is_empty() {
      current_line = &current_line.variations[0];
    }
    if current_line.game_over {
      search_complete = true;
    }

    // By default try looping with evaluate_all = false
    evaluate_all = false;
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
  info!(
    "Using {suggested_time_ms} ms to find a move for position {}",
    game_state.to_string()
  );
  game_state.update_game_phase();
  debug!("Game is in the {:?} phase", game_state.game_phase);
  let deadline = Instant::now() + Duration::from_millis(suggested_time_ms);

  if let Ok(chess_lines) = select_best_move(game_state, deadline) {
    display_lines(0, &chess_lines);

    // In super time pressure, we may not get around to evaluate moves at depth 1:
    if chess_lines[0].eval.is_none() {
      // Just play the first move from the list
      return Ok(chess_lines[0].chess_move.to_string());
    }

    // Check how tight is the evaluation between the best lines
    let mut move_cutoff = 0;
    let best_eval = chess_lines[0].eval.unwrap();
    loop {
      if (move_cutoff + 1) < chess_lines.len()
        && chess_lines[move_cutoff + 1].eval.is_some()
        && (best_eval - chess_lines[move_cutoff + 1].eval.unwrap()).abs() < 0.15
      {
        move_cutoff += 1;
      } else {
        break;
      }
    }

    // Select a move amongs the best moves:
    let index = if move_cutoff > 1 {
      let mut rng = rand::thread_rng();
      rng.gen_range(0..move_cutoff)
    } else {
      move_cutoff
    };
    debug!("Playing {index}th best move");

    return Ok(chess_lines[index].chess_move.to_string());
  }

  // Fallback on playing a random move:
  warn!("Eval went wrong. Playing a random move!");
  let move_list = game_state.get_moves();
  if move_list.is_empty() {
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
  for (i, line) in chess_lines.iter().enumerate().take(number_of_lines) {
    let mut moves: String = String::new();
    let mut current_line = line;
    moves += current_line.chess_move.to_string().as_str();
    moves += " ";

    while !current_line.variations.is_empty() {
      current_line = &current_line.variations[0];
      moves += current_line.chess_move.to_string().as_str();
      moves += " ";
    }

    if current_line.game_over {
      moves += "/ Game Over";
    }
    if current_line.permutation {
      moves += "/ Permutation";
    }
    println!(
      "Line {:<2} Eval: {:<7} - {}",
      i,
      format!("{:.4}", chess_lines[i].eval.unwrap_or(f32::NAN)),
      moves
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::chess::model::game_state::GamePhase;

  #[test]
  fn test_sorting_moves() {
    // This is a forced checkmate in 1:
    let fen = "8/8/8/8/2nN4/1q6/ppP1NPPP/1k2K2R w K - 0 1";
    let mut game_state = GameState::from_string(fen);
    game_state.get_moves();
    assert!(true == game_state.available_moves_computed);

    let move_nothing = Move::from_string("e1d1");
    let move_nothing_2 = Move::from_string("h2h3");
    let move_castle_check = Move::from_string("e1g1");
    let move_capture = Move::from_string("d4b3");
    let move_check = Move::from_string("e2c3");
    assert_eq!(
      Ordering::Greater,
      best_move_potential(&String::from(fen), &move_nothing, &move_castle_check)
    );
    assert_eq!(
      Ordering::Less,
      best_move_potential(&String::from(fen), &move_castle_check, &move_nothing)
    );
    assert_eq!(
      Ordering::Less,
      best_move_potential(&String::from(fen), &move_castle_check, &move_check)
    );
    assert_eq!(
      Ordering::Less,
      best_move_potential(&String::from(fen), &move_castle_check, &move_check)
    );
    assert_eq!(
      Ordering::Less,
      best_move_potential(&String::from(fen), &move_capture, &move_check)
    );
    assert_eq!(
      Ordering::Less,
      best_move_potential(&String::from(fen), &move_capture, &move_castle_check)
    );
    assert_eq!(
      Ordering::Equal,
      best_move_potential(&String::from(fen), &move_nothing, &move_nothing_2)
    );

    game_state
      .move_list
      .sort_by(|a, b| best_move_potential(&String::from(fen), a, b));

    assert_eq!("c2b3", game_state.move_list[0].to_string());
    assert_eq!("d4b3", game_state.move_list[1].to_string());
    assert_eq!("e1g1", game_state.move_list[2].to_string());
  }

  #[test]
  fn test_select_best_move_checkmate_in_one() {
    // This is a forced checkmate in 1:
    let fen = "1n4nr/5ppp/1N6/1P2p3/1P6/4kP2/1B1NP1PP/R3KB1R w KQ - 1 36";
    let mut game_state = GameState::from_string(fen);
    let deadline = Instant::now() + Duration::new(1, 0);
    let chess_lines =
      select_best_move(&mut game_state, deadline).expect("This should not be an error");
    display_lines(10, &chess_lines);
    let expected_move = Move::from_string("b6d5");
    assert_eq!(chess_lines[0].eval.unwrap(), 200.0);
    assert_eq!(expected_move, chess_lines[0].chess_move);
  }

  #[test]
  fn test_select_best_move_checkmate_in_one_for_black() {
    // This is a forced checkmate in 1:
    let fen = "8/8/2p1pkp1/p3p3/P1P1P1P1/6q1/7q/3K4 b - - 2 55";
    let mut game_state = GameState::from_string(fen);
    let deadline = Instant::now() + Duration::new(1, 0);
    let chess_lines =
      select_best_move(&mut game_state, deadline).expect("This should not be an error");
    display_lines(0, &chess_lines);
    let expected_move = Move::from_string("g3g1");
    assert_eq!(chess_lines[0].eval.unwrap(), -200.0);
    assert_eq!(expected_move, chess_lines[0].chess_move);
  }

  #[test]
  fn test_select_best_move_checkmate_in_two() {
    // This is a forced checkmate in 2: c1b2 d4e3 b6d5
    let fen = "1n4nr/5ppp/1N6/1P2p3/1P1k4/5P2/1p1NP1PP/R1B1KB1R w KQ - 0 35";
    let mut game_state = GameState::from_string(fen);
    let deadline = Instant::now() + Duration::new(5, 0);
    let chess_lines =
      select_best_move(&mut game_state, deadline).expect("This should not be an error");
    display_lines(10, &chess_lines);
    println!("----------------------------------");
    display_lines(5, &chess_lines[0].variations[0].variations);
    println!("{:?}", &chess_lines[0]);
    println!("----------------------------------");

    let expected_move = Move::from_string("c1b2");
    assert_eq!(chess_lines[0].eval.unwrap(), 196.0);
    assert_eq!(expected_move, chess_lines[0].chess_move);
  }

  #[test]
  fn test_select_best_defensive_move() {
    // Only good defense is : h8f8
    let fen = "r1bqk2r/ppppbp1p/2n5/3Bp1pQ/4P3/3P4/PPPN1PPP/R3K1NR b KQq - 0 7";
    let mut game_state = GameState::from_string(fen);
    let deadline = Instant::now() + Duration::new(5, 0);
    let chess_lines =
      select_best_move(&mut game_state, deadline).expect("This should not be an error");
    display_lines(10, &chess_lines);

    let expected_move = Move::from_string("h8f8");
    //assert_eq!(chess_lines[0].eval.unwrap(), 200.0);
    assert_eq!(expected_move, chess_lines[0].chess_move);
  }

  // Game https://lichess.org/Xjgkf4pp seemed really off. Testing some of the positions here
  #[test]
  fn test_select_pawn_capture() {
    let fen = "r2q1rk1/1pp1ppbp/p2p1np1/P7/6bP/R1N1Pn2/1PPP1PP1/2BQKB1R w K - 0 11";
    let mut game_state = GameState::from_string(fen);
    let deadline = Instant::now() + Duration::new(2, 10000000);
    let chess_lines =
      select_best_move(&mut game_state, deadline).expect("This should not be an error");
    display_lines(3, &chess_lines);
    display_lines(3, &chess_lines[2].variations[0].variations);

    let expected_move = Move::from_string("g2f3");
    //assert_eq!(chess_lines[0].eval.unwrap(), 200.0);
    assert_eq!(expected_move, chess_lines[0].chess_move);
  }

  #[test]
  fn evaluate_checkmate_with_castle() {
    let fen = "8/8/8/8/2nN4/1q6/ppP1NPPP/1k2K2R w K - 0 1";
    let mut game_state = GameState::from_string(fen);
    let deadline = Instant::now() + Duration::new(0, 10_000_000);
    let chess_lines =
      select_best_move(&mut game_state, deadline).expect("This should not be an error");
    display_lines(0, &chess_lines);
    assert_eq!("e1g1", chess_lines[0].chess_move.to_string());
    assert_eq!(chess_lines[0].eval.unwrap(), 200.0);
  }

  // From game : https://lichess.org/47V8eE5x -
  // Did not capture the knight, it was very obvious to capture.
  // Spent 2900 ms to come up with this crap: d7d5
  #[test]
  fn capture_the_damn_knight() {
    let fen = "rnb2r1k/pppp2pp/5N2/8/1bB5/8/PPPPQPPP/RNB1K2R b KQ - 0 9";
    let mut game_state = GameState::from_string(fen);
    let deadline = Instant::now() + Duration::new(3, 0);
    let chess_lines =
      select_best_move(&mut game_state, deadline).expect("This should not be an error");
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
    let mut game_state = GameState::from_string(fen);
    let deadline = Instant::now() + Duration::new(3, 0);
    let chess_lines =
      select_best_move(&mut game_state, deadline).expect("This should not be an error");
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
      permutation: false,
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
    let mut game_state = GameState::from_string(fen);
    let deadline = Instant::now() + Duration::new(3, 0);
    let chess_lines =
      select_best_move(&mut game_state, deadline).expect("This should not be an error");
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
      Here we blundered a rook playing e2f2
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
    let mut game_state = GameState::from_string(fen);
    let deadline = Instant::now() + Duration::new(1, 0);
    let chess_lines =
      select_best_move(&mut game_state, deadline).expect("This should not be an error");
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

  #[test]
  fn save_the_bishop() {
    /*
     [2023-06-26T13:51:05Z DEBUG schnecken_bot::lichess::api] Lichess get answer: {"nowPlaying":[{"color":"white","fen":"2kr1b1r/ppp2ppp/2nqp3/3n1BP1/8/3P1N1P/PPP1PP2/R1BQK2R w KQ - 0 12","fullId":"AHbg0nGCsiMN","gameId":"AHbg0nGC","hasMoved":true,"isMyTurn":true,"lastMove":"e7e6","opponent":{"id":"sargon-1ply","rating":1233,"username":"BOT sargon-1ply"},"perf":"blitz","rated":true,"secondsLeft":160,"source":"friend","speed":"blitz","status":{"id":20,"name":"started"},"variant":{"key":"standard","name":"Standard"}}]}
     [2023-06-26T13:51:05Z INFO  schnecken_bot] Trying to find a move for game id AHbg0nGC
     [2023-06-26T13:51:05Z INFO  schnecken_bot::chess::engine::core] Using 1777 ms to find a move
     Line 0 Eval: -1.8000004 - f5e6 d6e6 e2e4
     Line 1 Eval: -4.4000006 - f3g1 e6f5
     Line 2 Eval: -16.820002 - c2c3 f8e7
     Line 3 Eval: -17.800003 - a2a3 f8e7
     Line 4 Eval: -17.860003 - f3e5 f8e7
    */
    let fen = "2kr1b1r/ppp2ppp/2nqp3/3n1BP1/8/3P1N1P/PPP1PP2/R1BQK2R w KQ - 0 12";
    let mut game_state = GameState::from_string(fen);
    let deadline = Instant::now() + Duration::new(2, 0);
    assert_eq!(Some(GamePhase::Opening), game_state.game_phase);
    let chess_lines =
      select_best_move(&mut game_state, deadline).expect("This should not be an error");
    display_lines(0, &chess_lines);
    // This should be quite obvious to find, it's the only move that saves the bishop
    if "f5e4" != chess_lines[0].chess_move.to_string() {
      assert!(false, "Come on, the only good move is f5e4")
    }
  }

  #[test]
  fn king_should_not_disappear() {
    let fen = "8/2p3pk/3p2p1/4r2p/R7/P1N3PP/1B3P2/5K2 b - - 0 40";
    let mut game_state = GameState::from_string(fen);
    let deadline = Instant::now() + Duration::new(3, 0);
    let chess_lines =
      select_best_move(&mut game_state, deadline).expect("This should not be an error");
    display_lines(0, &chess_lines);
    // Eval should be quite bad:
    assert!(
      chess_lines[0].eval.unwrap_or(0.0) > 4.0,
      "Eval should be clearly in favor of white"
    );
  }

  #[test]
  fn test_eval_stalemate_lines() {
    let fen = "8/2k5/6K1/2p1b3/6q1/8/8/8 w - - 4 71";
    let mut game_state = GameState::from_string(fen);
    let deadline = Instant::now() + Duration::new(15, 0);
    let mut chess_lines = select_best_move(&mut game_state, deadline).expect("This should work");
    display_lines(0, &chess_lines);
    display_lines(0, &chess_lines[0].variations);
    println!("----------------------------------");
    //display_lines(0, &chess_lines[0].variations[14].variations);
    /*
    println!("----------------------------------");
    println!(
      "Game Over: {}, moves computed: {}, number of moves: {}",
      &chess_lines[0].variations[14].variations[0].variations[0].game_over,
      &chess_lines[0].variations[14].variations[0].variations[0]
        .game_state
        .available_moves_computed,
      &chess_lines[0].variations[14].variations[0].variations[0]
        .game_state
        .move_list
        .len()
    );
    println!(
      "is permutation : {}",
      &chess_lines[0].variations[14].variations[0].variations[0].permutation
    );
    println!(
      "is permutation before : {}",
      &chess_lines[0].variations[14].variations[0].permutation
    );
    display_lines(0, &chess_lines[0].variations[23].variations);
    */

    assert!(
      chess_lines[0].eval.unwrap_or(0.0) < -100.0,
      "We are in a checkmate in 2 situation."
    );
  }

  #[test]
  fn test_avoid_threefold_repetitions() {
    /* Looks like we had a permutation bug that lead us into some 3-fold repetitions
     [2023-07-04T12:36:47Z INFO  schnecken_bot::chess::engine::core] Using 1211 ms to find a move
       Line 0 Eval: 10.71348 - d1e2 / Permutation
       Line 1 Eval: 6.581044 - h2h3 / Permutation
       Line 2 Eval: 6.461045 - g3g2 / Permutation
       Line 3 Eval: 6.431045 - a1b1 / Permutation
       Line 4 Eval: 6.391044 - g3g1 / Permutation
    */
    let fen = "r7/1p4p1/5p1p/b3n1k1/p3P1P1/PbN3R1/1P1K3P/R1BB4 w - - 10 45";
    let mut game_state = GameState::from_string(fen);
    let deadline = Instant::now() + Duration::new(1, 211_000_000);
    game_state.last_positions.push_back(String::from(
      "r7/1p4p1/5p1p/b3n1k1/p3P1P1/PbN3R1/1P1K3P/R1BB4",
    ));
    game_state.last_positions.push_back(String::from(
      "r7/1p4p1/5p1p/b3n1k1/p1b1P1P1/P1N3R1/1P1K3P/R1BB4",
    ));
    game_state.last_positions.push_back(String::from(
      "r7/1p4p1/5p1p/b3n1k1/p1b1P1P1/P1N3R1/1P1KB2P/R1B5",
    ));
    game_state.last_positions.push_back(String::from(
      "r7/1p4p1/5p1p/b3n1k1/p3P1P1/PbN3R1/1P1KB2P/R1B5",
    ));
    game_state.last_positions.push_back(String::from(
      "r7/1p4p1/5p1p/b3n1k1/p3P1P1/PbN3R1/1P1K3P/R1BB4",
    ));
    game_state.last_positions.push_back(String::from(
      "r7/1p4p1/5p1p/b3n1k1/p1b1P1P1/P1N3R1/1P1K3P/R1BB4",
    ));
    game_state.last_positions.push_back(String::from(
      "r7/1p4p1/5p1p/b3n1k1/p1b1P1P1/P1N3R1/1P1KB2P/R1B5",
    ));
    game_state.last_positions.push_back(String::from(
      "r7/1p4p1/2n2p1p/b5k1/p1b1P1P1/P1N3R1/1P1KB2P/R1B5",
    ));

    let mut chess_lines = select_best_move(&mut game_state, deadline).expect("This should work");
    display_lines(5, &chess_lines);
    println!("-----------------------------------------");
    display_lines(5, &chess_lines[0].variations);
    if "d1e2" == chess_lines[0].chess_move.to_string() {
      assert!(false, "Come on, do not repeat when winning!")
    }
  }

  #[test]
  fn test_do_not_give_material_up() {
    // https://lichess.org/PsC4jC2o
    let fen = "6n1/6pr/4Pk2/1P2bb1p/R2Np3/1P6/4K3/2r5 b - - 2 34";
    let mut game_state = GameState::from_string(fen);
    let deadline = Instant::now() + Duration::new(1, 377_000_000);
    let chess_lines = select_best_move(&mut game_state, deadline).expect("This should work");
    display_lines(10, &chess_lines);
    for i in 0..10 {
      if "c1e1" == chess_lines[i].chess_move.to_string() {
        assert!(false, "Come on, giving up material is not good!")
      }
    }
  }

  #[test]
  fn find_checkmate_in_two() {
    // https://lichess.org/YVidO7Iw
    let fen = "r1kq1b1r/4ppp1/2p2n2/p1Pp1b2/Q2P1B1p/2N1P3/PP3PPP/3RK1NR w K - 0 15";
    let mut game_state = GameState::from_string(fen);
    let deadline = Instant::now() + Duration::new(1, 722_000_000);
    let chess_lines = select_best_move(&mut game_state, deadline).expect("This should work");
    display_lines(5, &chess_lines);
    display_lines(0, &chess_lines);
    assert_eq!("a4c6", chess_lines[0].chess_move.to_string());
    assert_eq!(chess_lines[0].eval.unwrap(), 196.0);
  }

  #[test]
  fn promote_this_pawn() {
    let fen = "8/P7/4kN2/4P3/1K3P2/4P3/8/8 w - - 7 76";
    let mut game_state = GameState::from_string(fen);
    let deadline = Instant::now() + Duration::new(0, 855_000_000);
    let chess_lines = select_best_move(&mut game_state, deadline).expect("This should work");
    assert_eq!("a7a8Q", chess_lines[0].chess_move.to_string());
  }

  #[test]
  fn capture_to_be_up_the_exchange() {
    // Game: https://lichess.org/L4ANgIdY/
    let fen = "6nr/1n1k2pp/p3p3/3pPqB1/2p2P1P/brN2R2/4B3/R1Q4K w - - 33 43";
    let mut game_state = GameState::from_string(fen);
    let deadline = Instant::now() + Duration::from_millis(5000);
    let chess_lines = select_best_move(&mut game_state, deadline).expect("This should work");
    display_lines(5, &chess_lines);
    println!("----------------------------------");
    display_lines(5, &chess_lines[0].variations);
    assert_eq!("a1a3", chess_lines[0].chess_move.to_string());
  }

  #[test]
  fn capture_the_free_piece() {
    // Game: https://lichess.org/PVxISRua
    let fen = "2krr3/1pp3pp/5b2/P1n1pp2/5P2/2P3P1/N3PBRP/4K3 w - - 0 31";
    let mut game_state = GameState::from_string(fen);
    let deadline = Instant::now() + Duration::from_millis(1434);
    let chess_lines = select_best_move(&mut game_state, deadline).expect("This should work");
    display_lines(5, &chess_lines);
    println!("----------------------------------");
    display_lines(5, &chess_lines[0].variations);
    assert_eq!("f2c5", chess_lines[0].chess_move.to_string());
  }

  #[test]
  fn capture_the_hanging_piece() {
    // Game: https://lichess.org/4T57NamT
    let fen = "r2qk2r/2p2pp1/p3p2p/n2p1b2/1b1P1P2/2P1PN2/PP4PP/R1B1KB1n w Qkq - 0 15";
    let mut game_state = GameState::from_string(fen);
    let deadline = Instant::now() + Duration::from_millis(7863);
    let chess_lines = select_best_move(&mut game_state, deadline).expect("This should work");
    display_lines(5, &chess_lines);
    println!("----------------------------------");
    display_lines(5, &chess_lines[0].variations);
    assert_eq!("c3b4", chess_lines[0].chess_move.to_string());
  }

  #[test]
  fn save_the_last_knight() {
    // Game: https://lichess.org/iavzLpKc
    let fen = "4r1k1/1p6/7p/p4p2/Pb1p1P2/1PN3P1/2P1P1K1/r7 w - - 0 34";
    let mut game_state = GameState::from_string(fen);
    let deadline = Instant::now() + Duration::from_millis(7863);
    let chess_lines = select_best_move(&mut game_state, deadline).expect("This should work");
    display_lines(5, &chess_lines);
    println!("----------------------------------");
    display_lines(5, &chess_lines[0].variations);
    assert_eq!("c3d5", chess_lines[0].chess_move.to_string());
  }

  #[test]
  fn king_should_capture_too() {
    // Game: https://lichess.org/adsFtw5A/black#77
    let fen = "8/ppp3kR/b7/3P1p2/3P1q2/PP6/6P1/3R2K1 b - - 0 39";
    let mut game_state = GameState::from_string(fen);
    let deadline = Instant::now() + Duration::from_millis(1308);
    let chess_lines = select_best_move(&mut game_state, deadline).expect("This should work");
    display_lines(5, &chess_lines);
    println!("----------------------------------");
    display_lines(5, &chess_lines[0].variations);
    assert_eq!("g7h7", chess_lines[0].chess_move.to_string());
  }
}
