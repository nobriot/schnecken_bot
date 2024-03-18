use std::str::FromStr;

#[derive(Copy, Debug, Clone, Eq, PartialEq, Default)]
pub enum PlayStyle {
  /// Normal play style for the engine
  #[default]
  Normal,
  /// Engine will try to play very safe lines. Kind of good if the opponent is
  /// Stronger and we just want to draw
  Conservative,
  /// Try spectacular sacrifices to get to the king.
  Aggressive,
  /// Use this with weaker opponents, to play dangerous/provocative lines
  /// like the bongcloud.
  Provocative,
}

impl FromStr for PlayStyle {
  type Err = ();

  fn from_str(input: &str) -> Result<PlayStyle, Self::Err> {
    match input.to_lowercase().as_str() {
      "normal" => Ok(PlayStyle::Normal),
      "conservative" => Ok(PlayStyle::Conservative),
      "aggressive" => Ok(PlayStyle::Aggressive),
      "provocative" => Ok(PlayStyle::Provocative),
      _ => Err(()),
    }
  }
}
