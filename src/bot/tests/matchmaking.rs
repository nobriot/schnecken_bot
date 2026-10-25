use crate::bot::matchmaking::*;
use lichess::types::{Clock, PerfRating, Speed, User, UserPerfs};

/// Creates a user with the same rating in bullet, blitz and rapid
fn user(id: &str, title: Option<&str>, rating: u32) -> User {
  user_with_ratings(id, title, Some(rating), Some(rating), Some(rating))
}

fn user_with_ratings(id: &str,
                     title: Option<&str>,
                     bullet: Option<u32>,
                     blitz: Option<u32>,
                     rapid: Option<u32>)
                     -> User {
  let perf = |rating: Option<u32>| {
    rating.map(|rating| PerfRating { games: 100,
                                     rating,
                                     prov: false })
  };
  User { id:            id.to_string(),
         username:      id.to_string(),
         title:         title.map(String::from),
         perfs:         UserPerfs { bullet:    perf(bullet),
                                    blitz:     perf(blitz),
                                    rapid:     perf(rapid),
                                    classical: None, },
         disabled:      false,
         tos_violation: false, }
}

fn clock(initial: i32, increment: i32) -> Clock {
  Clock { initial,
          increment,
          totaltime: None }
}

#[test]
fn test_speed_for_clock() {
  assert_eq!(Speed::Bullet, speed_for_clock(&clock(60, 0)));
  assert_eq!(Speed::Bullet, speed_for_clock(&clock(120, 1)));
  assert_eq!(Speed::Blitz, speed_for_clock(&clock(180, 0)));
  assert_eq!(Speed::Blitz, speed_for_clock(&clock(180, 2)));
  assert_eq!(Speed::Rapid, speed_for_clock(&clock(600, 0)));
  assert_eq!(Speed::Classical, speed_for_clock(&clock(1800, 0)));
}

#[test]
fn test_closest_rating_difference() {
  let us = user_with_ratings("schnecken_bot", Some("BOT"), Some(1600), Some(1500), Some(1550));
  let them = user_with_ratings("other_bot", Some("BOT"), Some(2000), Some(1550), None);
  assert_eq!(Some(50), closest_rating_difference(&us, &them));

  let no_common_rating = user_with_ratings("new_bot", Some("BOT"), None, None, None);
  assert_eq!(None, closest_rating_difference(&us, &no_common_rating));
}

#[test]
fn test_should_follow() {
  let us = user("schnecken_bot", Some("BOT"), 1500);
  // Many games and similar rating
  assert!(should_follow(&us, &user("other_bot", Some("BOT"), 1650), 101));
  assert!(should_follow(&us, &user("other_bot", Some("BOT"), 1300), 500));
  // Not enough games
  assert!(!should_follow(&us, &user("other_bot", Some("BOT"), 1500), 100));
  // Rating too different
  assert!(!should_follow(&us, &user("other_bot", Some("BOT"), 1701), 500));
}

#[test]
fn test_should_unfollow() {
  let us = user("schnecken_bot", Some("BOT"), 1500);
  // Bots too far from our rating are unfollowed
  assert!(should_unfollow(&us, &user("strong_bot", Some("BOT"), 2000)));
  assert!(should_unfollow(&us, &user("weak_bot", Some("BOT"), 1000)));
  // Bots within 400 are kept
  assert!(!should_unfollow(&us, &user("similar_bot", Some("BOT"), 1900)));
  // A bot close to us in one time control only is kept
  let blitz_only = user_with_ratings("blitz_bot", Some("BOT"), Some(2500), Some(1550), Some(2500));
  assert!(!should_unfollow(&us, &blitz_only));
  // Bots without rating in common are kept
  assert!(!should_unfollow(&us, &user_with_ratings("new_bot", Some("BOT"), None, None, None)));
  // Never unfollow humans
  assert!(!should_unfollow(&us, &user("strong_human", None, 2500)));
  assert!(!should_unfollow(&us, &user("titled_human", Some("GM"), 2800)));
  // Never unfollow the original author, even if this account becomes a bot
  assert!(!should_unfollow(&us, &user("SchnellSchnecke", Some("BOT"), 2500)));
  assert!(is_always_followed("schnellschnecke"));
}

#[test]
fn test_is_challengeable_bot() {
  let us = user("schnecken_bot", Some("BOT"), 1500);
  assert!(is_challengeable_bot(&us, &user("other_bot", Some("BOT"), 1500)));
  // Never challenge humans
  assert!(!is_challengeable_bot(&us, &user("human", None, 1500)));
  assert!(!is_challengeable_bot(&us, &user("titled_human", Some("IM"), 1500)));
  // Not ourselves
  assert!(!is_challengeable_bot(&us, &user("Schnecken_Bot", Some("BOT"), 1500)));
  // Not closed accounts
  let mut closed = user("closed_bot", Some("BOT"), 1500);
  closed.disabled = true;
  assert!(!is_challengeable_bot(&us, &closed));
}

#[test]
fn test_challenge_order() {
  let candidates = vec![user("far", Some("BOT"), 2000),
                        user("close", Some("BOT"), 1510),
                        user_with_ratings("unrated", Some("BOT"), None, None, None),
                        user("closer", Some("BOT"), 1505),
                        user("medium", Some("BOT"), 1600),];
  let mut sorted = candidates.clone();
  sort_by_rating_difference(&mut sorted, 1500, Speed::Blitz);
  let ids: Vec<&str> = sorted.iter().map(|u| u.id.as_str()).collect();
  assert_eq!(vec!["closer", "close", "medium", "far", "unrated"], ids);

  // The first pick is random among the 3 closest, then all the others follow
  for _ in 0..20 {
    let ordered = challenge_order(candidates.clone(), 1500, Speed::Blitz);
    assert_eq!(candidates.len(), ordered.len());
    assert!(["closer", "close", "medium"].contains(&ordered[0].id.as_str()));
    assert_eq!("far", ordered[3].id);
    assert_eq!("unrated", ordered[4].id);
  }
}

#[test]
fn test_select_new_opponents() {
  let us = user("schnecken_bot", Some("BOT"), 1500);
  let online_bots = vec![user("schnecken_bot", Some("BOT"), 1500),
                         user("followed_bot", Some("BOT"), 1500),
                         user("good_match", Some("BOT"), 1650),
                         user("too_strong", Some("BOT"), 1750),
                         user("too_weak", Some("BOT"), 1200),
                         user("not_a_bot", None, 1500),
                         user_with_ratings("no_blitz", Some("BOT"), Some(1500), None, Some(1500)),];
  let following = vec![String::from("followed_bot")];
  let selected = select_new_opponents(&us, online_bots, &following, Speed::Blitz);
  let ids: Vec<&str> = selected.iter().map(|u| u.id.as_str()).collect();
  assert_eq!(vec!["good_match"], ids);
}
