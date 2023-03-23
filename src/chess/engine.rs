use log::*;
use rand::Rng;
use std::str::FromStr;
use std::time::{Duration, Instant};

// From our module
use crate::chess::eval::ChessEval;
use crate::chess::model::board::Move;
use crate::chess::model::game_state::GameState;
use crate::chess::model::piece::*;
use crate::chess::theory::*;

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
pub fn select_best_move(fen: &str, depth: u8, deadline: Instant) -> Result<(Move, f32), ()> {
  debug!("eval {} at depth {}", fen, depth);

  // Check if we have been thinking too much:
  let current_time = Instant::now();
  if current_time > deadline {
    //debug!("We have been thinking too much, returning no evaluation");
    return Err(());
  }

  let mut game_state = GameState::from_string(fen);

  if depth == 0 {
    return Ok((Move::default(), game_state.evaluate_position()));
  }

  // Get the list of moves to assess:
  let candidates = ranked_moves(game_state);
  if candidates.len() == 0 {
    return Ok((Move::default(), game_state.evaluate_position()));
  }

  let mut move_list = String::new();
  for m in &candidates {
    move_list += format!("{m} ").as_str();
  }
  debug!("list of moves: {move_list}");

  let mut first_move = true;
  let mut best_move = candidates[0];
  let mut best_eval: f32 = 0.0;
  for m in candidates {
    game_state.apply_move(m);
    if let Ok((_, eval)) = select_best_move(&game_state.to_string(), depth - 1, deadline) {
      if first_move
        || ((best_eval < eval) && game_state.side_to_play == Color::White)
        || ((best_eval > eval) && game_state.side_to_play == Color::Black)
      {
        best_move = m;
        best_eval = eval;
      }
    }

    // Stop if we found a checkmate, no need to look at other moves for that line
    if ((best_eval == 200.0) && (game_state.side_to_play == Color::White))
      || ((best_eval == -200.0) && (game_state.side_to_play == Color::Black))
    {
      debug!("Checkmate detected for {}", game_state.to_string().as_str());
      return Ok((best_move, best_eval));
    }

    first_move = false;
    game_state = GameState::from_string(fen);
  }

  return Ok((best_move, best_eval));
}

pub fn play_move(game_state: &GameState) -> Result<String, ()> {
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
  info!("We should decide for a reasonable amount of time.");
  let deadline = Instant::now() + Duration::new(5, 0);

  if let Ok((chess_move, evaluation)) = select_best_move(&fen, 6, deadline) {
    debug!(
      "Selecting move {} with evaluation {:?}",
      chess_move, evaluation
    );
    return Ok(chess_move.to_string());
  }

  // Fallback on playing a random move:
  warn!("Eval went wrong. Playing a random move!");
  let mut move_list = game_state.get_moves();
  if move_list.len() == 0 {
    warn!("Cannot compute any move from fen: {fen}");
    return Err(());
  }

  let mut rng = rand::thread_rng();
  let random_legal_move = rng.gen_range(0..move_list.len());
  return Ok(move_list[random_legal_move].to_string());
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_evaluation_position() {}
}
