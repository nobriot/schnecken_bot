use super::handle::GameHandle;
use super::message::GameMessage;
use chess::engine::Engine;
use chess::model::game_state::START_POSITION_FEN;
use chess::model::moves::Move;
use lichess::api::LichessApi;
use lichess::types::Color;
use log::*;
use rand::Rng;
use std::sync::{mpsc, Arc};
use tokio::runtime::Handle;

static MESSAGE_HAVE_TO_LEAVE: &str = "Sorry, I have to leave. I'll resign now!";
static BOT_VERSION: &'static str = env!("CARGO_PKG_VERSION");
static BOT_NAME: &'static str = env!("CARGO_PKG_NAME");

pub struct Game {
  /// Channel to receive messages from the bot or whoever is controlling the
  /// game
  rx:        mpsc::Receiver<GameMessage>,
  /// Lichess API instance to interact with the server
  api:       LichessApi,
  /// Start FEN
  start_fen: String,
  /// Short Lichess Game ID, used in URLs
  id:        String,
  /// Color played by the bot in the ongoing game
  color:     lichess::types::Color,
  // Chess engine instance used to analyze the game
  engine:    Engine,
}

impl Game {
  /// Allocates all the resources for playing a game on Lichess.
  /// returns a thread handle and a channel transmitter to send messages to the
  /// game.
  pub fn new(game: lichess::types::GameStart, api: &LichessApi) -> GameHandle {
    // Communication with the game instance
    let (tx, rx) = mpsc::channel();

    // Create a new engine for playing
    let mut engine = Engine::new(false);
    engine.set_position(START_POSITION_FEN);
    engine.resize_cache_tables(1024); // Use 1024 MB for cache tables.

    let mut bot_game: Game = Game { rx,
                                    api: api.clone(),
                                    start_fen: String::from(START_POSITION_FEN),
                                    id: game.game_id.clone(),
                                    color: game.color,
                                    engine };

    // Start the game loop
    // Spawn blocking as calculating chess moves is CPU intense and would block the
    // async runtime
    let handle = tokio::task::spawn_blocking(move || {
      let rt_handle = Handle::current();
      rt_handle.block_on(bot_game.game_loop());
    });

    // Return a handle to the game
    GameHandle { tx,
                 handle: Arc::new(handle),
                 id: game.game_id.clone() }
  }

  /// Main loop for ongoing games. We dispatch events to the thread taking care
  /// of the game
  pub async fn game_loop(&mut self) {
    loop {
      match self.rx.recv() {
        Ok(GameMessage::Start(game)) => {
          println!("Received a Game Start : {:?}", game);
          self.api
              .write_in_chat(&self.id,
                             format!("Hey there! I am {} v{}", BOT_NAME, BOT_VERSION).as_str())
              .await;
          self.api
              .write_in_spectator_room(&self.id,
                             format!("Hey there! I am {} v{}", BOT_NAME, BOT_VERSION).as_str())
              .await;
        },
        Ok(GameMessage::Update(game)) => {
          println!("Received a Game Update: {:?}", game);
          self.play(game).await;
        },
        Ok(GameMessage::End(_game)) => {
          println!("Game {} is over", self.id);
          break;
        },
        Ok(GameMessage::Resign) => {
          println!("Resining game {}", &self.id);
          let _ = self.api.resign_game(&self.id).await;
        },
        Ok(GameMessage::Terminate) => {
          println!("Leaving game {}", &self.id);
          self.api.write_in_chat(&self.id, MESSAGE_HAVE_TO_LEAVE).await;
          self.api.write_in_spectator_room(&self.id, MESSAGE_HAVE_TO_LEAVE).await;
          let _ = self.api.resign_game(&self.id).await;
          break;
        },
        Ok(GameMessage::OpponentGone(opt_t)) => {
          if opt_t.is_some() {
            let timeout = opt_t.unwrap();
            info!("Opponent gone. Claiming victory after timeout {}", timeout);
            self.api.claim_victory_after_timeout(timeout, &self.id).await;
          }
        },
        Ok(o) => {
          println!("Received a Game Message : {:?}", o);
        },
        Err(_) => {
          info!("Game channel closed. Exiting game loop for game {}.",
                self.id);
          break;
        },
      }
    }
  }

  /// Plays a move in a game if it is ongoing and our turn
  pub async fn play(&mut self, game: lichess::types::GameState) {
    // Check if we just got a notification that the game is over, and make sure to
    // remove the game from our list if that's the case.
    if game.status != lichess::types::GameStatus::Started {
      // Write a well played / goodbye message
      // self.api.send_end_of_game_message(&game_id_clone, game_state.winner).await;
      return;
    }

    // debug!("Update engine and play if needed for GameState: {:?}", game);
    debug!("Play: game {} {:?} {}", self.id, self.color, self.start_fen);

    // Update whether it is our turn
    let move_list = Move::string_to_vec(game.moves.as_str());
    let is_our_turn = match self.color {
      Color::White => move_list.len() % 2 == 0,
      Color::Black => move_list.len() % 2 == 1,
    };

    if !is_our_turn {
      return;
    }

    debug!("It's our turn on game {}", self.id);

    // Make sure the engine knows the latest move:
    let move_count: usize = self.engine.position.move_count.into();
    if move_list.len() > move_count {
      for m in move_list.iter().skip(move_count) {
        self.engine.apply_move(m.to_string().as_str());
      }
    }

    info!("Trying to find a move for game {}", self.id);
    let (time_left, mut increment_ms) = match self.color {
      Color::White => (game.wtime, game.winc),
      Color::Black => (game.btime, game.binc),
    };

    if increment_ms > 60_000 {
      increment_ms = 60_000;
    }

    // Play as quick as possible if we have less than 10 seconds left
    let suggested_time_ms =
      if time_left < 10_000 { 100 } else { (time_left / 90) + increment_ms * 10 / 9 };

    info!("Using {} ms to find a move for position {}",
          suggested_time_ms,
          self.engine.position.to_fen());

    self.engine.options.max_search_time = suggested_time_ms;
    // let _ = self.engine.go_async().await;
    let engine_clone = self.engine.clone();
    let engine_search_handle = tokio::task::spawn_blocking(move || engine_clone.go());
    let _ = engine_search_handle.await.unwrap();
    // while !engine_handle.is_finished() {}

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

    let move_index = rand::thread_rng().gen_range(0..cutoff);
    let mv = analysis.get(move_index).variation.get_first_move().unwrap();
    let eval = analysis.get(move_index).eval;
    info!("Playing Line {} ({})  as {:?} for GameID {} - eval: {}",
          move_index, mv, self.color, self.id, eval);

    // Make the move
    self.api.make_move(&self.id, &mv.to_string(), false).await;
  }
}
