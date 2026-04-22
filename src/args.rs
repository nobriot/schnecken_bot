use clap::Parser;

#[derive(Parser)]
#[command(name = "schnecken_bot", about = "Lichess bot with integrated engine")]
pub struct Args {
  /// Lichess API token
  #[arg(long = "api-token")]
  pub api_token: Option<String>,

  /// Engine cache table size in MB
  #[arg(long = "cache-size")]
  pub cache_table_size: Option<usize>,

  /// Engine play style: normal, conservative, aggressive, provocative
  #[arg(long = "play-style")]
  pub play_style: Option<String>,
}
