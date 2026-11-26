//! The runs that drive a browser: the render record of one URL, and the page checks of a built
//! site or a running server. Everything they compute from what the browser returns lives in
//! `render` and `checks`, where the tests reach it without a browser.

use crate::cdp::{Connection, Guard};
use crate::checks::{
    self, AlwaysVisible, Candidate, Config, KeyboardConfig, NamedViewport, Pages, Ready,
};
use crate::journey::{Edge, Journey, Step};
use crate::render::{self, Declarations, FileRef, NodeRender, Record, Shot};
use crate::session::{Browser, Conn, Page, READY, Setup};
use crate::target::Site;
use crate::value::{f64_of, map, s};
use crate::ws::WebSocket;
use std::path::Path;
use std::time::{Duration, Instant};
use uxui_doc::yaml::Value;

pub const NODE_PROBE: &str = include_str!("probes/node.js");
pub const OVERFLOW_PROBE: &str = include_str!("probes/overflow.js");
pub const CANDIDATE_PROBE: &str = include_str!("probes/candidate.js");
/// The focused element's focus mark, as `checks::focus_marked` reads it.
pub const FOCUS_MARK_PROBE: &str = include_str!("probes/focus-mark.js");
/// The watch of an activation: `watch`, `mark`, `report` (`checks::acted`).
pub const ACTIVATE_PROBE: &str = include_str!("probes/activate.js");
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

/// Sets the page's clock to a declared instant before the page's own scripts: called with the
/// instant in milliseconds and whether the clock advances.
pub const CLOCK_INIT: &str = include_str!("probes/clock.js");

/// The script that runs before the page's own for the configuration's clock, if it declares one.
pub fn clock_init(config: &Config) -> Option<String> {
    config
        .clock
        .map(|c| format!("{}({},{})", CLOCK_INIT.trim_end(), c.at_ms, c.advance))
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

/// The request guard of a run on `site`: its origin, with the configuration's methods.
pub fn guard(config: &Config, site: &Site) -> Guard {
    Guard::new(&site.origin, &config.request_methods)
}

/// Reads the text at `url` in a page of its own behind `guard`, on the browser's own connection:
/// `Ok(Err)` when the server does not answer it with 200 (a refused input), `Err` when the browser
/// failed. What a running server's sitemap is read with.
pub fn fetch_text(
    conn: &mut Conn,
    guard: &Guard,
    url: &str,
) -> Result<Result<String, String>, String> {
    let page = Page::open(
        conn,
        &Setup {
            viewport: &render::Viewport {
                width: 800,
                height: 600,
            },
            mode: "light",
            reduced_motion: true,
            guard: Some(guard),
            init_scripts: &[],
        },
    )?;
    let result = (|| {
        let status = page.navigate(conn, url)?;
        if status != 200 {
            return Ok(Err(format!("{url}: HTTP status {status}")));
        }
        // The document as the server sent it: the page itself may be a viewer's rendering of it.
        let text = page.eval(
            conn,
            "fetch(location.href, { cache: 'no-store' }).then((r) => r.text())",
        )?;
        Ok(Ok(text.str().unwrap_or_default().to_string()))
    })();
    page.close(conn)?;
    result
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

/// Runs the checks of `config` on every page of `site` in `browser`, up to `jobs` page and viewport
/// pairs at a time, each worker on a DevTools connection of its own; returns one recorded page per
/// path (what `checks::page_result` reads).
pub fn check_pages(
    browser: &Browser,
    config: &Config,
    site: &Site,
    paths: &[String],
    jobs: usize,
    mut progress: impl FnMut(&str, &Value),
) -> Result<Vec<Value>, String> {
    let ws_url = browser.process.ws_url.clone();
    let guard = guard(config, site);
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
            match measure_viewport(conn, config, site, &guard, path, vp) {
                Err(Failure::Browser(e)) => Err(format!("{path} {}: {e}", vp.name)),
                other => Ok(other),
            }
        },
        |mut conn: Conn| conn.transport_mut().close(),
    );
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
    Ok(out)
}

