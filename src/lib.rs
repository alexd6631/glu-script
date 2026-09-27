#![deny(clippy::all)]
pub mod core;

use napi_derive::napi;

use crate::core::{compiler::Compiler, scanner::Scanner};

#[napi]
pub fn test_scan(s: String) {
  let scanner = Scanner::new(&s);
  for token in scanner {
    dbg!(token);
  }
}

#[napi]
pub fn test_parse(s: String) {
  let mut compiler = Compiler::new(&s);
  let ast = compiler.compile();

  dbg!(ast);
}
