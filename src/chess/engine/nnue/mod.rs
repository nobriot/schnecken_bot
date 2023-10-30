pub mod functions;
pub mod preprocessing;

use crate::engine::nnue::functions::*;
use crate::model::game_state::GameState;
use crate::model::piece::*;

use ndarray::{Array, Array2, Zip};
use ndarray_rand;
use ndarray_rand::RandomExt;
use std::fs::File;
use std::io::prelude::*;
use std::io::Write;
use std::io::{BufReader, BufWriter};

// This is a very good read on using Neural Networks for chess:
// https://github.com/asdfjkl/neural_network_chess/releases

// Looks like the best way to encode the inputs is using a massive list of
// piece/square boolean values
// https://www.chessprogramming.org/NNUE
// So x1 = white pawns on a1, x2 = white pawns on a2...
// First half of the input layer is the side to play, second half is the opposite color

// ---------------------------------------------------------------------------
// Constant
/// Magic bytes for our nnue file format
const MAGIC_BYTES: &str = "nnue";

/// #### Hyperparameters for Neural Network training
/// They can be tuned for each layer.
///
#[derive(Debug)]
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

impl HyperParameters {
  fn zeros() -> Self {
    Self {
      learning_rate: 0.0,
      beta_1: 0.0,
      beta_2: 0.0,
      lambda: 1.0,
      dropout: 0.0,
    }
  }
}

impl Default for HyperParameters {
  fn default() -> Self {
    Self {
      learning_rate: 0.05,
      beta_1: 0.9,
      beta_2: 0.999,
      lambda: 1.0,
      dropout: 0.0,
    }
  }
}

/// #### Activation cache
///
/// Contains all the intermediate calculation values for both forward and backward
/// propagation
///
#[allow(non_snake_case)]
#[derive(Debug)]
pub struct LayerCache {
  /// Linear value cache
  pub Z: Array2<f32>,
  /// Activation value cache
  pub A: Array2<f32>,
  /// Weight gradients, dW
  pub dW: Array2<f32>,
  /// Weight gradients Momentum, vdW
  pub vdW: Array2<f32>,
  /// Weight gradients Momentum squared, sdW
  pub sdW: Array2<f32>,
  /// biases gradients, db
  pub db: f32,
  /// Bias gradients Momentum, vdb
  pub vdb: f32,
  /// Bias gradients Momentum squared, sdb
  pub sdb: f32,
  /// Linear value gradient cache. dZ
  pub dZ: Array2<f32>,
  /// Activation gradient cache. dA
  pub dA: Array2<f32>,
}

impl LayerCache {
  /// Layer cache is just arrays of zeros to start with.
  ///
  /// ### Arguments
  ///
  /// * `layer_size`: Number of nodes in the layer for the current cache
  /// * `previous_layer_size`: Number of nodes in the previous layer for the current cache
  fn new(layer_size: usize, previous_layer_size: usize) -> Self {
    Self {
      Z: Array2::zeros((layer_size, previous_layer_size)),
      A: Array2::zeros((layer_size, previous_layer_size)),
      dW: Array2::zeros((layer_size, previous_layer_size)),
      vdW: Array2::zeros((layer_size, previous_layer_size)),
      sdW: Array2::zeros((layer_size, previous_layer_size)),
      db: 0.0,
      vdb: 0.0,
      sdb: 0.0,
      dZ: Array2::zeros((layer_size, previous_layer_size)),
      dA: Array2::zeros((layer_size, previous_layer_size)),
    }
  }
}

/// #### Layer state
/// Contains all the internal variables/parameters of a layer
///
/// Weights have to be of dimension (L,L-1), L being the number of nodes in a layer
///
#[allow(non_snake_case)]
#[derive(Debug)]
pub struct LayerState {
  /// Weights.(W)
  pub W: Array2<f32>,
  /// Biases, denoted b
  pub b: f32,
  /// activation cache
  pub cache: LayerCache,
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
        ndarray_rand::rand_distr::Normal::new(0.0, 1.0).unwrap(),
      ),
      b: 0.0,
      cache: LayerCache::new(layer_size, previous_layer_size),
    }
  }
}

/// #### Layer in a neural network
/// They can be tuned for each layer.
///
#[derive(Debug)]
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
#[derive(Debug)]
pub enum Activation {
  ReLU,
  ClippedReLU,
  ExtendedClippedReLU,
  Tanh,
  Sigmoid,
  None,
}

