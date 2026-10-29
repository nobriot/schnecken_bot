use std::fs::File;
use std::io::{self, Write};

/// Writes the logs both to stderr and to a log file
pub struct TeeWriter {
  file: File,
}

impl TeeWriter {
  pub fn new(file: File) -> Self {
    TeeWriter { file }
  }
}

impl Write for TeeWriter {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    io::stderr().write_all(buf)?;
    self.file.write_all(buf)?;
    Ok(buf.len())
  }

  fn flush(&mut self) -> io::Result<()> {
    io::stderr().flush()?;
    self.file.flush()
  }
}
