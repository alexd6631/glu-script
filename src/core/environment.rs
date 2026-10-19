use std::{
  collections::HashMap,
  sync::{Arc, RwLock},
};

use crate::core::value::Value;

#[derive(Debug, Default)]
pub struct Environment {
  enclosing: Option<Arc<Environment>>,
  values: RwLock<HashMap<Arc<str>, Value>>,
}

impl Environment {
  pub fn new(enclosing: Arc<Environment>) -> Self {
    Self {
      enclosing: Some(enclosing),
      values: Default::default(),
    }
  }

  pub fn get(&self, key: &str) -> Option<Value> {
    if let Some(value) = self.values.read().unwrap().get(key) {
      Some(value.clone())
    } else if let Some(enclosing) = &self.enclosing {
      enclosing.get(key)
    } else {
      None
    }
  }

  pub fn set(&self, key: Arc<str>, value: Value) {
    self.values.write().unwrap().insert(key, value);
  }
}
