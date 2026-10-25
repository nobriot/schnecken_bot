//! Rules to pick opponents to challenge, and which users to follow/unfollow.
//!
//! We challenge bots we follow, picking a similar rating whenever possible.
//! If none of them is available, we look for new opponents in the list of
//! online bots. We never challenge humans.
use lichess::types::{Clock, Speed, User};
use rand::Rng;

// -----------------------------------------------------------------------------
// Constants

/// Users we always follow, and never unfollow (the original bot author)
pub const ALWAYS_FOLLOW: &[&str] = &["schnellschnecke"];
/// Minimum number of games played against an opponent before we follow them
pub const FOLLOW_MIN_GAMES: u64 = 100;
/// Maximum rating difference with an opponent before we follow them
pub const FOLLOW_MAX_RATING_DIFFERENCE: u32 = 200;
/// Rating difference above which we unfollow a bot
pub const UNFOLLOW_MIN_RATING_DIFFERENCE: u32 = 400;
/// Maximum rating difference for new opponents found in the online bots list
pub const NEW_OPPONENT_MAX_RATING_DIFFERENCE: u32 = 200;
/// Number of closest-rated candidates among which we pick randomly, so that
/// we do not always challenge the same opponent
const CANDIDATES_TO_PICK_FROM: usize = 3;

// -----------------------------------------------------------------------------
// Functions

/// Returns true if this user must always be followed
pub fn is_always_followed(user_id: &str) -> bool {
  ALWAYS_FOLLOW.iter().any(|id| id.eq_ignore_ascii_case(user_id))
}

/// Determines the Lichess speed category of a clock, using the Lichess
/// estimated game duration: initial time + 40 * increment
pub fn speed_for_clock(clock: &Clock) -> Speed {
  let estimated_duration = clock.initial + 40 * clock.increment;
  match estimated_duration {
    ..180 => Speed::Bullet,
    180..480 => Speed::Blitz,
    480..1500 => Speed::Rapid,
    _ => Speed::Classical,
  }
}

/// Smallest rating difference between 2 users, across bullet, blitz and rapid
/// (the time controls we play).
///
/// ### Returns
///
/// None if the users have no rating in common.
pub fn closest_rating_difference(us: &User, them: &User) -> Option<u32> {
  [Speed::Bullet, Speed::Blitz, Speed::Rapid].into_iter()
                                             .filter_map(|speed| {
                                               Some(us.rating(speed)?.abs_diff(them.rating(speed)?))
                                             })
                                             .min()
}

/// Checks if we should start following an opponent: we played a lot of games
/// together and have a similar rating.
pub fn should_follow(us: &User, them: &User, games_played: u64) -> bool {
  games_played > FOLLOW_MIN_GAMES
  && closest_rating_difference(us, them).is_some_and(|d| d <= FOLLOW_MAX_RATING_DIFFERENCE)
}

/// Checks if we should unfollow a user. We only unfollow bots, never humans,
/// when their rating is too far from ours in all time controls we play.
pub fn should_unfollow(us: &User, them: &User) -> bool {
  them.is_bot()
  && !is_always_followed(&them.id)
  && closest_rating_difference(us, them).is_some_and(|d| d > UNFOLLOW_MIN_RATING_DIFFERENCE)
}

/// Checks if a user is a bot we could challenge: never humans, never
/// ourselves, never closed accounts.
pub fn is_challengeable_bot(us: &User, them: &User) -> bool {
  them.is_bot() && !them.id.eq_ignore_ascii_case(&us.id) && !them.disabled && !them.tos_violation
}

/// Sorts candidates by rating difference with us for a given speed (closest
/// first). Candidates without a rating for that speed come last.
pub fn sort_by_rating_difference(candidates: &mut [User], our_rating: u32, speed: Speed) {
  candidates.sort_by_key(|user| user.rating(speed).map_or(u32::MAX, |r| r.abs_diff(our_rating)));
}

/// Orders the candidates in which we should try to challenge them: one of the
/// closest-rated candidates first (picked randomly), then the others by rating
/// difference.
pub fn challenge_order(mut candidates: Vec<User>, our_rating: u32, speed: Speed) -> Vec<User> {
  sort_by_rating_difference(&mut candidates, our_rating, speed);
  let pick_range = candidates.len().min(CANDIDATES_TO_PICK_FROM);
  if pick_range > 1 {
    let first = rand::rng().random_range(0..pick_range);
    candidates[..=first].rotate_right(1);
  }
  candidates
}

/// Filters new opponents from the list of online bots: challengeable bots we
/// do not already follow, with a rating close to ours for the speed we want to
/// play.
pub fn select_new_opponents(us: &User,
                            online_bots: Vec<User>,
                            following: &[String],
                            speed: Speed)
                            -> Vec<User> {
  let Some(our_rating) = us.rating(speed) else {
    return Vec::new();
  };

  online_bots.into_iter()
             .filter(|bot| is_challengeable_bot(us, bot))
             .filter(|bot| !following.iter().any(|id| id.eq_ignore_ascii_case(&bot.id)))
             .filter(|bot| {
               bot.rating(speed)
                  .is_some_and(|r| r.abs_diff(our_rating) <= NEW_OPPONENT_MAX_RATING_DIFFERENCE)
             })
             .collect()
}
