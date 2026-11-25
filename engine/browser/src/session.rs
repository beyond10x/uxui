//! A browser and its pages, driven over the DevTools protocol. Each page lives in a browser context
//! of its own (as a fresh Playwright context does): no cache, storage or theme choice carries over
//! from one page to the next.

use crate::cdp::{Connection, Guard};
use crate::chrome::Process;
use crate::render::Viewport;
use crate::value::{map, s};
use crate::ws::WebSocket;
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;
use uxui_doc::yaml::Value;

pub type Conn = Connection<WebSocket<TcpStream>>;

pub struct Browser {
    pub process: Process,
    pub conn: Conn,
    /// `Browser.getVersion`'s product, `Chrome/<version>`.
    pub product: String,
}

impl Browser {
    /// Starts Chromium with the default switches and `added` after them (`chrome::switches`).
    pub fn start(profile: &Path, added: &[String]) -> Result<Browser, String> {
        let binary = crate::chrome::find()?;
        let process = Process::start(&binary, profile, added)?;
        let socket = WebSocket::connect(&process.ws_url, Duration::from_secs(10))?;
        let mut conn = Connection::new(socket);
        let version = conn.call(None, "Browser.getVersion", map(vec![]))?;
        let product = version.get_str("product").unwrap_or("unknown").to_string();
        Ok(Browser {
            process,
            conn,
            product,
        })
    }

    /// Asks the browser to close, then ends its process by PID.
    pub fn close(mut self) {
        let _ = self.conn.call(None, "Browser.close", map(vec![]));
        self.conn.transport_mut().close();
        self.process.end(Duration::from_secs(5));
    }
}

/// What a page is opened with.
pub struct Setup<'a> {
    pub viewport: &'a Viewport,
    pub mode: &'a str,
    pub reduced_motion: bool,
    /// When set, it answers every request of the page (`cdp::Guard`).
    pub guard: Option<&'a Guard>,
    /// Scripts that run before any script of each document.
    pub init_scripts: &'a [&'a str],
}

pub struct Page {
    pub session: String,
    context: String,
}

fn media(mode: &str, reduced_motion: bool) -> Value {
    let feature = |name: &str, value: &str| map(vec![("name", s(name)), ("value", s(value))]);
    let mut features = vec![feature("prefers-color-scheme", mode)];
    if reduced_motion {
        features.push(feature("prefers-reduced-motion", "reduce"));
    }
    map(vec![("features", Value::List(features))])
}

impl Page {
    pub fn open(conn: &mut Conn, setup: &Setup) -> Result<Page, String> {
        let ctx = conn.call(
            None,
            "Target.createBrowserContext",
            map(vec![("disposeOnDetach", Value::Bool(true))]),
        )?;
        let context = ctx
            .get_str("browserContextId")
            .ok_or("no browserContextId")?
            .to_string();
        let target = conn.call(
            None,
            "Target.createTarget",
            map(vec![
                ("url", s("about:blank")),
                ("browserContextId", s(&context)),
            ]),
        )?;
        let target_id = target.get_str("targetId").ok_or("no targetId")?.to_string();
        let attached = conn.call(
            None,
            "Target.attachToTarget",
            map(vec![
                ("targetId", s(&target_id)),
                ("flatten", Value::Bool(true)),
            ]),
        )?;
        let session = attached
            .get_str("sessionId")
            .ok_or("no sessionId")?
            .to_string();
        let page = Page { session, context };
        page.call(conn, "Page.enable", map(vec![]))?;
        page.call(conn, "DOM.enable", map(vec![]))?;
        page.call(conn, "Accessibility.enable", map(vec![]))?;
        page.set_viewport(conn, setup.viewport)?;
        page.set_mode(conn, setup.mode, setup.reduced_motion)?;
        page.call(
            conn,
            "Emulation.setFocusEmulationEnabled",
            map(vec![("enabled", Value::Bool(true))]),
        )?;
        if let Some(guard) = setup.guard {
            conn.guard = Some(guard.clone());
            page.call(
                conn,
                "Fetch.enable",
                map(vec![(
                    "patterns",
                    Value::List(vec![map(vec![("urlPattern", s("*"))])]),
                )]),
            )?;
        }
        for script in setup.init_scripts {
            page.call(
                conn,
                "Page.addScriptToEvaluateOnNewDocument",
                map(vec![("source", s(script))]),
            )?;
        }
        Ok(page)
    }

