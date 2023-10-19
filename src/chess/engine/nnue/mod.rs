pub mod functions;
pub mod preprocessing;

use crate::engine::nnue::functions::*;
use crate::model::game_state::GameState;
use crate::model::piece::*;

use ndarray::{Array, Array2, Zip};
use ndarray_rand;
use ndarray_rand::RandomExt;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

// This is a very good read on using Neural Networks for chess:
// https://github.com/asdfjkl/neural_network_chess/releases

// Looks like the best way to encode the inputs is using a massive list of
// piece/square boolean values
// https://www.chessprogramming.org/NNUE
// So x1 = white pawns on a1, x2 = white pawns on a2...
// First half of the input layer is the side to play, second half is the opposite color

/// #### Hyperparameters for Neural Network training
/// They can be tuned for each layer.
///
pub struct HyperParameters {
  /// Learning rate used for gradient descent
  pub learning_rate: f32,
  /// Momentum for the gradient descent (recommended to set $beta_1 = 0.9$)
  pub beta_1: f32,
  /// RMSProp for the gradient descent (recommended to set $beta_2 = 0.999$)
  pub beta_2: f32,
  /// Used for L2/Frobenius norm for the gradient descent.
  pub lambda: f32,
  /// Dropout rate, should be between 0 and 1 used for training.
  /// Should only be used on large layers, usually set to 0.1
  pub dropout: f32,
}

impl Default for HyperParameters {
  fn default() -> Self {
    Self {
      learning_rate: 0.1,
      beta_1: 0.9,
      beta_2: 0.999,
      lambda: 1.0,
      dropout: 0.0,
    }
  }
}

/// #### Layer state
/// Contains all the internal variables/parameters of a layer
///
/// Weights have to be of dimension (L,L-1), L being the number of nodes in a layer
///
#[allow(non_snake_case)]
pub struct LayerState {
  /// Weights.(W)
  pub W: Array2<f32>,
  /// Biases, denoted b
  pub b: f32,
  /// Linear value cache
  pub Z: Array2<f32>,
  /// Activation value cache
  pub A: Array2<f32>,
  /// Weight gradients, dW
  pub dW: Array2<f32>,
  /// biases gradients, db
  pub db: f32,
  /// Linear value gradient cache. dZ
  pub dZ: Array2<f32>,
  /// Activation gradient cache. dA
  pub dA: Array2<f32>,
}

impl LayerState {
  /// Creates a new Layer, using the number of nodes from the previous layer
  ///
  /// Weights are initialized with zeros. Note that they must be updated to
  /// random small numbers (to break symmetry) before training the model
  ///
  /// Biases are initialized with zeros
  ///
  ///
  pub fn new(layer_size: usize, previous_layer_size: usize) -> Self {
    // Create all the internal variables for the layer
    LayerState {
      W: Array::random(
        (layer_size, previous_layer_size),
        ndarray_rand::rand_distr::Normal::new(0.0, 0.01).unwrap(),
      ),
      b: 0.0,
      Z: Array2::zeros((layer_size, previous_layer_size)),
      A: Array2::zeros((layer_size, previous_layer_size)),
      dW: Array2::zeros((layer_size, previous_layer_size)),
      db: 0.0,
      dZ: Array2::zeros((layer_size, previous_layer_size)),
      dA: Array2::zeros((layer_size, previous_layer_size)),
    }
  }
}

/// #### Layer in a neural network
/// They can be tuned for each layer.
///
pub struct Layer {
  /// Number of nodes for that layer.
  pub nodes: usize,
  /// Tuning parameters
  pub param: HyperParameters,
  /// Activation function
  pub a: Activation,
  /// Weight, biases and gradients
  pub state: LayerState,
}

/// Enum representing the different activation function we would use for the
/// layer.
pub enum Activation {
  ReLU,
  ClippedReLU,
  ExtendedClippedReLU,
  Tanh,
  Sigmoid,
  None,
}

/// # NNUE
///
/// Here we try to build a Neural Network with 3 layers (not counting A0, input layer)
///
#[repr(align(32))]
pub struct NNUE {
  pub layers: Vec<Layer>,
}

// Regularization for the weights:
// It's a bit like normalization for the activation layer.
// 1. if using ReLU, then initialize with w = rand / sqrt(2/n[l-1])
// 2. if using tanh, then initialize with w = rand / sqrt(1/n[l-1])
// 3. if using sigmoid,... let's not use sigmoid

