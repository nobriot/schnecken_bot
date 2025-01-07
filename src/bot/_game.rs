use super::bot_control::BotControlState;
use std::thread;
use chess::engine::config::play_style::PlayStyle;
use chess::engine::Engine;
use chess::model::game_state::START_POSITION_FEN;
use lichess::api::LichessApi;
use log::info;
use std::sync::mpsc;
use std::sync::mpsc::channel;
use super::game_message::GameMessage;

#[derive(Clone)]
pub struct BotGame {
  rx:         mpsc::Receiver<GameMessage>,
  api:        LichessApi,
  /// Color played by the bot in the ongoing game
  color:      lichess::types::Color,
  /// Start FEN
  start_fen:  String,
  /// Short Lichess Game ID, used in URLs
  id:         String,
  /// Whether it got started, ever
  has_moved:  bool,
  /// If it is our turn or not
  is_my_turn: bool,
  /// list of moves with algebraic notation: "e2e4 e7e5 d2d4 f8b4 .."
  move_list:  String,
  rated:      bool,
  clock:      GameClock,
  // Chess engine instance used to analyze the game
  engine:     Engine,
}


impl BotGame {
  /// Allocates all the resources for playing a game on Lichess.
  /// returns a thread handle and a channel transmitter to send messages to the
  /// game.
  pub fn new(game: lichess::types::GameStart,
             api: &LichessApi)
             -> (handle, mpsc::Sender<GameMessages>) {

    let (tx, rx) = mpsc::channel();
    let mut engine = Engine::new(false);
    engine.set_position(START_POSITION_FEN);
    engine.resize_cache_tables(1024); // Use 1024 MB for cache tables.

    let mut bot_game: BotGame = BotGame { rx,
            api: api.clone(),
                                          color: game.color,
                                          start_fen: String::from(START_POSITION_FEN),
                                          id: game.game_id,
                                          has_moved: game.has_moved,
                                          is_my_turn: game.is_my_turn,
                                          move_list: game.last_move.unwrap_or_default(),
                                          rated: game.rated,
                                          clock: GameClock { white_time:      game.seconds_left,
                                                             white_increment: 0,
                                                             black_time:      game.seconds_left,
                                                             black_increment: 0, },
                                          engine };

    let handle = tokio::spawn(async move { bot_game.run().await });

    (handle, tx)
  }

  pub async fn run(&mut self) {
    loop {
      match self.rx.recv() {
        Ok(GameMessage::Start(game)) => {
          println!("Received a Game Start : {:?}", game);
        },
        Ok(GameMessage::Update(game)) => {
          println!("Received a Game Start : {:?}", game);
        },
        Ok(GameMessage::Terminate) => {
          break;
        },
        Err(_) => {
          info!("Game channel closed. Exiting game loop.");
          self.
          break;
        },
      }
    }
  }

  /// Get the Lichess ID of a game
  pub fn get_id(&self) -> String {
    self.id.clone()
  }

  /// Looks at the game outcome and sends a message depending on who won
  ///
  /// ### Arguments
  ///
  /// * `game_id`:  Identifier of the game that just finished
  /// * `winner`:   Option of a color indicating who won.
  pub fn send_end_of_game_message(&self, game_id: &str, winner: Option<lichess::types::Color>) {
    let message = match (self.color, winner) {
      // This is a draw:
      (_, None) => "Good game",
      (me, Some(w)) => {
        if me == w {
          "Always a pleasure to win =)"
        } else {
          "Well played ! I'll get my revanche next time ;-)"
        }
      },
    };

    // Write a goodbye message
    let api_clone = self.api.clone();
    let game_id_clone = String::from(game_id);
    tokio::spawn(async move { api_clone.write_in_chat(&game_id_clone, message).await });
  }
}
