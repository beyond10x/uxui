//! Small builders and readers over `uxui_doc::yaml::Value`, the one value type of the crate.

use uxui_doc::yaml::Value;

pub fn s(text: &str) -> Value {
    Value::Str(text.to_string())
}

pub fn map(entries: Vec<(&str, Value)>) -> Value {
    Value::Map(
        entries
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    )
}

/// A number as JSON writes it: an integer when it is one.
pub fn num(x: f64) -> Value {
    if x.fract() == 0.0 && x.abs() < 9.0e15 {
        Value::Int(x as i64)
    } else {
        Value::Float(x)
    }
}

pub fn f64_of(v: &Value) -> Option<f64> {
    match v {
        Value::Int(n) => Some(*n as f64),
        Value::Float(f) => Some(*f),
        _ => None,
    }
}

pub fn get_f64(v: &Value, key: &str) -> Option<f64> {
    v.get(key).and_then(f64_of)
}

pub fn get_i64(v: &Value, key: &str) -> Option<i64> {
    match v.get(key) {
        Some(Value::Int(n)) => Some(*n),
        _ => None,
    }
}

pub fn get_bool(v: &Value, key: &str) -> Option<bool> {
    match v.get(key) {
        Some(Value::Bool(b)) => Some(*b),
        _ => None,
    }
}

/// A list of strings under `key`; a missing key is an empty list.
pub fn get_strs(v: &Value, key: &str) -> Result<Vec<String>, String> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(Value::List(items)) => items
            .iter()
            .map(|i| {
                i.scalar_text()
                    .ok_or_else(|| format!("`{key}` holds a value that is not text"))
            })
            .collect(),
        Some(_) => Err(format!("`{key}` is not a list")),
    }
}

/// A decimal as text (ratchet's form): at most `places` decimals, no trailing zeros.
pub fn decimal(x: f64, places: usize) -> String {
    let text = format!("{x:.places$}");
    let text = if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        text
    };
    if text == "-0" { "0".to_string() } else { text }
}
