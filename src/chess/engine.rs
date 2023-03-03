use rand::Rng;

pub fn play_move(fen: &str) -> String {
    // For now we just ignore the position and try to play random moves!
    let mut rng = rand::thread_rng();
    let source_letter = ((rng.gen_range(0..8)) + b'a') as char;
    let source_number = (rng.gen_range(0..8)) + 1;
    let destination_letter = ((rng.gen_range(0..8)) + b'a') as char;
    let destination_number = (rng.gen_range(0..8)) + 1;

    let chess_move: String = String::from(format!(
        "{}{}{}{}",
        source_letter, source_number, destination_letter, destination_number
    ));
    return chess_move;
}