impl NNUE {
  /// Size of the input layer, has to be squares x piece_types x 2 (color)
  const LAYER_0_SIZE: usize = 64 * 6 * 2;
  const LAYER_1_SIZE: usize = 500;
  const LAYER_2_SIZE: usize = 16;
  const LAYER_3_SIZE: usize = 1;

  /// Creates a new NNUE
  ///
  /// For now the number of layers and their size is just hardcoded in there.
  ///
  pub fn new() -> Self {
    // Input Layer (L0):
    let l0 = Layer {
      nodes: Self::LAYER_0_SIZE,
      param: HyperParameters::default(),
      a: Activation::None,
      state: LayerState::new(Self::LAYER_0_SIZE, 1),
    };

    // Create the "normal" layers, L1, L2, etc...
    let l1 = Layer {
      nodes: Self::LAYER_1_SIZE,
      param: HyperParameters::default(),
      a: Activation::ClippedReLU,
      state: LayerState::new(Self::LAYER_1_SIZE, Self::LAYER_0_SIZE),
    };

    let l2 = Layer {
      nodes: Self::LAYER_2_SIZE,
      param: HyperParameters::default(),
      a: Activation::ClippedReLU,
      state: LayerState::new(Self::LAYER_2_SIZE, Self::LAYER_1_SIZE),
    };

    let l3 = Layer {
      nodes: Self::LAYER_3_SIZE,
      param: HyperParameters::default(),
      a: Activation::Tanh,
      state: LayerState::new(Self::LAYER_3_SIZE, Self::LAYER_2_SIZE),
    };

    let mut nnue = NNUE { layers: Vec::new() };
    nnue.layers.push(l0);
    nnue.layers.push(l1);
    nnue.layers.push(l2);
    nnue.layers.push(l3);

    nnue
  }

  /// Calculates the prediction/output given the current state of the NNUE.
  ///
  /// input can be of the size of a mini batch
  ///
  ///
  ///
  #[allow(non_snake_case)]
  pub fn forward_propagation(&mut self, input: &[&GameState]) -> Vec<f32> {
    // Convert the board to our input layer.
    self.layers[0].state.A = NNUE::game_state_to_input_layer(input);

    // This is A0, input data:
    let mut A_prev = self.layers[0].state.A.clone();

    for i in 1..self.layers.len() {
      //println!("Forward prop: Layer {i} - {} nodes", self.layers[i].nodes);

      // Zl = Wl.A(l-1) + bl
      let mut Zl = self.layers[i].state.W.dot(&A_prev);
      Zl += self.layers[i].state.b;

      //println!("Done");

      // Save Zl onto the layer:
      self.layers[i].state.Z = Zl.clone();

      // Al = gl(Zl) where gl is the activation function for that layer
      match self.layers[i].a {
        Activation::ReLU => Zip::from(&mut Zl).for_each(|a| *a = relu(*a)),
        Activation::ClippedReLU => Zip::from(&mut Zl).for_each(|a| *a = clipped_relu(*a)),
        Activation::ExtendedClippedReLU => {
          Zip::from(&mut Zl).for_each(|a| *a = extended_clipped_relu(*a, 200.0))
        },
        Activation::Tanh => Zip::from(&mut Zl).for_each(|a| *a = a.tanh()),
        Activation::Sigmoid => Zip::from(&mut Zl).for_each(|a| *a = sigmoid(*a)),
        Activation::None => {},
      };
      let Al = Zl;
      self.layers[i].state.A = Al.clone();

      A_prev = Al;
    }

    // Calculate prediction
    //println!("A for last layer: {:?}", A_prev);
    let mut Y_hat = vec![0.0; A_prev.len()];
    for c in 0..A_prev.len() {
      Y_hat[c] = A_prev[[0, c]];
    }

    Y_hat
  }

