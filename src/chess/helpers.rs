use chess::*;
use log::*;
use std::str::FromStr;

// Constants
pub const START_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn get_fen_from_move_list(move_list: &str) -> String {
    if move_list.is_empty() {
        return String::from(START_POSITION_FEN);
    }

    let board_result = Board::from_str(START_POSITION_FEN);
    let mut board: Board;
    match board_result {
        Ok(b) => {
            board = b;
        }
        Err(_) => return String::from(START_POSITION_FEN),
    }

    let moves: Vec<&str> = move_list.split(' ').collect();
    for chess_move in moves {
        let chess_m = ChessMove::from_str(&chess_move);
        if let Err(_) = chess_m {
            error!("Error parsing moves.");
            return String::from(START_POSITION_FEN);
        }

        board = board.make_move_new(chess_m.unwrap());
    }

    return board.to_string();
}

mod tests {
    #[test]
    fn test_fen_from_moves() {
        use crate::chess::helpers::*;

        assert_eq!(get_fen_from_move_list(""), START_POSITION_FEN);
        assert_eq!(
            get_fen_from_move_list("32423562 34 3 2 4 "),
            String::from(START_POSITION_FEN)
        );
        assert_eq!(
            get_fen_from_move_list("32423562 34 3 2 4 "),
            String::from(START_POSITION_FEN)
        );

        assert_eq!(
            get_fen_from_move_list("e2e4 c7c5 f2f4 d7d6 g1f3 b8c6 f1c4 g8f6 d2d3 g7g6 e1g1 f8g7"),
            String::from("r1bqk2r/pp2ppbp/2np1np1/2p5/2B1PP2/3P1N2/PPP3PP/RNBQ1RK1 w kq - 0 1")
        );
    }
}
