use super::engine::configure_engine;
use super::events::{self, EvalEvent, GameEvents};
use super::handle::GameHandle;
use super::message::GameMessage;
use chess::engine::Engine;
use chess::model::game_state::START_POSITION_FEN;
use chess::model::moves::Move;
use lichess::api::LichessApi;
use lichess::types::Color;
use log::*;
use rand::Rng;
use std::sync::{Arc, mpsc};
use std::time::Duration;
use tokio::runtime::Handle;

static MESSAGE_HAVE_TO_LEAVE: &str = "Sorry, I have to leave. I'll resign now!";

pub struct Game {
  /// Channel to receive messages from the bot or whoever is controlling the
  /// game
  rx:              mpsc::Receiver<GameMessage>,
  /// Lichess API instance to interact with the server
  api:             LichessApi,
  /// Start FEN
  #[allow(dead_code)]
  _start_fen:      String,
  /// Short Lichess Game ID, used in URLs
  id:              String,
  /// Color played by the bot in the ongoing game
  color:           lichess::types::Color,
  // Chess engine instance used to analyze the game
  engine:          Engine,
  /// Opponent name, used in chat messages
  opponent_name:   String,
  /// True if the opponent is a bot
  opponent_is_bot: bool,
  /// What already happened during the game
  events:          GameEvents,
}

impl Game {
  /// Allocates all the resources for playing a game on Lichess.
  /// returns a thread handle and a channel transmitter to send messages to the
  /// game.
  #[allow(clippy::new_ret_no_self)]
  pub fn new(game: lichess::types::GameStart,
             api: &LichessApi,
             engine_config: &crate::config::EngineConfig)
             -> GameHandle {
    println!("Game::new with game data: {:?}", game);

    // Communication with the game instance
    let (tx, rx) = mpsc::channel();

    // Create a new engine for playing
    let engine = configure_engine(&game, engine_config);
    let opponent_name = events::display_name(&game.opponent.username).to_string();
    let opponent_is_bot = game.opponent_is_bot();

    let mut bot_game: Game = Game { rx,
                                    api: api.clone(),
                                    _start_fen: game.fen
                                                    .unwrap_or(String::from(START_POSITION_FEN)),
                                    id: game.game_id.clone(),
                                    color: game.color,
                                    engine,
                                    opponent_name,
                                    opponent_is_bot,
                                    events: GameEvents::default() };

    // Start the game loop
    // Spawn blocking as calculating chess moves is CPU intense and would block
    // the async runtime
    let handle = tokio::task::spawn_blocking(move || {
      let rt_handle = Handle::current();
      rt_handle.block_on(bot_game.game_loop());
    });

    // Return a handle to the game
    GameHandle { tx,
                 handle: Arc::new(handle),
                 id: game.game_id.clone(),
                 opponent_id: game.opponent.id.clone() }
  }

  /// Writes a message in the game chat and the spectator room
  async fn say(&self, message: &str) {
    self.api.write_in_chat(&self.id, message).await;
    self.api.write_in_spectator_room(&self.id, message).await;
  }

  /// Writes a couple of message
  async fn start_of_game_announcement(&self) {
    self.say(&events::greeting(&self.opponent_name)).await;
  }

  /// Says goodbye depending on the result, only once per game
  async fn end_of_game_announcement(&mut self, game: &lichess::types::GameState) {
    use lichess::types::GameStatus;
    // Nothing to say if the game never really happened
    if matches!(game.status, GameStatus::Aborted | GameStatus::NoStart) {
      return;
    }
    if self.events.on_game_over() {
      self.say(events::game_over_message(game.winner, self.color)).await;
    }
  }

  /// Returns true if the opponent has made at least one move based on the
  /// moves string and our color.
  fn opponent_has_moved(moves: &str, color: Color) -> bool {
    let move_count = if moves.is_empty() { 0 } else { moves.split_whitespace().count() };
    match color {
      Color::White => move_count >= 2, // we played, they replied
      Color::Black => move_count >= 1, // they played first
    }
  }

  /// Main loop for ongoing games. We dispatch events to the thread taking care
  /// of the game
  pub async fn game_loop(&mut self) {
    // Annonce greetings on the game
    self.start_of_game_announcement().await;

    let mut opponent_moved = false;
    const ABORT_TIMEOUT: Duration = Duration::from_secs(30);

    loop {
      let msg = if opponent_moved {
        self.rx.recv().map_err(|_| ())
      } else {
        self.rx.recv_timeout(ABORT_TIMEOUT).map_err(|_| ())
      };

      match msg {
        Ok(GameMessage::Start(game)) => {
          debug!("Received a Game Start : {:?}", game);
        },
        Ok(GameMessage::Update(game)) => {
          debug!("Received a Game Update: {:?}", game);
          if !opponent_moved && Self::opponent_has_moved(&game.moves, self.color) {
            opponent_moved = true;
          }
          self.play(game).await;
        },
        Ok(GameMessage::End(game)) => {
          info!("Game {} is over", self.id);
          self.end_of_game_announcement(&game).await;
          break;
        },
        Ok(GameMessage::Resign) => {
          info!("Resining game {}", self.id);
          let _ = self.api.resign_game(&self.id).await;
        },
        Ok(GameMessage::Terminate) => {
          info!("Leaving game {}", self.id);
          // We already say goodbye here, no message at the end of the game
          self.events.on_game_over();
          self.say(MESSAGE_HAVE_TO_LEAVE).await;
          let _ = self.api.resign_game(&self.id).await;
        },
        Ok(GameMessage::OpponentGone(opt_t)) => {
          if let Some(timeout) = opt_t {
            info!("Opponent gone. Claiming victory after timeout {}", timeout);
            self.api.claim_victory_after_timeout(timeout, &self.id).await;
          }
        },
        Ok(GameMessage::ChatMessage(m)) => {
          info!("Message on the chat: {:?}", m);
        },
        Ok(o) => {
          info!("Received a Game Message : {:?}", o);
        },
        Err(_) => {
          if !opponent_moved {
            info!("Opponent did not move in {}s, aborting game {}",
                  ABORT_TIMEOUT.as_secs(),
                  self.id);
            let _ = self.api.abort_game(&self.id).await;
            break;
          }
          debug!("Game channel closed. Exiting game loop for game {}.", self.id);
          break;
        },
      }
    }
  }

