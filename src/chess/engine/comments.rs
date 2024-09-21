// Type definitions

/// High level position evaluation
///
#[derive(Copy, Debug, Clone, Eq, PartialEq, Default)]
pub enum HighLevelEvaluation {
  WhiteIsMating,
  WhiteIsCrushing,
  WhiteIsBetter,
  #[default]
  PositionIsEqual,
  BlackIsBetter,
  BlackIsCrushing,
  BlackIsMating,
}

/// Events/changes that can happen during a game
/// that do deserve commenting on
///
#[derive(Copy, Debug, Clone, Eq, PartialEq)]
pub enum GameEvents {
  /// The opponent delivered an epic mate
  /// Epic mates are : 
  /// Very complicated positions where many pieces (both same side and opponent participating in the mating net)
  /// Forks the king/queen for a win
  /// Smothered mate
  OpponentEpicMate,
  /// We delivered an epic mate
  EpicMate,
  /// The opponent eval just dropped
  OpponentBlunder,
  /// Our eval just dropped
  Blunder,
  /// The opponent is low on time
  OpponentLowOnTime,
  /// We are low on time
  LowOnTime,
  /// The opponent swindled us
  OpponentSwindled,
  /// We swindled the opponent
  Swindled,
  /// Opponent just found a draw to save our butt
  OpponentSavedByADraw,
  /// We just found a draw to save our butt
  SavedByADraw,
}

impl ToString for GameEvents {
  /// Converts a GameEvent into a string
  fn to_string(&self) -> String {
    match self {
      GameEvents::OpponentEpicMate => String::from("Oh my goddess! This is epic in a bad way!!"),
      GameEvents::EpicMate => String::from("Call an ambulance!"),
      GameEvents::OpponentBlunder => String::from("Did you just blunder ??"),
      GameEvents::Blunder => String::from("What did I just do ?!"),
      GameEvents::OpponentLowOnTime => String::from("You in a hurry ??"),
      GameEvents::LowOnTime => String::from("Hmm I'd better speed up..."),
      GameEvents::OpponentSwindled => String::from("Oh damn!"),
      GameEvents::Swindled => String::from("Swindle time!"),
      GameEvents::OpponentSavedByADraw => String::from("I missed that amazing draw!"),
      GameEvents::SavedByADraw => String::from("Sneaky draw my friend! Better luck next time"),
    }
  }
}

/// List of things we could tell to trash talk during a game.
///
#[derive(Copy, Debug, Clone, Eq, PartialEq)]
pub enum TrashTalk {
  /// To another human that has a positive score against our
  /// developer(s) that we will be merciless.
  AvengeMyCreator,
  /// Tell to the other engine that he is just an engine, while we are a BOT 🤖
  YouAreJustAChessEngine,
  /// Tell humans that computers are just better
  YouAreJustHuman,
  /// If we get to flag an opponent,
  TooSlowMyFriend,
  /// If we think the opponent can resign
  YouCanAlreadyResign,
  /// If we are losing but refuse to surrender
  WhatAreYouWaitingToCheckmateMe,
  /// Oh no my queen, of course.
  /// For example when we sac the queen without recapture/checkmate before 2 ply
  /// It has to lead to an increase in eval, else we should not announce it.
  OhNoMyQueen,
  /// Just announce that the Horsey is tricky, when we landed a tactic with a
  /// horsey
  HowDoesTheKnightMove,
}

impl ToString for TrashTalk {
  /// Converts a GameEvent into a string
  fn to_string(&self) -> String {
    match self {
        TrashTalk::AvengeMyCreator => String::from("I see that you have a positive match-up against my developer... I'll do all I can to crush you! Good luck ;-)"),
        TrashTalk::YouAreJustAChessEngine => String::from("Hey there ... You're just a simple engine it seems. I am a chess bot :-D."),
        TrashTalk::YouAreJustHuman => String::from("Hey there ... It seems that you're just a simple human. Good luck!"),
        TrashTalk::TooSlowMyFriend => String::from("Consider working on your speed... Slow and steady wins the race"),
        TrashTalk::YouCanAlreadyResign => String::from("consider working on your speed... Slow and steady wins the race"),
        TrashTalk::WhatAreYouWaitingToCheckmateMe => String::from("What are you waiting to checkmate me?? Don't you know the rules ?"),
        TrashTalk::OhNoMyQueen => String::from("Oh no my queen!"),
        TrashTalk::HowDoesTheKnightMove => String::from("How does the knight move?"),
    }
  }
}