    pub fn call(&self, conn: &mut Conn, method: &str, params: Value) -> Result<Value, String> {
        conn.call(Some(&self.session), method, params)
    }

    pub fn set_viewport(&self, conn: &mut Conn, viewport: &Viewport) -> Result<(), String> {
        self.call(
            conn,
            "Emulation.setDeviceMetricsOverride",
            map(vec![
                ("width", Value::Int(viewport.width)),
                ("height", Value::Int(viewport.height)),
                ("deviceScaleFactor", Value::Int(1)),
                ("mobile", Value::Bool(false)),
            ]),
        )?;
        Ok(())
    }

    pub fn set_mode(
        &self,
        conn: &mut Conn,
        mode: &str,
        reduced_motion: bool,
    ) -> Result<(), String> {
        self.call(
            conn,
            "Emulation.setEmulatedMedia",
            media(mode, reduced_motion),
        )?;
        Ok(())
    }

    /// Loads `url` and waits for its `load` event; returns the HTTP status of the document.
    pub fn navigate(&self, conn: &mut Conn, url: &str) -> Result<i64, String> {
        conn.forget(Some(&self.session));
        let nav = self.call(conn, "Page.navigate", map(vec![("url", s(url))]))?;
        if let Some(error) = nav.get_str("errorText") {
            return Err(format!("{url}: {error}"));
        }
        conn.wait_event(
            Some(&self.session),
            "Page.loadEventFired",
            Duration::from_secs(30),
        )?;
        let status = self.eval(
            conn,
            "(performance.getEntriesByType('navigation')[0] || {}).responseStatus || 0",
        )?;
        Ok(match status {
            Value::Int(n) => n,
            _ => 0,
        })
    }

    /// Evaluates an expression in the page, awaiting a promise, and returns its JSON value.
    pub fn eval(&self, conn: &mut Conn, expression: &str) -> Result<Value, String> {
        let r = self.call(
            conn,
            "Runtime.evaluate",
            map(vec![
                ("expression", s(expression)),
                ("returnByValue", Value::Bool(true)),
                ("awaitPromise", Value::Bool(true)),
            ]),
        )?;
        value_of(&r)
    }

    /// Calls a function expression (`probes/*.js`) with JSON arguments.
    pub fn run(&self, conn: &mut Conn, function: &str, args: &[Value]) -> Result<Value, String> {
        let args: Vec<String> = args.iter().map(uxui_doc::json::write).collect();
        self.eval(conn, &format!("({function})({})", args.join(",")))
    }

    /// Calls `function` with `this` bound to a remote object.
    pub fn run_on(
        &self,
        conn: &mut Conn,
        object: &str,
        function: &str,
        args: &[Value],
    ) -> Result<Value, String> {
        let arguments = args
            .iter()
            .map(|a| map(vec![("value", a.clone())]))
            .collect();
        let r = self.call(
            conn,
            "Runtime.callFunctionOn",
            map(vec![
                ("objectId", s(object)),
                ("functionDeclaration", s(function)),
                ("arguments", Value::List(arguments)),
                ("returnByValue", Value::Bool(true)),
                ("awaitPromise", Value::Bool(true)),
            ]),
        )?;
        value_of(&r)
    }

    /// Remote objects of the elements `selector` matches inside the first match of `scope`, in
    /// document order.
    pub fn query(
        &self,
        conn: &mut Conn,
        selector: &str,
        scope: Option<&str>,
    ) -> Result<Vec<String>, String> {
        let root = match scope {
            Some(sc) => format!("document.querySelector({})", uxui_doc::json::string(sc)),
            None => "document".to_string(),
        };
        let expression = format!(
            "(() => {{ const r = {root}; return r ? Array.from(r.querySelectorAll({})) : []; }})()",
            uxui_doc::json::string(selector)
        );
        let r = self.call(
            conn,
            "Runtime.evaluate",
            map(vec![
                ("expression", s(&expression)),
                ("returnByValue", Value::Bool(false)),
            ]),
        )?;
        if r.get("exceptionDetails").is_some() {
            return Err(format!("selector `{selector}` is not valid"));
        }
        let Some(list) = r.get("result").and_then(|o| o.get_str("objectId")) else {
            return Ok(Vec::new());
        };
        let props = self.call(
            conn,
            "Runtime.getProperties",
            map(vec![
                ("objectId", s(list)),
                ("ownProperties", Value::Bool(true)),
            ]),
        )?;
        let mut items: Vec<(usize, String)> = props
            .get("result")
            .map(Value::list)
            .unwrap_or_default()
            .iter()
            .filter_map(|p| {
                let index = p.get_str("name")?.parse::<usize>().ok()?;
                let id = p.get("value")?.get_str("objectId")?;
                Some((index, id.to_string()))
            })
            .collect();
        items.sort();
        Ok(items.into_iter().map(|(_, id)| id).collect())
    }