  /// Back propagate using gradient descent.
  ///
  /// ```math
  /// dZ^{[l]} = dA^{[l]} \times g'(Z^{[l]})
  /// dW^{[l]} = \frac{\partial \mathcal{J} }{\partial W^{[l]}} = \frac{1}{m} dZ^{[l]} A^{[l-1] T}
  /// db^{[l]} = \frac{\partial \mathcal{J} }{\partial b^{[l]}} = \frac{1}{m} \sum_{i = 1}^{m} dZ^{[l](i)}
  /// dA^{[l-1]} = \frac{\partial \mathcal{L} }{\partial A^{[l-1]}} = W^{[l] T} dZ^{[l]} \tag{10}
  /// ````
  ///
  #[allow(non_snake_case)]
  pub fn backwards_propagation(&mut self, AL: &[f32], Y: &[f32]) {
    assert_eq!(AL.len(), Y.len());

    // Calculate dc/dAL
    let dC = cost_derivative_vector(AL, Y);
    //println!("dC : {:?}", dC);
    let mut dA_prev = dC;

    let m = AL.len() as f32;
    //println!("Number of samples: {m}");

    // Start from the last layer, back-propagating all the way to layer 1
    for i in (1..self.layers.len()).rev() {
      //println!("backprop : Layer {i} - {} nodes", self.layers[i].nodes);

      self.layers[i].state.dA = dA_prev;

      //println!("dAl-1 dimensions: {}x{}", dA_prev.len(), dA_prev[0].len());
      //println!("dZl");
      let mut dZl = self.layers[i].state.Z.clone();
      match self.layers[i].a {
        Activation::ReLU => Zip::from(&mut dZl).for_each(|a| *a = relu_backwards(*a)),
        Activation::ClippedReLU => {
          Zip::from(&mut dZl).for_each(|a| *a = clipped_relu_backwards(*a))
        },
        Activation::ExtendedClippedReLU => {
          Zip::from(&mut dZl).for_each(|a| *a = extended_clipped_relu_backwards(*a, 200.0))
        },
        Activation::Tanh => Zip::from(&mut dZl).for_each(|a| *a = tanh_backwards(*a)),
        Activation::Sigmoid => Zip::from(&mut dZl).for_each(|a| *a = sigmoid_backwards(*a)),
        Activation::None => {},
      };

      dZl *= &self.layers[i].state.dA;
      //println!("dZl = {:?}", dZl);
      //println!("dZl dimensions: {}x{}", dZl.len(), dZl[0].len());

      // Cache the result
      self.layers[i].state.dZ = dZl.clone();

      //println!("dWl");
      let mut dWl = dZl.dot(&self.layers[i - 1].state.A.t());
      dWl /= m;
      self.layers[i].state.dW = dWl.clone();
      //println!("dWl = {:?}", dWl);
      //println!("dWl dimensions: {}x{}", dWl.len(), dWl[0].len());

      //println!("dbl");
      self.layers[i].state.db = (1.0 / m) * dZl.sum();
      //println!("dbl = {:?}", dbl);
      //println!("dbl dimensions: {}x{}", dbl.len(), dbl[0].len());

      //println!("dA_prev");
      dA_prev = self.layers[i].state.W.t().dot(&dZl);
    }
  }

  /// Checks if gradients seem to match the calculated values
  ///
  /// Calculates the value by forward propagating using tiny increments in the
  /// weights.
  ///
  /// Then compare it to the previously calculated gradient value
  ///
  ///
  #[allow(non_snake_case)]
  pub fn check_weight_gradient(&mut self, input: &[&GameState], y: &[f32], layer: usize) {
    // Do that for all rows/columns of the layer Weight matrix:
    const EPSILON: f32 = 10e-6;

    let shape_c = self.layers[layer].state.W.shape()[0];
    let shape_r = self.layers[layer].state.W.shape()[1];
    println!("Layer size: {}x{}", shape_c, shape_r);

    for c in 0..shape_c {
      for r in 0..shape_r {
        // Capture a weight at a given layer, row, colum:
        let W = self.layers[layer].state.W[[c, r]];
        let dW = self.layers[layer].state.dW[[c, r]];

        // Update W to W + epsilon
        self.layers[layer].state.W[[c, r]] = W + EPSILON;
        let y_hat_plus = self.forward_propagation(input);
        let cost_plus = cost_vector(&y_hat_plus, y);

        // Update W to W - epsilon
        self.layers[layer].state.W[[c, r]] = W - EPSILON;
        let y_hat_minus = self.forward_propagation(input);
        let cost_minus = cost_vector(&y_hat_minus, y);

        // Calculate gradient based on these tiny offsets
        let test_dW = (cost_plus - cost_minus).sum() / (2.0 * EPSILON * y.len() as f32);

        // Restore W
        self.layers[layer].state.W[[c, r]] = W;

        println!("dW[{c}][{r}] : {} - test_dW: {} ", dW, test_dW);
        //println!("cost+ : {:?} ", cost_plus);
        //println!("cost- : {:?} ", cost_minus);
        assert!((test_dW - dW).abs() < 0.01);
      }
    }
  }

