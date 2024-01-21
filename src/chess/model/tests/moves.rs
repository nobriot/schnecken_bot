use crate::model::moves::*;

#[test]
fn square_to_string_test() {
  assert_eq!("a1", square_to_string(0));
  assert_eq!("b1", square_to_string(1));
  assert_eq!("c1", square_to_string(2));
  assert_eq!("d1", square_to_string(3));
  assert_eq!("e1", square_to_string(4));
  assert_eq!("f1", square_to_string(5));
  assert_eq!("g1", square_to_string(6));
  assert_eq!("h1", square_to_string(7));
  assert_eq!("a2", square_to_string(8));
  assert_eq!("b2", square_to_string(9));
  assert_eq!("c2", square_to_string(10));
  assert_eq!("d2", square_to_string(11));
  assert_eq!("e2", square_to_string(12));
  assert_eq!("f2", square_to_string(13));
  assert_eq!("g2", square_to_string(14));
  assert_eq!("h2", square_to_string(15));
  assert_eq!("a8", square_to_string(56));
  assert_eq!("b8", square_to_string(57));
  assert_eq!("c8", square_to_string(58));
  assert_eq!("d8", square_to_string(59));
  assert_eq!("e8", square_to_string(60));
  assert_eq!("f8", square_to_string(61));
  assert_eq!("g8", square_to_string(62));
  assert_eq!("h8", square_to_string(63));
}

#[test]
fn string_to_square_test() {
  assert_eq!(0, string_to_square("a1"));
  assert_eq!(1, string_to_square("b1"));
  assert_eq!(2, string_to_square("c1"));
  assert_eq!(3, string_to_square("d1"));
  assert_eq!(4, string_to_square("e1"));
  assert_eq!(5, string_to_square("f1"));
  assert_eq!(6, string_to_square("g1"));
  assert_eq!(7, string_to_square("h1"));
  assert_eq!(8, string_to_square("a2"));
  assert_eq!(9, string_to_square("b2"));
  assert_eq!(10, string_to_square("c2"));
  assert_eq!(11, string_to_square("d2"));
  assert_eq!(12, string_to_square("e2"));
  assert_eq!(13, string_to_square("f2"));
  assert_eq!(14, string_to_square("g2"));
  assert_eq!(15, string_to_square("h2"));
  assert_eq!(56, string_to_square("a8"));
  assert_eq!(57, string_to_square("b8"));
  assert_eq!(58, string_to_square("c8"));
  assert_eq!(59, string_to_square("d8"));
  assert_eq!(60, string_to_square("e8"));
  assert_eq!(61, string_to_square("f8"));
  assert_eq!(62, string_to_square("g8"));
  assert_eq!(63, string_to_square("h8"));
}

#[test]
fn move_to_string() {
  let m = mv!(0, 1, Promotion::WhiteBishop);
  assert_eq!("a1b1B", m.to_string());

  let m = mv!(63, 1);
  assert_eq!("h8b1", m.to_string());

  let m = mv!(9, 1, Promotion::BlackQueen);
  assert_eq!("b2b1q", m.to_string());
  assert_eq!(m, Move::from_string(m.to_string().as_str()));
}

#[test]
fn vec_to_string() {
  let mut vec = Vec::new();
  vec.push(mv!(0, 1, Promotion::WhiteBishop));
  vec.push(mv!(63, 0));

  assert_eq!("a1b1B h8a1", Move::vec_to_string(&vec));
}

#[test]
fn string_to_vec() {
  let moves = "a1b1B h8a1";
  let vec = Move::string_to_vec(moves);
  let m0 = mv!(0, 1, Promotion::WhiteBishop);
  let m1 = mv!(63, 0);
  assert_eq!(vec[0], m0);
  assert_eq!(vec[1], m1);
}
