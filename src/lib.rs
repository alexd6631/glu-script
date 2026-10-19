#![deny(clippy::all)]
pub mod core;

use std::{collections::HashMap, ptr, rc::Rc};

use napi::{Env, Unknown, bindgen_prelude::*, check_status, sys};
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

#[napi(ts_return_type = "Promise<unknown>")]
pub async fn test_evaluate_async(s: String) -> Result<GluValue> {
  let res = napi::tokio::task::spawn_blocking(move || {
    let mut compiler = Compiler::new(&s);
    if let Some(ast) = compiler.compile() {
      Evaluator::new()
        .evaluate(&ast)
        .map(|v| SendValue::from_value(&v))
        .map_err(|e| {
          Error::new(
            Status::InvalidArg,
            format!("failed to evaluate expression: {e}"),
          )
        })
    } else {
      Err(Error::new(Status::InvalidArg, "failed to parse expression"))
    }
  })
  .await
  .map_err(|e| Error::new(Status::GenericFailure, format!("evaluator panicked: {e}")))?;

  Ok(GluValue(res?))
}

pub struct GluValue(SendValue);

impl TypeName for GluValue {
  fn type_name() -> &'static str {
    "unknown"
  }

  fn value_type() -> ValueType {
    ValueType::Unknown
  }
}

impl ValidateNapiValue for GluValue {}

impl ToNapiValue for GluValue {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    unsafe { send_value_to_raw(env, &val.0) }
  }
}

#[derive(Debug)]
enum SendValue {
  Nil,
  Bool(bool),
  Number(f64),
  String(String),
  List(Vec<SendValue>),
  Object(Vec<(String, SendValue)>),
  Closure,
  Native,
}

impl SendValue {
  fn from_value(v: &Value) -> Self {
    match v {
      Value::Nil => Self::Nil,
      Value::Bool(b) => Self::Bool(*b),
      Value::Number(n) => Self::Number(*n),
      Value::String(s) => Self::String(s.to_string()),
      Value::List(values) => Self::List(values.iter().map(Self::from_value).collect()),
      Value::Object(hash_map) => Self::Object(
        hash_map
          .iter()
          .map(|(k, v)| (k.to_string(), Self::from_value(v)))
          .collect(),
      ),
      Value::Closure(_) => Self::Closure,
      Value::Native(_) => Self::Native,
    }
  }
}

unsafe fn send_value_to_raw(env: sys::napi_env, v: &SendValue) -> Result<sys::napi_value> {
  unsafe {
    let mut out = ptr::null_mut();
    match v {
      SendValue::Nil => check_status!(sys::napi_get_null(env, &mut out))?,
      SendValue::Bool(b) => check_status!(sys::napi_get_boolean(env, *b, &mut out))?,
      SendValue::Number(n) => check_status!(sys::napi_create_double(env, *n, &mut out))?,
      SendValue::String(s) => {
        check_status!(sys::napi_create_string_utf8(
          env,
          s.as_ptr() as *const std::os::raw::c_char,
          s.len() as isize,
          &mut out,
        ))?;
      }
      SendValue::List(values) => {
        check_status!(sys::napi_create_array_with_length(
          env,
          values.len(),
          &mut out,
        ))?;
        for (i, item) in values.iter().enumerate() {
          check_status!(sys::napi_set_element(
            env,
            out,
            i as u32,
            send_value_to_raw(env, item)?,
          ))?;
        }
      }
      SendValue::Object(entries) => {
        check_status!(sys::napi_create_object(env, &mut out))?;
        for (key, value) in entries.iter() {
          let mut raw_key = ptr::null_mut();
          check_status!(sys::napi_create_string_utf8(
            env,
            key.as_ptr() as *const std::os::raw::c_char,
            key.len() as isize,
            &mut raw_key,
          ))?;
          check_status!(sys::napi_set_property(
            env,
            out,
            raw_key,
            send_value_to_raw(env, value)?,
          ))?;
        }
      }
      SendValue::Closure => {
        check_status!(sys::napi_create_string_utf8(
          env,
          c"<closure>".as_ptr() as *const std::os::raw::c_char,
          9,
          &mut out,
        ))?;
      }
      SendValue::Native => {
        check_status!(sys::napi_create_string_utf8(
          env,
          c"<native_fn>".as_ptr() as *const std::os::raw::c_char,
          11,
          &mut out,
        ))?;
      }
    }

    Ok(out)
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

fn to_array(env: &Env, values: Rc<Vec<Value>>) -> Result<Unknown<'_>> {
  let mut array = env.create_array(values.len() as u32)?;
  for (i, obj) in values.iter().enumerate() {
    array.set(i as u32, to_js(obj.clone(), env)?)?;
  }
  array.into_unknown(env)
}

fn to_object(env: &Env, hash_map: Rc<HashMap<Rc<str>, Value>>) -> Result<Unknown<'_>> {
  let mut object = Object::new(env)?;
  for (key, value) in hash_map.iter() {
    object.set(key.as_ref(), to_js(value.clone(), env)?)?;
  }
  object.into_unknown(env)
}
