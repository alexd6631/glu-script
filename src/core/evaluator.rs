use std::{collections::HashMap, rc::Rc};

use crate::core::{
  ast::{AstNode, AstObjectKey, BOpType, ClosureNode, IfNode, LOpType, UOpType},
  environment::Environment,
  value::{Closure, Value},
};

pub const MAX_CALL_DEPTH: usize = 1024;

pub struct Evaluator {
  environment: Rc<Environment>,
  call_depth: usize,
}

impl Evaluator {
  pub fn new() -> Self {
    Self {
      environment: Rc::new(Environment::default()),
      call_depth: 0,
    }
  }

  pub fn evaluate(&mut self, node: &AstNode) -> Result<Value, String> {
    match node {
      AstNode::Literal(l) => Ok(l.clone()),
      AstNode::List(items) => self.list(items),
      AstNode::Object(entries) => self.object(entries),
      AstNode::Block(items) => self.block(items),
      AstNode::Binary(op_type, left, right) => self.binary(*op_type, left, right),
      AstNode::Logical(op_type, left, right) => self.logical(*op_type, left, right),
      AstNode::Index(target, index) => self.index(target, index),
      AstNode::Property(target, name) => self.property(target, name),
      AstNode::Unary(op_type, ast_node) => self.unary(*op_type, ast_node),
      AstNode::Binding(id, expr) => self.binding(id, expr),
      AstNode::VariableRef(id) => self.variable_ref(id),
      AstNode::If(node) => self.if_expression(node),
      AstNode::Closure(node) => self.closure(node.clone()),
      AstNode::Call(callee, args) => self.call(callee, args),
      AstNode::Error => Err("Invalid expression.".to_string()),
    }
  }

  fn binding(&mut self, id: &Rc<str>, expr: &AstNode) -> Result<Value, String> {
    let value = self.evaluate(expr)?;
    self.environment.set(id.clone(), value.clone());
    Ok(value)
  }

  fn variable_ref(&mut self, id: &Rc<str>) -> Result<Value, String> {
    self
      .environment
      .get(id)
      .ok_or_else(|| format!("Undefined variable '{id}'."))
  }

  fn unary(&mut self, op_type: UOpType, node: &AstNode) -> Result<Value, String> {
    let value = self.evaluate(node)?;

    match op_type {
      UOpType::Not => Ok(Value::Bool(!is_truthy(&value))),
      UOpType::Negate => match value {
        Value::Number(n) => Ok(Value::Number(-n)),
        _ => Err("Operand must be a number.".to_string()),
      },
    }
  }

  fn logical(
    &mut self,
    op_type: LOpType,
    left: &AstNode,
    right: &AstNode,
  ) -> Result<Value, String> {
    let lhs = self.evaluate(left)?;

    match op_type {
      LOpType::And => {
        if is_truthy(&lhs) {
          self.evaluate(right)
        } else {
          Ok(lhs)
        }
      }
      LOpType::Or => {
        if is_truthy(&lhs) {
          Ok(lhs)
        } else {
          self.evaluate(right)
        }
      }
    }
  }

