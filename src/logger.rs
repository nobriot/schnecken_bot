use crate::tee_writer::TeeWriter;
use log::LevelFilter;
use std::path::Path;

/// Initializes the logger, optionally also writing the logs to `log_file`
///
/// The level is `level` if given, else the one from `RUST_LOG`, else info. Per-module directives
/// from `RUST_LOG` (e.g. `RUST_LOG=chess=trace`) still apply on top of `level`.
pub fn init_logger(level: Option<LevelFilter>, log_file: Option<&Path>) {
  let mut builder = env_logger::Builder::new();
  builder.filter_level(LevelFilter::Info).parse_default_env().format_timestamp_millis();
  if let Some(level) = level {
    builder.filter_level(level);
  }

  if let Some(path) = log_file {
    match std::fs::OpenOptions::new().create(true).append(true).open(path) {
      Ok(file) => {
        builder.target(env_logger::Target::Pipe(Box::new(TeeWriter::new(file))));
      },
      Err(e) => {
        eprintln!("Failed to open log file {}: {e}", path.display());
        std::process::exit(1);
      },
    }
  }

  builder.init();
}
