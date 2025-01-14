use serde::{Deserialize, Serialize};

// Reasons for declining a challenge
// pub const DECLINE_GENERIC: &str = "generic";
// pub const DECLINE_TOO_FAST: &str = "tooFast";
// pub const DECLINE_TOO_SLOW: &str = "tooSlow";
// pub const DECLINE_STANDARD: &str = "standard";
// pub const DECLINE_CASUAL: &str = "casual";
// pub const DECLINE_NOT_BOTS: &str = "noBot";
// pub const DECLINE_ONLY_BOTS: &str = "onlyBot";
pub const DECLINE_RATED: &str = "rated";
pub const DECLINE_VARIANT: &str = "variant";
pub const DECLINE_LATER: &str = "later";
pub const DECLINE_TIME_CONTROL: &str = "timeControl";

/// Game information contained for GameStart / GameFinish events
#[derive(Debug, Deserialize, Serialize)]
pub struct GameStart {
  #[serde(rename = "gameId")]
  pub game_id:      String,
  pub color:        Color,
  pub fen:          Option<String>,
  #[serde(rename = "hasMoved")]
  pub has_moved:    bool,
  #[serde(rename = "isMyTurn")]
  pub is_my_turn:   bool,
  #[serde(rename = "lastMove")]
  pub last_move:    Option<String>,
  pub speed:        String,
  pub rated:        bool,
  pub opponent:     Player,
  #[serde(rename = "secondsLeft")]
  pub seconds_left: usize,
  pub winner:       Option<Color>,
}

impl GameStart {
  pub fn opponent_is_bot(&self) -> bool {
    self.opponent.username.contains("BOT ")
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum Color {
  White,
  Black,
}

/// Full game state
#[derive(Debug, Deserialize, Serialize)]
pub struct GameFull {
  #[serde(rename = "type")]
  pub typ:     String,
  pub id:      String,
  pub rated:   bool,
  pub variant: Variant,
  pub clock:   Clock,
  pub speed:   Speed,

  #[serde(rename = "initialFen")]
  pub initial_fen: String,
  pub state:       GameState,
}

/// Game state object received during the games
#[derive(Debug, Deserialize, Serialize)]
pub struct GameState {
  pub moves:  String,
  pub wtime:  usize,
  pub btime:  usize,
  pub winc:   usize,
  pub binc:   usize,
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
  pub id:               String,
  pub challenger:       Challenger,
  #[serde(rename = "destUser")]
  pub destination_user: Challenger,
  pub rated:            bool,
  pub variant:          Variant,
  #[serde(rename = "timeControl")]
  pub time_control:     TimeControl,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Challenger {
  pub id:     String,
  pub name:   String,
  pub online: Option<bool>,
  pub rating: usize,
  pub title:  Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TimeControl {
  pub increment:    Option<usize>,
  pub limit:        Option<usize>,
  pub show:         Option<String>,
  #[serde(rename = "type")]
  pub control_type: TimeControlType,
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TimeControlType {
  Clock,
  Correspondence,
  Unlimited,
}

/// Player title, can be any FIDE title for titled players and BOT for bots.
#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum Title {
  CM,
  FM,
  IM,
  GM,
  WFM,
  WIM,
  WGM,
  BOT,
}

/// Clock used for the game.
#[derive(Serialize, Deserialize, Debug)]
pub struct Clock {
  pub initial:   i32,
  pub increment: i32,
  pub totaltime: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Player {
  pub id:          String,
  #[serde(alias = "name")]
  pub username:    String,
  pub rating:      usize,
  pub provisional: Option<bool>,
  pub title:       Option<Title>,
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
  pub key:   VariantKey,
  pub name:  String,
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

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct ChatMessage {
  pub room:         ChatRoom,
  pub text:         String,
  #[serde(rename = "type")]
  pub message_type: String,
  pub username:     String,
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ChatRoom {
  Spectator,
  Player,
}
