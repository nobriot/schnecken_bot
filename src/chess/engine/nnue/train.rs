use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::process::ExitCode;

use chess::engine::nnue::preprocessing::*;
use chess::engine::nnue::*;
use chess::model::piece::Color;

pub const LICHESS_DATABASE_FILE: &str = "engine/nnue/data/training_set.pgn";
pub const OUTPUT_TRAINING_FILE: &str = "engine/nnue/data/training_set.csv";
pub const NNUE_OUTPUT_FILE: &str = "engine/nnue/data/net.nuue";
pub const MINI_BATCH_SIZE: usize = 10000;
pub const NUMBER_OF_EPOCH: usize = 200;

const ERROR: &str = "\x1B[31m\x1B[1m\x1B[4mError\x1B[24m: \x1B[0m\x1B[31m";

// Main function
#[allow(non_snake_case)]
fn main() -> ExitCode {
  println!("\n\x1B[4mWelcome to \x1B[1mNNUE training\x1B[0m. 🙂\n");

  // Process the PGN files into a file containing training data
  println!(
    "Pre-processing database file {} into a training set.",
    LICHESS_DATABASE_FILE
  );

  let input_file = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), LICHESS_DATABASE_FILE);
  let output_file = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), OUTPUT_TRAINING_FILE);

  if let Err(error) = create_training_data_from_pgn_file(input_file.as_str(), output_file.as_str())
  {
    println!(
      "{ERROR}: Could not pre-process the data: {}. Exiting\n",
      error
    );
    return ExitCode::FAILURE;
  }

  // ---------------------------------------------------------------------------
  // Load the training data.
  println!("Load training file.");
  let training_set_result = load_training_set_in_cache(output_file.as_str());
  if let Err(error) = training_set_result {
    println!(
      "{ERROR}: Could not load the training data in to a cache {}. Exiting\n",
      error
    );
    return ExitCode::FAILURE;
  }
  let training_cache = training_set_result.unwrap();
  println!("We have {} training samples\n", training_cache.len());

  // ---------------------------------------------------------------------------
  // Instantiante the NNUE and train it
  println!("Loading the NNUE from file {NNUE_OUTPUT_FILE}");
  let mut nnue = NNUE::load(NNUE_OUTPUT_FILE).unwrap_or_default();

  let number_of_mini_batches = training_cache.len() / MINI_BATCH_SIZE;

  for e in 0..NUMBER_OF_EPOCH {
    for i in 0..(number_of_mini_batches - 1) {
      let mut training = Vec::new();
      let mut evals = Vec::new();
      for j in 0..MINI_BATCH_SIZE {
        let index = i * MINI_BATCH_SIZE + j;
        let game_state = &training_cache[index].0;
        training.push(game_state);
        let mut eval = (training_cache[index].1 / 6.0).tanh();
        if game_state.board.side_to_play == Color::Black {
          eval = -eval;
        }
        evals.push(eval);
      }
      nnue.game_state_to_input_layer(&training);
      let Y_hat = nnue.forward_propagation();
      nnue.backwards_propagation(&Y_hat, &evals);
      nnue.update_parameters();

      println!(
        "Cost after iteration {}: {}",
        i + 1,
        functions::total_cost(&Y_hat, &evals) / MINI_BATCH_SIZE as f32
      );
    }
    println!("Epoch {e} completed");
  }

  // ---------------------------------------------------------------------------
  // Run the NNUE against the test set
  println!("Now let's see our predictions:");
  let mut testing = Vec::new();
  let mut evals = Vec::new();
  for j in 0..MINI_BATCH_SIZE {
    let index = (number_of_mini_batches - 1) * MINI_BATCH_SIZE + j;
    let game_state = &training_cache[index].0;
    testing.push(game_state);
    let mut eval = (training_cache[index].1 / 6.0).tanh();
    if game_state.board.side_to_play == Color::Black {
      eval = -eval;
    }
    evals.push(eval);
  }

  nnue.game_state_to_input_layer(&testing);
  let predictions = nnue.forward_propagation();
  let output_file = File::create("predictions.csv").unwrap();
  let mut writer = BufWriter::new(output_file);
  for i in 0..predictions.len() {
    writer
      .write_fmt(format_args!(
        "{};{};{}\n",
        evals[i],
        predictions[i],
        testing[i].to_fen()
      ))
      .unwrap();
  }

  println!(
    "Cost on test set: {}",
    functions::total_cost(&predictions, &evals)
  );

  // ---------------------------------------------------------------------------
  // Save the NNUE so it can be restored later
  println!("Saving the NNUE to file {NNUE_OUTPUT_FILE}");
  nnue.save(NNUE_OUTPUT_FILE);

  println!("");
  println!("Done! 🙂");

  return ExitCode::SUCCESS;
}
