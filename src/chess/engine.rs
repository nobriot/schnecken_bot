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

// For now we just apply the entire line and evaluate the end result
pub fn select_best_move(fen: &str, depth: u8, deadline: Instant) -> Result<(Move, f32), ()> {
  //debug!("eval {} at depth {}", fen, depth);

  let mut game_state = GameState::from_string(fen);

  if depth == 0 {
    return Ok((Move::default(), game_state.evaluate_position()));
  }

  // Check if we have been thinking too much:
  let current_time = Instant::now();
  if current_time > deadline {
    // Abort looking at the line by returning no move and a very bad evaluation.
    if game_state.side_to_play == Color::White {
      return Ok((Move::default(), -200.0));
    } else {
      return Ok((Move::default(), 200.0));
    }
  }

  // Else try to calculate a little deep:
  let candidates = game_state.get_moves();
  let mut first_move = true;
  let mut best_move = candidates[0];
  let mut best_eval: f32 = 0.0;
  for m in candidates {
    game_state.apply_move(m);
    if let Ok((_, eval)) = select_best_move(&game_state.to_string(), depth - 1, deadline) {
      if first_move || ((best_eval < eval) && game_state.side_to_play == Color::White) {
        best_move = m;
        best_eval = eval;
      } else if first_move || ((best_eval > eval) && game_state.side_to_play == Color::Black) {
        best_move = m;
        best_eval = eval;
      }
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
  warn!("We should decide for a reasonable amount of time.");
  let deadline = Instant::now() + Duration::new(1, 0);

  if let Ok((chess_move, evaluation)) = select_best_move(&fen, 10, deadline) {
    info!("Selecting move with evaluation {:?}", evaluation);
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
