#![deny(clippy::all)]
pub mod core;

use std::{collections::HashMap, sync::Arc};

use napi::{Env, Unknown, bindgen_prelude::*, sys};
use napi_derive::napi;

use crate::core::{compiler::Compiler, evaluator::Evaluator, scanner::Scanner, value::Value};

#[napi]
pub fn test_scan(s: String) {
  let scanner = Scanner::new(&s);
  for token in scanner {
    dbg!(token);
  }
}

#[napi]
pub fn test_parse(s: String) {
  let mut compiler = Compiler::new(&s);
  let ast = compiler.compile();

  dbg!(ast);
}

#[napi]
pub fn test_evaluate(env: &Env, s: String) -> Result<Unknown<'_>> {
  let mut compiler = Compiler::new(&s);
  if let Some(ast) = compiler.compile() {
    let res = Evaluator::new().evaluate(&ast).map_err(|e| {
      Error::new(
        Status::InvalidArg,
        format!("failed to evaluate expression: {e}"),
      )
    })?;

    to_js(res, env)
  } else {
    Err(Error::new(Status::InvalidArg, "failed to parse expression"))
  }
}

impl ToNapiValue for Value {
  unsafe fn to_napi_value(raw_env: sys::napi_env, v: Self) -> Result<sys::napi_value> {
    let env = Env::from_raw(raw_env);
    Ok(to_js(v, &env)?.raw())
  }
}

#[napi(ts_return_type = "Promise<unknown>")]
pub async fn test_evaluate_async(s: String) -> Result<Value> {
  let mut compiler = Compiler::new(&s);
  if let Some(ast) = compiler.compile() {
    Evaluator::new().evaluate(&ast).map_err(|e| {
      Error::new(
        Status::InvalidArg,
        format!("failed to evaluate expression: {e}"),
      )
    })
  } else {
    Err(Error::new(Status::InvalidArg, "failed to parse expression"))
  }
}

fn to_js(v: Value, env: &Env) -> Result<Unknown<'_>> {
  let js_value = match v {
    Value::Nil => ().into_unknown(env)?,
    Value::Bool(b) => b.into_unknown(env)?,
    Value::Number(n) => n.into_unknown(env)?,
    Value::String(s) => s.into_unknown(env)?,
    Value::List(values) => to_array(env, values)?,
    Value::Object(hash_map) => to_object(env, hash_map)?,
    Value::Closure(_) => "<closure>".into_unknown(env)?,
    Value::Native(_) => "<native_fn>".into_unknown(env)?,
  };

  Ok(js_value)
}

fn to_array(env: &Env, values: Arc<Vec<Value>>) -> Result<Unknown<'_>> {
  let mut array = env.create_array(values.len() as u32)?;
  for (i, obj) in values.iter().enumerate() {
    array.set(i as u32, to_js(obj.clone(), env)?)?;
  }
  array.into_unknown(env)
}

fn to_object(env: &Env, hash_map: Arc<HashMap<Arc<str>, Value>>) -> Result<Unknown<'_>> {
  let mut object = Object::new(env)?;
  for (key, value) in hash_map.iter() {
    object.set(key.as_ref(), to_js(value.clone(), env)?)?;
  }
  object.into_unknown(env)
}