/// Measures `path` at `vp` in a browser context of its own, as a fresh Playwright context: a page
/// reused from one path to the next read a layout shift of the first consumer's site as 0 where a
/// fresh one reads 0.0004 (`docs/browser-adapter.md`, Timings), and saved no time.
fn measure_viewport(
    conn: &mut Conn,
    config: &Config,
    site: &Site,
    guard: &Guard,
    path: &str,
    vp: &NamedViewport,
) -> Result<Value, Failure> {
    // A page with no place on the server (outside the base path, under a mount) is not served.
    let url = site
        .page_url(&config.base_path, path)
        .map_err(Failure::Page)?;
    let first = &config.modes[0];
    let ready_init = ready_init(config);
    let clock_init = clock_init(config);
    let mut init_scripts: Vec<&str> = clock_init.as_deref().into_iter().collect();
    init_scripts.push(SHIFTS_INIT);
    init_scripts.extend(ready_init.as_deref());
    let start = Instant::now();
    let page = Page::open(
        conn,
        &Setup {
            viewport: &vp.viewport(),
            mode: first,
            reduced_motion: config.reduced_motion,
            guard: Some(guard),
            init_scripts: &init_scripts,
        },
    )?;
    let mut timing = vec![("open", ms(start))];
    let result = measure_open(conn, &page, config, &url, path, vp, &mut timing);
    let closing = Instant::now();
    page.close(conn)?;
    timing.push(("close", ms(closing)));
    timing.push(("total", ms(start)));
    let mut record = result?;
    if let Value::Map(entries) = &mut record {
        entries.push(("timing".to_string(), map(timing)));
    }
    Ok(record)
}

/// The checks of `path` at `vp` on an open page; the milliseconds of each phase go to `timing`
/// (`run.timing`, `docs/browser-adapter.md`).
fn measure_open(
    conn: &mut Conn,
    page: &Page,
    config: &Config,
    url: &str,
    path: &str,
    vp: &NamedViewport,
    timing: &mut Vec<(&'static str, Value)>,
) -> Result<Value, Failure> {
    let t = Instant::now();
    let status = page.navigate(conn, url)?;
    if status != 200 {
        return Err(Failure::Page(format!("HTTP status {status}")));
    }
    let ready = ready(conn, page, config)?;
    let load = ms(t);
    let t = Instant::now();
    page.wait(conn, config.settle_ms)?;
    let settle = ms(t);
    let entries = page.eval(conn, "window.__uxbShifts")?;
    let shifts = map(vec![("ready", ready), ("entries", entries)]);
    let mut units = Vec::new();
    let mut walks = Vec::new();
    let mut loads = 1;
    let mut mode_times = Vec::new();
    for (i, mode) in config.modes.iter().enumerate() {
        let t = Instant::now();
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
        let mut times = vec![("mode", s(mode)), ("switch", ms(t))];
        let t = Instant::now();
        page.eval(conn, "window.scrollTo(0, 0)")?;
        let strs = |l: &[String]| Value::List(l.iter().map(|e| s(e)).collect());
        let rule = &config.layout.overflow;
        let overflow = page.run(
            conn,
            OVERFLOW_PROBE,
            &[strs(&rule.elements), strs(&rule.exclude)],
        )?;
        times.push(("overflow", ms(t)));
        let t = Instant::now();
        let top = visible_items(conn, page, config, path, vp)?;
        times.push(("top", ms(t)));
        let t = Instant::now();
        page.eval(
            conn,
            "window.scrollTo(0, document.documentElement.scrollHeight)",
        )?;
        page.wait(conn, config.scroll_settle_ms)?;
        times.push(("scroll_settle", ms(t)));
        let t = Instant::now();
        let bottom = visible_items(conn, page, config, path, vp)?;
        page.eval(conn, "window.scrollTo(0, 0)")?;
        times.push(("bottom", ms(t)));
        units.push(map(vec![
            ("mode", s(mode)),
            ("mode_applied", Value::Bool(applied)),
            ("overflow", overflow),
            ("visible", map(vec![("top", top), ("bottom", bottom)])),
        ]));
        if let Some(k) = &config.keyboard
            && walks_at(k, mode, &vp.name)
        {
            let t = Instant::now();
            let plan = walk_plan(k, walks.is_empty());
            let mut walk_times = WalkTimes::default();
            let walk = keyboard_walk(conn, page, config, k, url, &plan, &mut walk_times)?;
            walks.push(map(vec![("mode", s(mode)), ("walk", walk)]));
            loads += walk_times.loads.len();
            let mut kt = vec![("ms", ms(t))];
            kt.extend(walk_times.reset.map(|r| ("reset", r)));
            kt.push(("loads", Value::List(walk_times.loads)));
            kt.push(("tabs", Value::Int(walk_times.tabs)));
            times.push(("keyboard", map(kt)));
        }
        mode_times.push(map(times));
    }
    timing.push(("loads", Value::Int(loads as i64)));
    timing.push(("load", load));
    timing.push(("settle", settle));
    timing.push(("modes", Value::List(mode_times)));
    Ok(map(vec![
        ("viewport", s(&vp.name)),
        ("shifts", shifts),
        ("units", Value::List(units)),
        ("keyboard", Value::List(walks)),
    ]))
}

/// How a check of the keyboard walk starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Start {
    /// On the page as the checks left it, made the same as a fresh load.
    Reset,
    /// On a new load of the page.
    Load,
}

