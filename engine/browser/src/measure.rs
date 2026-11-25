//! The runs that drive a browser: the render record of one URL, and the page checks of a built
//! site. Everything they compute from what the browser returns lives in `render` and `checks`,
//! where the tests reach it without a browser.

use crate::cdp::Connection;
use crate::checks::{
    self, AlwaysVisible, Candidate, Config, KeyboardConfig, NamedViewport, Pages, Ready,
};
use crate::render::{self, Declarations, FileRef, NodeRender, Record, Shot};
use crate::session::{Browser, Conn, Page, READY, Setup};
use crate::value::{f64_of, map, s};
use crate::ws::WebSocket;
use std::path::Path;
use std::time::Duration;
use uxui_doc::yaml::Value;

pub const NODE_PROBE: &str = include_str!("probes/node.js");
pub const OVERFLOW_PROBE: &str = include_str!("probes/overflow.js");
pub const CANDIDATE_PROBE: &str = include_str!("probes/candidate.js");
pub const SHIFTS_INIT: &str = include_str!("probes/shifts.js");
/// Waits for the page's ready condition (watched, not polled, up to the declared time), then fonts
/// and two frames; answers what `checks::ready_time` reads.
pub const READY_PROBE: &str = include_str!("probes/ready.js");
/// The trap a global ready condition needs before the page's own scripts: called with its name.
const READY_GLOBAL_INIT: &str = include_str!("probes/ready-global.js");
/// How long a DevTools call waits for its answer (`cdp::Connection`), beside the ready time.
const CALL_TIMEOUT: Duration = Duration::from_secs(30);
/// The focused element's label (its `aria-label`, else its text) and `href`.
const ACTIVE_PROBE: &str = "(() => { const a = document.activeElement; return { label: a ? (a.getAttribute('aria-label') ?? a.innerText ?? '').trim() : '', href: a && a.getAttribute ? (a.getAttribute('href') ?? '') : '' }; })()";
/// Whether focus is on the skip link's target or inside it.
const PAST_PROBE: &str = "(function (href) { const t = href.startsWith('#') ? document.getElementById(href.slice(1)) : null; const a = document.activeElement; return !!t && !!a && a !== document.body && (t === a || t.contains(a)); })";
/// Whether focus is on `this` or inside it.
const FOCUS_WITHIN: &str = "function () { const a = document.activeElement; return !!a && (this === a || this.contains(a)); }";
/// How long a page may take to apply a mode it is switched to.
const MODE_WAIT_MS: u64 = 5_000;
/// Resolves once `<html>` has `attr` equal to `mode`, or after `ms`: the page is watched, not polled.
const MODE_WAIT: &str = "(function (attr, mode, ms) { const el = document.documentElement; const done = () => el.getAttribute(attr) === mode; if (done()) return true; return new Promise((resolve) => { let timer; const watch = new MutationObserver(() => { if (done()) { watch.disconnect(); clearTimeout(timer); resolve(true); } }); watch.observe(el, { attributes: true, attributeFilter: [attr] }); timer = setTimeout(() => { watch.disconnect(); resolve(done()); }, ms); }); })";

/// The script that runs before the page's own for the configuration's ready condition, if it needs
/// one: a global's trap.
pub fn ready_init(config: &Config) -> Option<String> {
    match &config.ready {
        Some(Ready::Global(name)) => Some(format!(
            "{}({})",
            READY_GLOBAL_INIT.trim_end(),
            uxui_doc::json::string(name)
        )),
        _ => None,
    }
}

/// Ready after a load: without a condition, fonts and two frames as in 0.3.0; with one, the
/// condition, then fonts and two frames. Returns the ready instant (`performance.now()`); a page
/// that does not set its condition in time is the page's failure.
fn ready(conn: &mut Conn, page: &Page, config: &Config) -> Result<Value, Failure> {
    let Some(condition) = &config.ready else {
        return Ok(page.eval(conn, READY)?);
    };
    let (kind, value) = match condition {
        Ready::Selector(css) => ("selector", css),
        Ready::Global(name) => ("global", name),
    };
    conn.set_timeout(CALL_TIMEOUT + Duration::from_millis(config.ready_timeout_ms));
    let answer = page.run(
        conn,
        READY_PROBE,
        &[
            s(kind),
            s(value),
            Value::Int(config.ready_timeout_ms as i64),
        ],
    );
    conn.set_timeout(CALL_TIMEOUT);
    let answer = answer?;
    checks::ready_time(config, &answer).map_err(Failure::Page)?;
    Ok(answer.get("ready").cloned().unwrap_or(Value::Null))
}

