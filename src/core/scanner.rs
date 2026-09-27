use phf::phf_map;
use std::str;

pub type Line = u32;

#[derive(Clone)]
pub struct Scanner<'a> {
  source: &'a [u8],
  start: usize,
  current: usize,
  line: Line,
  done: bool,
}

impl<'a> Scanner<'a> {
  pub fn new(source: &'a str) -> Self {
    Scanner {
      source: source.as_bytes(),
      start: 0,
      current: 0,
      line: 1,
      done: false,
    }
  }

  pub fn scan_token(&mut self) -> Token<'a> {
    self.skip_whitespace();
    self.start = self.current;

    if self.is_at_end() {
      return self.make_token(TokenType::Eof);
    }

    let c = self.advance();

    if is_alpha(c) {
      return self.identifier();
    }

    if c.is_ascii_digit() {
      return self.number();
    }

    match c {
      b'(' => return self.make_token(TokenType::LeftParen),
      b')' => return self.make_token(TokenType::RightParen),
      b'{' => return self.make_token(TokenType::LeftBrace),
      b'}' => return self.make_token(TokenType::RightBrace),
      b':' => return self.make_token(TokenType::Colon),
      b';' => return self.make_token(TokenType::Semicolon),
      b',' => return self.make_token(TokenType::Comma),
      b'.' => return self.make_token(TokenType::Dot),
      b'-' => return self.make_token(TokenType::Minus),
      b'+' => return self.make_token(TokenType::Plus),
      b'/' => return self.make_token(TokenType::Slash),
      b'*' => return self.make_token(TokenType::Star),
      b'%' => return self.make_token(TokenType::Percent),
      b'!' => {
        let kind = if self.matches(b'=') {
          TokenType::BangEqual
        } else {
          TokenType::Bang
        };
        return self.make_token(kind);
      }
      b'=' => {
        let kind = if self.matches(b'=') {
          TokenType::EqualEqual
        } else {
          TokenType::Equal
        };
        return self.make_token(kind);
      }
      b'<' => {
        let kind = if self.matches(b'=') {
          TokenType::LessEqual
        } else {
          TokenType::Less
        };
        return self.make_token(kind);
      }
      b'>' => {
        let kind = if self.matches(b'=') {
          TokenType::GreaterEqual
        } else {
          TokenType::Greater
        };
        return self.make_token(kind);
      }
      b'"' => return self.string(),
      _ => {}
    }

    self.error_token("Unexpected character.")
  }

  fn string(&mut self) -> Token<'a> {
    while self.peek() != b'"' && !self.is_at_end() {
      if self.peek() == b'\n' {
        self.line += 1;
      }
      self.advance();
    }

    if self.is_at_end() {
      return self.error_token("Unterminated string");
    }

    // closing quote
    self.advance();
    self.make_token(TokenType::String)
  }

  fn number(&mut self) -> Token<'a> {
    while self.peek().is_ascii_digit() {
      self.advance();
    }

    if self.peek() == b'.' && self.peek_next().is_ascii_digit() {
      // consume "."
      self.advance();

      while self.peek().is_ascii_digit() {
        self.advance();
      }
    }

    self.make_token(TokenType::Number)
  }

  fn identifier(&mut self) -> Token<'a> {
    while is_alpha(self.peek()) || self.peek().is_ascii_digit() {
      self.advance();
    }

    self.make_token(self.identifier_type())
  }

  fn identifier_type(&self) -> TokenType {
    KEYWORDS
      .get(self.current_lexeme())
      .copied()
      .unwrap_or(TokenType::Identifier)
  }

  fn skip_whitespace(&mut self) {
    loop {
      let c = self.peek();

      match c {
        b' ' | b'\r' | b'\t' => {
          self.advance();
        }
        b'\n' => {
          self.line += 1;
          self.advance();
        }
        b'/' if self.peek_next() == b'/' => {
          while self.peek() != b'\n' && !self.is_at_end() {
            self.advance();
          }
        }
        _ => return,
      }
    }
  }
}

impl<'a> Scanner<'a> {
  fn make_token(&self, kind: TokenType) -> Token<'a> {
    let lexeme = self.current_lexeme();

    Token {
      kind,
      lexeme,
      line: self.line,
    }
  }

  fn current_lexeme(&self) -> &'a str {
    let s = &self.source[self.start..self.current];
    let lexeme = unsafe { str::from_utf8_unchecked(s) };
    lexeme
  }

  fn error_token(&self, message: &'static str) -> Token<'a> {
    Token {
      kind: TokenType::Error,
      lexeme: message,
      line: self.line,
    }
  }
}

impl Scanner<'_> {
  fn is_at_end(&self) -> bool {
    self.current >= self.source.len()
  }

  fn advance(&mut self) -> u8 {
    let c = self.source[self.current];
    self.current += 1;
    c
  }

  fn matches(&mut self, expected: u8) -> bool {
    if self.is_at_end() {
      return false;
    }
    if self.source[self.current] != expected {
      return false;
    }
    self.current += 1;
    true
  }

  fn peek(&self) -> u8 {
    if !self.is_at_end() {
      self.source[self.current]
    } else {
      b'\0'
    }
  }

  fn peek_next(&self) -> u8 {
    if self.current + 1 < self.source.len() {
      self.source[self.current + 1]
    } else {
      b'\0'
    }
  }
}

