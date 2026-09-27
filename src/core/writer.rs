use std::cell::RefCell;
use std::fmt::Write;
use std::rc::Rc;

#[derive(Clone)]
pub struct Writers {
  pub writer: Rc<RefCell<dyn Write>>,
  pub err_writer: Rc<RefCell<dyn Write>>,
  pub trace_writer: Rc<RefCell<dyn Write>>,
}

impl Default for Writers {
  fn default() -> Self {
    Self {
      writer: Rc::new(RefCell::new(StdoutWriter {})),
      err_writer: Rc::new(RefCell::new(StderrWriter {})),
      trace_writer: Rc::new(RefCell::new(StderrWriter {})),
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