/// ### NNUE
///
/// Just contains a bunch of Neural Net layers
///
#[derive(Debug)]
pub struct NNUE {
  /// Vector of Neural Net layers
  pub layers: Vec<Layer>,
  /// Keeping tracks of how many times we iterated, i.e. updated the parameters
  pub iterations: usize,
}

// Regularization for the weights:
// It's a bit like normalization for the activation layer.
// 1. if using ReLU, then initialize with w = rand / sqrt(2/n[l-1])
// 2. if using tanh, then initialize with w = rand / sqrt(1/n[l-1])
// 3. if using sigmoid,... let's not use sigmoid

impl Default for NNUE {
  fn default() -> Self {
    let mut nnue = NNUE::new();
    nnue.add_layer(
      NNUE::LAYER_1_SIZE,
      HyperParameters::default(),
      Activation::ClippedReLU,
    );
    nnue.add_layer(
      NNUE::LAYER_2_SIZE,
      HyperParameters::default(),
      Activation::ClippedReLU,
    );
    nnue.add_layer(
      NNUE::LAYER_3_SIZE,
      HyperParameters::default(),
      Activation::Tanh,
    );
    nnue
  }
}

#[allow(non_snake_case)]
impl NNUE {
  /// Size of the input layer, has to be squares x piece_types x 2 (color)
  const LAYER_0_SIZE: usize = 64 * 6 * 2;
  const LAYER_1_SIZE: usize = 64;
  const LAYER_2_SIZE: usize = 8;
  const LAYER_3_SIZE: usize = 1;

  /// Creates a new NNUE
  ///
  /// Will only contains the fixed size Input layer.
  /// More layers need to be added manually.
  ///
  /// ### Return value
  ///
  /// NNUE with an input layer.
  ///
  pub fn new() -> Self {
    // Input Layer (L0):
    let l0 = Layer {
      nodes: Self::LAYER_0_SIZE,
      param: HyperParameters::zeros(),
      a: Activation::None,
      state: LayerState::new(Self::LAYER_0_SIZE, 1),
    };

    let mut nnue = NNUE {
      layers: Vec::new(),
      iterations: 0,
    };
    nnue.layers.push(l0);

    nnue
  }

  /// Creates a new Neural Net without any layer
  ///
  /// ### Return value
  ///
  /// NNUE without any layer
  ///
  pub fn new_no_layer() -> Self {
    NNUE {
      layers: Vec::new(),
      iterations: 0,
    }
  }

  /// Adds a layer to the NNUE:
  pub fn add_layer(&mut self, nodes: usize, param: HyperParameters, a: Activation) {
    let last_layer_size = if self.layers.len() > 0 { self.layers.last().unwrap().nodes } else { 1 };
    let layer = Layer {
      nodes,
      param,
      a,
      state: LayerState::new(nodes, last_layer_size),
    };

    self.layers.push(layer);
  }

  pub fn f32_slice_to_input_layer(&mut self, input: &[f32]) {
    let mut a0: Array2<f32> = Array2::zeros((1, input.len()));

    for m in 0..input.len() {
      a0[[0, m]] = input[m];
    }

    self.layers[0].state.cache.A = a0;
  }

  /// Decays the learning rate
  ///
  /// ```
  /// let mut nnue = NNUE::default()
  /// nnue.decay_learning_rate(0.9);
  /// ```
  ///
  /// ### Arguments
  ///
  /// * `decay`: Factor by which the learning rate gets multiplied.
  ///
  ///
  pub fn decay_learning_rate(&mut self, decay: f32) {
    for i in 1..self.layers.len() {
      self.layers[i].param.learning_rate *= decay;
    }
  }