/// The pages a configuration names: its list, or its sitemap in the build.
pub fn pages(config: &Config, build: &Path) -> Result<Vec<String>, String> {
    match &config.pages {
        Pages::Paths(p) => Ok(p.clone()),
        Pages::Sitemap(file) => {
            let path = build.join(file);
            let xml =
                std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            Ok(checks::sitemap_paths(&xml))
        }
    }
}

/// A page that could not be measured, as opposed to a browser that failed.
#[derive(Debug)]
pub enum Failure {
    Page(String),
    Browser(String),
}

impl From<String> for Failure {
    fn from(e: String) -> Self {
        Failure::Browser(e)
    }
}

/// The recorded page of `path` from its viewports' results, in the configuration's viewport order:
/// every viewport's record, or the first failure. A page failure makes the page an `error` entry; a
/// browser failure fails the run.
pub fn page_record(
    path: &str,
    viewports: Vec<(String, Result<Value, Failure>)>,
) -> Result<Value, String> {
    let mut records = Vec::new();
    for (name, result) in viewports {
        match result {
            Ok(v) => records.push(v),
            Err(Failure::Page(error)) => {
                return Ok(map(vec![("path", s(path)), ("error", s(&error))]));
            }
            Err(Failure::Browser(e)) => return Err(format!("{path} {name}: {e}")),
        }
    }
    Ok(map(vec![
        ("path", s(path)),
        ("viewports", Value::List(records)),
    ]))
}

/// Runs the checks of `config` on every page of the site served at `origin` in one browser, up to
/// `jobs` page and viewport pairs at a time, each worker on a DevTools connection of its own;
/// returns one recorded page per path (what `checks::page_result` reads) and the browser's product.
pub fn check_pages(
    config: &Config,
    origin: &str,
    paths: &[String],
    profile: &Path,
    jobs: usize,
    mut progress: impl FnMut(&str, &Value),
) -> Result<(Vec<Value>, String), String> {
    let browser = Browser::start(profile)?;
    let ws_url = browser.process.ws_url.clone();
    let viewports = config.viewports.len();
    let measured = crate::pool::run(
        jobs,
        paths.len() * viewports,
        || {
            let socket = WebSocket::connect(&ws_url, Duration::from_secs(10))?;
            Ok(Connection::new(socket))
        },
        |conn: &mut Conn, item| {
            let (path, vp) = (
                &paths[item / viewports],
                &config.viewports[item % viewports],
            );
            // A browser failure stops every worker; a page failure is the page's result.
            match measure_viewport(conn, config, origin, path, vp) {
                Err(Failure::Browser(e)) => Err(format!("{path} {}: {e}", vp.name)),
                other => Ok(other),
            }
        },
        |mut conn: Conn| conn.transport_mut().close(),
    );
    let product = browser.product.clone();
    browser.close();
    let mut measured = measured?.into_iter();
    let mut out = Vec::new();
    for path in paths {
        let results = config
            .viewports
            .iter()
            .map(|vp| {
                let result = measured.next().expect("one result per page and viewport");
                (vp.name.clone(), result)
            })
            .collect();
        let page = page_record(path, results)?;
        progress(path, &page);
        out.push(page);
    }
    Ok((out, product))
}