/// How the checks of one walk start; each activation starts on a load of its own.
#[derive(Debug, PartialEq, Eq)]
pub struct WalkPlan {
    pub skip_link: Option<Start>,
    pub tabs: Start,
}

/// The plan of a walk. The `first` walk of a page and viewport starts on the page the checks just
/// measured, which they only scrolled and read: its first check (the skip link's, else the Tab walk)
/// starts on a reset (`RESET_PROBE`). Every other start is a load: the Tab walk after the skip
/// link's Enter (focus moved, and the fragment set the browser's starting point of the next Tab), and
/// each check of a walk after another walk.
pub fn walk_plan(k: &KeyboardConfig, first: bool) -> WalkPlan {
    let start = if first { Start::Reset } else { Start::Load };
    match k.skip_link {
        Some(_) => WalkPlan {
            skip_link: Some(start),
            tabs: Start::Load,
        },
        None => WalkPlan {
            skip_link: None,
            tabs: start,
        },
    }
}

/// Whether the walk runs in `mode` at the viewport named `viewport`.
fn walks_at(k: &KeyboardConfig, mode: &str, viewport: &str) -> bool {
    k.modes.iter().any(|m| m == mode)
        && k.viewports
            .as_ref()
            .is_none_or(|v| v.iter().any(|n| n == viewport))
}

/// The page loads of one page at the viewport named `viewport` as the run plans them: the measured
/// load and the loads of each walk's plan ([`walk_plan`]). Not counted, because the page decides
/// them: one more per activated target, and one where a reset finds focus or a selection on the
/// page (a page that focuses an element itself) and loads instead.
pub fn planned_loads(config: &Config, viewport: &str) -> usize {
    let Some(k) = &config.keyboard else { return 1 };
    let walks = config
        .modes
        .iter()
        .filter(|m| walks_at(k, m, viewport))
        .count();
    let loads_of = |plan: WalkPlan| {
        plan.skip_link
            .into_iter()
            .chain([plan.tabs])
            .filter(|s| *s == Start::Load)
            .count()
    };
    1 + (0..walks).map(|i| loads_of(walk_plan(k, i == 0))).sum::<usize>()
}

