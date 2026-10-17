use std::{collections::HashMap, fmt, rc::Rc};

use crate::core::{ast::ClosureNode, environment::Environment};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
  Nil,
  Bool(bool),
  Number(f64),
  String(Rc<str>),
  List(Rc<Vec<Value>>),
  Object(Rc<HashMap<Rc<str>, Value>>),
  Closure(Closure),
}

#[derive(Debug, Clone)]
pub struct Closure {
  pub node: Rc<ClosureNode>,
  pub env: Rc<Environment>,
}

impl PartialEq for Closure {
  fn eq(&self, other: &Self) -> bool {
    self.node == other.node && Rc::ptr_eq(&self.env, &other.env)
  }
}

impl fmt::Display for Value {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Value::Nil => write!(f, "nil"),
      Value::Bool(b) => write!(f, "{b}"),
      Value::Number(n) => write!(f, "{n}"),
      Value::String(s) => write!(f, "\"{s}\""),
      Value::List(items) => {
        write!(f, "[")?;
        for (i, item) in items.iter().enumerate() {
          if i > 0 {
            write!(f, ", ")?;
          }
          write!(f, "{item}")?;
        }
        write!(f, "]")
      }
      Value::Object(entries) => {
        write!(f, "{{")?;
        for (i, (key, value)) in entries.iter().enumerate() {
          if i > 0 {
            write!(f, ", ")?;
          }
          write!(f, "\"{key}\": {value}")?;
        }
        write!(f, "}}")
      }
      Value::Closure(_) => write!(f, "closure"),
    }
  }
}
