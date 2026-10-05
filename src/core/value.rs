use std::{collections::HashMap, rc::Rc};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
  Nil,
  Bool(bool),
  Number(f64),
  String(Rc<str>),
  List(Vec<Value>),
  Object(HashMap<Rc<str>, Value>),
}