/// Makes the measured page the same as a fresh load for the walk, when the checks left it so: no
/// element has focus and nothing is selected (the checks only scroll and read). Then it scrolls to
/// the top and answers true; else it changes nothing and answers false, and the walk loads the page.
const RESET_PROBE: &str = "(() => { const a = document.activeElement; const sel = window.getSelection(); const clean = (a === null || a === document.body) && (!sel || sel.rangeCount === 0); if (clean) window.scrollTo(0, 0); return clean; })()";

/// Milliseconds since `t`.
fn ms(t: Instant) -> Value {
    Value::Int(t.elapsed().as_millis() as i64)
}

/// The times of one walk (`run.timing`): its loads, its reset and its Tab presses.
#[derive(Default)]
struct WalkTimes {
    loads: Vec<Value>,
    reset: Option<Value>,
    tabs: i64,
}

/// The element a `click` or `type` step acts on (`probes/step-target.js`).
const STEP_TARGET_PROBE: &str = include_str!("probes/step-target.js");
/// A `wait` step (`probes/wait.js`).
const WAIT_PROBE: &str = include_str!("probes/wait.js");

/// Runs every journey of `journeys` at every viewport, up to `jobs` journey and viewport pairs at a
/// time as [`check_pages`] does; returns one recorded journey per journey (what
/// `journey::journey_result` reads).
pub fn check_journeys(
    browser: &Browser,
    config: &Config,
    site: &Site,
    journeys: &[&Journey],
    jobs: usize,
    mut progress: impl FnMut(&Journey, &Value),
) -> Result<Vec<Value>, String> {
    let ws_url = browser.process.ws_url.clone();
    let guard = guard(config, site);
    let viewports = config.viewports.len();
    let measured = crate::pool::run(
        jobs,
        journeys.len() * viewports,
        || {
            let socket = WebSocket::connect(&ws_url, Duration::from_secs(10))?;
            Ok(Connection::new(socket))
        },
        |conn: &mut Conn, item| {
            let (journey, vp) = (
                journeys[item / viewports],
                &config.viewports[item % viewports],
            );
            journey_viewport(conn, config, site, &guard, journey, vp)
                .map_err(|e| format!("journey `{}` {}: {e}", journey.name, vp.name))
        },
        |mut conn: Conn| conn.transport_mut().close(),
    );
    let mut measured = measured?.into_iter();
    let mut out = Vec::new();
    for journey in journeys {
        let records = (0..viewports)
            .map(|_| {
                measured
                    .next()
                    .expect("one result per journey and viewport")
            })
            .collect();
        let record = map(vec![
            ("journey", s(&journey.name)),
            ("page", s(&journey.page)),
            ("viewports", Value::List(records)),
        ]);
        progress(journey, &record);
        out.push(record);
    }
    Ok(out)
}

/// A journey at `vp`: per mode, a fresh browser context loaded in that mode, the steps, and at each
/// state its geometry. A step that fails ends the run of that mode; the error is the browser's
/// failure only when the browser failed.
fn journey_viewport(
    conn: &mut Conn,
    config: &Config,
    site: &Site,
    guard: &Guard,
    journey: &Journey,
    vp: &NamedViewport,
) -> Result<Value, String> {
    let url = site.page_url(&config.base_path, &journey.page)?;
    let ready_init = ready_init(config);
    let clock_init = clock_init(config);
    let mut init_scripts: Vec<&str> = clock_init.as_deref().into_iter().collect();
    init_scripts.extend(ready_init.as_deref());
    let mut runs = Vec::new();
    for mode in &config.modes {
        let page = Page::open(
            conn,
            &Setup {
                viewport: &vp.viewport(),
                mode,
                reduced_motion: config.reduced_motion,
                guard: Some(guard),
                init_scripts: &init_scripts,
            },
        )?;
        let result = journey_run(conn, &page, config, &url, journey, vp, mode);
        page.close(conn)?;
        runs.push(result?);
    }
    Ok(map(vec![
        ("viewport", s(&vp.name)),
        ("runs", Value::List(runs)),
    ]))
}

