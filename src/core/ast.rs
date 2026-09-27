#[derive(Debug, PartialEq)]
pub enum AstNode {
  String(String),
  Number(f64),
  Bool(bool),

  Unary(UOpType, Box<AstNode>),
  Binary(BOpType, Box<AstNode>, Box<AstNode>),
  Logical(LOpType, Box<AstNode>, Box<AstNode>),

  Error,
}

#[derive(Debug, PartialEq, Eq)]
pub enum UOpType {
  Not,
  Negate,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LOpType {
  And,
  Or,
}

#[derive(Debug, PartialEq, Eq)]
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
