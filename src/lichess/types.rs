use crate::chess;
use serde::{Deserialize, Serialize};

// Reasons for declining a challenge
/*
pub const DECLINE_GENERIC: &str = "generic";
pub const DECLINE_TOO_FAST: &str = "tooFast";
pub const DECLINE_TOO_SLOW: &str = "tooSlow";
pub const DECLINE_RATED: &str = "rated";
pub const DECLINE_CASUAL: &str = "casual";
pub const DECLINE_STANDARD: &str = "standard";
pub const DECLINE_NOT_BOTS: &str = "noBot";
pub const DECLINE_ONLY_BOTS: &str = "onlyBot";
*/
pub const DECLINE_VARIANT: &str = "variant";
pub const DECLINE_LATER: &str = "later";
pub const DECLINE_TIME_CONTROL: &str = "timeControl";

/// Game information contained for GameStart / GameFinish events
#[derive(Debug, Deserialize, Serialize)]
pub struct GameStart {
  #[serde(rename = "gameId")]
  pub game_id: String,
  pub color: chess::model::piece::Color,
  pub fen: Option<String>,
  #[serde(rename = "hasMoved")]
  pub has_moved: bool,
  #[serde(rename = "isMyTurn")]
  pub is_my_turn: bool,
  #[serde(rename = "lastMove")]
  pub last_move: Option<String>,
  #[serde(rename = "fullId")]
  pub speed: String,
  pub rated: bool,
  pub opponent: Player,
  #[serde(rename = "secondsLeft")]
  pub seconds_left: usize,
  //pub winner: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Clock {
  initial: i32,
  increment: i32,
  totaltime: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Player {
  id: String,
  username: String,
  rating: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Result {
  winner: Option<String>,
  status: String,
  clock: Option<Clock>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Speed {
  Blitz,
  Rapid,
  Classical,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Perf {
  Bullet,
  Blitz,
  Rapid,
  Classical,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Variant {
  Standard,
  Chess960,
  KingOfTheHill,
  ThreeCheck,
  Antichess,
  Atomic,
  Horde,
  RacingKings,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Status {
  Started,
  Aborted,
  Mate,
  Stalemate,
  Timeout,
  Resign,
  Draw,
  Outoftime,
  Cheat,
  NoStart,
}