/// Measures `path` at `vp` in a browser context of its own, as a fresh Playwright context: a page
/// reused from one path to the next read a layout shift of the first consumer's site as 0 where a
/// fresh one reads 0.0004 (`docs/browser-adapter.md`, Timings), and saved no time.
fn measure_viewport(
    conn: &mut Conn,
    config: &Config,
    origin: &str,
    path: &str,
    vp: &NamedViewport,
) -> Result<Value, Failure> {
    let first = &config.modes[0];
    let ready_init = ready_init(config);
    let mut init_scripts = vec![SHIFTS_INIT];
    init_scripts.extend(ready_init.as_deref());
    let page = Page::open(
        conn,
        &Setup {
            viewport: &vp.viewport(),
            mode: first,
            reduced_motion: config.reduced_motion,
            origin: Some(origin),
            init_scripts: &init_scripts,
        },
    )?;
    let result = measure_open(conn, &page, config, origin, path, vp);
    page.close(conn)?;
    result
}

fn measure_open(
    conn: &mut Conn,
    page: &Page,
    config: &Config,
    origin: &str,
    path: &str,
    vp: &NamedViewport,
) -> Result<Value, Failure> {
    let url = format!("{origin}{path}");
    let status = page.navigate(conn, &url)?;
    if status != 200 {
        return Err(Failure::Page(format!("HTTP status {status}")));
    }
    let ready = ready(conn, page, config)?;
    page.wait(conn, config.settle_ms)?;
    let entries = page.eval(conn, "window.__uxbShifts")?;
    let shifts = map(vec![("ready", ready), ("entries", entries)]);
    let mut units = Vec::new();
    let mut walks = Vec::new();
    for (i, mode) in config.modes.iter().enumerate() {
        if i > 0 {
            page.set_mode(conn, mode, config.reduced_motion)?;
            if let Some(attr) = &config.mode_attribute {
                page.run(
                    conn,
                    MODE_WAIT,
                    &[s(attr), s(mode), Value::Int(MODE_WAIT_MS as i64)],
                )?;
            }
            page.wait(conn, config.mode_settle_ms)?;
        }
        let applied = match &config.mode_attribute {
            Some(attr) => mode_of(conn, page, attr)?.as_deref() == Some(mode.as_str()),
            None => true,
        };
        page.eval(conn, "window.scrollTo(0, 0)")?;
        let overflow = page.run(conn, OVERFLOW_PROBE, &[])?;
        let top = visible_items(conn, page, config, path, vp)?;
        page.eval(
            conn,
            "window.scrollTo(0, document.documentElement.scrollHeight)",
        )?;
        page.wait(conn, config.scroll_settle_ms)?;
        let bottom = visible_items(conn, page, config, path, vp)?;
        page.eval(conn, "window.scrollTo(0, 0)")?;
        units.push(map(vec![
            ("mode", s(mode)),
            ("mode_applied", Value::Bool(applied)),
            ("overflow", overflow),
            ("visible", map(vec![("top", top), ("bottom", bottom)])),
        ]));
        if let Some(k) = &config.keyboard
            && k.modes.contains(mode)
            && k.viewports.as_ref().is_none_or(|v| v.contains(&vp.name))
        {
            let walk = keyboard_walk(conn, page, config, k, &url)?;
            walks.push(map(vec![("mode", s(mode)), ("walk", walk)]));
        }
    }
    Ok(map(vec![
        ("viewport", s(&vp.name)),
        ("shifts", shifts),
        ("units", Value::List(units)),
        ("keyboard", Value::List(walks)),
    ]))
}

fn mode_of(conn: &mut Conn, page: &Page, attr: &str) -> Result<Option<String>, String> {
    let expr = format!(
        "document.documentElement.getAttribute({})",
        uxui_doc::json::string(attr)
    );
    Ok(page.eval(conn, &expr)?.str().map(str::to_string))
}

/// The remote objects that may be `item`: role-and-name matches from the accessibility tree, then
/// CSS matches inside the scope, in that order.
fn objects(
    conn: &mut Conn,
    page: &Page,
    tree: &Value,
    item: &AlwaysVisible,
) -> Result<Vec<String>, String> {
    let l = &item.locate;
    let mut out = Vec::new();
    for backend in checks::ax_candidates(tree, &l.role, &l.name) {
        // A node that left the DOM since the tree was read has no object; it is no candidate.
        if let Ok(object) = page.resolve(conn, backend) {
            out.push(object);
        }
    }
    if let Some(css) = &l.css {
        out.extend(page.query(conn, css, l.scope.as_deref())?);
    }
    Ok(out)
}

