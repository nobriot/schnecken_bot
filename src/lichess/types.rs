use serde::{Deserialize, Serialize};

use crate::chess::model::piece::Color;

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
  pub color: Color,
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

/// Game state object received during the games
#[derive(Debug, Deserialize, Serialize)]
pub struct GameState {
  pub moves: String,
  pub wtime: usize,
  pub btime: usize,
  pub winc: usize,
  pub binc: usize,
  pub status: GameStatus,
  pub winner: Option<Color>,
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GameStatus {
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

#[derive(Debug, Deserialize, Serialize)]
pub struct Challenge {
  pub id: String,
  pub challenger: Challenger,
  #[serde(rename = "destUser")]
  pub destination_user: Challenger,
  pub rated: bool,
  pub variant: Variant,
  #[serde(rename = "timeControl")]
  pub time_control: TimeControl,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Challenger {
  pub id: String,
  pub name: String,
  pub online: bool,
  pub rating: usize,
  pub title: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TimeControl {
  pub increment: usize,
  pub limit: usize,
  pub show: String,
  #[serde(rename = "type")]
  pub control_type: TimeControlType,
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TimeControlType {
  Clock,
  Correspondence,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Clock {
  pub initial: i32,
  pub increment: i32,
  pub totaltime: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Player {
  id: String,
  username: String,
  rating: usize,
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Speed {
  Bullet,
  Blitz,
  Rapid,
  Classical,
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Perf {
  Bullet,
  Blitz,
  Rapid,
  Classical,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Variant {
  pub key: VariantKey,
  pub name: String,
  pub short: String,
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VariantKey {
  Standard,
  Chess960,
  KingOfTheHill,
  ThreeCheck,
  Antichess,
  Atomic,
  Horde,
  RacingKings,
}
