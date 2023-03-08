use chess::*;
use log::*;
use rand::Rng;
use std::str::FromStr;
use std::time::{Duration, Instant};

// From our module
use crate::chess::eval::ChessEval;
use crate::chess::theory::*;

trait EngineEval {
    // Engine evaluation of a certain static position, without any depth
    fn eval_fen(fen: &str) -> Result<ChessEval, ()>;

    // Engine evaluation of a certain fen, using a certain depth
    fn eval(fen: &str, depth: u8) -> Result<(ChessEval, ChessMove), ()>;
}

pub fn eval_fen(fen: &str) -> Result<ChessEval, ()> {
    // Let just do a simple material count for now
    let board_result = Board::from_str(fen);
    let board: Board;
    match board_result {
        Ok(b) => {
            board = b;
        }
        Err(_) => return Err(()),
    }

    let mut chess_eval: ChessEval = ChessEval::Score(0.0);
    for square_index in 0..64 {
        unsafe {
            let current_square = Square::new(square_index);

            match board.piece_on(current_square) {
                Some(piece) => {
                    let mut score_offset: f32 = match piece {
                        Piece::Pawn => 1.0,
                        Piece::Knight => 3.0,
                        Piece::Bishop => 3.1,
                        Piece::Rook => 5.0,
                        Piece::Queen => 10.0,
                        Piece::King => 0.0,
                    };

                    if let Some(Color::Black) = board.color_on(current_square) {
                        score_offset = score_offset * (-1.0);
                    }

                    let chess_eval_offset = ChessEval::Score(score_offset);
                    chess_eval += chess_eval_offset;
                }
                _ => {}
            }
        }
    }

    return Ok(chess_eval);
}

pub fn eval(fen: &str, depth: u8, deadline: Instant) -> Result<(ChessEval, Option<ChessMove>), ()> {
    //debug!("eval {} at depth {}", fen, depth);

    let board_result = Board::from_str(fen);
    let board: Board;
    match board_result {
        Ok(b) => {
            board = b;
        }
        Err(_) => return Err(()),
    }

    if depth == 0 {
        // We've reached the end of the line, just evaluate the position and return.
        if let Ok(new_eval) = eval_fen(&board.to_string()) {
            return Ok((new_eval, None));
        } else {
            return Err(());
        }
    }

    // Check if we have been thinking too much:
    let current_time = Instant::now();
    if current_time > deadline {
        // Abort looking at the line by returning no move and a very bad evaluation.
        if board.side_to_move() == Color::White {
            return Ok((ChessEval::Checkmate(-1), None));
        } else {
            return Ok((ChessEval::Checkmate(1), None));
        }
    }

    // Check our available moves.
    let mut chess_eval: ChessEval = ChessEval::Score(0.0);
    let move_list = MoveGen::new_legal(&board);

    if move_list.len() == 0 {
        warn!("Cannot compute any move from fen: {fen} ??");
        return Err(());
    }

    let mut best_move_option: Option<ChessMove> = Option::None;
    let mut first_move: bool = true;
    for chess_move in move_list {
        let new_board = board.make_move_new(chess_move);

        let new_chess_eval;
        if let Ok((branch_eval, _)) = eval(&new_board.to_string(), depth - 1, deadline) {
            new_chess_eval = branch_eval;
        } else {
            warn!("Error evaluating position {} at depth {}", fen, depth);
            return Err(());
        }

        if first_move {
            chess_eval = new_chess_eval;
            best_move_option = Some(chess_move);
            first_move = false;
        } else {
            if board.side_to_move() == Color::White {
                if chess_eval < new_chess_eval {
                    chess_eval = new_chess_eval;
                    best_move_option = Some(chess_move);
                }
            } else {
                if chess_eval > new_chess_eval {
                    chess_eval = new_chess_eval;
                    best_move_option = Some(chess_move);
                }
            }
        }
    }

    return Ok((chess_eval, best_move_option));
}

pub fn play_move(fen: &str) -> Result<String, ()> {
    // Check if it is a known position
    if let Some(moves) = get_theory_moves(fen) {
        info!("We are in theory! Easy");
        let mut rng = rand::thread_rng();
        let random_good_move = rng.gen_range(0..moves.len());
        return Ok(moves[random_good_move].to_string());
    }

    // Try to evaluate ourselves.
    warn!("We should decide for a reasonable amount of time.");
    let deadline = Instant::now() + Duration::new(1, 0);

    if let Ok((chess_eval, best_move_option)) = eval(fen, 3, deadline) {
        if let Some(chess_move) = best_move_option {
            info!("Selecting move with evaluation {:?}", chess_eval);
            return Ok(chess_move.to_string());
        }
    }

    // Fallback on playing a random move:
    warn!("Eval went wrong. Playing a random move!");

    let board_result = Board::from_str(fen);
    let board: Board;
    match board_result {
        Ok(b) => {
            board = b;
        }
        Err(_) => return Err(()),
    }

    let mut move_list = MoveGen::new_legal(&board);
    if move_list.len() == 0 {
        warn!("Cannot compute any move from fen: {fen}");
        return Err(());
    }

    // Just Play a random move for now:
    let mut rng = rand::thread_rng();
    let random_legal_move = rng.gen_range(0..move_list.len());
    return Ok(move_list.nth(random_legal_move).unwrap().to_string());
}

#[cfg(test)]
mod tests {
    use crate::chess::engine::*;
    use crate::chess::eval::ChessEval;

    //#[test]
    fn eval_start_position() {
        let start_position =
            String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(Ok(ChessEval::Score(0.0)), eval_fen(&start_position));

        let deadline = Instant::now() + Duration::new(1, 0);
        let eval_result = eval(&start_position, 2, deadline);
        let (eval, chess_move_option) = eval_result.unwrap();
        println!("Chess Move: {:?}", chess_move_option);
        println!("Chess Eval: {:?}", eval);
    }

    //#[test]
    fn eval_single_pawn_for_white() {
        let single_white_pawn = String::from("3k4/8/8/8/8/8/4P3/3K4 w - - 0 1");
        assert_eq!(Ok(ChessEval::Score(1.0)), eval_fen(&single_white_pawn));

        let deadline = Instant::now() + Duration::new(1, 0);
        let eval_result = eval(&single_white_pawn, 2, deadline);
        let (eval, chess_move_option) = eval_result.unwrap();
        println!("Chess Move: {:?}", chess_move_option);
        println!("Chess Eval: {:?}", eval);
    }
}
