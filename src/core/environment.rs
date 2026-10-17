use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::core::value::Value;

#[derive(Debug, Default)]
pub struct Environment {
  enclosing: Option<Rc<Environment>>,
  values: RefCell<HashMap<Rc<str>, Value>>,
}

impl Environment {
  pub fn new(enclosing: Rc<Environment>) -> Self {
    Self {
      enclosing: Some(enclosing),
      values: Default::default(),
    }
  }

  pub fn get(&self, key: &str) -> Option<Value> {
    if let Some(value) = self.values.borrow().get(key) {
      Some(value.clone())
    } else if let Some(enclosing) = &self.enclosing {
      enclosing.get(key)
    } else {
      None
    }
  }

  pub fn set(&self, key: Rc<str>, value: Value) {
    self.values.borrow_mut().insert(key, value);
  }
}
