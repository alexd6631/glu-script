use crate::core::ast::AstNode;

pub struct Evaluator {}

impl Evaluator {
  fn evaluate(node: &AstNode) {
    match node {
      AstNode::String(s) => todo!(),
      AstNode::Number(n) => todo!(),
      AstNode::Bool(b) => todo!(),
      AstNode::Binary(op_type, left, right) => todo!(),
      AstNode::Error => todo!(),
      AstNode::Unary(uop_type, ast_node) => todo!(),
      AstNode::Logical(lop_type, left, right) => todo!(),
    }
  }
}
