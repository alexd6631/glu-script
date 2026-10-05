use std::collections::HashMap;

use crate::core::{
  ast::{AstNode, AstObjectKey, BOpType, LOpType, UOpType},
  value::Value,
};

pub struct Evaluator {}

impl Evaluator {
  pub fn evaluate(node: &AstNode) -> Result<Value, String> {
    match node {
      AstNode::Literal(l) => Ok(l.clone()),
      AstNode::List(items) => Self::list(items),
      AstNode::Object(entries) => Self::object(entries),
      AstNode::Block(items) => Self::block(items),
      AstNode::Binary(op_type, left, right) => Self::binary(*op_type, left, right),
      AstNode::Logical(op_type, left, right) => Self::logical(*op_type, left, right),
      AstNode::Unary(op_type, ast_node) => Self::unary(*op_type, ast_node),
      AstNode::Error => Err("Invalid expression.".to_string()),
    }
  }

  fn unary(op_type: UOpType, node: &AstNode) -> Result<Value, String> {
    let value = Self::evaluate(node)?;

    match op_type {
      UOpType::Not => Ok(Value::Bool(!is_truthy(&value))),
      UOpType::Negate => match value {
        Value::Number(n) => Ok(Value::Number(-n)),
        _ => Err("Operand must be a number.".to_string()),
      },
    }
  }

  fn logical(op_type: LOpType, left: &AstNode, right: &AstNode) -> Result<Value, String> {
    let lhs = Self::evaluate(left)?;

    match op_type {
      LOpType::And => {
        if is_truthy(&lhs) {
          Self::evaluate(right)
        } else {
          Ok(lhs)
        }
      }
      LOpType::Or => {
        if is_truthy(&lhs) {
          Ok(lhs)
        } else {
          Self::evaluate(right)
        }
      }
    }
  }

  fn binary(op_type: BOpType, left: &AstNode, right: &AstNode) -> Result<Value, String> {
    let lhs = Self::evaluate(left)?;
    let rhs = Self::evaluate(right)?;

    match op_type {
      BOpType::Equal => Ok(Value::Bool(lhs == rhs)),
      BOpType::NotEqual => Ok(Value::Bool(lhs != rhs)),
      BOpType::Add => match (&lhs, &rhs) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
        (Value::String(l), Value::String(r)) => Ok(Value::String(format!("{}{}", l, r).into())),
        _ => Err("Operands must be two numbers or two strings.".to_string()),
      },
      BOpType::Sub => Ok(Value::Number(check_number(&lhs)? - check_number(&rhs)?)),
      BOpType::Mul => Ok(Value::Number(check_number(&lhs)? * check_number(&rhs)?)),
      BOpType::Div => Ok(Value::Number(check_number(&lhs)? / check_number(&rhs)?)),
      BOpType::Mod => Ok(Value::Number(check_number(&lhs)? % check_number(&rhs)?)),
      BOpType::Less => Ok(Value::Bool(check_number(&lhs)? < check_number(&rhs)?)),
      BOpType::LessEqual => Ok(Value::Bool(check_number(&lhs)? <= check_number(&rhs)?)),
      BOpType::Greater => Ok(Value::Bool(check_number(&lhs)? > check_number(&rhs)?)),
      BOpType::GreaterEqual => Ok(Value::Bool(check_number(&lhs)? >= check_number(&rhs)?)),
    }
  }

  fn list(nodes: &[AstNode]) -> Result<Value, String> {
    let values: Result<Vec<Value>, String> = nodes.iter().map(Self::evaluate).collect();
    Ok(Value::List(values?))
  }

  fn object(entries: &[(AstObjectKey, AstNode)]) -> Result<Value, String> {
    let mut fields = HashMap::with_capacity(entries.len());

    for (key, value) in entries {
      let key = match key {
        AstObjectKey::Literal(k) => k.clone(),
        AstObjectKey::Computed(node) => match Self::evaluate(node)? {
          Value::String(s) => s,
          _ => return Err("Object keys must be strings.".to_string()),
        },
      };

      fields.insert(key, Self::evaluate(value)?);
    }

    Ok(Value::Object(fields))
  }

  fn block(nodes: &[AstNode]) -> Result<Value, String> {
    let mut result = Value::Nil;

    for node in nodes {
      result = Self::evaluate(node)?;
    }

    Ok(result)
  }
}

fn check_number(value: &Value) -> Result<f64, String> {
  match value {
    Value::Number(n) => Ok(*n),
    _ => Err("Operands must be numbers.".to_string()),
  }
}

fn is_truthy(value: &Value) -> bool {
  !matches!(value, Value::Nil | Value::Bool(false))
}
