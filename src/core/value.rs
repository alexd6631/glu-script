use std::{collections::HashMap, rc::Rc};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
  Nil,
  Bool(bool),
  Number(f64),
  String(Rc<str>),
  List(Rc<Vec<Value>>),
  Object(Rc<HashMap<Rc<str>, Value>>),
}
