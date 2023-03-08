use crate::chess::model::board::Board;
use crate::chess::model::piece::Color;

pub const START_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub struct GameState {
  side_to_play: Color,
  is_check: bool,
  is_checkmate: bool,
  en_passant_square: u8,
  board: Board,
  ply: u8,
  move_count: u8,
}

