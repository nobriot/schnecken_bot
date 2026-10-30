use crate::tee_writer::TeeWriter;
use log::{LevelFilter, Log, Metadata, Record};
use std::path::Path;
use std::sync::{OnceLock, RwLock};

/// Logger whose filter can be changed at runtime: env_logger handles the formatting and the
/// output, but its filter is fixed once built, so we filter the records ourselves.
struct ReloadableLogger {
  inner:  env_logger::Logger,
  filter: RwLock<env_filter::Filter>,
}

static LOGGER: OnceLock<ReloadableLogger> = OnceLock::new();

impl Log for ReloadableLogger {
  fn enabled(&self, metadata: &Metadata) -> bool {
    self.filter.read().unwrap().enabled(metadata)
  }

  fn log(&self, record: &Record) {
    if self.filter.read().unwrap().matches(record) {
      self.inner.log(record);
    }
  }

  fn flush(&self) {
    self.inner.flush();
  }
}

/// Builds the log filter: `level` if given, else the level from `RUST_LOG`, else info.
/// Per-module directives from `RUST_LOG` (e.g. `RUST_LOG=chess=trace`) still apply on top of
/// `level`.
fn build_filter(level: Option<LevelFilter>) -> env_filter::Filter {
  let mut builder = env_filter::Builder::new();
  builder.filter_level(LevelFilter::Info);
  if let Ok(rust_log) = std::env::var("RUST_LOG") {
    builder.parse(&rust_log);
  }
  if let Some(level) = level {
    builder.filter_level(level);
  }
  builder.build()
}

/// Initializes the logger, optionally also writing the logs to `log_file`
///
/// See `build_filter` for how `level` is used.
pub fn init_logger(level: Option<LevelFilter>, log_file: Option<&Path>) {
  // Let everything through env_logger, the filtering happens in ReloadableLogger
  let mut builder = env_logger::Builder::new();
  builder.filter_level(LevelFilter::Trace).format_timestamp_millis();

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

  let filter = build_filter(level);
  let max_level = filter.filter();
  let logger = LOGGER.get_or_init(|| ReloadableLogger { inner:  builder.build(),
                                                        filter: RwLock::new(filter), });
  log::set_logger(logger).expect("Logger already initialized");
  log::set_max_level(max_level);
}

/// Changes the log level at runtime. Per-module directives from `RUST_LOG` are kept.
pub fn set_log_level(level: LevelFilter) {
  let Some(logger) = LOGGER.get() else {
    return;
  };
  let filter = build_filter(Some(level));
  let max_level = filter.filter();
  *logger.filter.write().unwrap() = filter;
  log::set_max_level(max_level);
}
