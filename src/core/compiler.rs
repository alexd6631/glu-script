use std::{collections::HashMap, sync::OnceLock};

use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::core::{
  ast::{AstNode, AstObjectKey, BOpType, LOpType, UOpType},
  parser::Parser,
  scanner::{Scanner, TokenType},
  value::Value,
  writer::Writers,
};

pub type RawNumber = f64;

pub struct Compiler<'a> {
  parser: Parser<'a>,
}

impl<'a> Compiler<'a> {
  pub fn new(source: &'a str) -> Self {
    let scanner = Scanner::new(source);
    let parser = Parser::new(scanner, Writers::default());

    Self { parser }
  }

  pub fn compile(&mut self) -> AstNode {
    self.parser.advance();
    self.expression()
  }

  fn expression(&mut self) -> AstNode {
    self.parse_precedence(Precedence::Assignment)
  }

  fn literal(&self, _can_assign: bool) -> AstNode {
    let token = self.parser.get_previous();
    let token_type = token.kind;
    let lexeme = token.lexeme;

    match token_type {
      TokenType::False => AstNode::Literal(Value::Bool(false)),
      TokenType::True => AstNode::Literal(Value::Bool(true)),
      TokenType::Nil => AstNode::Literal(Value::Nil),
      TokenType::String => {
        let s = &lexeme[1..lexeme.len() - 1];
        AstNode::Literal(Value::String(s.into()))
      }
      TokenType::Number => {
        let value: RawNumber = lexeme.parse().unwrap();
        AstNode::Literal(Value::Number(value))
      }
      _ => panic!("unreachable"),
    }
  }

  fn grouping(&mut self, _can_assign: bool) -> AstNode {
    let node = self.expression();
    self
      .parser
      .consume(TokenType::RightParen, "Expect ')' after expression.");

    node
  }

  fn unary(&mut self, _can_assign: bool) -> AstNode {
    let prev = self.parser.get_previous().clone();
    let operator_type = prev.kind;

    let inner = self.parse_precedence(Precedence::Unary);
    match operator_type {
      TokenType::Bang => AstNode::Unary(UOpType::Not, inner.into()),
      TokenType::Minus => AstNode::Unary(UOpType::Negate, inner.into()),
      _ => panic!("unreachable"),
    }
  }

  fn binary(&mut self, lhs: AstNode, _can_assign: bool) -> AstNode {
    let prev = self.parser.get_previous().clone();
    let operator_type = prev.kind;

    let rule = get_rule(operator_type);
    let rhs = self.parse_precedence(rule.precedence.next());

    let ast_op_type = match operator_type {
      TokenType::Plus => BOpType::Add,
      TokenType::Minus => BOpType::Sub,
      TokenType::Star => BOpType::Mul,
      TokenType::Slash => BOpType::Div,
      TokenType::Percent => BOpType::Mod,
      TokenType::Less => BOpType::Less,
      TokenType::LessEqual => BOpType::LessEqual,
      TokenType::Greater => BOpType::Greater,
      TokenType::GreaterEqual => BOpType::GreaterEqual,
      TokenType::EqualEqual => BOpType::Equal,
      TokenType::BangEqual => BOpType::NotEqual,
      _ => panic!("unreachable"),
    };

    AstNode::Binary(ast_op_type, Box::new(lhs), Box::new(rhs))
  }

  fn logical(&mut self, lhs: AstNode, _can_assign: bool) -> AstNode {
    let prev = self.parser.get_previous().clone();
    let operator_type = prev.kind;

    let rule = get_rule(operator_type);
    let rhs = self.parse_precedence(rule.precedence.next());

    let ast_op_type = match operator_type {
      TokenType::And => LOpType::And,
      TokenType::Or => LOpType::Or,
      _ => panic!("unreachable"),
    };

    AstNode::Logical(ast_op_type, Box::new(lhs), Box::new(rhs))
  }

  fn list(&mut self, _can_assign: bool) -> AstNode {
    let mut items = vec![];
    if !self.parser.check(TokenType::RightBracket) {
      loop {
        items.push(self.expression());
        if !self.parser.matches(TokenType::Comma) {
          break;
        }
      }
    }

    self.parser.consume(TokenType::RightBracket, "Expected ']'");

    AstNode::List(items)
  }

  fn block_or_object_literal(&mut self, _can_assign: bool) -> AstNode {
    if self.look_object_literal_ahead() {
      self.object_literal(_can_assign)
    } else {
      self.block(_can_assign)
    }
  }

