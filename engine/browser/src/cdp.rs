//! A DevTools-protocol client: JSON commands with ids, responses matched by id, events queued until
//! asked for, and a request guard that keeps a page on its own server. Sessions are flat
//! (`Target.attachToTarget` with `flatten: true`): a command for a page carries its `sessionId`.

use std::collections::VecDeque;
use std::time::{Duration, Instant};
use uxui_doc::yaml::Value;

#[derive(Debug)]
pub enum Incoming {
    Response {
        id: i64,
        session: Option<String>,
        result: Result<Value, String>,
    },
    Event {
        method: String,
        params: Value,
        session: Option<String>,
    },
}

pub fn parse_incoming(text: &str) -> Result<Incoming, String> {
    let v = crate::json::parse(text)?;
    let session = v.get_str("sessionId").map(str::to_string);
    if let Some(Value::Int(id)) = v.get("id") {
        let result = match (v.get("result"), v.get("error")) {
            (_, Some(error)) => Err(format!(
                "{}: {}",
                error
                    .get("code")
                    .and_then(Value::scalar_text)
                    .unwrap_or_default(),
                error.get_str("message").unwrap_or("no message")
            )),
            (Some(result), None) => Ok(result.clone()),
            (None, None) => Ok(Value::Map(Vec::new())),
        };
        return Ok(Incoming::Response {
            id: *id,
            session,
            result,
        });
    }
    if let Some(method) = v.get_str("method") {
        return Ok(Incoming::Event {
            method: method.to_string(),
            params: v.get("params").cloned().unwrap_or(Value::Map(Vec::new())),
            session,
        });
    }
    Err(format!(
        "a DevTools message that is neither a response nor an event: {text}"
    ))
}

pub fn command(id: i64, method: &str, params: &Value, session: Option<&str>) -> String {
    let mut entries = vec![
        ("id".to_string(), Value::Int(id)),
        ("method".to_string(), Value::Str(method.to_string())),
        ("params".to_string(), params.clone()),
    ];
    if let Some(s) = session {
        entries.push(("sessionId".to_string(), Value::Str(s.to_string())));
    }
    uxui_doc::json::write(&Value::Map(entries))
}

/// Where messages go and come from: the WebSocket to the browser, or a script in tests.
pub trait Transport {
    fn send(&mut self, text: &str) -> Result<(), String>;
    /// The next message, or `None` when none arrived by `deadline`.
    fn recv(&mut self, deadline: Instant) -> Result<Option<String>, String>;
}

impl<S: crate::ws::Stream> Transport for crate::ws::WebSocket<S> {
    fn send(&mut self, text: &str) -> Result<(), String> {
        self.send_text(text)
    }
    fn recv(&mut self, deadline: Instant) -> Result<Option<String>, String> {
        crate::ws::WebSocket::recv(self, deadline)
    }
}

/// Lets requests to one origin through (and `data:` URLs, and `blob:` URLs of it), fails every
/// other: a measured page reaches nothing but the server under test. To that origin only the
/// declared methods go (by default `GET` and `HEAD`): a measurement does not change the state of
/// the server it measures.
#[derive(Clone, Debug)]
pub struct Guard {
    /// `scheme://host:port`, without a trailing slash.
    pub origin: String,
    /// The HTTP methods a request to the origin may have.
    pub methods: Vec<String>,
}

impl Guard {
    pub fn new(origin: &str, methods: &[String]) -> Guard {
        Guard {
            origin: origin.to_string(),
            methods: methods.to_vec(),
        }
    }

    /// Whether a request with `method` to `url` goes on: a `data:` URL always; a URL of the origin
    /// with a declared method; nothing else.
    pub fn admits(&self, method: &str, url: &str) -> bool {
        if url.starts_with("data:") {
            return true;
        }
        self.allows(url) && self.methods.iter().any(|m| m == method)
    }

    /// Whether `url` is of the origin (or a `data:` URL), whatever its method.
    pub fn allows(&self, url: &str) -> bool {
        if url.starts_with("data:") {
            return true;
        }
        let url = url.strip_prefix("blob:").unwrap_or(url);
        url.strip_prefix(self.origin.as_str())
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
    }
}

/// Events kept for a later `wait_event`; older ones are dropped beyond this.
const MAX_QUEUED: usize = 10_000;

pub struct Connection<T> {
    transport: T,
    next_id: i64,
    events: VecDeque<(Option<String>, String, Value)>,
    timeout: Duration,
    /// When set, every paused request (`Fetch.requestPaused`) is answered by it.
    pub guard: Option<Guard>,
    /// The URLs the guard failed, in order.
    pub blocked: Vec<String>,
}

