use crate::core::value::Value;

#[derive(Debug, PartialEq)]
pub enum AstNode {
  Literal(Value),
  List(Vec<AstNode>),

  Unary(UOpType, Box<AstNode>),
  Binary(BOpType, Box<AstNode>, Box<AstNode>),
  Logical(LOpType, Box<AstNode>, Box<AstNode>),

  Error,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UOpType {
  Not,
  Negate,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LOpType {
  And,
  Or,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BOpType {
  Add,
  Sub,
  Mul,
  Div,
  Mod,
  Less,
  LessEqual,
  Greater,
  GreaterEqual,
  Equal,
  NotEqual,
}