/// One run of a journey: `failed` is `{step, error}` for the step that failed (step 0: the page did
/// not load or get ready), or null.
fn journey_run(
    conn: &mut Conn,
    page: &Page,
    config: &Config,
    url: &str,
    journey: &Journey,
    vp: &NamedViewport,
    mode: &str,
) -> Result<Value, String> {
    let mut states = Vec::new();
    let record = |applied: bool, states: Vec<Value>, failed: Option<(usize, String)>| {
        let failed = failed.map_or(Value::Null, |(step, error)| {
            map(vec![
                ("step", Value::Int(step as i64)),
                ("error", s(&error)),
            ])
        });
        map(vec![
            ("mode", s(mode)),
            ("mode_applied", Value::Bool(applied)),
            ("states", Value::List(states)),
            ("failed", failed),
        ])
    };
    let status = page.navigate(conn, url)?;
    if status != 200 {
        return Ok(record(
            true,
            states,
            Some((0, format!("HTTP status {status}"))),
        ));
    }
    match ready(conn, page, config) {
        Ok(_) => {}
        Err(Failure::Page(e)) => return Ok(record(true, states, Some((0, e)))),
        Err(Failure::Browser(e)) => return Err(e),
    }
    page.wait(conn, config.settle_ms)?;
    let applied = match &config.mode_attribute {
        Some(attr) => {
            page.run(
                conn,
                MODE_WAIT,
                &[s(attr), s(mode), Value::Int(MODE_WAIT_MS as i64)],
            )?;
            mode_of(conn, page, attr)?.as_deref() == Some(mode)
        }
        None => true,
    };
    if !applied {
        return Ok(record(false, states, None));
    }
    for (i, step) in journey.steps.iter().enumerate() {
        if let Some(error) = journey_step(conn, page, config, journey, vp, step, &mut states)? {
            return Ok(record(true, states, Some((i + 1, error))));
        }
    }
    Ok(record(true, states, None))
}

/// One step; `Ok(Some(error))` when the page did not let it happen.
fn journey_step(
    conn: &mut Conn,
    page: &Page,
    config: &Config,
    journey: &Journey,
    vp: &NamedViewport,
    step: &Step,
    states: &mut Vec<Value>,
) -> Result<Option<String>, String> {
    match step {
        Step::Click(css) => {
            let answer = page.run(conn, STEP_TARGET_PROBE, &[s(css), Value::Bool(false)])?;
            if let Some(error) = answer.get_str("error") {
                return Ok(Some(error.to_string()));
            }
            let (Some(x), Some(y)) = (
                answer.get("x").and_then(f64_of),
                answer.get("y").and_then(f64_of),
            ) else {
                return Err(format!("click `{css}`: no point to click"));
            };
            conn.forget(Some(&page.session));
            page.click(conn, x, y)?;
            // A click that follows a link: the new document is loaded before the next step.
            navigated(conn, page);
        }
        Step::Type { css, text } => {
            let answer = page.run(conn, STEP_TARGET_PROBE, &[s(css), Value::Bool(true)])?;
            if let Some(error) = answer.get_str("error") {
                return Ok(Some(error.to_string()));
            }
            page.insert_text(conn, text)?;
        }
        Step::Press(key) => {
            conn.forget(Some(&page.session));
            page.press(conn, key)?;
            navigated(conn, page);
        }
        Step::Wait(css) => {
            conn.set_timeout(CALL_TIMEOUT + Duration::from_millis(journey.timeout_ms));
            let answer = page.run(
                conn,
                WAIT_PROBE,
                &[s(css), Value::Int(journey.timeout_ms as i64)],
            );
            conn.set_timeout(CALL_TIMEOUT);
            let answer = answer?;
            if get_bool_of(&answer, "invalid") {
                return Ok(Some("is not a valid selector".to_string()));
            }
            if !get_bool_of(&answer, "met") {
                return Ok(Some(format!(
                    "no rendered element matched it within {} ms",
                    journey.timeout_ms
                )));
            }
        }
        Step::Scroll(edge) => {
            let to = match edge {
                Edge::Top => "window.scrollTo(0, 0)",
                Edge::Bottom => "window.scrollTo(0, document.documentElement.scrollHeight)",
            };
            page.eval(conn, to)?;
            page.wait(conn, config.scroll_settle_ms)?;
        }
        Step::State(name) => {
            // What the steps changed is laid out and drawn: fonts, two frames, the settle time.
            page.eval(conn, READY)?;
            page.wait(conn, config.settle_ms)?;
            let strs = |l: &[String]| Value::List(l.iter().map(|e| s(e)).collect());
            let rule = &config.layout.overflow;
            let overflow = page.run(
                conn,
                OVERFLOW_PROBE,
                &[strs(&rule.elements), strs(&rule.exclude)],
            )?;
            let visible = visible_items(conn, page, config, &journey.page, vp)?;
            states.push(map(vec![
                ("state", s(name)),
                ("overflow", overflow),
                ("visible", visible),
            ]));
        }
    }
    Ok(None)
}

