use crate::tee_writer::TeeWriter;
use log::LevelFilter;
use std::path::Path;

/// Initializes the logger, optionally also writing the logs to `log_file`
pub fn init_logger(log_file: Option<&Path>) {
  let mut builder = env_logger::Builder::new();
  builder.filter_level(LevelFilter::Info).parse_default_env().format_timestamp_millis();

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