  fn binary(&mut self, op_type: BOpType, left: &AstNode, right: &AstNode) -> Result<Value, String> {
    let lhs = self.evaluate(left)?;
    let rhs = self.evaluate(right)?;

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

  fn list(&mut self, nodes: &[AstNode]) -> Result<Value, String> {
    let mut values = Vec::with_capacity(nodes.len());

    for node in nodes {
      values.push(self.evaluate(node)?);
    }

    Ok(Value::List(Rc::new(values)))
  }

  fn object(&mut self, entries: &[(AstObjectKey, AstNode)]) -> Result<Value, String> {
    let mut fields = HashMap::with_capacity(entries.len());

    for (key, value) in entries {
      let key = match key {
        AstObjectKey::Literal(k) => k.clone(),
        AstObjectKey::Computed(node) => match self.evaluate(node)? {
          Value::String(s) => s,
          _ => return Err("Object keys must be strings.".to_string()),
        },
      };

      fields.insert(key, self.evaluate(value)?);
    }

    Ok(Value::Object(Rc::new(fields)))
  }

  fn index(&mut self, target: &AstNode, index: &AstNode) -> Result<Value, String> {
    let target = self.evaluate(target)?;
    let index = self.evaluate(index)?;

    let idx = match index {
      Value::Number(n) if n >= 0.0 && n.fract() == 0.0 => n as usize,
      Value::Number(_) => return Err("Index must be a non-negative integer.".to_string()),
      _ => return Err("Index must be a number.".to_string()),
    };

    match target {
      Value::List(items) => items
        .get(idx)
        .cloned()
        .ok_or_else(|| "Index out of range.".to_string()),
      Value::String(s) => s
        .chars()
        .nth(idx)
        .map(|c| Value::String(c.to_string().into()))
        .ok_or_else(|| "Index out of range.".to_string()),
      _ => Err("Only lists and strings can be indexed.".to_string()),
    }
  }

  fn property(&mut self, target: &AstNode, name: &Rc<str>) -> Result<Value, String> {
    match self.evaluate(target)? {
      Value::Object(fields) => fields
        .get(name)
        .cloned()
        .ok_or_else(|| format!("Undefined property '{name}'.")),
      _ => Err("Only objects have properties.".to_string()),
    }
  }

  fn block(&mut self, nodes: &[AstNode]) -> Result<Value, String> {
    let enclosing = self.environment.clone();
    self.environment = Rc::new(Environment::new(enclosing.clone()));

    let mut result = Value::Nil;

    for node in nodes {
      result = self.evaluate(node)?;
    }

    self.environment = enclosing;
    Ok(result)
  }

  fn if_expression(&mut self, node: &IfNode) -> Result<Value, String> {
    if is_truthy(&self.evaluate(&node.condition)?) {
      self.evaluate(&node.then_branch)
    } else if let Some(else_branch) = &node.else_branch {
      self.evaluate(else_branch)
    } else {
      Ok(Value::Nil)
    }
  }

  fn closure(&mut self, node: Rc<ClosureNode>) -> Result<Value, String> {
    let env = self.environment.clone();
    Ok(Value::Closure(Closure { node, env }))
  }

  fn call(&mut self, callee: &AstNode, args: &[AstNode]) -> Result<Value, String> {
    let callee_value = self.evaluate(callee)?;

    let Value::Closure(closure) = callee_value else {
      return Err("Only closures can be called.".to_string());
    };

    if closure.node.params.len() != args.len() {
      return Err(format!(
        "Expected {} argument(s) but got {}.",
        closure.node.params.len(),
        args.len()
      ));
    }

    let mut arg_values = Vec::with_capacity(args.len());
    for arg in args {
      arg_values.push(self.evaluate(arg)?);
    }

    let call_env = Rc::new(Environment::new(closure.env.clone()));

    for (param, value) in closure.node.params.iter().zip(arg_values) {
      call_env.set(param.clone(), value);
    }

    if self.call_depth >= MAX_CALL_DEPTH {
      return Err(format!("Recursion limit of {MAX_CALL_DEPTH} exceeded."));
    }

    let saved_env = self.environment.clone();
    self.environment = call_env;

    self.call_depth += 1;
    let result = self.evaluate(&closure.node.body);
    self.call_depth -= 1;

    self.environment = saved_env;

    result
  }
}

impl Default for Evaluator {
  fn default() -> Self {
    Self::new()
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

#[cfg(test)]
mod tests {
  use super::*;

  fn evaluate(source: &str) -> Result<Value, String> {
    let ast = crate::core::compiler::Compiler::new(source)
      .compile()
      .unwrap();

    Evaluator::new().evaluate(&ast)
  }

  #[test]
  fn binding_and_reference() {
    assert_eq!(evaluate("{ let x = 1 + 2; x }"), Ok(Value::Number(3.0)));
  }

  #[test]
  fn binding_returns_bound_value() {
    assert_eq!(evaluate("{ let x = 5 }"), Ok(Value::Number(5.0)));
  }

  #[test]
  fn nested_block_shadowing() {
    assert_eq!(
      evaluate("{ let x = 1; { let x = 2; x } + x }"),
      Ok(Value::Number(3.0))
    );
  }

  #[test]
  fn undefined_variable() {
    assert_eq!(
      evaluate("foo"),
      Err("Undefined variable 'foo'.".to_string())
    );
  }

  #[test]
  fn block_scopes_are_popped() {
    assert_eq!(
      evaluate("{ { let x = 1; 2 }; { x } }"),
      Err("Undefined variable 'x'.".to_string())
    );
  }

  #[test]
  fn list_indexed_access() {
    assert_eq!(evaluate("[10, 20, 30][1]"), Ok(Value::Number(20.0)));
  }

  #[test]
  fn string_indexed_access() {
    assert_eq!(evaluate("\"hello\"[1]"), Ok(Value::String("e".into())));
  }

  #[test]
  fn chained_indexed_access() {
    assert_eq!(evaluate("[[1, 2], [3, 4]][1][0]"), Ok(Value::Number(3.0)));
  }

  #[test]
  fn indexed_access_through_variable() {
    assert_eq!(evaluate("{ let l = [1, 2]; l[0] }"), Ok(Value::Number(1.0)));
  }

  #[test]
  fn index_out_of_range() {
    assert_eq!(
      evaluate("[1, 2][5]"),
      Err("Index out of range.".to_string())
    );
  }

  #[test]
  fn fractional_index() {
    assert_eq!(
      evaluate("[1, 2][0.5]"),
      Err("Index must be a non-negative integer.".to_string())
    );
  }

  #[test]
  fn non_number_index() {
    assert_eq!(
      evaluate("[1, 2][\"a\"]"),
      Err("Index must be a number.".to_string())
    );
  }

  #[test]
  fn non_indexable_target() {
    assert_eq!(
      evaluate("1[0]"),
      Err("Only lists and strings can be indexed.".to_string())
    );
  }

  #[test]
  fn object_property_access() {
    assert_eq!(evaluate("{a: 1}.a"), Ok(Value::Number(1.0)));
  }

  #[test]
  fn chained_property_access() {
    assert_eq!(
      evaluate(r#"{a: {b: "x"}}.a.b"#),
      Ok(Value::String("x".into()))
    );
  }

  #[test]
  fn property_access_through_variable() {
    assert_eq!(
      evaluate(r#"{ let o = {n: 1}; o.n }"#),
      Ok(Value::Number(1.0))
    );
  }

  #[test]
  fn missing_property() {
    assert_eq!(
      evaluate("{a: 1}.b"),
      Err("Undefined property 'b'.".to_string())
    );
  }

  #[test]
  fn non_object_property_target() {
    assert_eq!(
      evaluate("[1, 2].a"),
      Err("Only objects have properties.".to_string())
    );
  }

  #[test]
  fn if_truthy_condition() {
    assert_eq!(evaluate("if (true) 1 else 2"), Ok(Value::Number(1.0)));
  }

  #[test]
  fn if_falsy_condition() {
    assert_eq!(evaluate("if (false) 1 else 2"), Ok(Value::Number(2.0)));
  }

  #[test]
  fn if_no_else_falsy() {
    assert_eq!(evaluate("if (false) 1"), Ok(Value::Nil));
  }

  #[test]
  fn if_no_else_truthy() {
    assert_eq!(evaluate("if (true) 42"), Ok(Value::Number(42.0)));
  }

  #[test]
  fn if_truthiness_number() {
    assert_eq!(
      evaluate(r#"if (1) "yes" else "no""#),
      Ok(Value::String("yes".into()))
    );
  }

  #[test]
  fn if_truthiness_nil() {
    assert_eq!(evaluate("if (nil) 1 else 2"), Ok(Value::Number(2.0)));
  }

  #[test]
  fn if_else_if_chain() {
    assert_eq!(
      evaluate("if (false) 1 else if (true) 2 else 3"),
      Ok(Value::Number(2.0))
    );
  }

  #[test]
  fn if_else_if_chain_all_falsy() {
    assert_eq!(
      evaluate("if (false) 1 else if (false) 2 else 3"),
      Ok(Value::Number(3.0))
    );
  }

  #[test]
  fn if_inside_block() {
    assert_eq!(
      evaluate(r#"{ let x = 5; if (x > 3) "big" else "small" }"#),
      Ok(Value::String("big".into()))
    );
  }

  #[test]
  fn if_assigned_to_variable() {
    assert_eq!(
      evaluate("{ let x = if (true) 10 else 20; x }"),
      Ok(Value::Number(10.0))
    );
  }

  #[test]
  fn if_with_block_condition_and_then() {
    assert_eq!(
      evaluate("if ({ let x = 5; x > 3 }) { let y = 10; y * 2 } else { 0 }"),
      Ok(Value::Number(20.0))
    );
  }

  #[test]
  fn if_with_block_condition_and_else() {
    assert_eq!(
      evaluate("if ({ let x = 1; x > 3 }) { 10 } else { let y = 7; y + 1 }"),
      Ok(Value::Number(8.0))
    );
  }

  #[test]
  fn closure_evaluates_to_closure_value() {
    let result = evaluate("|x| x + 1").unwrap();
    assert!(matches!(result, Value::Closure(_)));
  }

  #[test]
  fn closure_no_params() {
    let result = evaluate("|| 42").unwrap();
    let Value::Closure(c) = result else {
      panic!("expected closure");
    };
    assert!(c.node.params.is_empty());
    assert_eq!(*c.node.body, AstNode::Literal(Value::Number(42.0)));
  }

  #[test]
  fn closure_captures_params() {
    let result = evaluate("|a, b| a + b").unwrap();
    let Value::Closure(c) = result else {
      panic!("expected closure");
    };
    assert_eq!(c.node.params.len(), 2);
    assert_eq!(&*c.node.params[0], "a");
    assert_eq!(&*c.node.params[1], "b");
  }

  #[test]
  fn closure_display() {
    let result = evaluate("|x| x").unwrap();
    assert_eq!(result.to_string(), "closure");
  }

  #[test]
  fn closure_in_binding() {
    let result = evaluate("{ let f = |x| x * 2; f }").unwrap();
    assert!(matches!(result, Value::Closure(_)));
  }

  #[test]
  fn closure_with_block_body() {
    let result = evaluate("|x| { let y = x; y + 1 }").unwrap();
    let Value::Closure(c) = result else {
      panic!("expected closure");
    };
    assert_eq!(c.node.params.len(), 1);
    assert!(matches!(*c.node.body, AstNode::Block(_)));
  }

  #[test]
  fn call_closure_basic() {
    assert_eq!(evaluate("(|x| x + 1)(5)"), Ok(Value::Number(6.0)));
  }

  #[test]
  fn call_closure_multiple_args() {
    assert_eq!(evaluate("(|x, y| x + y)(3, 4)"), Ok(Value::Number(7.0)));
  }

  #[test]
  fn call_closure_no_args() {
    assert_eq!(evaluate("(|| 42)()"), Ok(Value::Number(42.0)));
  }

  #[test]
  fn call_closure_in_binding() {
    assert_eq!(
      evaluate("{ let f = |x| x * 2; f(21) }"),
      Ok(Value::Number(42.0))
    );
  }

  #[test]
  fn call_closure_with_block_body() {
    assert_eq!(
      evaluate("(|x| { let y = x; y + 1 })(5)"),
      Ok(Value::Number(6.0))
    );
  }

  #[test]
  fn call_closure_captures_environment() {
    assert_eq!(
      evaluate("{ let z = 10; let f = |x| x + z; f(5) }"),
      Ok(Value::Number(15.0))
    );
  }

  #[test]
  fn call_wrong_arg_count_too_few() {
    assert_eq!(
      evaluate("(|x, y| x + y)(1)"),
      Err("Expected 2 argument(s) but got 1.".to_string())
    );
  }

  #[test]
  fn call_wrong_arg_count_too_many() {
    assert_eq!(
      evaluate("(|x| x)(1, 2)"),
      Err("Expected 1 argument(s) but got 2.".to_string())
    );
  }

  #[test]
  fn call_non_closure() {
    assert_eq!(
      evaluate("(1)(2)"),
      Err("Only closures can be called.".to_string())
    );
  }

  #[test]
  fn call_chained_closures() {
    assert_eq!(evaluate("(|x| |y| x + y)(3)(4)"), Ok(Value::Number(7.0)));
  }

  #[test]
  fn call_nested() {
    assert_eq!(
      evaluate("{ let add = |a, b| a + b; add(add(1, 2), 3) }"),
      Ok(Value::Number(6.0))
    );
  }

  #[test]
  fn call_args_evaluated_in_caller_env() {
    assert_eq!(
      evaluate("{ let a = 10; let f = |x| x + a; { let a = 100; f(1) } }"),
      Ok(Value::Number(11.0))
    );
  }

  #[test]
  fn call_depth_limit_returns_error() {
    assert_eq!(
      evaluate("{ let f = |n| f(n); f(0) }"),
      Err(format!("Recursion limit of {MAX_CALL_DEPTH} exceeded."))
    );
  }

  #[test]
  fn call_within_depth_limit_succeeds() {
    std::thread::Builder::new()
      .stack_size(64 * 1024 * 1024)
      .spawn(move || {
        let source = format!(
          "{{ let f = |n| if (n == 0) 0 else f(n - 1); f({}) }}",
          MAX_CALL_DEPTH - 1
        );
        assert_eq!(evaluate(&source), Ok(Value::Number(0.0)));
      })
      .unwrap()
      .join()
      .unwrap();
  }

  #[test]
  fn call_depth_exceeding_limit_returns_error() {
    std::thread::Builder::new()
      .stack_size(64 * 1024 * 1024)
      .spawn(move || {
        let source = format!(
          "{{ let f = |n| if (n == 0) 0 else f(n - 1); f({}) }}",
          MAX_CALL_DEPTH
        );
        assert_eq!(
          evaluate(&source),
          Err(format!("Recursion limit of {MAX_CALL_DEPTH} exceeded."))
        );
      })
      .unwrap()
      .join()
      .unwrap();
  }
}
