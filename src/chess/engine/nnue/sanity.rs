use chess::engine::nnue::*;
use rand::Rng;
use std::process::ExitCode;

pub const BATCH_SIZE: usize = 100;
pub const NUMBER_OF_EPOCH: usize = 100000;

// Main function
#[allow(non_snake_case)]
fn main() -> ExitCode {
  let mut rng = rand::thread_rng();

  println!("\n\x1B[4mWelcome to \x1B[1mNNUE testing\x1B[0m. 🙂\n");

  // ---------------------------------------------------------------------------
  // Instantiante the NNUE and train it
  println!("Creating a neural net with 5 layers of 8 neurons");
  let mut nnue = NNUE::new_no_layer();
  nnue.add_layer(1, HyperParameters::default(), Activation::None);
  nnue.add_layer(8, HyperParameters::default(), Activation::ReLU);
  nnue.add_layer(8, HyperParameters::default(), Activation::ReLU);
  nnue.add_layer(8, HyperParameters::default(), Activation::ReLU);
  nnue.add_layer(8, HyperParameters::default(), Activation::ReLU);
  nnue.add_layer(1, HyperParameters::default(), Activation::None);

  // Let's just have it learn a polynomial:
  let mut training = Vec::new();
  let mut labels = Vec::new();

  for _ in 0..BATCH_SIZE {
    let value = (rng.r#gen::<f32>() - 0.5) * 200.0; // Uniform between -100 and 100.
    training.push(value);
    labels.push(value.powf(2.0) - 5.0 * value + 45.0);
  }

  nnue.f32_slice_to_input_layer(&training);
  let mut last_cost: f32 = f32::INFINITY;

  for e in 0..NUMBER_OF_EPOCH {
    let Y_hat = nnue.forward_propagation();

    let new_cost = functions::total_cost(&Y_hat, &labels) / BATCH_SIZE as f32;
    if new_cost > 2.0 * last_cost {
      nnue.decay_learning_rate(0.9);
    }
    last_cost = new_cost;

    nnue.backwards_propagation(&Y_hat, &labels);
    nnue.update_parameters();

    if e % 1000 == 0 {
      println!("Cost after iteration {}: {}", e, last_cost);
    }
  }

  // ---------------------------------------------------------------------------
  // Run the NNUE against the test set
  println!("Now let's see our predictions:");
  let mut testing = Vec::new();
  let mut labels = Vec::new();
  for _ in 0..BATCH_SIZE {
    let value = (rng.r#gen::<f32>() - 0.5) * 200.0; // Uniform between -100 and 100.
    testing.push(value);
    labels.push(value.powf(2.0) - 5.0 * value + 45.0);
  }

  nnue.f32_slice_to_input_layer(&testing);
  let predictions = nnue.forward_propagation();

  for i in 0..5 {
    println!("testing: {} - true value : {} prediction : {} - error: {}",
             testing[i],
             labels[i],
             predictions[i],
             (labels[i] - predictions[i]).powf(2.0));
  }

  println!("Cost on test set: {}", functions::total_cost(&predictions, &labels));

  println!("");
  println!("Done! 🙂");

  return ExitCode::SUCCESS;
}
