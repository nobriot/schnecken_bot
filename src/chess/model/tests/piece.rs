use crate::model::piece::*;

  #[test]
  fn color_display_test() {
    let mut color = Color::Black;
    assert_eq!("Black", format!("{color}"));
    color = Color::White;
    assert_eq!("White", format!("{color}"));
  }
  #[test]
  fn piece_type_display_test() {
    let mut piece_type = PieceType::King;
    assert_eq!("King", format!("{piece_type}"));
    piece_type = PieceType::Queen;
    assert_eq!("Queen", format!("{piece_type}"));
    piece_type = PieceType::Rook;
    assert_eq!("Rook", format!("{piece_type}"));
    piece_type = PieceType::Bishop;
    assert_eq!("Bishop", format!("{piece_type}"));
    piece_type = PieceType::Knight;
    assert_eq!("Knight", format!("{piece_type}"));
    piece_type = PieceType::Pawn;
    assert_eq!("Pawn", format!("{piece_type}"));
  }

  #[test]
  fn piece_display_test() {
    let mut piece = Piece {
      p_type: PieceType::King,
      color: Color::White,
    };
    assert_eq!("K", format!("{piece}"));
    piece.p_type = PieceType::Queen;
    assert_eq!("Q", format!("{piece}"));
    piece.p_type = PieceType::Rook;
    assert_eq!("R", format!("{piece}"));
    piece.p_type = PieceType::Bishop;
    assert_eq!("B", format!("{piece}"));
    piece.p_type = PieceType::Knight;
    assert_eq!("N", format!("{piece}"));
    piece.p_type = PieceType::Pawn;
    assert_eq!("P", format!("{piece}"));

    piece.color = Color::Black;
    piece.p_type = PieceType::King;
    assert_eq!("k", format!("{piece}"));
    piece.p_type = PieceType::Queen;
    assert_eq!("q", format!("{piece}"));
    piece.p_type = PieceType::Rook;
    assert_eq!("r", format!("{piece}"));
    piece.p_type = PieceType::Bishop;
    assert_eq!("b", format!("{piece}"));
    piece.p_type = PieceType::Knight;
    assert_eq!("n", format!("{piece}"));
    piece.p_type = PieceType::Pawn;
    assert_eq!("p", format!("{piece}"));
  }

  #[test]
  fn piece_type_material_value() {
    let mut piece = Piece {
      p_type: PieceType::King,
      color: Color::White,
    };
    assert_eq!(KING_VALUE, piece.material_value());
    piece.p_type = PieceType::Queen;
    assert_eq!(QUEEN_VALUE, piece.material_value());
    piece.p_type = PieceType::Rook;
    assert_eq!(ROOK_VALUE, piece.material_value());
    piece.p_type = PieceType::Bishop;
    assert_eq!(BISHOP_VALUE, piece.material_value());
    piece.p_type = PieceType::Knight;
    assert_eq!(KNIGHT_VALUE, piece.material_value());
    piece.p_type = PieceType::Pawn;
    assert_eq!(PAWN_VALUE, piece.material_value());
  }

  #[test]
  fn piece_as_char() {
    let mut piece = Piece {
      p_type: PieceType::King,
      color: Color::White,
    };

    assert_eq!('K', piece.as_char());
    piece.p_type = PieceType::Queen;
    assert_eq!('Q', piece.as_char());
    piece.p_type = PieceType::Rook;
    assert_eq!('R', piece.as_char());
    piece.p_type = PieceType::Bishop;
    assert_eq!('B', piece.as_char());
    piece.p_type = PieceType::Knight;
    assert_eq!('N', piece.as_char());
    piece.p_type = PieceType::Pawn;
    assert_eq!('P', piece.as_char());

    piece.color = Color::Black;
    piece.p_type = PieceType::King;

    assert_eq!('k', piece.as_char());
    piece.p_type = PieceType::Queen;
    assert_eq!('q', piece.as_char());
    piece.p_type = PieceType::Rook;
    assert_eq!('r', piece.as_char());
    piece.p_type = PieceType::Bishop;
    assert_eq!('b', piece.as_char());
    piece.p_type = PieceType::Knight;
    assert_eq!('n', piece.as_char());
    piece.p_type = PieceType::Pawn;
    assert_eq!('p', piece.as_char());
  }

  #[test]
  fn piece_as_u8() {
    let mut piece = Piece {
      p_type: PieceType::King,
      color: Color::White,
    };

    assert_eq!(WHITE_KING, piece.as_u8());
    piece.p_type = PieceType::Queen;
    assert_eq!(WHITE_QUEEN, piece.as_u8());
    piece.p_type = PieceType::Rook;
    assert_eq!(WHITE_ROOK, piece.as_u8());
    piece.p_type = PieceType::Bishop;
    assert_eq!(WHITE_BISHOP, piece.as_u8());
    piece.p_type = PieceType::Knight;
    assert_eq!(WHITE_KNIGHT, piece.as_u8());
    piece.p_type = PieceType::Pawn;
    assert_eq!(WHITE_PAWN, piece.as_u8());

    piece.color = Color::Black;
    piece.p_type = PieceType::King;

    assert_eq!(BLACK_KING, piece.as_u8());
    piece.p_type = PieceType::Queen;
    assert_eq!(BLACK_QUEEN, piece.as_u8());
    piece.p_type = PieceType::Rook;
    assert_eq!(BLACK_ROOK, piece.as_u8());
    piece.p_type = PieceType::Bishop;
    assert_eq!(BLACK_BISHOP, piece.as_u8());
    piece.p_type = PieceType::Knight;
    assert_eq!(BLACK_KNIGHT, piece.as_u8());
    piece.p_type = PieceType::Pawn;
    assert_eq!(BLACK_PAWN, piece.as_u8());
  }

  #[test]
  fn full_loop_u8() {
    let mut initial_piece = Piece {
      p_type: PieceType::King,
      color: Color::White,
    };

    assert_eq!(
      initial_piece,
      Piece::from_u8(initial_piece.as_u8()).unwrap()
    );

    initial_piece.p_type = PieceType::Queen;
    assert_eq!(
      initial_piece,
      Piece::from_u8(initial_piece.as_u8()).unwrap()
    );
    initial_piece.p_type = PieceType::Rook;
    assert_eq!(
      initial_piece,
      Piece::from_u8(initial_piece.as_u8()).unwrap()
    );
    initial_piece.p_type = PieceType::Bishop;
    assert_eq!(
      initial_piece,
      Piece::from_u8(initial_piece.as_u8()).unwrap()
    );
    initial_piece.p_type = PieceType::Knight;
    assert_eq!(
      initial_piece,
      Piece::from_u8(initial_piece.as_u8()).unwrap()
    );
    initial_piece.p_type = PieceType::Pawn;
    assert_eq!(
      initial_piece,
      Piece::from_u8(initial_piece.as_u8()).unwrap()
    );

    initial_piece.p_type = PieceType::King;
    initial_piece.color = Color::Black;
    assert_eq!(
      initial_piece,
      Piece::from_u8(initial_piece.as_u8()).unwrap()
    );
    initial_piece.p_type = PieceType::Queen;
    assert_eq!(
      initial_piece,
      Piece::from_u8(initial_piece.as_u8()).unwrap()
    );
    initial_piece.p_type = PieceType::Rook;
    assert_eq!(
      initial_piece,
      Piece::from_u8(initial_piece.as_u8()).unwrap()
    );
    initial_piece.p_type = PieceType::Bishop;
    assert_eq!(
      initial_piece,
      Piece::from_u8(initial_piece.as_u8()).unwrap()
    );
    initial_piece.p_type = PieceType::Knight;
    assert_eq!(
      initial_piece,
      Piece::from_u8(initial_piece.as_u8()).unwrap()
    );
    initial_piece.p_type = PieceType::Pawn;
    assert_eq!(
      initial_piece,
      Piece::from_u8(initial_piece.as_u8()).unwrap()
    );
  }

  #[test]
  fn full_loop_char() {
    let mut initial_piece = Piece {
      p_type: PieceType::King,
      color: Color::White,
    };

    assert_eq!(
      initial_piece,
      Piece::from_char(initial_piece.as_char()).unwrap()
    );

    initial_piece.p_type = PieceType::Queen;
    assert_eq!(
      initial_piece,
      Piece::from_char(initial_piece.as_char()).unwrap()
    );
    initial_piece.p_type = PieceType::Rook;
    assert_eq!(
      initial_piece,
      Piece::from_char(initial_piece.as_char()).unwrap()
    );
    initial_piece.p_type = PieceType::Bishop;
    assert_eq!(
      initial_piece,
      Piece::from_char(initial_piece.as_char()).unwrap()
    );
    initial_piece.p_type = PieceType::Knight;
    assert_eq!(
      initial_piece,
      Piece::from_char(initial_piece.as_char()).unwrap()
    );
    initial_piece.p_type = PieceType::Pawn;
    assert_eq!(
      initial_piece,
      Piece::from_char(initial_piece.as_char()).unwrap()
    );

    initial_piece.p_type = PieceType::King;
    initial_piece.color = Color::Black;
    assert_eq!(
      initial_piece,
      Piece::from_char(initial_piece.as_char()).unwrap()
    );
    initial_piece.p_type = PieceType::Queen;
    assert_eq!(
      initial_piece,
      Piece::from_char(initial_piece.as_char()).unwrap()
    );
    initial_piece.p_type = PieceType::Rook;
    assert_eq!(
      initial_piece,
      Piece::from_char(initial_piece.as_char()).unwrap()
    );
    initial_piece.p_type = PieceType::Bishop;
    assert_eq!(
      initial_piece,
      Piece::from_char(initial_piece.as_char()).unwrap()
    );
    initial_piece.p_type = PieceType::Knight;
    assert_eq!(
      initial_piece,
      Piece::from_char(initial_piece.as_char()).unwrap()
    );
    initial_piece.p_type = PieceType::Pawn;
    assert_eq!(
      initial_piece,
      Piece::from_char(initial_piece.as_char()).unwrap()
    );
  }

