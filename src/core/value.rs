use std::{
  collections::HashMap,
  fmt::{self, Debug},
  sync::Arc,
};

use crate::core::{ast::ClosureNode, environment::Environment, evaluator::Evaluator};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
  Nil,
  Bool(bool),
  Number(f64),
  String(Arc<str>),
  List(Arc<Vec<Value>>),
  Object(Arc<HashMap<Arc<str>, Value>>),
  Closure(Closure),
  Native(NativeFunction),
}

#[derive(Debug, Clone)]
pub struct Closure {
  pub node: Arc<ClosureNode>,
  pub env: Arc<Environment>,
}

impl PartialEq for Closure {
  fn eq(&self, other: &Self) -> bool {
    self.node == other.node && Arc::ptr_eq(&self.env, &other.env)
  }
}

pub type NativeFn = dyn Fn(&mut Evaluator, &[Value]) -> Result<Value, String> + Send + Sync;

#[derive(Clone)]
pub struct NativeFunction {
  pub function: Arc<NativeFn>,
  pub arity: u16,
}

impl Debug for NativeFunction {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "<native>")
  }
}

impl PartialEq for NativeFunction {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.function, &other.function) && self.arity == other.arity
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
      Value::Closure(_) => write!(f, "<closure>"),
      Value::Native(_) => write!(f, "<native_fn>"),
    }
  }
}
