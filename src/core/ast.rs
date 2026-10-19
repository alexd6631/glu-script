use std::sync::Arc;

use crate::core::value::Value;

#[derive(Debug, PartialEq)]
pub enum AstNode {
  Literal(Value),
  List(Vec<AstNode>),
  Object(Vec<(AstObjectKey, AstNode)>),
  Block(Vec<AstNode>),

  Unary(UOpType, Box<AstNode>),
  Binary(BOpType, Box<AstNode>, Box<AstNode>),
  Logical(LOpType, Box<AstNode>, Box<AstNode>),
  Index(Box<AstNode>, Box<AstNode>),
  Property(Box<AstNode>, Arc<str>),

  Binding(Arc<str>, Box<AstNode>),
  VariableRef(Arc<str>),

  If(IfNode),
  Closure(Arc<ClosureNode>),
  Call(Box<AstNode>, Vec<AstNode>),

  Error,
}

#[derive(Debug, PartialEq)]
pub struct IfNode {
  pub condition: Box<AstNode>,
  pub then_branch: Box<AstNode>,
  pub else_branch: Option<Box<AstNode>>,
}

#[derive(Debug, PartialEq)]
pub struct ClosureNode {
  pub params: Vec<Arc<str>>,
  pub body: Box<AstNode>,
}

#[derive(Debug, PartialEq)]
pub enum AstObjectKey {
  Literal(Arc<str>),
  Computed(AstNode),
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
