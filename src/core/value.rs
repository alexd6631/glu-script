use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
  Nil,
  Bool(bool),
  Number(f64),
  String(Rc<str>),
}