  /// Calculates the prediction/output given the current state of the NNUE.
  ///
  /// input can be of the size of a mini batch
  ///
  ///
  ///
  pub fn forward_propagation(&mut self) -> Vec<f32> {
    // This is A0, input data:
    let mut A_prev = self.layers[0].state.cache.A.clone();

    for i in 1..self.layers.len() {
      //println!("Forward prop: Layer {i} - {} nodes", self.layers[i].nodes);

      // Zl = Wl.A(l-1) + bl
      let mut Zl = self.layers[i].state.W.dot(&A_prev);
      Zl += self.layers[i].state.b;
      //println!("Zl: {:?}", Zl);

      // Normalize Z to have zero mean and unit standard deviation
      //let mean = Zl.mean().unwrap();
      //let sigma = Zl.std(0.0);
      //Zip::from(&mut Zl).par_for_each(|a| *a = (*a - mean) / (sigma + f32::EPSILON));
      //println!("Zl normalized: {:?}", Zl);

      // Save Zl onto the layer:
      self.layers[i].state.cache.Z = Zl.clone();

      // Al = gl(Zl) where gl is the activation function for that layer
      match self.layers[i].a {
        Activation::ReLU => Zip::from(&mut Zl).par_for_each(|a| *a = relu(*a)),
        Activation::ClippedReLU => Zip::from(&mut Zl).par_for_each(|a| *a = clipped_relu(*a)),
        Activation::ExtendedClippedReLU => {
          Zip::from(&mut Zl).par_for_each(|a| *a = extended_clipped_relu(*a, 200.0))
        },
        Activation::Tanh => Zip::from(&mut Zl).par_for_each(|a| *a = a.tanh()),
        Activation::Sigmoid => Zip::from(&mut Zl).par_for_each(|a| *a = sigmoid(*a)),
        Activation::None => {},
      };
      let Al = Zl;
      self.layers[i].state.cache.A = Al.clone();

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

  /// Calculates the prediction/output given the current state of the NNUE.
  /// Does not cache any of the intermediate values, so it can run faster
  ///
  /// Note: Do not apply backwards propagation after calling this function
  ///
  /// input can be of the size of a mini batch
  ///
  ///
  pub fn predict(&mut self) -> Vec<f32> {
    // This is A0, input data:
    let mut A_prev = self.layers[0].state.cache.A.clone();

    for i in 1..self.layers.len() {
      // Calculate Z for that layer:
      let mut Zl = self.layers[i].state.W.dot(&A_prev);
      Zl += self.layers[i].state.b;

      // Al = gl(Zl) where gl is the activation function for that layer
      match self.layers[i].a {
        Activation::ReLU => Zip::from(&mut Zl).par_for_each(|a| *a = relu(*a)),
        Activation::ClippedReLU => Zip::from(&mut Zl).par_for_each(|a| *a = clipped_relu(*a)),
        Activation::ExtendedClippedReLU => {
          Zip::from(&mut Zl).par_for_each(|a| *a = extended_clipped_relu(*a, 200.0))
        },
        Activation::Tanh => Zip::from(&mut Zl).par_for_each(|a| *a = a.tanh()),
        Activation::Sigmoid => Zip::from(&mut Zl).par_for_each(|a| *a = sigmoid(*a)),
        Activation::None => {},
      };

      let Al = Zl;

      A_prev = Al;
    }

    // Calculate predictions
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
  /// dA^{[l-1]} = \frac{\partial \mathcal{L} }{\partial A^{[l-1]}} = W^{[l] T} dZ^{[l]}
  /// ````
  ///
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

      self.layers[i].state.cache.dA = dA_prev;

      //println!("dAl-1 dimensions: {}x{}", dA_prev.len(), dA_prev[0].len());
      //println!("dZl");
      let mut dZl = self.layers[i].state.cache.Z.clone();
      match self.layers[i].a {
        Activation::ReLU => Zip::from(&mut dZl).par_for_each(|a| *a = relu_backwards(*a)),
        Activation::ClippedReLU => {
          Zip::from(&mut dZl).par_for_each(|a| *a = clipped_relu_backwards(*a))
        },
        Activation::ExtendedClippedReLU => {
          Zip::from(&mut dZl).par_for_each(|a| *a = extended_clipped_relu_backwards(*a, 200.0))
        },
        Activation::Tanh => Zip::from(&mut dZl).par_for_each(|a| *a = tanh_backwards(*a)),
        Activation::Sigmoid => Zip::from(&mut dZl).par_for_each(|a| *a = sigmoid_backwards(*a)),
        // No activation means that we have to skip the derivative of the activation layer. Replace everything with 1.0
        Activation::None => dZl.par_mapv_inplace(|_| 1.0),
      };

      dZl *= &self.layers[i].state.cache.dA;
      //println!("dZl = {:?}", dZl);
      //println!("dZl dimensions: {}x{}", dZl.len(), dZl[0].len());

      // Cache the result
      self.layers[i].state.cache.dZ = dZl.clone();

      //println!("dWl");
      let dWl = dZl.dot(&self.layers[i - 1].state.cache.A.t()) / m;
      self.layers[i].state.cache.dW = dWl.clone();
      //println!("dWl = {:?}", dWl);
      //println!("dWl dimensions: {}x{}", dWl.len(), dWl[0].len());

      //println!("dbl");
      self.layers[i].state.cache.db = (1.0 / m) * dZl.sum();
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
  pub fn check_weight_gradient(&mut self, y: &[f32], layer: usize) {
    // Do that for all rows/columns of the layer Weight matrix:
    const EPSILON: f32 = 1e-5;

    let shape_c = self.layers[layer].state.W.shape()[0];
    let shape_r = self.layers[layer].state.W.shape()[1];
    println!("Layer size: {}x{}", shape_c, shape_r);
    let m = y.len() as f32;

    // Select 20 points randomly
    for _ in 0..20 {
      let c = rand::random::<usize>() % shape_c;
      let r = rand::random::<usize>() % shape_r;

      // Capture a weight at a given layer, row, colum:
      let W = self.layers[layer].state.W[[c, r]];
      let dW = self.layers[layer].state.cache.dW[[c, r]];

      // Update W to W + epsilon
      self.layers[layer].state.W[[c, r]] = W + EPSILON;
      let y_hat_plus = self.forward_propagation();
      let cost_plus = total_cost(&y_hat_plus, y) / m;

      // Update W to W - epsilon
      self.layers[layer].state.W[[c, r]] = W - EPSILON;
      let y_hat_minus = self.forward_propagation();
      let cost_minus = total_cost(&y_hat_minus, y) / m;

      // Calculate gradient based on these tiny offsets
      let test_dW = (cost_plus - cost_minus) / (2.0 * EPSILON);

      // Restore W
      self.layers[layer].state.W[[c, r]] = W;

      println!("cost diff: {:?} ", cost_plus - cost_minus);
      println!("W {} - dW[{c}][{r}] : {} - test_dW: {} \n", W, dW, test_dW);
      assert!((test_dW - dW).abs() < 0.05);
    }
  }

  pub fn check_bias_gradient(&mut self, y: &[f32], layer: usize) {
    // Do that for all rows/columns of the layer Weight matrix:
    const EPSILON: f32 = 1e-5;

    // Capture a weight at a given layer, row, colum:
    let b = self.layers[layer].state.b;
    let db = self.layers[layer].state.cache.db;
    let m = y.len() as f32;

    // Update b to b + epsilon
    self.layers[layer].state.b = b + EPSILON;
    let y_hat_plus = self.forward_propagation();
    let cost_plus = total_cost(&y_hat_plus, y) / m;

    // Update b to b - epsilon
    self.layers[layer].state.b = b - EPSILON;
    let y_hat_minus = self.forward_propagation();
    let cost_minus = total_cost(&y_hat_minus, y) / m;

    // Calculate gradient based on these tiny offsets
    let test_db = (cost_plus - cost_minus) / (2.0 * EPSILON);

    // Restore b
    self.layers[layer].state.b = b;

    //println!("y_hat +: {:?} / cost {:?}", y_hat_plus, cost_plus);
    //println!("y_hat -: {:?}  / cost {:?}", y_hat_minus, cost_minus);
    //println!("cost diff: {:?} ", cost_plus - cost_minus);
    println!("b: {} - db : {} - test_db: {} \n", b, db, test_db);
    assert!((test_db - db).abs() < 0.05);
  }

  /// Update the parameters using back-propagation gradient calculation
  ///
  ///
  #[allow(non_snake_case)]
  pub fn update_parameters(&mut self) {
    // Increment the number of times we updated the parameters
    self.iterations += 1;

    for i in 1..self.layers.len() {
      //println!("Updating layer {}: {:?}", i, self.layers[i].state.W);
      // Compute the new momentum:
      let beta_1 = self.layers[i].param.beta_1;
      let beta_2 = self.layers[i].param.beta_2;
      let learning_rate = self.layers[i].param.learning_rate;

      // Make a separate copy of dW that we can modify locally
      let mut dW = self.layers[i].state.cache.dW.clone();
      Zip::from(&mut self.layers[i].state.cache.vdW)
        .and(&dW)
        .par_for_each(|vdw, &dw| *vdw = beta_1 * *vdw + (1.0 - beta_1) * dw);

      // Now square the dW and compute sdW
      Zip::from(&mut dW).par_for_each(|dw| *dw = dw.powf(2.0));
      Zip::from(&mut self.layers[i].state.cache.sdW)
        .and(&dW)
        .par_for_each(|sdw, &dw2| *sdw = beta_2 * *sdw + (1.0 - beta_2) * dw2);

      // Apply the momentum update on the parameters:
      let vdW = self.layers[i].state.cache.vdW.clone();
      let sdW = self.layers[i].state.cache.sdW.clone();

      let beta_1_correction = 1.0 - beta_1.powf(self.iterations as f32);
      let beta_2_correction = 1.0 - beta_2.powf(self.iterations as f32);

      // W = W - alpha* (vdW/(1-beta^t)) / sqrt( sdW/((1-beta2^t))) + epsilon
      Zip::from(&mut self.layers[i].state.W)
        .and(&vdW)
        .and(&sdW)
        .par_for_each(|w, &vdw, &sdw| {
          *w -= learning_rate * (vdw / beta_1_correction)
            / (f32::sqrt(sdw / beta_2_correction) + f32::EPSILON);
        });

      // Regular W = W - alpha*dW
      /*
      Zip::from(&mut self.layers[i].state.W)
        .and(&dW)
        .par_for_each(|w, &dw| *w -= learning_rate * dw);
      */

      // New vdb / sdb
      self.layers[i].state.cache.vdb =
        self.layers[i].state.cache.vdb * beta_1 + (1.0 - beta_1) * self.layers[i].state.cache.db;
      self.layers[i].state.cache.sdb = self.layers[i].state.cache.sdb * beta_2
        + (1.0 - beta_2) * self.layers[i].state.cache.db.powf(2.0);

      // Update b:
      /*
      println!(
        "Updating b: {:?} - {} - {} ",
        self.layers[i].state.b, beta_1_correction, beta_2_correction
      );
      println!("sdb: {} ", self.layers[i].state.cache.sdb);
      println!("vdb: {} ", self.layers[i].state.cache.vdb);
      println!("Denom: {}", (f32::sqrt(self.layers[i].state.cache.sdb)));
      */

      self.layers[i].state.b -= learning_rate
        * (self.layers[i].state.cache.vdb / beta_1_correction)
        / (f32::sqrt(self.layers[i].state.cache.sdb / beta_2_correction) + f32::EPSILON);
      //println!("Updated layer {}: {:?}", i, self.layers[i].state.b);

      // Decay a little bit the learning rate
      //self.layers[i].param.learning_rate *= 1.0 / (1.0 + 0.000001 * self.iterations as f32);
    }
  }

  /// TODO: Write description
  ///
  ///
  ///
  pub fn game_state_to_input_layer(&mut self, input: &[&GameState]) {
    let mut a0: Array2<f32> = Array2::zeros((Self::LAYER_0_SIZE, input.len()));

    for m in 0..input.len() {
      // ptp : piece to play (side to play) ; op: opposite pieces
      let flip_board = input[m].board.side_to_play == Color::Black;
      let (ptp, op) = match input[m].board.side_to_play {
        Color::White => (input[m].board.pieces.white, input[m].board.pieces.black),
        Color::Black => (input[m].board.pieces.black, input[m].board.pieces.white),
      };

      // Let's do: rook (offset = 0), queens (offset = 1 x 64), bishops (offset = 2 x 64), knights (offset = 3 x 64), king (offset = 4 x 64), pawn (offset = 5 x 64)
      for (mut i, piece) in ptp {
        if flip_board {
          i = 63 - i;
        }
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
      for (mut i, piece) in op {
        if flip_board {
          i = 63 - i;
        }
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

    self.layers[0].state.cache.A = a0;
  }

  /// Converts any sized type to a slice of bytes.
  ///
  unsafe fn as_bytes<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
  }

  /// Converts any sized type to a slice of bytes.
  ///
  unsafe fn as_mut_bytes<T: Sized>(p: &T) -> &mut [u8] {
    ::core::slice::from_raw_parts_mut((p as *const T) as *mut u8, ::core::mem::size_of::<T>())
  }

  /// Saves the NNUE bytes to a file, so it can be loaded again later
  ///
  pub fn save(&self, output_file: &str) -> std::io::Result<()> {
    let mut writer = BufWriter::new(File::create(output_file)?);

    // Write a magic byte
    // https://en.wikipedia.org/wiki/List_of_file_signatures
    //
    writer.write_all(MAGIC_BYTES.as_bytes())?;
    for i in 1..self.layers.len() {
      // Format will be: Layer size - Activation - Weights - Bias
      let cols = self.layers[i].state.W.shape()[0];
      let rows = self.layers[i].state.W.shape()[1];
      writer.write_all(unsafe { NNUE::as_bytes(&cols) })?;
      writer.write_all(unsafe { NNUE::as_bytes(&self.layers[i].a) })?;
      // Then dump the Weights and bias
      for c in 0..cols {
        for r in 0..rows {
          let bytes = unsafe { NNUE::as_bytes(&self.layers[i].state.W[[c, r]]) };
          writer.write_all(bytes)?;
        }
      }
      let bytes = unsafe { NNUE::as_bytes(&self.layers[i].state.b) };
      writer.write_all(bytes)?;
    }

    Ok(())
  }

  /// Saves the NNUE bytes to a file, so it can be loaded again later
  ///
  pub fn load(input_file: &str) -> std::io::Result<Self> {
    let file = File::open(input_file)?;
    let mut reader = BufReader::new(file);
    let mut nnue = Self::new();
    let mut layer = 0;

    let mut magic_bytes = [0; MAGIC_BYTES.len()];
    reader.read_exact(&mut magic_bytes)?;
    if magic_bytes != MAGIC_BYTES.as_bytes() {
      println!("Error: Trying to read NNUE format on wrong file: {input_file}");
      return Err(std::io::Error::from_raw_os_error(22));
    }

    loop {
      layer += 1;
      let layer_size: usize = 0;
      assert!(nnue.layers.last().is_some());
      let last_layer_size = nnue.layers.last().unwrap().nodes;
      let activation = Activation::None;

      if let Err(_) = reader.read_exact(&mut unsafe { NNUE::as_mut_bytes(&layer_size) }) {
        break;
      }
      reader.read_exact(&mut unsafe { NNUE::as_mut_bytes(&activation) })?;

      println!("Layer size: {layer_size} - Activation: {:?}", activation);
      nnue.add_layer(layer_size, HyperParameters::default(), activation);

      for c in 0..layer_size {
        for r in 0..last_layer_size {
          reader
            .read_exact(&mut unsafe { NNUE::as_mut_bytes(&nnue.layers[layer].state.W[[c, r]]) })?;
        }
      }
      reader.read_exact(&mut unsafe { NNUE::as_mut_bytes(&nnue.layers[layer].state.b) })?;
    }

    Ok(nnue)
  }
}

//------------------------------------------------------------------------------
// Tests
#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
  use super::*;

  #[test]
  fn test_game_state_to_input_layer() {
    let game_state = GameState::default();

    let mut nnue = NNUE::default();
    nnue.game_state_to_input_layer(&vec![&game_state]);
    let a0 = nnue.layers[0].state.cache.A.clone();
    //println!("{:?}", a0);

    // We expect 32 pieces on the board:
    assert_eq!(32.0, a0.sum());

    let game_state = GameState::from_fen("r1b1r1k1/ppp4p/3p3b/8/4P3/7P/PP2Q1P1/RN2K3 b - - 2 0");
    nnue.game_state_to_input_layer(&vec![&game_state]);
    let a0 = nnue.layers[0].state.cache.A.clone();
    assert_eq!(19.0, a0.sum());
  }

  #[test]
  fn test_forward_propagation() {
    let game_state_1 = GameState::default();
    let game_state_2 =
      GameState::from_fen("rnbqkbnr/ppp1pppp/8/3p4/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 2");
    let mut nnue = NNUE::default();
    let mini_batch = vec![&game_state_1, &game_state_2];

    nnue.game_state_to_input_layer(&mini_batch);
    let Y_hat = nnue.forward_propagation();

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

    let mut nnue = NNUE::default();
    let mini_batch = vec![&game_state_1, &game_state_2, &game_state_3, &game_state_4];
    let mut previous_cost = 100000.0;

    for _ in 0..100 {
      nnue.game_state_to_input_layer(&mini_batch);
      let Y_hat = nnue.forward_propagation();

      println!("Prediction: {:?}", Y_hat);
      nnue.backwards_propagation(&Y_hat, &evals);

      let new_cost = total_cost(&Y_hat, &evals);
      println!("new cost: {} - old cost:  {}", new_cost, previous_cost);
      //assert!(new_cost < previous_cost);
      previous_cost = new_cost;
    }

    nnue.game_state_to_input_layer(&mini_batch);
    let Y_hat = nnue.forward_propagation();
    println!("Prediction: {:?}", Y_hat);
    println!("True labels: {:?}", evals);
    println!("Cost {}", total_cost(&Y_hat, &evals));
  }

  #[test]
  fn test_gradient_checking_layer_3() {
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

    let mut nnue = NNUE::default();
    let mini_batch = vec![&game_state_1, &game_state_2, &game_state_3, &game_state_4];
    nnue.game_state_to_input_layer(&mini_batch);
    let Y_hat = nnue.forward_propagation();
    println!("Prediction: {:?}", Y_hat);
    println!("Actual: {:?}", evals);
    println!("Total cost: {:?}", total_cost(&Y_hat, &evals));

    nnue.backwards_propagation(&Y_hat, &evals);

    nnue.check_weight_gradient(&evals, 1);
  }

  #[test]
  fn test_gradient_checking_layer_2() {
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

    let mut nnue = NNUE::default();
    let mini_batch = vec![&game_state_1, &game_state_2, &game_state_3, &game_state_4];
    nnue.game_state_to_input_layer(&mini_batch);
    let Y_hat = nnue.forward_propagation();
    println!("Prediction: {:?}", Y_hat);
    println!("Actual: {:?}", evals);
    println!("Total cost: {:?}", total_cost(&Y_hat, &evals));

    nnue.backwards_propagation(&Y_hat, &evals);
    nnue.check_weight_gradient(&evals, 2);
  }

  #[test]
  fn test_gradient_checking_layer_1() {
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

    let mut nnue = NNUE::default();
    let mini_batch = vec![&game_state_1, &game_state_2, &game_state_3, &game_state_4];
    nnue.game_state_to_input_layer(&mini_batch);
    let Y_hat = nnue.forward_propagation();
    println!("Prediction: {:?}", Y_hat);
    println!("Actual: {:?}", evals);
    println!("Total cost: {:?}", total_cost(&Y_hat, &evals));

    nnue.backwards_propagation(&Y_hat, &evals);
    nnue.check_weight_gradient(&evals, 1);
  }

  #[test]
  fn test_gradient_checking_bias_layer_3() {
    let game_state_4 = GameState::from_fen("5k2/p1p5/1p5p/6p1/5p1P/2b1P3/Pr5B/3rNKR1 w - - 2 31");
    let mut evals: Vec<f32> = vec![-199.0];
    for i in 0..evals.len() {
      evals[i] = (evals[i] / 15.0).tanh();
    }

    let mut nnue = NNUE::default();

    let mini_batch = vec![&game_state_4];
    nnue.game_state_to_input_layer(&mini_batch);
    let Y_hat = nnue.forward_propagation();
    println!("Prediction: {:?}", Y_hat);
    println!("Actual: {:?}", evals);
    println!("Total cost: {:?}", total_cost(&Y_hat, &evals));

    nnue.backwards_propagation(&Y_hat, &evals);
    nnue.check_bias_gradient(&evals, 3);
  }

  #[test]
  fn test_saving_the_nnue() {
    let mut nnue = NNUE::default();

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

    let mini_batch = vec![&game_state_1, &game_state_2, &game_state_3, &game_state_4];
    // Spin the NNUE a little bit:
    nnue.game_state_to_input_layer(&mini_batch);
    let initial_Y_hat = nnue.forward_propagation();

    for _ in 0..100 {
      nnue.game_state_to_input_layer(&mini_batch);
      let Y_hat = nnue.forward_propagation();
      nnue.backwards_propagation(&Y_hat, &evals);
      nnue.update_parameters();
    }

    nnue.game_state_to_input_layer(&mini_batch);
    let Y_hat = nnue.forward_propagation();

    // Now save to a file:
    let _ = nnue.save("super_net.nnue");

    // Now I expect that if we reload, we get the same prediction again
    let mut nnue_2 = NNUE::load("super_net.nnue").unwrap();

    nnue_2.game_state_to_input_layer(&mini_batch);
    let new_Y_hat = nnue_2.forward_propagation();
    println!("Prediction before: {:?}", Y_hat);
    println!("Prediction after: {:?}", new_Y_hat);
    assert_ne!(initial_Y_hat, Y_hat);
    assert_eq!(Y_hat, new_Y_hat);

    std::fs::remove_file("super_net.nnue").unwrap();
  }
}
