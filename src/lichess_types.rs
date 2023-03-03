use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
    pub color: Color,
    pub fen: String,
    pub full_id: String,
    pub id: String,
    pub has_moved: bool,
    pub is_my_turn: bool,
    pub last_move: String,
    pub opponent: Player,
    pub rated: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Color {
    White,
    Black,
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
