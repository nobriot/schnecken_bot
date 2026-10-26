use crate::bot::scopes::*;
use crate::bot::tournaments::*;
use lichess::types::{
  ARENA_STATUS_CREATED, ARENA_STATUS_FINISHED, ArenaClock, ArenaTournament, Color, GameStart,
  Player, Variant, VariantKey,
};

const HOUR_MS: u64 = 3600 * 1000;
const MINUTE_MS: u64 = 60 * 1000;

fn arena(starts_at: u64, finishes_at: u64) -> ArenaTournament {
  ArenaTournament { id: "hL7vMrFQ".to_string(),
                    full_name: "Daily Bot Arena".to_string(),
                    rated: true,
                    variant: Variant { key:   VariantKey::Standard,
                                       name:  "Standard".to_string(),
                                       short: "Std".to_string(), },
                    clock: ArenaClock { limit: 180, increment: 0 },
                    starts_at,
                    finishes_at,
                    status: ARENA_STATUS_CREATED,
                    bots_allowed: Some(true),
                    team_battle: None }
}

fn game(tournament_id: Option<&str>,
        has_moved: bool,
        rating: Option<usize>,
        opponent: usize)
        -> GameStart {
  GameStart { game_id: "5IrD6Gzz".to_string(),
              color: Color::White,
              fen: None,
              has_moved,
              is_my_turn: true,
              last_move: None,
              speed: "blitz".to_string(),
              rated: true,
              opponent: Player { id:          "maia1".to_string(),
                                 username:    "BOT maia1".to_string(),
                                 rating:      opponent,
                                 provisional: None,
                                 title:       None, },
              seconds_left: 180,
              winner: None,
              rating,
              tournament_id: tournament_id.map(String::from) }
}

#[test]
fn test_should_join() {
  assert!(should_join(&arena(0, HOUR_MS)));

  let mut a = arena(0, HOUR_MS);
  a.bots_allowed = None;
  assert!(should_join(&a));

  a.bots_allowed = Some(false);
  assert!(!should_join(&a));

  let mut a = arena(0, HOUR_MS);
  a.rated = false;
  assert!(!should_join(&a));

  let mut a = arena(0, HOUR_MS);
  a.variant.key = VariantKey::Chess960;
  assert!(!should_join(&a));

  let mut a = arena(0, HOUR_MS);
  a.status = ARENA_STATUS_FINISHED;
  assert!(!should_join(&a));
}

#[test]
fn test_should_join_time_controls() {
  let with_clock = |limit, increment| {
    let mut a = arena(0, HOUR_MS);
    a.clock = ArenaClock { limit, increment };
    a
  };

  // Bullet, blitz and rapid
  assert!(should_join(&with_clock(60, 0)));
  assert!(should_join(&with_clock(60, 1)));
  assert!(should_join(&with_clock(240, 1)));
  assert!(should_join(&with_clock(300, 6)));
  assert!(should_join(&with_clock(600, 5)));
  // Classical: 25+1, 20+10, 1.5+60, 60+3 (seen in the daily-bot-tournaments
  // team)
  assert!(!should_join(&with_clock(1500, 1)));
  assert!(!should_join(&with_clock(1200, 10)));
  assert!(!should_join(&with_clock(90, 60)));
  assert!(!should_join(&with_clock(3600, 3)));
}

#[test]
fn test_tournament_timing() {
  let start = 10 * HOUR_MS;
  let end = 12 * HOUR_MS;
  let a = arena(start, end);

  // Long before the start
  assert!(!is_active(&a, start - HOUR_MS));
  assert!(!is_running(&a, start - HOUR_MS));
  // Shortly before the start: we stay available, but Lichess cannot pair us
  assert!(is_active(&a, start - 10 * MINUTE_MS));
  assert!(!is_running(&a, start - 10 * MINUTE_MS));
  // During the tournament
  assert!(is_active(&a, start));
  assert!(is_running(&a, start));
  assert!(is_running(&a, end - 1));
  assert!(!is_over(&a, end - 1));
  // After the end
  assert!(!is_active(&a, end));
  assert!(!is_running(&a, end));
  assert!(is_over(&a, end));
}

#[test]
fn test_should_berserk() {
  assert!(should_berserk(&game(Some("t"), false, Some(1600), 1500)));
  assert!(should_berserk(&game(Some("t"), false, Some(1700), 1500)));
  // Not enough rating difference
  assert!(!should_berserk(&game(Some("t"), false, Some(1599), 1500)));
  assert!(!should_berserk(&game(Some("t"), false, Some(1400), 1500)));
  // Not a tournament game
  assert!(!should_berserk(&game(None, false, Some(1700), 1500)));
  // Game already started
  assert!(!should_berserk(&game(Some("t"), true, Some(1700), 1500)));
  // Unknown rating
  assert!(!should_berserk(&game(Some("t"), false, None, 1500)));
}

#[test]
fn test_missing_scopes() {
  let all: Vec<String> = REQUIRED_SCOPES.iter().map(|s| s.scope.to_string()).collect();
  assert!(missing_scopes(&all).is_empty());

  let granted: Vec<String> =
    all.iter().filter(|s| s.as_str() != TOURNAMENT_SCOPE).cloned().collect();
  let missing = missing_scopes(&granted);
  assert_eq!(1, missing.len());
  assert_eq!(TOURNAMENT_SCOPE, missing[0].scope);

  let message = missing_scopes_message(&missing);
  assert!(message.contains("tournament:write (joining tournaments)"));
  // The link to create a new token asks for all the scopes
  for scope in REQUIRED_SCOPES {
    assert!(message.contains(&format!("scopes[]={}", scope.scope)));
  }
}

#[test]
fn test_pick_battle_team() {
  let teams = |ids: &[&str]| ids.iter().map(|id| id.to_string()).collect::<Vec<String>>();
  let battle = teams(&["chess-pro-elnur-uktamov", "daily-bot-tournaments"]);

  // Not a member of any of the teams
  assert_eq!(None, pick_battle_team(&battle, &[]));
  assert_eq!(None, pick_battle_team(&battle, &teams(&["coders"])));

  // A single team in common (case insensitive)
  assert_eq!(Some("daily-bot-tournaments".to_string()),
             pick_battle_team(&battle, &teams(&["coders", "Daily-Bot-Tournaments"])));

  // Several teams in common: any of them
  let ours = teams(&["daily-bot-tournaments", "chess-pro-elnur-uktamov"]);
  for _ in 0..20 {
    let team = pick_battle_team(&battle, &ours).unwrap();
    assert!(battle.contains(&team));
  }
}