impl<T: Transport> Connection<T> {
    pub fn new(transport: T) -> Self {
        Connection {
            transport,
            next_id: 0,
            events: VecDeque::new(),
            timeout: Duration::from_secs(30),
            guard: None,
            blocked: Vec::new(),
        }
    }

    /// How long a call waits for its response.
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    pub fn transport(&self) -> &T {
        &self.transport
    }

    pub fn transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }

    fn send(&mut self, session: Option<&str>, method: &str, params: &Value) -> Result<i64, String> {
        self.next_id += 1;
        let id = self.next_id;
        self.transport.send(&command(id, method, params, session))?;
        Ok(id)
    }

    /// Handles one message; returns it when it is the response to `wanted`.
    fn take(
        &mut self,
        text: &str,
        wanted: Option<i64>,
    ) -> Result<Option<Result<Value, String>>, String> {
        match parse_incoming(text)? {
            Incoming::Response { id, result, .. } => Ok((Some(id) == wanted).then_some(result)),
            Incoming::Event {
                method,
                params,
                session,
            } => {
                if method == "Fetch.requestPaused" && self.guard.is_some() {
                    self.answer_paused(&params, session.as_deref())?;
                } else {
                    if self.events.len() >= MAX_QUEUED {
                        self.events.pop_front();
                    }
                    self.events.push_back((session, method, params));
                }
                Ok(None)
            }
        }
    }

    fn answer_paused(&mut self, params: &Value, session: Option<&str>) -> Result<(), String> {
        let Some(guard) = &self.guard else {
            return Ok(());
        };
        let request_id = params.get_str("requestId").unwrap_or_default().to_string();
        let request = params.get("request");
        let url = request
            .and_then(|r| r.get_str("url"))
            .unwrap_or_default()
            .to_string();
        let request_method = request
            .and_then(|r| r.get_str("method"))
            .unwrap_or_default();
        let mut answer = vec![("requestId".to_string(), Value::Str(request_id))];
        let method = if guard.admits(request_method, &url) {
            "Fetch.continueRequest"
        } else {
            answer.push((
                "errorReason".to_string(),
                Value::Str("BlockedByClient".to_string()),
            ));
            self.blocked.push(url);
            "Fetch.failRequest"
        };
        self.send(session, method, &Value::Map(answer))?;
        Ok(())
    }

    /// Sends a command and returns its result; events that arrive meanwhile are queued.
    pub fn call(
        &mut self,
        session: Option<&str>,
        method: &str,
        params: Value,
    ) -> Result<Value, String> {
        let id = self.send(session, method, &params)?;
        let deadline = Instant::now() + self.timeout;
        loop {
            let Some(text) = self.transport.recv(deadline)? else {
                if Instant::now() >= deadline {
                    return Err(format!("{method}: no answer within {:?}", self.timeout));
                }
                continue;
            };
            if let Some(result) = self.take(&text, Some(id))? {
                return result.map_err(|e| format!("{method}: {e}"));
            }
        }
    }

    fn queued(&mut self, session: Option<&str>, method: &str) -> Option<Value> {
        let at = self
            .events
            .iter()
            .position(|(s, m, _)| s.as_deref() == session && m == method)?;
        self.events.remove(at).map(|(_, _, params)| params)
    }

    /// The first event `method` of `session`, queued or arriving within `timeout`.
    pub fn wait_event(
        &mut self,
        session: Option<&str>,
        method: &str,
        timeout: Duration,
    ) -> Result<Value, String> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(params) = self.queued(session, method) {
                return Ok(params);
            }
            if Instant::now() >= deadline {
                return Err(format!("no {method} within {timeout:?}"));
            }
            if let Some(text) = self.transport.recv(deadline)? {
                self.take(&text, None)?;
            }
        }
    }

    /// Handles messages for `duration`: a wait during which the guard keeps answering.
    pub fn pump(&mut self, duration: Duration) -> Result<(), String> {
        let deadline = Instant::now() + duration;
        while Instant::now() < deadline {
            if let Some(text) = self.transport.recv(deadline)? {
                self.take(&text, None)?;
            }
        }
        Ok(())
    }

    /// Drops the queued events of `session`.
    pub fn forget(&mut self, session: Option<&str>) {
        self.events.retain(|(s, _, _)| s.as_deref() != session);
    }
}