  /// Update the parameters using back-propagation gradient calculation
  ///
  ///
  #[allow(non_snake_case)]
  pub fn update_parameters(&mut self) {
    for i in 1..self.layers.len() {
      let learning_rate = self.layers[i].param.learning_rate;

      let dW = self.layers[i].state.dW.clone();
      // W = W - alpha*dW
      Zip::from(&mut self.layers[i].state.W)
        .and(&dW)
        .for_each(|w, &dw| {
          *w -= learning_rate * dw;
        });

      self.layers[i].state.b -= learning_rate * self.layers[i].state.db;
    }
  }

  /// Calculates the prediction/output given the current state of the NNUE.
  /// TODO: Write description
  ///
  /// Predicting is just running a forward propagation with a single game state
  /// (instead of mini batch)
  ///
  /// ### Arguments
  ///
  /// * `self`:   reference to the NNUE
  /// * `input`:  reference to a game State to evaluate
  ///
  /// ### Return value
  ///
  /// white's perpective f32 evaluation of the game state, included in [-1;1]
  ///
  #[allow(non_snake_case)]
  pub fn predict(&mut self, input: &GameState) -> f32 {
    let y_hat = self.forward_propagation(&vec![input]);

    // Evaluation is relative to the side to play here.
    // So 6.0 when it is black to play actually means -6.0

    match input.board.side_to_play {
      Color::White => y_hat[0],
      Color::Black => -y_hat[0],
    }
  }

  /// TODO: Write description
  ///
  ///
  ///
  pub fn game_state_to_input_layer(input: &[&GameState]) -> Array2<f32> {
    let mut a0: Array2<f32> = Array2::zeros((Self::LAYER_0_SIZE, input.len()));

    for m in 0..input.len() {
      // ptp : piece to play (side to play) ; op: opposite pieces
      let (ptp, op) = match input[m].board.side_to_play {
        Color::White => (input[m].board.pieces.white, input[m].board.pieces.black),
        Color::Black => (input[m].board.pieces.black, input[m].board.pieces.white),
      };

      // Let's do: rook (offset = 0), queens (offset = 1 x 64), bishops (offset = 2 x 64), knights (offset = 3 x 64), king (offset = 4 x 64), pawn (offset = 5 x 64)
      for (i, piece) in ptp {
        match piece {
          PieceType::King => a0[[i as usize + 4 * 64, m]] = 1.0,
          PieceType::Queen => a0[[i as usize + 1 * 64, m]] = 1.0,
          PieceType::Rook => a0[[i as usize, m]] = 1.0,
          PieceType::Bishop => a0[[i as usize + 2 * 64, m]] = 1.0,
          PieceType::Knight => a0[[i as usize + 3 * 64, m]] = 1.0,
          PieceType::Pawn => a0[[i as usize + 5 * 64, m]] = 1.0,
        }
      }
      // Same for opponent pieces, except that we have a 384 offset to everything
      for (i, piece) in op {
        match piece {
          PieceType::King => a0[[i as usize + 384 + 4 * 64, m]] = 1.0,
          PieceType::Queen => a0[[i as usize + 384 + 1 * 64, m]] = 1.0,
          PieceType::Rook => a0[[i as usize + 384, m]] = 1.0,
          PieceType::Bishop => a0[[i as usize + 384 + 2 * 64, m]] = 1.0,
          PieceType::Knight => a0[[i as usize + 384 + 3 * 64, m]] = 1.0,
          PieceType::Pawn => a0[[i as usize + 384 + 5 * 64, m]] = 1.0,
        }
      }
    }
    a0
  }

  /// Saves the NNUE bytes to a file, so it can be loaded again later
  ///
  pub fn save(&self, output_file: &str) -> std::io::Result<()> {
    let mut writer = BufWriter::new(File::create(output_file)?);

    writer.write_all("NNUE Data\n".as_bytes())?;
    for i in 0..self.layers.len() {}
    todo!("Finish this");

    Ok(())
  }
}

impl Default for NNUE {
  fn default() -> Self {
    Self::new()
  }
}

