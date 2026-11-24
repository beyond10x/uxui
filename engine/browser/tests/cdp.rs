//! The DevTools-protocol client over a scripted transport: commands, responses, events, errors and
//! the request guard that keeps a run on its own server.

use std::collections::VecDeque;
use std::time::{Duration, Instant};
use ux_browser::cdp::{self, Connection, Guard, Incoming, Transport};
use ux_browser::json;
use uxui_doc::yaml::Value;

/// Hands out scripted messages one per `recv` and keeps every sent text.
#[derive(Default)]
struct Script {
    incoming: VecDeque<String>,
    sent: Vec<String>,
}

impl Transport for Script {
    fn send(&mut self, text: &str) -> Result<(), String> {
        self.sent.push(text.to_string());
        Ok(())
    }
    fn recv(&mut self, _deadline: Instant) -> Result<Option<String>, String> {
        Ok(self.incoming.pop_front())
    }
}

fn script(lines: &[&str]) -> Script {
    Script {
        incoming: lines.iter().map(|s| s.to_string()).collect(),
        sent: Vec::new(),
    }
}

fn map(entries: Vec<(&str, Value)>) -> Value {
    Value::Map(
        entries
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    )
}

#[test]
fn a_command_carries_id_method_params_and_session() {
    let text = cdp::command(
        3,
        "Runtime.evaluate",
        &map(vec![("expression", Value::Str("1+1".into()))]),
        Some("S1"),
    );
    let v = json::parse(&text).unwrap();
    assert_eq!(v.get("id"), Some(&Value::Int(3)));
    assert_eq!(v.get_str("method"), Some("Runtime.evaluate"));
    assert_eq!(v.get_str("sessionId"), Some("S1"));
    assert_eq!(v.get("params").unwrap().get_str("expression"), Some("1+1"));
    let browser = json::parse(&cdp::command(4, "Browser.getVersion", &map(vec![]), None)).unwrap();
    assert_eq!(browser.get("sessionId"), None);
}

#[test]
fn reads_responses_errors_and_events() {
    match cdp::parse_incoming(r#"{"id":2,"result":{"a":1},"sessionId":"S"}"#).unwrap() {
        Incoming::Response {
            id,
            session,
            result,
        } => {
            assert_eq!((id, session.as_deref()), (2, Some("S")));
            assert_eq!(result.unwrap().get("a"), Some(&Value::Int(1)));
        }
        other => panic!("not a response: {other:?}"),
    }
    match cdp::parse_incoming(
        r#"{"id":5,"error":{"code":-32000,"message":"No node with given id"}}"#,
    )
    .unwrap()
    {
        Incoming::Response { result, .. } => {
            assert_eq!(result.unwrap_err(), "-32000: No node with given id");
        }
        other => panic!("not a response: {other:?}"),
    }
    match cdp::parse_incoming(
        r#"{"method":"Page.loadEventFired","params":{"timestamp":1.5},"sessionId":"S"}"#,
    )
    .unwrap()
    {
        Incoming::Event {
            method,
            params,
            session,
        } => {
            assert_eq!(
                (method.as_str(), session.as_deref()),
                ("Page.loadEventFired", Some("S"))
            );
            assert_eq!(params.get("timestamp"), Some(&Value::Float(1.5)));
        }
        other => panic!("not an event: {other:?}"),
    }
    assert!(cdp::parse_incoming(r#"{"neither":true}"#).is_err());
}

#[test]
fn a_call_waits_for_its_own_id_and_keeps_the_events_on_the_way() {
    let transport = script(&[
        r#"{"method":"Page.frameNavigated","params":{"n":1},"sessionId":"S"}"#,
        r#"{"id":99,"result":{}}"#,
        r#"{"id":1,"result":{"frameId":"F"},"sessionId":"S"}"#,
    ]);
    let mut conn = Connection::new(transport);
    let result = conn
        .call(
            Some("S"),
            "Page.navigate",
            map(vec![("url", Value::Str("about:blank".into()))]),
        )
        .unwrap();
    assert_eq!(result.get_str("frameId"), Some("F"));
    let event = conn
        .wait_event(Some("S"), "Page.frameNavigated", Duration::from_millis(10))
        .unwrap();
    assert_eq!(event.get("n"), Some(&Value::Int(1)));
}

#[test]
fn an_error_response_is_an_error_naming_the_method() {
    let transport = script(&[r#"{"id":1,"error":{"code":-32601,"message":"'X.y' wasn't found"}}"#]);
    let mut conn = Connection::new(transport);
    let err = conn.call(None, "X.y", map(vec![])).unwrap_err();
    assert!(err.contains("X.y") && err.contains("-32601"), "{err}");
}

#[test]
fn a_call_with_no_answer_times_out() {
    let mut conn = Connection::new(script(&[]));
    conn.set_timeout(Duration::from_millis(1));
    let err = conn
        .call(None, "Browser.getVersion", map(vec![]))
        .unwrap_err();
    assert!(err.contains("Browser.getVersion"), "{err}");
}

#[test]
fn the_guard_lets_its_origin_through_and_fails_everything_else() {
    let transport = script(&[
        r#"{"method":"Fetch.requestPaused","params":{"requestId":"r1","request":{"url":"http://127.0.0.1:4000/llm/"}},"sessionId":"S"}"#,
        r#"{"method":"Fetch.requestPaused","params":{"requestId":"r2","request":{"url":"https://fonts.example.org/a.woff2"}},"sessionId":"S"}"#,
        r#"{"method":"Fetch.requestPaused","params":{"requestId":"r3","request":{"url":"data:image/png;base64,AA=="}},"sessionId":"S"}"#,
        r#"{"id":1,"result":{}}"#,
    ]);
    let mut conn = Connection::new(transport);
    conn.guard = Some(Guard {
        origin: "http://127.0.0.1:4000".to_string(),
    });
    conn.call(None, "Browser.getVersion", map(vec![])).unwrap();
    let sent: Vec<Value> = conn
        .transport()
        .sent
        .iter()
        .map(|t| json::parse(t).unwrap())
        .collect();
    let answers: Vec<(String, String)> = sent
        .iter()
        .filter(|v| v.get_str("method").is_some_and(|m| m.starts_with("Fetch.")))
        .map(|v| {
            (
                v.get_str("method").unwrap().to_string(),
                v.get("params")
                    .unwrap()
                    .get_str("requestId")
                    .unwrap()
                    .to_string(),
            )
        })
        .collect();
    assert_eq!(
        answers,
        [
            ("Fetch.continueRequest".to_string(), "r1".to_string()),
            ("Fetch.failRequest".to_string(), "r2".to_string()),
            ("Fetch.continueRequest".to_string(), "r3".to_string()),
        ]
    );
    assert!(sent.iter().all(|v| v.get_str("sessionId") == Some("S")
        || v.get_str("method") == Some("Browser.getVersion")));
    assert_eq!(conn.blocked, ["https://fonts.example.org/a.woff2"]);
}

#[test]
fn an_origin_prefix_is_not_its_origin() {
    let guard = Guard {
        origin: "http://127.0.0.1:4000".to_string(),
    };
    assert!(guard.allows("http://127.0.0.1:4000/"));
    assert!(guard.allows("blob:http://127.0.0.1:4000/x"));
    assert!(!guard.allows("http://127.0.0.1:40001/"));
    assert!(!guard.allows("http://127.0.0.1:4000.example.org/"));
}