fn get_bool_of(v: &Value, key: &str) -> bool {
    matches!(v.get(key), Some(Value::Bool(true)))
}

fn mode_of(conn: &mut Conn, page: &Page, attr: &str) -> Result<Option<String>, String> {
    let expr = format!(
        "document.documentElement.getAttribute({})",
        uxui_doc::json::string(attr)
    );
    Ok(page.eval(conn, &expr)?.str().map(str::to_string))
}

/// The remote objects that may be `item`: role-and-name matches from the accessibility tree, then
/// CSS matches inside the scope (`css`, or every element with `aria-current` for an item with
/// `current` and neither), in that order.
fn objects(
    conn: &mut Conn,
    page: &Page,
    tree: &Value,
    item: &AlwaysVisible,
) -> Result<Vec<String>, String> {
    let l = &item.locate;
    let mut out = Vec::new();
    if let Some(role) = &l.role {
        for backend in checks::ax_candidates(tree, role, &l.name) {
            // A node that left the DOM since the tree was read has no object; it is no candidate.
            if let Ok(object) = page.resolve(conn, backend) {
                out.push(object);
            }
        }
    }
    if let Some(css) = l.selector() {
        out.extend(page.query(conn, css, l.scope.as_deref())?);
    }
    Ok(out)
}

/// The accessibility tree when one of `items` is located by role; else no tree is read.
fn ax_tree_for<'a>(
    conn: &mut Conn,
    page: &Page,
    mut items: impl Iterator<Item = &'a AlwaysVisible>,
) -> Result<Value, String> {
    if items.any(|a| a.locate.role.is_some()) {
        page.ax_tree(conn)
    } else {
        Ok(Value::Null)
    }
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
    let tree = ax_tree_for(conn, page, applicable.iter().copied())?;
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

/// The keyboard walk (`docs/browser-adapter.md`, The keyboard walk): with a skip link, on a fresh
/// page the first Tab stop must be the skip link and Enter on it (or one more Tab) must put focus
/// into its target; on another fresh page, up to `max_tabs` Tabs must reach each target, and with
/// `focus_mark` the focused element is read on the Tab that reaches one; then each reached target of
/// `activates` is activated on a fresh load of its own. A fresh page is a load, or the measured page
/// reset where `plan` says so ([`walk_plan`]). With a ready condition each load waits for it.
fn keyboard_walk(
    conn: &mut Conn,
    page: &Page,
    config: &Config,
    k: &KeyboardConfig,
    url: &str,
    plan: &WalkPlan,
    times: &mut WalkTimes,
) -> Result<Value, Failure> {
    let fresh_load = |conn: &mut Conn, times: &mut WalkTimes| -> Result<(), Failure> {
        let t = Instant::now();
        page.navigate(conn, url)?;
        if config.ready.is_some() {
            ready(conn, page, config)?;
        }
        times.loads.push(ms(t));
        Ok(())
    };
    let fresh = |conn: &mut Conn, start: Start, times: &mut WalkTimes| -> Result<(), Failure> {
        if start == Start::Reset {
            let t = Instant::now();
            if page.eval(conn, RESET_PROBE)? == Value::Bool(true) {
                // The scroll's handlers run, as a load's scripts have run before its first Tab.
                page.eval(conn, READY)?;
                times.reset = Some(ms(t));
                return Ok(());
            }
        }
        fresh_load(conn, times)
    };
    let mut record = Vec::new();
    if let Some(skip_link) = &k.skip_link {
        fresh(conn, plan.skip_link.unwrap_or(Start::Load), times)?;
        page.press(conn, "Tab")?;
        times.tabs += 1;
        let active = page.eval(conn, ACTIVE_PROBE)?;
        let label = active.get_str("label").unwrap_or_default().to_string();
        let href = active.get_str("href").unwrap_or_default().to_string();
        let mut after_enter = false;
        let mut after_tab = Value::Null;
        if skip_link.matches(&label) {
            page.press(conn, "Enter")?;
            page.wait(conn, k.focus_settle_ms)?;
            after_enter = page.run(conn, PAST_PROBE, &[s(&href)])? == Value::Bool(true);
            if !after_enter {
                page.press(conn, "Tab")?;
                times.tabs += 1;
                after_tab = page.run(conn, PAST_PROBE, &[s(&href)])?;
            }
        }
        record.push(("first_label", s(&label)));
        record.push(("after_enter", Value::Bool(after_enter)));
        record.push(("after_tab", after_tab));
    }
    fresh(conn, plan.tabs, times)?;
    let targets = keyboard_targets(conn, page, config, k)?;
    let mut reached: Vec<Option<usize>> = targets.iter().map(|_| None).collect();
    let mut marks = Vec::new();
    let mut tabs = Vec::new();
    for press in 1..=k.max_tabs {
        if targets
            .iter()
            .zip(&reached)
            .all(|((_, object), r)| object.is_none() || r.is_some())
        {
            break;
        }
        page.press(conn, "Tab")?;
        times.tabs += 1;
        let mut here = Vec::new();
        for (i, (name, object)) in targets.iter().enumerate() {
            let Some(object) = object else { continue };
            if reached[i].is_none()
                && page.run_on(conn, object, FOCUS_WITHIN, &[])? == Value::Bool(true)
            {
                reached[i] = Some(press);
                here.push(s(name));
                if k.focus_mark {
                    marks.push((name.clone(), page.eval(conn, FOCUS_MARK_PROBE)?));
                }
            }
        }
        tabs.push(Value::List(here));
    }
    let mut activations = Vec::new();
    for (i, (name, _)) in targets.iter().enumerate() {
        if let Some(presses) = reached[i]
            && k.activates.contains(name)
        {
            fresh_load(conn, times)?;
            times.tabs += presses as i64;
            activations.push((name.clone(), activate(conn, page, config, k, i, presses)?));
        }
    }
    record.push((
        "located",
        Value::Map(
            targets
                .iter()
                .map(|(n, o)| (n.clone(), Value::Bool(o.is_some())))
                .collect(),
        ),
    ));
    record.push(("tabs", Value::List(tabs)));
    if k.focus_mark {
        record.push(("marks", Value::Map(marks)));
    }
    if !k.activates.is_empty() {
        record.push(("activations", Value::Map(activations)));
    }
    Ok(map(record))
}

/// The walk's targets on the loaded page, in `must_reach` order: each with the first visible
/// candidate's remote object, or none.
fn keyboard_targets(
    conn: &mut Conn,
    page: &Page,
    config: &Config,
    k: &KeyboardConfig,
) -> Result<Vec<(String, Option<String>)>, String> {
    let walked = config
        .always_visible
        .iter()
        .filter(|a| k.must_reach.contains(&a.name));
    let tree = ax_tree_for(conn, page, walked)?;
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
            if checks::located(&item.locate, std::slice::from_ref(&c)).is_some() {
                found = Some(object);
                break;
            }
        }
        targets.push((name.clone(), found));
    }
    Ok(targets)
}

