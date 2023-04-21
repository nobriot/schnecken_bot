use log::*;
use rand::Rng;
use std::cmp::Ordering;
use std::time::{Duration, Instant};

// From our module
use crate::chess::model::board::Move;
use crate::chess::model::game_state::GameState;
use crate::chess::model::piece::*;
use crate::chess::theory::*;

#[derive(Debug, Clone)]
pub struct ChessLine {
  pub line: Vec<Move>,
  pub eval: Option<f32>,
  pub game_over: bool,
}

pub fn highest_evaluation(a: &ChessLine, b: &ChessLine) -> Ordering {
  match (a.eval, b.eval) {
    (None, _) => return Ordering::Greater,
    (_, None) => return Ordering::Less,
    (_, _) => {
      let a_value = a.eval.unwrap();
      let b_value = b.eval.unwrap();
      if a_value > b_value {
        return Ordering::Less;
      } else if a_value < b_value {
        return Ordering::Greater;
      } else {
        // Shortest line is better:
        if a.line.len() > b.line.len() {
          return Ordering::Greater;
        } else if a.line.len() < b.line.len() {
          return Ordering::Less;
        } else {
          return Ordering::Equal;
        }
      }
    },
  }
}

pub fn lowest_evaluation(a: &ChessLine, b: &ChessLine) -> Ordering {
  match (a.eval, b.eval) {
    (None, _) => return Ordering::Greater,
    (_, None) => return Ordering::Less,
    (_, _) => {
      let a_value = a.eval.unwrap();
      let b_value = b.eval.unwrap();
      if a_value > b_value {
        return Ordering::Greater;
      } else if a_value < b_value {
        return Ordering::Less;
      } else {
        // Shortest line is better:
        if a.line.len() > b.line.len() {
          return Ordering::Greater;
        } else if a.line.len() < b.line.len() {
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
// 3. Captures
// 4. Tempos - pins ?
// 5. All the rest
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

    match (new_game_state.checks, capture) {
      (2, _) => moves_arrays[0].push(*m),
      (1, _) => moves_arrays[1].push(*m),
      (0, true) => moves_arrays[2].push(*m),
      (_, _) => moves_arrays[4].push(*m),
    }
  }

  let mut moves = Vec::new();
  for index in 0..moves_arrays.len() {
    moves.append(&mut moves_arrays[index]);
  }

  moves
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
    let mut line: Vec<Move> = Vec::new();
    line.push(m);
    let chess_line = ChessLine {
      line,
      eval: None,
      game_over: false,
    };
    chess_lines.push(chess_line);
  }

  // Process all the moves:
  for i in 0..chess_lines.len() {
    let mut new_game_state = game_state.clone();
    new_game_state.apply_moves(&chess_lines[i].line);
    let (eval, game_over) = new_game_state.evaluate_position();
    chess_lines[i].eval = Some(eval);
    chess_lines[i].game_over = game_over;
  }

  // Rank the moves by eval
  if game_state.side_to_play == Color::White {
    chess_lines.sort_by(|a, b| highest_evaluation(a, b));
  } else {
    chess_lines.sort_by(|a, b| lowest_evaluation(a, b));
  }

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

    // Find the shortest non-game over line:
    let mut index = 0;
    let mut depth = usize::MAX;
    for i in 0..chess_lines.len() {
      if chess_lines[i].line.len() < depth && chess_lines[i].game_over == false {
        depth = chess_lines[i].line.len();
        index = i;
      }
    }
    if depth == usize::MAX {
      // We have nothing to look at ? (all the lines are Game Over)
      return Ok(chess_lines);
    }

    let mut line_game_state = game_state.clone();
    line_game_state.apply_moves(&chess_lines[index].line);

    let candidate_moves = ranked_moves(line_game_state);
    // Add all the moves to the chess lines:
    let base_line: Vec<Move> = chess_lines[index].line.clone();
    chess_lines.remove(index);
    for m in candidate_moves {
      let mut new_line = base_line.clone();
      new_line.push(m);
      let new_chess_line = ChessLine {
        line: new_line,
        eval: None,
        game_over: false,
      };
      chess_lines.push(new_chess_line);
    }

    // Find all the un-evaluated positions and evaluate them:
    for i in 0..chess_lines.len() {
      if chess_lines[i].eval.is_none() {
        let mut new_game_state = game_state.clone();
        new_game_state.apply_moves(&chess_lines[i].line);
        let (eval, game_over) = new_game_state.evaluate_position();
        chess_lines[i].eval = Some(eval);
        chess_lines[i].game_over = game_over;
      }
    }

    // Sort again by evaluation
    // Rank the moves by eval
    if game_state.side_to_play == Color::White {
      chess_lines.sort_by(|a, b| highest_evaluation(a, b));
    } else {
      chess_lines.sort_by(|a, b| lowest_evaluation(a, b));
    }
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
    return Ok(chess_lines[0].line[0].to_string());
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
  if number_of_lines == 0 {
    number_of_lines = chess_lines.len();
  }
  for i in 0..number_of_lines {
    let mut moves: String = String::new();
    for m in &chess_lines[i].line {
      moves += m.to_string().as_str();
      moves += " ";
    }
    let game_over: &str;
    if chess_lines[i].game_over {
      game_over = "/ Game Over";
    } else {
      game_over = "";
    }
    debug!(
      "Eval: {} - {} {}",
      chess_lines[i].eval.unwrap_or(f32::NAN),
      moves,
      game_over
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
    assert_eq!(expected_move, chess_lines[0].line[0]);
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
    assert_eq!(expected_move, chess_lines[0].line[0]);
  }

  #[test]
  fn test_select_best_move_checkmate_in_two() {
    // This is a forced checkmate in 2: c1b2 d4e3 b6d5
    let fen = "1n4nr/5ppp/1N6/1P2p3/1P1k4/5P2/1p1NP1PP/R1B1KB1R w KQ - 0 35";
    let deadline = Instant::now() + Duration::new(5, 0);
    let chess_lines = select_best_move(fen, deadline).expect("This should not be an error");
    display_lines(10, &chess_lines);

    let expected_move = Move::from_string("c1b2");
    assert_eq!(chess_lines[0].eval.unwrap(), 200.0);
    assert_eq!(expected_move, chess_lines[0].line[0]);
  }

  #[test]
  fn test_select_best_defensive_move() {
    // Only good defense is : d7d5
    let fen = "r1bqkb1r/ppppnppp/2n5/4p3/2B1P3/5Q1N/PPPP1PPP/RNB1K2R b KQkq - 5 4";
    let deadline = Instant::now() + Duration::new(5, 0);
    let chess_lines = select_best_move(fen, deadline).expect("This should not be an error");
    display_lines(10, &chess_lines);

    let expected_move = Move::from_string("d7d5");
    //assert_eq!(chess_lines[0].eval.unwrap(), 200.0);
    assert_eq!(expected_move, chess_lines[0].line[0]);
  }
}
