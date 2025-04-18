use ndarray::Array2;

//------------------------------------------------------------------------------
// Activation functions

/// ReLU activation function
/// computes the ReLU value.
///
/// ### Arguments
///
/// * `x`: input
///
/// ### Return value
///
/// x if x > 0
/// 0 if x <= 0
#[inline]
pub fn relu(x: f32) -> f32 {
  x.max(0.0)
}

/// ReLU backwards
///
/// Returns the derivative of the ReLU function, which is:
///
/// ```math
/// f'(x) =  \begin{cases}1 & x > 0\\0 & x < 0\end{cases}
/// ```
///
/// ### Arguments
///
/// * `x`: input
///
/// ### Return value
///
/// Derivative value of the ReLU function
#[inline]
pub fn relu_backwards(x: f32) -> f32 {
  if x > 0.0 {
    1.0
  } else {
    0.0
  }
}

/// Sigmoid activation function
///
/// ### Arguments
///
/// * `x`: input
///
/// ### Return value
///
/// 1 / (1 + exp(-x))
#[inline]
pub fn sigmoid(x: f32) -> f32 {
  1.0 / (1.0 + (-x).exp())
}

/// Sigmoid activation function derivative
///
/// ```math
///  \sigma '(x) =  \sigma (x) \times (1-  \sigma (x))
/// ```
///
/// ### Arguments
///
/// * `x`: input
///
/// ### Return value
///
/// derivative of the sigmoid function at x.
#[inline]
pub fn sigmoid_backwards(x: f32) -> f32 {
  sigmoid(x) * (1.0 - sigmoid(x))
}

/// tanh activation function derivative
///
/// ```math
/// tanh '(x) = 1 - tanh^2 (x)
/// ```
///
/// ### Arguments
///
/// * `x`: input
///
/// ### Return value
///
/// derivative of the sigmoid function at x.
#[inline]
pub fn tanh_backwards(x: f32) -> f32 {
  1.0 - x.tanh() * x.tanh()
}

/// Clipped ReLU activation function
///
/// ### Arguments
///
/// * `x`: input
///
/// ### Return value
///
///  0.0 if        x <= 0.0
///  x   if  0.0 < x <  1.0
///  1.0 if  1.0 < x
#[inline]
pub fn clipped_relu(x: f32) -> f32 {
  x.clamp(0.0, 1.0)
}

/// Clipped ReLU activation function backwards
///
/// Returns the derivative of the clipped ReLU function.
///
/// ```math
/// f'(x) =  \begin{cases}1 & x  \in [0;1] \\0 & otherwise \end{cases}
/// ```
///
/// ### Arguments
///
/// * `x`: input
///
/// ### Return value
///
/// Derivative of the clippped ReLU function.
#[inline]
pub fn clipped_relu_backwards(x: f32) -> f32 {
  if (0.0..=1.0).contains(&x) {
    0.0
  } else {
    1.0
  }
}

/// Extended Clipped ReLU activation function
///
/// ### Arguments
///
/// * `x`: input
/// * `threshold`: Value at which we want to clip the ReLU.
///
/// ### Return value
///
///  -1.0 if        x <= 1-.0
///  x   if -1.0 < x  <  1.0
///  1.0 if  1.0 < x
#[inline]
pub fn extended_clipped_relu(x: f32, threshold: f32) -> f32 {
  x.max(-threshold).min(threshold)
}

/// Extended Clipped ReLU activation backwards function
///
/// Returns the derivative of the extended clipped ReLU function.
///
/// ```math
/// f'(x) =  \begin{cases}1 & x  \in [-t;t] \\0 & otherwise \end{cases}
/// ```
///
/// ### Arguments
///
/// * `x`: input
/// * `t`: threshold value at which we want to clip the ReLU.
///
/// ### Return value
///
/// Derivative of the extended clipped ReLU
#[inline]
pub fn extended_clipped_relu_backwards(x: f32, t: f32) -> f32 {
  if x > t || x < -t {
    0.0
  } else {
    1.0
  }
}

//------------------------------------------------------------------------------
// Cost functions

/// Cost/Loss function
///
/// Here I think we are relatively happy when the engine evals +12 instead of
/// +9. This difference should not matter as much as evaluating +2 instead of
/// -1.
///
/// Let's use the Quadratic cost
///
/// ### Arguments
///
/// * `a`: Prediction
/// * `y`: Real outcome (training sample)
///
/// ### Return value
///
/// ```math
/// c =  (a - y)^2
/// ```
#[inline]
pub fn cost(a: f32, y: f32) -> f32 {
  (a - y).powf(2.0)
}

/// Cost/Loss function computed element wise for 2 vectors.
///
/// See [cost] for the function used by the cost.
///
/// ### Arguments
///
/// * `a`: Prediction
/// * `y`: Real outcome (training sample)
///
/// ### Return value
///
/// Column vector with cost values
pub fn cost_vector(a: &[f32], y: &[f32]) -> Array2<f32> {
  assert_eq!(a.len(),
             y.len(),
             "Vectors sizes are must be identical: {} vs {}",
             a.len(),
             y.len());
  let mut cost_vector = Array2::zeros([1, a.len()]);

  for i in 0..a.len() {
    cost_vector[[0, i]] = cost(a[i], y[i]);
  }

  cost_vector
}

/// Cost/Loss function derivative with respect to 1
///
/// This returns $dA^{[L]} = \frac{\partial\mathcal{C} }{\partial A^{[L]}}$
///
/// See [cost] for the function used by the cost.
///
/// ### Arguments
///
/// * `a`: Prediction
/// * `y`: Real outcome (training sample)
///
/// ### Return value
///
/// Value of the derivative.
/// $$c =  {2 \times (a - y)$$
#[inline]
pub fn cost_derivative(a: f32, y: f32) -> f32 {
  2.0 * (a - y)
}

/// Cost/Loss function derivative on a slice
///
/// Returns a vector of cost derivatives
/// See [cost_derivative]
///
/// ### Arguments
///
/// * `a`: Prediction
/// * `y`: Real outcome (training sample)
///
/// ### Return value
///
/// Column Vector with derivatives
pub fn cost_derivative_vector(a: &[f32], y: &[f32]) -> Array2<f32> {
  assert_eq!(a.len(),
             y.len(),
             "Vectors sizes are must be identical: {} vs {}",
             a.len(),
             y.len());

  let mut cost_derivative_vector = Array2::zeros([1, a.len()]);

  for i in 0..a.len() {
    cost_derivative_vector[[0, i]] = cost_derivative(a[i], y[i]);
  }

  cost_derivative_vector
}

/// Applies the loss/cost function on a vector.
///
/// ### Arguments
///
/// * `a`: slice of f32 with all predictions
/// * `y`: Slices of f32 with Real outcome (training sample)
///
/// ### Return value
///
/// Cumumated cost value
pub fn total_cost(a: &[f32], y: &[f32]) -> f32 {
  assert_eq!(a.len(),
             y.len(),
             "Vectors sizes are different: {} vs {}",
             a.len(),
             y.len());

  let mut total_cost = 0.0;

  for i in 0..a.len() {
    total_cost += cost(a[i], y[i]);
  }

  total_cost
}