  fn look_object_literal_ahead(&self) -> bool {
    let mut lookahead_parser = self.parser.clone();

    // Heuristic to check if we branch to object literal parsing instead of a block
    // We check for one of those patterns, to match a first computed or literal property :
    // { [ ... ] :
    // { <IDENTIFIER | STRING> :

    if lookahead_parser.matches(TokenType::LeftBracket) {
      let mut level = 1;
      while !self.parser.check(TokenType::Eof) {
        if lookahead_parser.matches(TokenType::RightBracket) {
          level -= 1;
          if level == 0 {
            return lookahead_parser.matches(TokenType::Colon);
          }
        } else {
          if lookahead_parser.check(TokenType::LeftBracket) {
            level += 1;
          }
          lookahead_parser.advance();
        }
      }
      false
    } else {
      (lookahead_parser.matches(TokenType::Identifier)
        || lookahead_parser.matches(TokenType::String))
        && lookahead_parser.matches(TokenType::Colon)
    }
  }

  fn object_literal(&mut self, _can_assign: bool) -> AstNode {
    let mut entries = vec![];

    if !self.parser.check(TokenType::RightBrace) {
      loop {
        let key = if self.parser.matches(TokenType::Identifier) {
          AstObjectKey::Literal(self.parser.get_previous().lexeme.into())
        } else if self.parser.matches(TokenType::String) {
          let lexeme = self.parser.get_previous().lexeme;

          AstObjectKey::Literal(lexeme[1..lexeme.len() - 1].into())
        } else if self.parser.matches(TokenType::LeftBracket) {
          let key_expr = self.expression();
          self
            .parser
            .consume(TokenType::RightBracket, "expected right bracket");
          AstObjectKey::Computed(key_expr)
        } else {
          self
            .parser
            .error_at_current("expected string or identifier");
          AstObjectKey::Literal("error".into())
        };

        self.parser.consume(TokenType::Colon, "expected :");
        let value = self.expression();

        entries.push((key, value));

        if !self.parser.matches(TokenType::Comma) {
          break;
        }
      }
    }

    self.parser.consume(TokenType::RightBrace, "Expected '}'");

    AstNode::Object(entries)
  }

  fn block(&mut self, _can_assign: bool) -> AstNode {
    let mut items = vec![];
    if !self.parser.check(TokenType::RightBrace) {
      loop {
        items.push(self.expression());
        if !self.parser.matches(TokenType::Semicolon) {
          break;
        }
      }
    }

    self.parser.consume(TokenType::RightBrace, "Expected '}'");

    AstNode::Block(items)
  }

  fn parse_precedence(&mut self, precedence: Precedence) -> AstNode {
    self.parser.advance();
    let Some(prefix_rule) = get_rule(self.parser.get_previous().kind).prefix else {
      self.parser.error_at_current("Expect expression.");
      return AstNode::Error;
    };

    let can_assign = precedence <= Precedence::Assignment;
    let mut lhs = prefix_rule(self, can_assign);

    while precedence <= get_rule(self.parser.get_current().kind).precedence {
      self.parser.advance();
      let infix_rule = get_rule(self.parser.get_previous().kind).infix.unwrap();
      lhs = infix_rule(self, lhs, can_assign);
    }

    if can_assign && self.parser.matches(TokenType::Equal) {
      self.parser.error_at_current("Invalid assignment target.");
      return AstNode::Error;
    }

    lhs
  }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, IntoPrimitive, TryFromPrimitive, Ord, PartialOrd, Eq, PartialEq)]
enum Precedence {
  None,
  Assignment, // =
  Or,         // or
  And,        // and
  Equality,   // == !=
  Comparison, // < > <= >=
  Term,       // + -
  Factor,     // * /
  Unary,      // ! -
  Call,       // . ()
  Primary,
}

impl Precedence {
  fn next(self) -> Precedence {
    ((self as u8) + 1).try_into().unwrap()
  }
}

type PrefixParseFn = for<'a> fn(&mut Compiler<'a>, can_assign: bool) -> AstNode;
type InfixParseFn = for<'a> fn(&mut Compiler<'a>, lhs: AstNode, can_assign: bool) -> AstNode;

struct ParseRule {
  prefix: Option<PrefixParseFn>,
  infix: Option<InfixParseFn>,
  precedence: Precedence,
}

fn get_rule(token_type: TokenType) -> &'static ParseRule {
  rules().get(&token_type).unwrap_or(&DEFAULT_RULE)
}

