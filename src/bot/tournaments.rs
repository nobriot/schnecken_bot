//! Rules to join and play the arena tournaments of the Lichess team
//! [daily-bot-tournaments](https://lichess.org/team/daily-bot-tournaments).
//!
//! We join all its standard rated arenas allowing bots, except classical ones
//! (games take too long). While one of them is
//! running, we do not send challenges and decline challenges from other bots.
use super::matchmaking::speed_for_clock;
use lichess::types::{ARENA_STATUS_FINISHED, ArenaTournament, Clock, GameStart, Speed, VariantKey};
use rand::seq::IndexedRandom;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// -----------------------------------------------------------------------------
// Constants

/// Lichess team whose arena tournaments we join
pub const TEAM_ID: &str = "daily-bot-tournaments";
/// Interval between two lookups of the team arena tournaments
pub const ARENA_LOOKUP_INTERVAL: Duration = Duration::from_secs(10 * 60);
/// Interval between two "pair me asap" join requests while a tournament is
/// running. Lichess forgets them after one minute.
pub const PAIR_ME_REFRESH_INTERVAL: Duration = Duration::from_secs(45);
/// We stop sending challenges this long before a tournament starts, so that we
/// are not busy with another game when it starts
pub const PRE_TOURNAMENT_QUIET: Duration = Duration::from_secs(15 * 60);
/// We go berserk in tournament games when our rating is at least this much
/// higher than our opponent's
pub const BERSERK_MIN_RATING_ADVANTAGE: usize = 100;

// -----------------------------------------------------------------------------
// Types

/// An arena tournament we joined
#[derive(Debug)]
pub struct JoinedArena {
  pub arena: ArenaTournament,
  /// Team we play for, for team battles
  pub team:  Option<String>,
}

// -----------------------------------------------------------------------------
// Functions

/// Current time in milliseconds since the Unix epoch, like Lichess timestamps
pub fn now_ms() -> u64 {
  SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64)
}

/// Lichess speed category of an arena tournament, from its clock
pub fn arena_speed(arena: &ArenaTournament) -> Speed {
  speed_for_clock(&Clock { initial:   arena.clock.limit as i32,
                           increment: arena.clock.increment as i32,
                           totaltime: None, })
}

/// Checks if we should join an arena tournament: standard, rated, open to
/// bots, not finished, and bullet, blitz or rapid (classical games take too
/// long).
pub fn should_join(arena: &ArenaTournament) -> bool {
  arena.variant.key == VariantKey::Standard
  && arena.rated
  && arena.bots_allowed != Some(false)
  && arena.status != ARENA_STATUS_FINISHED
  && arena_speed(arena) != Speed::Classical
}

/// Checks if an arena tournament is running, i.e. Lichess can pair us
pub fn is_running(arena: &ArenaTournament, now_ms: u64) -> bool {
  arena.starts_at <= now_ms && now_ms < arena.finishes_at
}

/// Checks if an arena tournament is running or starts soon, in which case we
/// keep ourselves available for it
pub fn is_active(arena: &ArenaTournament, now_ms: u64) -> bool {
  let quiet_from = arena.starts_at.saturating_sub(PRE_TOURNAMENT_QUIET.as_millis() as u64);
  quiet_from <= now_ms && now_ms < arena.finishes_at
}

/// Checks if an arena tournament is over
pub fn is_over(arena: &ArenaTournament, now_ms: u64) -> bool {
  arena.finishes_at <= now_ms
}

/// Checks if we should go berserk in a game: tournament game, not started yet,
/// against an opponent rated well below us.
pub fn should_berserk(game: &GameStart) -> bool {
  game.tournament_id.is_some()
  && !game.has_moved
  && game.rating
         .is_some_and(|rating| game.opponent.rating + BERSERK_MIN_RATING_ADVANTAGE <= rating)
}

/// Picks the team we play for in a team battle: a random one among the
/// competing teams we are a member of.
///
/// ### Arguments
///
/// * `battle_teams` IDs of the teams competing in the team battle
/// * `our_teams`    IDs of the teams we are a member of
///
/// ### Returns
///
/// The team ID to join the tournament with, None if we are not a member of
/// any of the competing teams.
pub fn pick_battle_team(battle_teams: &[String], our_teams: &[String]) -> Option<String> {
  let candidates: Vec<&String> =
    battle_teams.iter()
                .filter(|team| our_teams.iter().any(|ours| ours.eq_ignore_ascii_case(team)))
                .collect();
  candidates.choose(&mut rand::rng()).map(|team| team.to_string())
}
