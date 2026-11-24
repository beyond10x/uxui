//! The JSON reader every DevTools-protocol message goes through.

use ux_browser::json;
use uxui_doc::yaml::Value;

#[test]
fn reads_nested_values_in_order() {
    let v =
        json::parse(r#" {"id": 7, "result": {"a": [1, -2.5, 3e2, true, false, null], "b": "x"}} "#)
            .unwrap();
    assert_eq!(v.get("id"), Some(&Value::Int(7)));
    let result = v.get("result").unwrap();
    assert_eq!(
        result.get("a").unwrap().list(),
        &[
            Value::Int(1),
            Value::Float(-2.5),
            Value::Float(300.0),
            Value::Bool(true),
            Value::Bool(false),
            Value::Null
        ]
    );
    let keys: Vec<&str> = result.entries().iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(keys, ["a", "b"]);
}

#[test]
fn reads_escapes_and_surrogate_pairs() {
    let v = json::parse(r#""a\"b\\c\/d\n\té😀""#).unwrap();
    assert_eq!(v, Value::Str("a\"b\\c/d\n\té\u{1f600}".to_string()));
}

#[test]
fn refuses_trailing_text_and_unclosed_values() {
    assert!(json::parse("{} x").is_err());
    assert!(json::parse("[1, 2").is_err());
    assert!(json::parse(r#"{"a" 1}"#).is_err());
    assert!(json::parse(r#""\ud83d""#).is_err());
}

#[test]
fn writes_what_it_reads() {
    let text = r#"{"a":[1,2.5,"x\n"],"b":{"c":null}}"#;
    assert_eq!(uxui_doc::json::write(&json::parse(text).unwrap()), text);
}
