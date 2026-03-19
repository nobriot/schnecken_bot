use anyhow::{Context, Result, anyhow};
use log::info;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
  #[serde(default)]
  pub lichess: LichessConfig,
  #[serde(default)]
  pub engine:  EngineConfig,
}

#[derive(Serialize, Deserialize, Default)]
pub struct LichessConfig {
  pub api_token: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EngineConfig {
  /// Cache table size in MB (default: 1024)
  #[serde(default = "EngineConfig::default_cache_table_size")]
  pub cache_table_size: usize,
  /// Play style: normal, conservative, aggressive, provocative (default:
  /// normal)
  #[serde(default)]
  pub play_style:       Option<String>,
}

impl Default for EngineConfig {
  fn default() -> Self {
    EngineConfig { cache_table_size: Self::default_cache_table_size(),
                   play_style:       None, }
  }
}

impl EngineConfig {
  fn default_cache_table_size() -> usize {
    1024
  }
}

fn config_path() -> Result<PathBuf> {
  let dir = dirs::config_dir().ok_or_else(|| anyhow!("Could not determine config directory"))?
                              .join("schnecken_bot");
  Ok(dir.join("config.toml"))
}

fn load_config() -> Result<Config> {
  let path = config_path()?;
  if !path.exists() {
    return Ok(Config::default());
  }
  let content = std::fs::read_to_string(&path).with_context(|| {
                                                format!("Failed to read config file: {}",
                                                        path.display())
                                              })?;
  toml::from_str(&content).with_context(|| "Failed to parse config file")
}

fn save_config(config: &Config) -> Result<()> {
  let path = config_path()?;
  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent).with_context(|| {
                                     format!("Failed to create config directory: {}",
                                             parent.display())
                                   })?;
  }
  let content = toml::to_string_pretty(config)?;
  std::fs::write(&path, &content).with_context(|| {
                                   format!("Failed to write config file: {}", path.display())
                                 })?;

  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
  }

  info!("Config saved to {}", path.display());
  Ok(())
}

pub fn resolve_token(cli_token: Option<String>) -> Result<String> {
  // 1. CLI argument
  if let Some(token) = cli_token {
    let token = token.trim().to_string();
    if !token.is_empty() {
      info!("Using API token from CLI argument");
      save_token(&token)?;
      return Ok(token);
    }
  }

  // 2. Environment variable
  if let Ok(token) = std::env::var("LICHESS_API_TOKEN") {
    let token = token.trim().to_string();
    if !token.is_empty() {
      info!("Using API token from LICHESS_API_TOKEN environment variable");
      save_token(&token)?;
      return Ok(token);
    }
  }

  // 3. Config file
  if let Ok(config) = load_config()
     && let Some(token) = config.lichess.api_token
  {
    let token = token.trim().to_string();
    if !token.is_empty() {
      info!("Using API token from config file");
      return Ok(token);
    }
  }

  // 4. Interactive prompt
  eprintln!("No Lichess API token found. Please enter your token:");
  let token = rpassword::read_password().context("Failed to read token from terminal")?
                                        .trim()
                                        .to_string();

  if token.is_empty() {
    return Err(anyhow!("No API token provided"));
  }

  info!("Using API token from interactive prompt");
  save_token(&token)?;
  Ok(token)
}

pub fn load_engine_config() -> EngineConfig {
  load_config().map(|c| c.engine).unwrap_or_default()
}

fn save_token(token: &str) -> Result<()> {
  let mut config = load_config().unwrap_or_default();
  config.lichess.api_token = Some(token.to_string());
  save_config(&config)
}