fn rules() -> &'static HashMap<TokenType, ParseRule> {
  static HASHMAP: OnceLock<HashMap<TokenType, ParseRule>> = OnceLock::new();
  HASHMAP.get_or_init(build_rules)
}

macro_rules! _parse_fn {
  (None) => {
    None
  };
  ($fn: ident) => {
    Some(|c, can_assign| c.$fn(can_assign))
  };
}

macro_rules! _parse_infix_fn {
  (None) => {
    None
  };
  ($fn: ident) => {
    Some(|c, lhs, can_assign| c.$fn(lhs, can_assign))
  };
}

macro_rules! _rule {
  (
        ($prefix: tt, $infix: tt, $precedence: ident)
    ) => {
    ParseRule {
      prefix: _parse_fn!($prefix),
      infix: _parse_infix_fn!($infix),
      precedence: Precedence::$precedence,
    }
  };
}

macro_rules! _rules {
    ($($token: ident => $rule: tt)*) => {
        let mut rules = HashMap::<TokenType, ParseRule>::new();

        $(
            rules.insert(
                TokenType::$token,
                _rule!($rule)
            );
        )*

        rules
    };
}

static DEFAULT_RULE: ParseRule = _rule!((None, None, None));

fn build_rules() -> HashMap<TokenType, ParseRule> {
  _rules! {
      LeftParen => (grouping, None, Call) // TODO implement call
      Number => (literal, None, None)
      Nil => (literal, None, None)
      True => (literal, None, None)
      False => (literal, None, None)
      // Identifier => (variable, None, None)
      String => (literal, None, None)

      BangEqual => (None, binary, Equality)
      EqualEqual => (None, binary, Equality)

      Greater => (None, binary, Comparison)
      GreaterEqual => (None, binary, Comparison)
      Less => (None, binary, Comparison)
      LessEqual => (None, binary, Comparison)

      Bang => (unary, None, None)
      Minus => (unary, binary, Term)
      Plus => (None, binary, Term)
      Star => (None, binary, Factor)
      Slash => (None, binary, Factor)
      Percent => (None, binary, Factor)

      And => (None, logical, And)
      Or => (None, logical, Or)

      // Dot => (None, dot, Call)

      LeftBracket => (list, None, None)
      LeftBrace => (block_or_object_literal, None, None)
  }
}

#[cfg(test)]
mod tests {
  use crate::core::{
    ast::{AstNode, AstObjectKey, BOpType, LOpType, UOpType},
    compiler::Compiler,
    value::Value,
  };

  fn bool(b: bool) -> AstNode {
    AstNode::Literal(Value::Bool(b))
  }

  fn num(n: f64) -> AstNode {
    AstNode::Literal(Value::Number(n))
  }

  fn unary(op: UOpType, node: AstNode) -> AstNode {
    AstNode::Unary(op, Box::new(node))
  }

  fn bin(op: BOpType, lhs: AstNode, rhs: AstNode) -> AstNode {
    AstNode::Binary(op, Box::new(lhs), Box::new(rhs))
  }

  fn log(op: LOpType, lhs: AstNode, rhs: AstNode) -> AstNode {
    AstNode::Logical(op, Box::new(lhs), Box::new(rhs))
  }

  fn str(s: &str) -> AstNode {
    AstNode::Literal(Value::String(s.into()))
  }

  #[test]
  fn test_parse_expression() {
    let mut c = Compiler::new("1 + 1 - 2 * 3 + 2/5");
    let node = c.compile();

    assert_eq!(
      node,
      bin(
        BOpType::Add,
        bin(
          BOpType::Sub,
          bin(BOpType::Add, num(1.0), num(1.0)),
          bin(BOpType::Mul, num(2.0), num(3.0))
        ),
        bin(BOpType::Div, num(2.0), num(5.0))
      )
    );
  }

  #[test]
  fn test_parse_comparison() {
    let mut c = Compiler::new("1 < 2 + 3");
    let node = c.compile();

    assert_eq!(
      node,
      bin(
        BOpType::Less,
        num(1.0),
        bin(BOpType::Add, num(2.0), num(3.0))
      )
    );
  }

  #[test]
  fn test_parse_comparison_left_associative() {
    let mut c = Compiler::new("1 <= 2 > 3 >= 4");
    let node = c.compile();

    assert_eq!(
      node,
      bin(
        BOpType::GreaterEqual,
        bin(
          BOpType::Greater,
          bin(BOpType::LessEqual, num(1.0), num(2.0)),
          num(3.0)
        ),
        num(4.0)
      )
    );
  }

