use std::fmt::Write;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Writers {
  pub writer: Arc<Mutex<dyn Write + Send>>,
  pub err_writer: Arc<Mutex<dyn Write + Send>>,
  pub trace_writer: Arc<Mutex<dyn Write + Send>>,
}

impl Default for Writers {
  fn default() -> Self {
    Self {
      writer: Arc::new(Mutex::new(StdoutWriter {})),
      err_writer: Arc::new(Mutex::new(StderrWriter {})),
      trace_writer: Arc::new(Mutex::new(StderrWriter {})),
    }
  }
}

pub struct StdoutWriter;

impl Write for StdoutWriter {
  fn write_str(&mut self, s: &str) -> std::fmt::Result {
    print!("{}", s);
    Ok(())
  }
}

pub struct StderrWriter;

impl Write for StderrWriter {
  fn write_str(&mut self, s: &str) -> std::fmt::Result {
    eprint!("{}", s);
    Ok(())
  }
}