impl<'a> Iterator for Scanner<'a> {
  type Item = Token<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.done {
      None
    } else {
      let token = self.scan_token();
      if token.kind == TokenType::Eof {
        self.done = true
      }
      Some(token)
    }
  }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum TokenType {
  Error,
  Eof,
  LeftParen,
  RightParen,
  LeftBrace,
  RightBrace,
  Semicolon,
  Comma,
  Dot,
  Colon,
  Minus,
  Plus,
  Slash,
  Star,
  Percent,
  BangEqual,
  Bang,
  EqualEqual,
  Equal,
  LessEqual,
  Less,
  GreaterEqual,
  Greater,
  String,
  Number,
  Identifier,
  And,
  Else,
  If,
  Nil,
  Print,
  Or,
  While,
  False,
  Let,
  For,
  Fun,
  True,
  Switch,
  Case,
  Default,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Token<'a> {
  pub kind: TokenType,
  pub lexeme: &'a str,
  pub line: u32,
}

impl Token<'static> {
  pub fn synthetic(name: &'static str) -> Self {
    Self {
      kind: TokenType::Eof,
      lexeme: name,
      line: 0,
    }
  }
}

fn is_alpha(c: u8) -> bool {
  c.is_ascii_alphabetic() || c == b'_'
}

static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "and" => TokenType::And,
    "case" => TokenType::Case,
    "default" => TokenType::Default,
    "else" => TokenType::Else,
    "false" => TokenType::False,
    "for" => TokenType::For,
    "fun" => TokenType::Fun,
    "if" => TokenType::If,
    "nil" => TokenType::Nil,
    "or" => TokenType::Or,
    "print" => TokenType::Print,
    "switch" => TokenType::Switch,
    "true" => TokenType::True,
    "let" => TokenType::Let,
    "while" => TokenType::While,
};

#[cfg(test)]
mod tests {
  use crate::core::scanner::TokenType::Eof;
  use crate::core::scanner::{Scanner, Token, TokenType};

  #[test]
  pub fn test_eof() {
    let scanner = Scanner::new("");
    let t: Vec<Token> = scanner.collect();

    assert_eq!(
      vec![Token {
        kind: Eof,
        lexeme: "",
        line: 1,
      }],
      t
    );
  }

  #[test]
  pub fn test_simple_lexemes() {
    let scanner = Scanner::new(" ( + . * )");
    let t: Vec<Token> = scanner.collect();

    assert_eq!(
      vec![
        Token {
          kind: TokenType::LeftParen,
          lexeme: "(",
          line: 1,
        },
        Token {
          kind: TokenType::Plus,
          lexeme: "+",
          line: 1,
        },
        Token {
          kind: TokenType::Dot,
          lexeme: ".",
          line: 1,
        },
        Token {
          kind: TokenType::Star,
          lexeme: "*",
          line: 1,
        },
        Token {
          kind: TokenType::RightParen,
          lexeme: ")",
          line: 1,
        },
        Token {
          kind: Eof,
          lexeme: "",
          line: 1,
        },
      ],
      t
    );
  }

  #[test]
  pub fn test_two_chars_lexemes() {
    let scanner = Scanner::new("! != =");
    let t: Vec<Token> = scanner.collect();

    assert_eq!(
      vec![
        Token {
          kind: TokenType::Bang,
          lexeme: "!",
          line: 1,
        },
        Token {
          kind: TokenType::BangEqual,
          lexeme: "!=",
          line: 1,
        },
        Token {
          kind: TokenType::Equal,
          lexeme: "=",
          line: 1,
        },
        Token {
          kind: Eof,
          lexeme: "",
          line: 1,
        },
      ],
      t
    );
  }

  #[test]
  pub fn test_newlines_and_comments() {
    let scanner = Scanner::new("() // comment\n!= // comment\n.");
    let t: Vec<Token> = scanner.collect();

    assert_eq!(
      vec![
        Token {
          kind: TokenType::LeftParen,
          lexeme: "(",
          line: 1
        },
        Token {
          kind: TokenType::RightParen,
          lexeme: ")",
          line: 1
        },
        Token {
          kind: TokenType::BangEqual,
          lexeme: "!=",
          line: 2
        },
        Token {
          kind: TokenType::Dot,
          lexeme: ".",
          line: 3
        },
        Token {
          kind: Eof,
          lexeme: "",
          line: 3
        }
      ],
      t
    );
  }

  #[test]
  pub fn test_string() {
    let scanner = Scanner::new("(\"Hello\nworld\")");
    let t: Vec<Token> = scanner.collect();

    assert_eq!(
      vec![
        Token {
          kind: TokenType::LeftParen,
          lexeme: "(",
          line: 1
        },
        Token {
          kind: TokenType::String,
          lexeme: "\"Hello\nworld\"",
          line: 2
        },
        Token {
          kind: TokenType::RightParen,
          lexeme: ")",
          line: 2
        },
        Token {
          kind: Eof,
          lexeme: "",
          line: 2
        }
      ],
      t
    );
  }

  #[test]
  pub fn test_numbers() {
    let scanner = Scanner::new("42,42.4,42.");
    let t: Vec<Token> = scanner.collect();

    assert_eq!(
      vec![
        Token {
          kind: TokenType::Number,
          lexeme: "42",
          line: 1
        },
        Token {
          kind: TokenType::Comma,
          lexeme: ",",
          line: 1
        },
        Token {
          kind: TokenType::Number,
          lexeme: "42.4",
          line: 1
        },
        Token {
          kind: TokenType::Comma,
          lexeme: ",",
          line: 1
        },
        Token {
          kind: TokenType::Number,
          lexeme: "42",
          line: 1
        },
        Token {
          kind: TokenType::Dot,
          lexeme: ".",
          line: 1
        },
        Token {
          kind: Eof,
          lexeme: "",
          line: 1
        }
      ],
      t
    );
  }
}