fn candidate(
    conn: &mut Conn,
    page: &Page,
    object: &str,
    scope: Option<&str>,
) -> Result<Value, String> {
    let scope = scope.map_or(Value::Null, s);
    page.run_on(conn, object, CANDIDATE_PROBE, &[scope])
}

fn visible_items(
    conn: &mut Conn,
    page: &Page,
    config: &Config,
    path: &str,
    vp: &NamedViewport,
) -> Result<Value, String> {
    let applicable: Vec<&AlwaysVisible> = config
        .always_visible
        .iter()
        .filter(|a| checks::applies(a, path, vp.width))
        .collect();
    if applicable.is_empty() {
        return Ok(Value::List(Vec::new()));
    }
    let tree = page.ax_tree(conn)?;
    let mut out = Vec::new();
    for item in applicable {
        let mut candidates = Vec::new();
        for object in objects(conn, page, &tree, item)? {
            candidates.push(candidate(
                conn,
                page,
                &object,
                item.locate.scope.as_deref(),
            )?);
        }
        out.push(map(vec![
            ("item", s(&item.name)),
            ("candidates", Value::List(candidates)),
        ]));
    }
    Ok(Value::List(out))
}

/// The keyboard walk of the first consumer: on a fresh load, the first Tab stop must be the skip
/// link and Enter on it (or one more Tab) must put focus into its target; on another fresh load, up
/// to `max_tabs` Tabs must reach each target. With a ready condition each fresh load waits for it.
fn keyboard_walk(
    conn: &mut Conn,
    page: &Page,
    config: &Config,
    k: &KeyboardConfig,
    url: &str,
) -> Result<Value, Failure> {
    let fresh_load = |conn: &mut Conn| -> Result<(), Failure> {
        page.navigate(conn, url)?;
        if config.ready.is_some() {
            ready(conn, page, config)?;
        }
        Ok(())
    };
    fresh_load(conn)?;
    page.press(conn, "Tab")?;
    let active = page.eval(conn, ACTIVE_PROBE)?;
    let label = active.get_str("label").unwrap_or_default().to_string();
    let href = active.get_str("href").unwrap_or_default().to_string();
    let mut after_enter = false;
    let mut after_tab = Value::Null;
    if k.skip_link.matches(&label) {
        page.press(conn, "Enter")?;
        page.wait(conn, k.focus_settle_ms)?;
        after_enter = page.run(conn, PAST_PROBE, &[s(&href)])? == Value::Bool(true);
        if !after_enter {
            page.press(conn, "Tab")?;
            after_tab = page.run(conn, PAST_PROBE, &[s(&href)])?;
        }
    }
    fresh_load(conn)?;
    let tree = page.ax_tree(conn)?;
    let mut targets: Vec<(String, Option<String>)> = Vec::new();
    for name in &k.must_reach {
        let item = config
            .always_visible
            .iter()
            .find(|a| &a.name == name)
            .expect("the configuration checks that every target is an item");
        let mut found = None;
        for object in objects(conn, page, &tree, item)? {
            let c = Candidate::from_value(&candidate(
                conn,
                page,
                &object,
                item.locate.scope.as_deref(),
            )?)?;
            if checks::first_visible(std::slice::from_ref(&c)).is_some() {
                found = Some(object);
                break;
            }
        }
        targets.push((name.clone(), found));
    }
    let mut reached: Vec<bool> = targets.iter().map(|_| false).collect();
    let mut tabs = Vec::new();
    for _ in 0..k.max_tabs {
        if targets
            .iter()
            .zip(&reached)
            .all(|((_, object), r)| object.is_none() || *r)
        {
            break;
        }
        page.press(conn, "Tab")?;
        let mut here = Vec::new();
        for (i, (name, object)) in targets.iter().enumerate() {
            let Some(object) = object else { continue };
            if !reached[i] && page.run_on(conn, object, FOCUS_WITHIN, &[])? == Value::Bool(true) {
                reached[i] = true;
                here.push(s(name));
            }
        }
        tabs.push(Value::List(here));
    }
    Ok(map(vec![
        ("first_label", s(&label)),
        ("after_enter", Value::Bool(after_enter)),
        ("after_tab", after_tab),
        (
            "located",
            Value::Map(
                targets
                    .iter()
                    .map(|(n, o)| (n.clone(), Value::Bool(o.is_some())))
                    .collect(),
            ),
        ),
        ("tabs", Value::List(tabs)),
    ]))
}

