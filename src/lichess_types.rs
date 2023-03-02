use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
    pub id: String,
    pub rated: bool,
    pub initial_fen: String,
    pub variant: Variant,
    pub speed: Speed,
    pub perf: Perf,
    pub created_at: String,
    pub last_move_at: String,
    pub turns: i32,
    pub status: Status,
    pub clock: Option<Clock>,
    pub white: Player,
    pub black: Player,
    pub moves: String,
    pub opening_name: Option<String>,
    pub opening_ply: Option<i32>,
    pub result: Option<Result>,
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
    name: String,
    title: Option<String>,
    rating: i32,
    provisional: bool,
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