//------------------------------------------------------------------------------
// Tests
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_game_state_to_input_layer() {
    let game_state = GameState::default();

    let a0 = NNUE::game_state_to_input_layer(&vec![&game_state]);
    println!("{:#?}", a0);

    // We expect 32 pieces on the board:
    let mut sum = 0;
    for i in 0..a0.shape()[1] {
      if a0[[0, i]] == 1.0 {
        sum += 1;
      }
    }
    assert_eq!(32, sum);
  }

  #[test]
  fn test_predict() {
    let game_state = GameState::default();
    let mut nnue = NNUE::new();

    let y_hat = nnue.predict(&game_state);

    assert_eq!(0.0, y_hat);
  }

  #[test]
  fn test_forward_propagation() {
    let game_state_1 = GameState::default();
    let game_state_2 =
      GameState::from_fen("rnbqkbnr/ppp1pppp/8/3p4/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 2");
    let mut nnue = NNUE::new();

    let mini_batch = vec![&game_state_1, &game_state_2];

    let Y_hat = nnue.forward_propagation(&mini_batch);

    println!("Y hat: {:?}", Y_hat);
  }

  #[test]
  fn test_training() {
    let game_state_1 =
      GameState::from_fen("rnbqkbnr/ppp1pppp/8/3p4/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 2");
    let game_state_2 =
      GameState::from_fen("rnbqkbnr/ppp1pppp/8/3p4/2PP4/8/PP2PPPP/RNBQKBNR b KQkq - 0 2");
    let game_state_3 =
      GameState::from_fen("rnbqkbnr/pp2pppp/2p5/3p4/2PP4/8/PP2PPPP/RNBQKBNR w KQkq - 0 3");
    let game_state_4 = GameState::from_fen("5k2/p1p5/1p5p/6p1/5p1P/2b1P3/Pr5B/3rNKR1 w - - 2 31");

    // Evals for games 1, 2, 3 and 4.
    let mut evals = vec![0.27, -0.29, 0.3, -199.0];
    for i in 0..evals.len() {
      evals[i] = (evals[i] + 200.0) / 400.0;
    }
    assert_eq!(0.0, total_cost(&evals, &evals));

    let mut nnue = NNUE::new();
    let mini_batch = vec![&game_state_1, &game_state_2, &game_state_3, &game_state_4];
    let mut previous_cost = 100000.0;

    for i in 0..100 {
      let Y_hat = nnue.forward_propagation(&mini_batch);
      println!("Prediction: {:?}", Y_hat);
      nnue.backwards_propagation(&Y_hat, &evals);

      let new_cost = total_cost(&Y_hat, &evals);
      println!("new cost: {} - old cost:  {}", new_cost, previous_cost);
      //assert!(new_cost < previous_cost);
      previous_cost = new_cost;
    }

    let Y_hat = nnue.forward_propagation(&mini_batch);
    println!("Prediction: {:?}", Y_hat);
    println!("True labels: {:?}", evals);
    println!("Cost {}", total_cost(&Y_hat, &evals));
  }

  #[test]
  fn test_gradient_checking() {
    let game_state_1 =
      GameState::from_fen("rnbqkbnr/ppp1pppp/8/3p4/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 2");
    let game_state_2 =
      GameState::from_fen("rnbqkbnr/ppp1pppp/8/3p4/2PP4/8/PP2PPPP/RNBQKBNR b KQkq - 0 2");
    let game_state_3 =
      GameState::from_fen("rnbqkbnr/pp2pppp/2p5/3p4/2PP4/8/PP2PPPP/RNBQKBNR w KQkq - 0 3");
    let game_state_4 = GameState::from_fen("5k2/p1p5/1p5p/6p1/5p1P/2b1P3/Pr5B/3rNKR1 w - - 2 31");
    let mut evals: Vec<f32> = vec![0.27, -0.29, 0.3, -199.0];
    for i in 0..evals.len() {
      evals[i] = (evals[i] / 15.0).tanh();
    }
    assert_eq!(0.0, total_cost(&evals, &evals));

    let mut nnue = NNUE::new();
    let mini_batch = vec![&game_state_1, &game_state_2, &game_state_3, &game_state_4];
    let Y_hat = nnue.forward_propagation(&mini_batch);
    println!("Prediction: {:?}", Y_hat);
    println!("Actual: {:?}", evals);
    println!("Total cost: {:?}", total_cost(&Y_hat, &evals));

    nnue.backwards_propagation(&Y_hat, &evals);

    nnue.check_weight_gradient(&mini_batch, &evals, 3);
  }
}