    /// The remote object of a DOM node named by its backend id (from the accessibility tree).
    pub fn resolve(&self, conn: &mut Conn, backend_id: i64) -> Result<String, String> {
        let r = self.call(
            conn,
            "DOM.resolveNode",
            map(vec![("backendNodeId", Value::Int(backend_id))]),
        )?;
        r.get("object")
            .and_then(|o| o.get_str("objectId"))
            .map(str::to_string)
            .ok_or_else(|| format!("node {backend_id} has no remote object"))
    }

    pub fn ax_tree(&self, conn: &mut Conn) -> Result<Value, String> {
        self.call(conn, "Accessibility.getFullAXTree", map(vec![]))
    }

    pub fn ax_of(&self, conn: &mut Conn, object: &str) -> Result<Value, String> {
        self.call(
            conn,
            "Accessibility.getPartialAXTree",
            map(vec![
                ("objectId", s(object)),
                ("fetchRelatives", Value::Bool(false)),
            ]),
        )
    }

    /// Presses a key as a keyboard does: `Tab`, `Enter` or `Space`.
    pub fn press(&self, conn: &mut Conn, key: &str) -> Result<(), String> {
        let (code, text) = match key {
            "Tab" => (9, None),
            "Enter" => (13, Some("\r")),
            "Space" => (32, Some(" ")),
            other => return Err(format!("key `{other}` is not supported")),
        };
        // The DOM `key` of the space bar is a space; its `code` is `Space`.
        let key_value = if key == "Space" { " " } else { key };
        let mut down = vec![
            (
                "type",
                s(if text.is_some() {
                    "keyDown"
                } else {
                    "rawKeyDown"
                }),
            ),
            ("key", s(key_value)),
            ("code", s(key)),
            ("windowsVirtualKeyCode", Value::Int(code)),
            ("nativeVirtualKeyCode", Value::Int(code)),
        ];
        if let Some(t) = text {
            down.push(("text", s(t)));
            down.push(("unmodifiedText", s(t)));
        }
        self.call(conn, "Input.dispatchKeyEvent", map(down))?;
        self.call(
            conn,
            "Input.dispatchKeyEvent",
            map(vec![
                ("type", s("keyUp")),
                ("key", s(key_value)),
                ("code", s(key)),
                ("windowsVirtualKeyCode", Value::Int(code)),
                ("nativeVirtualKeyCode", Value::Int(code)),
            ]),
        )?;
        Ok(())
    }

    pub fn screenshot(&self, conn: &mut Conn) -> Result<Vec<u8>, String> {
        let r = self.call(
            conn,
            "Page.captureScreenshot",
            map(vec![("format", s("png"))]),
        )?;
        crate::base64::decode(r.get_str("data").ok_or("no screenshot data")?)
    }

    /// Waits, answering the guard's requests meanwhile.
    pub fn wait(&self, conn: &mut Conn, ms: u64) -> Result<(), String> {
        conn.pump(Duration::from_millis(ms))
    }

    pub fn close(self, conn: &mut Conn) -> Result<(), String> {
        conn.forget(Some(&self.session));
        conn.call(
            None,
            "Target.disposeBrowserContext",
            map(vec![("browserContextId", s(&self.context))]),
        )?;
        conn.guard = None;
        Ok(())
    }
}

fn value_of(r: &Value) -> Result<Value, String> {
    if let Some(ex) = r.get("exceptionDetails") {
        let text = ex
            .get("exception")
            .and_then(|e| e.get_str("description"))
            .or_else(|| ex.get_str("text"))
            .unwrap_or("an exception");
        return Err(format!("in the page: {text}"));
    }
    Ok(r.get("result")
        .and_then(|o| o.get("value"))
        .cloned()
        .unwrap_or(Value::Null))
}

/// Resolves once fonts are loaded and two frames have been drawn; returns `performance.now()`.
pub const READY: &str = "(async () => { await document.fonts.ready; await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r))); return performance.now(); })()";
