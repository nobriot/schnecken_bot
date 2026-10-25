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

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
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

/// Rating of a user in a performance category (bullet, blitz, etc.)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PerfRating {
  #[serde(default)]
  pub games:  u32,
  pub rating: u32,
  /// Provisional rating (not enough games played yet)
  #[serde(default)]
  pub prov:   bool,
}

/// Ratings of a user, for the categories we play
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct UserPerfs {
  pub bullet:    Option<PerfRating>,
  pub blitz:     Option<PerfRating>,
  pub rapid:     Option<PerfRating>,
  pub classical: Option<PerfRating>,
}

/// User information, as returned by the user, account, following and online
/// bots endpoints
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
  pub id:            String,
  pub username:      String,
  /// FIDE title, or "BOT" for bots. Kept as a string, as Lichess has more
  /// titles than the `Title` enum (NM, LM, etc.)
  pub title:         Option<String>,
  #[serde(default)]
  pub perfs:         UserPerfs,
  #[serde(default)]
  pub disabled:      bool,
  #[serde(default, rename = "tosViolation")]
  pub tos_violation: bool,
}

impl User {
  /// Returns true if the user is a bot account
  pub fn is_bot(&self) -> bool {
    self.title.as_deref() == Some("BOT")
  }

  /// Returns the rating of the user for a game speed, if any
  pub fn rating(&self, speed: Speed) -> Option<u32> {
    let perf = match speed {
      Speed::Bullet => &self.perfs.bullet,
      Speed::Blitz => &self.perfs.blitz,
      Speed::Rapid => &self.perfs.rapid,
      Speed::Classical => &self.perfs.classical,
    };
    perf.as_ref().map(|p| p.rating)
  }
}

/// Real-time status of a user
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserStatus {
  pub id:      String,
  pub name:    String,
  #[serde(default)]
  pub online:  bool,
  #[serde(default)]
  pub playing: bool,
}

/// Crosstable between 2 users
#[derive(Debug, Clone)]
pub struct Crosstable {
  /// Score of the first user
  pub user_1_score: f64,
  /// Score of the second user
  pub user_2_score: f64,
  /// Number of games played between the 2 users
  pub games:        u64,
}