  #[test]
  fn test_parse_equality() {
    let mut c = Compiler::new("1 == 2 != 3 < 4");
    let node = c.compile();

    assert_eq!(
      node,
      bin(
        BOpType::NotEqual,
        bin(BOpType::Equal, num(1.0), num(2.0)),
        bin(BOpType::Less, num(3.0), num(4.0))
      )
    );
  }

  #[test]
  fn test_parse_unary() {
    let mut c = Compiler::new("-1");
    let node = c.compile();

    assert_eq!(node, unary(UOpType::Negate, num(1.0)))
  }

  #[test]
  fn test_grouping() {
    let mut c = Compiler::new("3 * (1 + 2)");
    let node = c.compile();

    assert_eq!(
      node,
      bin(
        BOpType::Mul,
        num(3.0),
        bin(BOpType::Add, num(1.0), num(2.0))
      )
    );
  }

  #[test]
  fn test_parse_modulo() {
    let mut c = Compiler::new("1 + 2 % 3");
    let node = c.compile();

    assert_eq!(
      node,
      bin(
        BOpType::Add,
        num(1.0),
        bin(BOpType::Mod, num(2.0), num(3.0))
      )
    );
  }

  #[test]
  fn test_parse_and_or() {
    let mut c = Compiler::new("1 or 2 and 3");
    let node = c.compile();

    assert_eq!(
      node,
      log(LOpType::Or, num(1.0), log(LOpType::And, num(2.0), num(3.0)))
    );
  }

  #[test]
  fn test_parse_and_or_left_associative() {
    let mut c = Compiler::new("1 and 2 or 3");
    let node = c.compile();

    assert_eq!(
      node,
      log(LOpType::Or, log(LOpType::And, num(1.0), num(2.0)), num(3.0))
    );
  }

  #[test]
  fn test_literal_string() {
    let mut c = Compiler::new("\"Hello\" + \" world\"");
    let node = c.compile();

    assert_eq!(node, bin(BOpType::Add, str("Hello"), str(" world")));
  }

  #[test]
  fn test_literal_true() {
    let mut c = Compiler::new("true");
    let node = c.compile();

    assert_eq!(node, AstNode::Literal(Value::Bool(true)));
  }

  #[test]
  fn test_literal_false() {
    let mut c = Compiler::new("false");
    let node = c.compile();

    assert_eq!(node, AstNode::Literal(Value::Bool(false)));
  }

  #[test]
  fn test_literal_nil() {
    let mut c = Compiler::new("nil");
    let node = c.compile();

    assert_eq!(node, AstNode::Literal(Value::Nil));
  }

  #[test]
  fn test_literal_number() {
    let mut c = Compiler::new("42.5");
    let node = c.compile();

    assert_eq!(node, num(42.5));
  }

  #[test]
  fn test_literal_list() {
    let mut c = Compiler::new("([42.5, \"hello\", false, true]");
    let node = c.compile();

    assert_eq!(
      node,
      AstNode::List(vec![num(42.5), str("hello"), bool(false), bool(true)])
    );
  }

  #[test]
  fn test_block() {
    let mut c = Compiler::new("{ 1 + 2; 2 + 2 }");
    let node = c.compile();

    assert_eq!(
      node,
      AstNode::Block(vec![
        bin(BOpType::Add, num(1.0), num(2.0)),
        bin(BOpType::Add, num(2.0), num(2.0))
      ])
    );
  }

  #[test]
  fn test_object_literal() {
    let mut c =
      Compiler::new(r#"{ "aaaa": 1, b: true, c: [1, 2, 3], ["Hello" + "world"]: "test"}"#);

    let node = c.compile();

    assert_eq!(
      node,
      AstNode::Object(vec![
        (AstObjectKey::Literal("aaaa".into()), num(1.0)),
        (AstObjectKey::Literal("b".into()), bool(true)),
        (
          AstObjectKey::Literal("c".into()),
          AstNode::List(vec![num(1.0), num(2.0), num(3.0)])
        ),
        (
          AstObjectKey::Computed(bin(BOpType::Add, str("Hello"), str("world"))),
          str("test")
        ),
      ])
    );
  }

  #[test]
  fn test_foo() {
    let mut c = Compiler::new(r#"{ [[1, 2, 3]]: 32 }"#);

    let node = c.compile();
    dbg!(node);
  }
}