/// The browser's events of a navigation of the page: a link followed, a form sent, a new hash.
const NAVIGATION_EVENTS: [&str; 3] = [
    "Page.frameRequestedNavigation",
    "Page.frameStartedLoading",
    "Page.navigatedWithinDocument",
];

/// How long a navigation an activation started may take to stop loading before the walk goes on.
const NAVIGATION_SETTLE: Duration = Duration::from_secs(10);

/// Whether the page reported a navigation since its events were last dropped. One that loads a
/// document is waited for (`Page.frameStoppedLoading`, at most [`NAVIGATION_SETTLE`]), so that the
/// page is not closed or loaded again while it loads.
fn navigated(conn: &mut Conn, page: &Page) -> bool {
    let seen: Vec<bool> = NAVIGATION_EVENTS
        .iter()
        .map(|e| {
            conn.wait_event(Some(&page.session), e, Duration::ZERO)
                .is_ok()
        })
        .collect();
    if seen[0] || seen[1] {
        // A load that does not stop in time is left to the next fresh load or to closing the page.
        let _ = conn.wait_event(
            Some(&page.session),
            "Page.frameStoppedLoading",
            NAVIGATION_SETTLE,
        );
    }
    seen.contains(&true)
}

/// One key on the focused target and what it did within `focus_settle_ms`.
fn press_and_watch(
    conn: &mut Conn,
    page: &Page,
    k: &KeyboardConfig,
    key: &str,
) -> Result<Value, Failure> {
    conn.forget(Some(&page.session));
    page.press(conn, key)?;
    page.wait(conn, k.focus_settle_ms)?;
    let went = || {
        map(vec![
            ("navigated", Value::Bool(true)),
            ("focus_moved", Value::Bool(false)),
            ("state_changed", Value::Bool(false)),
            ("dom_changed", Value::Bool(false)),
        ])
    };
    if navigated(conn, page) {
        return Ok(went());
    }
    match page.run(conn, ACTIVATE_PROBE, &[s("report")]) {
        // The probe's state is gone: a new document.
        Ok(Value::Null) => Ok(went()),
        Ok(facts) => Ok(facts),
        // A navigation that began after the events were read destroys the page's context.
        Err(_) if navigated(conn, page) => Ok(went()),
        Err(e) => Err(Failure::Browser(e)),
    }
}

