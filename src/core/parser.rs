use crate::core::{
  scanner::{Scanner, Token, TokenType},
  writer::Writers,
};

pub struct Parser<'a> {
  scanner: Scanner<'a>,
  current: Option<Token<'a>>,
  previous: Option<Token<'a>>,
  had_error: bool,
  panic_mode: bool,
  writers: Writers,
}

impl<'a> Parser<'a> {
  pub fn new(scanner: Scanner<'a>, writers: Writers) -> Self {
    Self {
      scanner,
      current: None,
      previous: None,
      had_error: false,
      panic_mode: false,
      writers,
    }
  }

  pub fn advance(&mut self) {
    self.previous.clone_from(&self.current);

    loop {
      self.current = Some(self.scanner.scan_token());

      if self.get_current().kind != TokenType::Error {
        break;
      }

      let msg = self.get_current().lexeme;
      self.error_at_current(msg);
    }
  }

  pub fn consume(&mut self, kind: TokenType, message: &str) {
    if self.get_current().kind == kind {
      self.advance();
      return;
    }

    self.error_at_current(message);
  }

  pub fn matches(&mut self, token_type: TokenType) -> bool {
    if !self.check(token_type) {
      return false;
    }
    self.advance();
    true
  }

  pub fn check(&self, token_type: TokenType) -> bool {
    self.get_current().kind == token_type
  }

  pub fn get_current(&self) -> &Token<'a> {
    self.current.as_ref().unwrap()
  }

  pub fn get_previous(&self) -> &Token<'a> {
    self.previous.as_ref().unwrap()
  }
}

impl<'a> Parser<'a> {
  pub fn error_at_current(&mut self, message: &str) {
    let token = self.get_current().clone();
    self.error_at(&token, message);
  }

  pub fn error_at(&mut self, token: &Token<'a>, message: &str) {
    if self.panic_mode {
      return;
    }
    self.panic_mode = true;
    let mut writer = self.writers.err_writer.borrow_mut();
    write!(writer, "[line {}] Error", token.line).unwrap();

    match token.kind {
      TokenType::Eof => write!(writer, " at end").unwrap(),
      TokenType::Error => {}
      _ => write!(writer, " at {}", token.lexeme).unwrap(),
    }
    writeln!(writer, ": {message}").unwrap();
    self.had_error = true;
  }
}
