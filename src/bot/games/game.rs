use super::engine::configure_engine;
use super::events::{self, ChatCommand, EvalEvent, GameEvents};
use super::handle::GameHandle;
use super::message::GameMessage;
use chess::engine::Engine;
use chess::engine::eval::position::{can_declare_draw, get_epic_mate};
use chess::model::game_state::{GameStatus as EngineGameStatus, START_POSITION_FEN};
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
  /// Our Lichess username, to recognize our messages in the chat
  bot_username:    String,
  /// We already played a move when the game was started (Lichess `hasMoved`),
  /// i.e. we are joining an ongoing game
  we_have_moved:   bool,
}

impl Game {
  /// Allocates all the resources for playing a game on Lichess.
  /// returns a thread handle and a channel transmitter to send messages to the
  /// game.
  #[allow(clippy::new_ret_no_self)]
  pub fn new(game: lichess::types::GameStart,
             api: &LichessApi,
             engine_config: &crate::config::EngineConfig,
             bot_username: &str)
             -> GameHandle {
    println!("Game::new with game data: {:?}", game);

    // Communication with the game instance
    let (tx, rx) = mpsc::channel();

    // Create a new engine for playing
    let engine = configure_engine(&game, engine_config);
    let opponent_name = events::display_name(&game.opponent.username).to_string();
    let opponent_is_bot = game.opponent_is_bot();
    let we_have_moved = game.has_moved;

    let mut bot_game: Game = Game { rx,
                                    api: api.clone(),
                                    _start_fen: game.fen
                                                    .unwrap_or(String::from(START_POSITION_FEN)),
                                    id: game.game_id.clone(),
                                    color: game.color,
                                    engine,
                                    opponent_name,
                                    opponent_is_bot,
                                    events: GameEvents::default(),
                                    bot_username: bot_username.to_string(),
                                    we_have_moved };

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

  /// Writes a comment in the game chat and the spectator room, unless we
  /// already posted the same comment in this game.
  async fn say(&mut self, message: &str) {
    if !self.events.should_say(message) {
      debug!("Not repeating comment in game {}: {message}", self.id);
      return;
    }
    self.api.write_in_chat(&self.id, message).await;
    self.api.write_in_spectator_room(&self.id, message).await;
  }

  /// Greets the opponent, only on new games: when rejoining an ongoing game
  /// (e.g. after a restart), restores what we already said from the chat
  /// instead.
  async fn start_of_game_announcement(&mut self) {
    let already_talked = match self.api.get_game_chat(&self.id).await {
      Ok(chat) => self.events.restore_from_chat(&chat, &self.bot_username),
      Err(_) => {
        // Could not get the chat: only greet if we did not play yet
        warn!("Could not get the chat of game {}, guessing if it is a new game", self.id);
        if self.we_have_moved {
          self.events.mark_ongoing_game();
        }
        self.we_have_moved
      },
    };

    if already_talked {
      info!("Rejoining ongoing game {}, not greeting again", self.id);
      return;
    }
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
        Ok(GameMessage::Full(game)) => {
          debug!("Received the full game: {:?}", game);
          // Start from the initial position: play() replays all the moves, so
          // that the engine knows the game history (threefold repetitions),
          // also when rejoining an ongoing game.
          let initial_fen = match game.initial_fen.as_str() {
            "startpos" | "" => START_POSITION_FEN,
            fen => fen,
          };
          self.engine.set_position(initial_fen);
          if !opponent_moved && Self::opponent_has_moved(&game.state.moves, self.color) {
            opponent_moved = true;
          }
          self.play(game.state).await;
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
          info!("Resigning and leaving game {}", self.id);
          // We already say goodbye here, no message at the end of the game
          self.events.on_game_over();
          self.say(MESSAGE_HAVE_TO_LEAVE).await;
          let _ = self.api.resign_game(&self.id).await;
          break;
        },
        Ok(GameMessage::Leave) => {
          info!("Leaving game {} without resigning", self.id);
          break;
        },
        Ok(GameMessage::OpponentGone(opt_t)) => {
          if let Some(timeout) = opt_t {
            info!("Opponent gone. Claiming victory after timeout {}", timeout);
            self.api.claim_victory_after_timeout(timeout, &self.id).await;
          }
        },
        Ok(GameMessage::ChatMessage(m)) => {
          info!("Message on the chat: {:?}", m);
          self.on_chat_message(m).await;
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

  /// Handles chat commands (`/eval`, `/quiet`, `/chatty`)
  async fn on_chat_message(&mut self, message: lichess::types::ChatMessage) {
    let Some(command) = events::parse_chat_command(&message.text) else {
      return;
    };

    // Turning the commentary on/off is for the opponent and the bot owners,
    // not for any spectator
    if matches!(command, ChatCommand::Quiet | ChatCommand::Chatty)
       && !events::can_toggle_commentary(&message.room, &message.username)
    {
      info!("Ignoring {command:?} from spectator {} in game {}", message.username, self.id);
      return;
    }

    // Answer in the room where the command was sent
    let answer = match command {
      ChatCommand::Eval => Some(self.events.eval_report()),
      ChatCommand::Quiet => self.events.set_quiet().then(|| events::QUIET_ACKNOWLEDGED.to_string()),
      ChatCommand::Chatty => {
        self.events.set_chatty().then(|| events::CHATTY_ACKNOWLEDGED.to_string())
      },
    };
    info!("{} sent {command:?} in game {}", message.username, self.id);
    if let Some(answer) = answer {
      self.api.write_in_chat_room(&self.id, message.room, &answer).await;
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
    if self.events.on_moves_played(move_list.len()) && !self.events.is_quiet() {
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

    // A threefold repetition (or the 50 moves rule) is on the board: claim the
    // draw unless we are better. No need to search, the game is over.
    if can_declare_draw(&self.engine.position) != EngineGameStatus::Ongoing
       && events::should_claim_draw(self.events.last_eval())
    {
      info!("Draw available on the board for game {}, claiming it", self.id);
      if self.api.offer_draw(&self.id).await.is_ok() {
        return;
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
    let mut suggested_time_ms =
      if time_left < 10_000 { 200 } else { (time_left / 90) + increment_ms * 10 / 9 };

    // Against a human short on time and winning: play very fast, try to win on
    // time
    let opponent_time_left = match self.color {
      Color::White => game.btime,
      Color::Black => game.wtime,
    };
    if self.events.update_flagging(!self.opponent_is_bot, time_left, opponent_time_left) {
      info!("Trying to flag the opponent in game {}", self.id);
      if !self.events.is_quiet() {
        self.say(events::ATTEMPTING_TO_FLAG).await;
      }
    }
    if self.events.is_flagging() {
      suggested_time_ms = suggested_time_ms.min(events::flagging_move_time_ms(opponent_time_left));
      info!("Flagging: using {suggested_time_ms}ms (opponent has {opponent_time_left}ms left)");
    }

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
    self.events.set_last_eval(move_number, eval_for_us);
    for event in self.events.after_search(eval_for_us, self.opponent_is_bot) {
      info!("Game event {event:?} for game {} (eval {best_eval})", self.id);
      // Commentary, unless we were asked to be quiet
      if let Some(message) = event.message(move_number, eval_for_us)
         && !self.events.is_quiet()
      {
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

    // If our move gives a threefold repetition (or reaches the 50 moves rule),
    // the engine picked a draw as the best we have: claim it with the move.
    let mut after_our_move = self.engine.position.clone();
    after_our_move.apply_move(&mv);
    let claim_draw = can_declare_draw(&after_our_move) != EngineGameStatus::Ongoing;
    if claim_draw {
      info!("Our move {mv} gives a draw by repetition/50 moves, claiming it");
    }

    // Is our move an epic mate (smothered, or lots of pieces in the net)?
    let epic_mate = get_epic_mate(&after_our_move.board);

    // Make the move
    self.api.make_move(&self.id, &mv.to_string(), claim_draw).await;

    if let Some(epic_mate) = epic_mate {
      info!("Epic mate {epic_mate:?} in game {}", self.id);
      if !self.events.is_quiet() {
        self.say(&events::epic_mate_message(epic_mate)).await;
      }
    }
  }
}