/// A number the page returned.
pub fn number(v: &Value) -> Option<f64> {
    f64_of(v)
}

/// What a render run returns: the record and, for `--capture`, the raw probes and AX entries.
pub struct Rendered {
    pub record: Value,
    pub capture: Value,
    pub png: Vec<u8>,
}

pub struct RenderInputs<'a> {
    pub url: &'a str,
    pub declarations: &'a Declarations,
    pub declarations_file: FileRef,
    pub shot_file: &'a str,
    pub settle_ms: u64,
    pub profile: &'a Path,
}

pub fn render_url(inputs: &RenderInputs) -> Result<Rendered, String> {
    let mut browser = Browser::start(inputs.profile)?;
    let result = render_in(&mut browser, inputs);
    browser.close();
    result
}

fn render_in(browser: &mut Browser, inputs: &RenderInputs) -> Result<Rendered, String> {
    let d = inputs.declarations;
    let conn = &mut browser.conn;
    let page = Page::open(
        conn,
        &Setup {
            viewport: &d.viewport,
            mode: &d.mode,
            reduced_motion: true,
            origin: None,
            init_scripts: &[],
        },
    )?;
    let status = page.navigate(conn, inputs.url)?;
    if status != 200 {
        return Err(format!("{}: HTTP status {status}", inputs.url));
    }
    page.eval(conn, READY)?;
    page.wait(conn, inputs.settle_ms)?;
    let styles = Value::List(d.styles.iter().map(|p| s(p)).collect());
    let mut nodes: Vec<NodeRender> = Vec::new();
    let mut raw = Vec::new();
    for decl in &d.nodes {
        let probe = page.run(conn, NODE_PROBE, &[s(&decl.selector), styles.clone()])?;
        let objects = page.query(conn, &decl.selector, None)?;
        let ax_raw = match objects.first() {
            Some(object) => Some(page.ax_of(conn, object)?),
            None => None,
        };
        let ax = ax_raw.as_ref().and_then(render::ax_entry);
        nodes.push(render::node_render(decl, &d.styles, &probe, ax)?);
        let mut entry = vec![("name", s(&decl.name)), ("probe", probe)];
        if let Some(a) = ax_raw {
            entry.push(("ax", trim_ax(&a)));
        }
        raw.push(map(entry));
    }
    let png = page.screenshot(conn)?;
    let (width, height) = render::png_size(&png).ok_or("the screenshot is not a PNG")?;
    page.close(conn)?;
    let record = render::record_value(&Record {
        url: inputs.url.to_string(),
        declarations: d,
        declarations_file: FileRef {
            path: inputs.declarations_file.path.clone(),
            digest: inputs.declarations_file.digest.clone(),
        },
        browser: browser.product.clone(),
        nodes,
        screenshot: Some(Shot {
            file: inputs.shot_file.to_string(),
            digest: digest(&png),
            width,
            height,
        }),
    });
    Ok(Rendered {
        record,
        capture: map(vec![("nodes", Value::List(raw))]),
        png,
    })
}

/// A partial AX tree cut to the node itself: what `render::ax_entry` reads.
fn trim_ax(partial: &Value) -> Value {
    let first = partial
        .get("nodes")
        .and_then(|n| n.list().first().cloned())
        .map(|node| {
            let keep = ["ignored", "role", "name", "backendDOMNodeId"];
            Value::Map(
                node.entries()
                    .iter()
                    .filter(|(k, _)| keep.contains(&k.as_str()))
                    .cloned()
                    .collect(),
            )
        });
    map(vec![("nodes", Value::List(first.into_iter().collect()))])
}

/// `sha256:<hex>` of the bytes.
pub fn digest(bytes: &[u8]) -> String {
    uxui_doc::sha256::hex(bytes)
}