  /// Plays a move in a game if it is ongoing and our turn
  pub async fn play(&mut self, game: lichess::types::GameState) {
    // Check if we just got a notification that the game is over, and make sure
    // to remove the game from our list if that's the case.
    if game.status != lichess::types::GameStatus::Started {
      // Write a good game / well played / goodbye message
      self.end_of_game_announcement(&game).await;
      return;
    }

    // debug!("Update engine and play if needed for GameState: {:?}", game);
    // debug!("Play: game {} {:?} {}", self.id, self.color, self.start_fen);

    // Update whether it is our turn
    let move_list = Move::string_to_vec(game.moves.as_str());
    if self.events.on_moves_played(move_list.len()) {
      self.say(events::HAVE_FUN).await;
    }

    let is_our_turn = match self.color {
      Color::White => move_list.len().is_multiple_of(2),
      Color::Black => !move_list.len().is_multiple_of(2),
    };

    if !is_our_turn {
      return;
    }

    info!("It's our turn on game {}", self.id);

    // Make sure the engine knows the latest move:
    let move_count: usize = self.engine.position.move_count.into();
    if move_list.len() > move_count {
      for m in move_list.iter().skip(move_count) {
        self.engine.apply_move(m.to_string().as_str());
      }
    }

    debug!("Trying to find a move for game {}", self.id);
    let (time_left, mut increment_ms) = match self.color {
      Color::White => (game.wtime, game.winc),
      Color::Black => (game.btime, game.binc),
    };

    if increment_ms > 60_000 {
      increment_ms = 60_000;
    }

    // Play as quick as possible if we have less than 10 seconds left
    let suggested_time_ms =
      if time_left < 10_000 { 200 } else { (time_left / 90) + increment_ms * 10 / 9 };

    info!("Using {}ms (max depth {}) to find a move for position {}",
          suggested_time_ms,
          self.engine.options.max_depth,
          self.engine.position.to_fen());

    self.engine.options.max_search_time = suggested_time_ms;
    self.engine.go();

    // Select randomly one of the good moves.
    let analysis = self.engine.get_analysis();
    let best_eval = analysis.get_eval().unwrap_or(f32::NAN);
    let mut cutoff = 1;
    // We are in trouble if the engine could not find a move
    if analysis.is_empty() {
      error!("Empty result from the engine.");
      self.api.write_in_spectator_room(&self.id, "Error: Could not find a move to play.").await;
      self.api.write_in_chat(&self.id, "Error: Could not find a move to play.").await;
      let _ = self.api.resign_game(&self.id).await;
      return;
    }

    let eval_for_us = match self.color {
      Color::White => best_eval,
      Color::Black => -best_eval,
    };
    // Move number, as in the game notation, of the move we are about to play
    let move_number = move_list.len() / 2 + 1;
    for event in self.events.after_search(eval_for_us, self.opponent_is_bot) {
      info!("Game event {event:?} for game {} (eval {best_eval})", self.id);
      if let Some(message) = event.message(move_number, eval_for_us) {
        self.say(&message).await;
      }
      if event == EvalEvent::Resign {
        if events::is_forced_mate_in_one_against(&self.engine.position.board) {
          info!("Opponent mates in one whatever we play, resigning game {}", self.id);
          // We say "Well played!" now, no need to say it again at the end
          self.events.on_game_over();
          self.say(events::WELL_PLAYED).await;
          let _ = self.api.resign_game(&self.id).await;
          return;
        }
        debug!("Eval says mate in one against us, but it is not forced. Playing on.");
      }
    }

    while analysis.len() > cutoff {
      if analysis.get(cutoff).eval.is_nan() {
        break;
      }
      if (best_eval - analysis.get(cutoff).eval).abs() > 0.015 {
        break;
      } else {
        cutoff += 1;
      }
    }

    let move_index = rand::rng().random_range(0..cutoff);
    let mv = analysis.get(move_index).variation.get_first_move().unwrap();
    let eval = analysis.get(move_index).eval;
    info!("Playing Line {} ({})  as {:?} for GameID {} - eval: {}",
          move_index, mv, self.color, self.id, eval);

    // Make the move
    self.api.make_move(&self.id, &mv.to_string(), false).await;
  }
}
