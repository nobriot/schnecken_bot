use crate::model::game_state::START_POSITION_FEN;
use crate::model::piece_set::*;

#[test]
fn test_piece_masks() {
  let mut pieces: PieceSet = PieceSet::new();

  for i in 0..64 {
    assert_eq!(NO_PIECE, pieces.get_usize(i));
  }
  assert_eq!(0, pieces.all());
  assert_eq!(None, pieces.white.get_king());
  assert_eq!(None, pieces.black.get_king());

  pieces.add(WHITE_QUEEN, 0);
  pieces.add(BLACK_KING, 1);

  assert_eq!(WHITE_QUEEN, pieces.get(0));
  assert_eq!(BLACK_KING, pieces.get(1));
  assert_eq!(None, pieces.white.get_king());
  assert_eq!(Some(1), pieces.black.get_king());

  // Now the king becomes a rook:
  pieces.update(WHITE_ROOK, 1);
  assert_eq!(WHITE_QUEEN, pieces.get(0));
  assert_eq!(WHITE_ROOK, pieces.get(1));
  assert_eq!(None, pieces.white.get_king());
  assert_eq!(None, pieces.black.get_king());

  // Now the queen becomes a king:
  pieces.update(WHITE_KING, 0);
  assert_eq!(WHITE_KING, pieces.get(0));
  assert_eq!(WHITE_ROOK, pieces.get(1));
  assert_eq!(Some(0), pieces.white.get_king());
  assert_eq!(None, pieces.black.get_king());
  assert_eq!(0b11, pieces.all());

  // Try removal
  pieces.remove(0);
  assert_eq!(NO_PIECE, pieces.get(0));
  assert_eq!(WHITE_ROOK, pieces.get(1));
  assert_eq!(None, pieces.white.get_king());
  assert_eq!(None, pieces.black.get_king());
  assert_eq!(0b10, pieces.all());
}

#[test]
fn test_piece_mask_iterator() {
  let mut pieces: PieceMasks = PieceMasks::default_white_piece_set();

  assert_eq!(Some((4, PieceType::King)), pieces.next());
  assert_eq!(Some((3, PieceType::Queen)), pieces.next());
  assert_eq!(Some((0, PieceType::Rook)), pieces.next());
  assert_eq!(Some((7, PieceType::Rook)), pieces.next());
  assert_eq!(Some((2, PieceType::Bishop)), pieces.next());
  assert_eq!(Some((5, PieceType::Bishop)), pieces.next());
  assert_eq!(Some((1, PieceType::Knight)), pieces.next());
  assert_eq!(Some((6, PieceType::Knight)), pieces.next());
  assert_eq!(Some((8, PieceType::Pawn)), pieces.next());
  assert_eq!(Some((9, PieceType::Pawn)), pieces.next());
  assert_eq!(Some((10, PieceType::Pawn)), pieces.next());
  assert_eq!(Some((11, PieceType::Pawn)), pieces.next());
  assert_eq!(Some((12, PieceType::Pawn)), pieces.next());
  assert_eq!(Some((13, PieceType::Pawn)), pieces.next());
  assert_eq!(Some((14, PieceType::Pawn)), pieces.next());
  assert_eq!(Some((15, PieceType::Pawn)), pieces.next());
  assert_eq!(None, pieces.next());

  let mut pieces: PieceMasks = PieceMasks::new();

  assert_eq!(None, pieces.next());
  assert_eq!(None, pieces.next());
  assert_eq!(None, pieces.next());

  let mut pieces: PieceMasks = PieceMasks::default_black_piece_set();
  assert_eq!(Some((60, PieceType::King)), pieces.next());
  assert_eq!(Some((59, PieceType::Queen)), pieces.next());
  assert_eq!(Some((56, PieceType::Rook)), pieces.next());
  assert_eq!(Some((63, PieceType::Rook)), pieces.next());
}

#[test]
fn test_piece_set() {
  let piece_set: PieceSet = PieceSet::default();
  let piece_set_from_fen: PieceSet = PieceSet::from_fen(START_POSITION_FEN);

  assert_eq!(piece_set, piece_set_from_fen);
}
