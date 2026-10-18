use std::rc::Rc;

use crate::core::{
  evaluator::{Evaluator, is_truthy},
  value::{NativeFunction, Value},
};

use super::value::Closure;

pub fn register_stdlib(evaluator: &Evaluator) {
  evaluator.register_native_function(
    "hello".into(),
    NativeFunction {
      function: Rc::new(|_, _| Ok(Value::String("hello".into()))),
      arity: 0,
    },
  );

  evaluator.register_native_function(
    "map".into(),
    NativeFunction {
      function: Rc::new(|eval, args| {
        let (list, closure) = require_list_and_closure(args)?;

        let mut vec = Vec::new();
        for item in list.iter() {
          vec.push(eval.call_closure(&closure, std::slice::from_ref(&item))?);
        }

        Ok(Value::List(Rc::new(vec)))
      }),
      arity: 2,
    },
  );

  evaluator.register_native_function(
    "filter".into(),
    NativeFunction {
      function: Rc::new(|eval, args| {
        let (list, closure) = require_list_and_closure(args)?;

        let mut vec = Vec::new();
        for item in list.iter().cloned() {
          let keep = eval.call_closure(&closure, std::slice::from_ref(&item))?;
          if is_truthy(&keep) {
            vec.push(item);
          }
        }

        Ok(Value::List(Rc::new(vec)))
      }),
      arity: 2,
    },
  );

  evaluator.register_native_function(
    "fold".into(),
    NativeFunction {
      function: Rc::new(|eval, args| {
        let Value::List(list) = args[0].clone() else {
          return Err("Expected array".to_string());
        };
        let mut acc = args[1].clone();
        let Value::Closure(closure) = args[2].clone() else {
          return Err("Expected closure".to_string());
        };

        for item in list.iter().cloned() {
          acc = eval.call_closure(&closure, &[acc, item])?;
        }

        Ok(acc)
      }),
      arity: 3,
    },
  );

  evaluator.register_native_function(
    "reduce".into(),
    NativeFunction {
      function: Rc::new(|eval, args| {
        let (list, closure) = require_list_and_closure(args)?;

        let mut iter = list.iter();
        let mut acc = iter
          .next()
          .cloned()
          .ok_or("Cannot reduce an empty array.")?;
        for item in iter.cloned() {
          acc = eval.call_closure(&closure, &[acc, item])?;
        }

        Ok(acc)
      }),
      arity: 2,
    },
  );

  evaluator.register_native_function(
    "any".into(),
    NativeFunction {
      function: Rc::new(|eval, args| {
        let (list, closure) = require_list_and_closure(args)?;

        for item in list.iter() {
          if is_truthy(&eval.call_closure(&closure, std::slice::from_ref(&item))?) {
            return Ok(Value::Bool(true));
          }
        }

        Ok(Value::Bool(false))
      }),
      arity: 2,
    },
  );

  evaluator.register_native_function(
    "every".into(),
    NativeFunction {
      function: Rc::new(|eval, args| {
        let (list, closure) = require_list_and_closure(args)?;

        for item in list.iter() {
          if !is_truthy(&eval.call_closure(&closure, std::slice::from_ref(&item))?) {
            return Ok(Value::Bool(false));
          }
        }

        Ok(Value::Bool(true))
      }),
      arity: 2,
    },
  );

  evaluator.register_native_function(
    "find".into(),
    NativeFunction {
      function: Rc::new(|eval, args| {
        let (list, closure) = require_list_and_closure(args)?;

        for item in list.iter().cloned() {
          if is_truthy(&eval.call_closure(&closure, std::slice::from_ref(&item))?) {
            return Ok(item);
          }
        }

        Ok(Value::Nil)
      }),
      arity: 2,
    },
  );
}

fn require_list_and_closure(args: &[Value]) -> Result<(Rc<Vec<Value>>, Closure), String> {
  let Value::List(list) = args[0].clone() else {
    return Err("Expected array".to_string());
  };
  let Value::Closure(closure) = args[1].clone() else {
    return Err("Expected closure".to_string());
  };

  Ok((list, closure))
}