/// The activation of target `i` on a fresh load: `presses` Tabs, focus on the target or inside it
/// again, a quiet watch, Enter, and Space when Enter did not act (`probes/activate.js`).
fn activate(
    conn: &mut Conn,
    page: &Page,
    config: &Config,
    k: &KeyboardConfig,
    i: usize,
    presses: usize,
) -> Result<Value, Failure> {
    let targets = keyboard_targets(conn, page, config, k)?;
    for _ in 0..presses {
        page.press(conn, "Tab")?;
    }
    let refocused = match &targets[i].1 {
        Some(object) => page.run_on(conn, object, FOCUS_WITHIN, &[])? == Value::Bool(true),
        None => false,
    };
    let mut out = vec![
        ("refocused", Value::Bool(refocused)),
        ("enter", Value::Null),
        ("space", Value::Null),
    ];
    if !refocused {
        return Ok(map(out));
    }
    page.run(conn, ACTIVATE_PROBE, &[s("watch")])?;
    page.wait(conn, k.focus_settle_ms)?;
    page.run(conn, ACTIVATE_PROBE, &[s("mark")])?;
    let enter = press_and_watch(conn, page, k, "Enter")?;
    let acted = checks::KeyFacts::from_value(&enter)
        .map_err(Failure::Browser)?
        .any();
    out[1].1 = enter;
    if !acted {
        out[2].1 = press_and_watch(conn, page, k, "Space")?;
    }
    Ok(map(out))
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
    let mut browser = Browser::start(inputs.profile, &[])?;
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
            guard: None,
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
