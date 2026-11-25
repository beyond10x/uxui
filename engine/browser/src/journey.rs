//! Journeys (`docs/browser-adapter.md`, Journeys): steps named by CSS selector that bring a page from
//! its load into a state (open a menu, fill a form, open a `details`), and the states at which the
//! checks that need no fresh load run: page scroll, overflow and the always-visible items. The
//! browser run (`measure.rs`) records each state's geometry; [`journey_result`] turns it into values,
//! as `checks::page_result` does for a page after load.

use crate::checks::{self, Config, NamedViewport, OverflowRaw, UnitResult};
use crate::value::{get_bool, get_i64};
use uxui_doc::yaml::Value;

/// How long a `wait` step waits for its selector, by default.
pub const DEFAULT_TIMEOUT_MS: u64 = 5_000;

/// The keys a `press` step may name: the keys the keyboard walk presses, and Escape.
pub const KEYS: &[&str] = &["Tab", "Enter", "Space", "Escape"];

/// The position the always-visible items of a state are checked at: where the steps left the page.
pub const POSITION: &str = "state";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Top,
    Bottom,
}

impl Edge {
    pub fn word(self) -> &'static str {
        match self {
            Edge::Top => "top",
            Edge::Bottom => "bottom",
        }
    }
}

/// One step of a journey.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// A click at the centre of the first rendered element the selector matches.
    Click(String),
    /// The text typed into the first rendered element the selector matches, in place of its value.
    Type { css: String, text: String },
    /// A key pressed on the focused element ([`KEYS`]).
    Press(String),
    /// Until the selector matches a rendered element, at most the journey's `timeout_ms`.
    Wait(String),
    /// The page scrolled to its top or its bottom, then the scroll settle time.
    Scroll(Edge),
    /// A state: the checks run here.
    State(String),
}

impl Step {
    /// The step as a reason names it: `click `#toggle``.
    pub fn describe(&self) -> String {
        match self {
            Step::Click(css) => format!("click `{css}`"),
            Step::Type { css, .. } => format!("type `{css}`"),
            Step::Press(key) => format!("press `{key}`"),
            Step::Wait(css) => format!("wait `{css}`"),
            Step::Scroll(edge) => format!("scroll `{}`", edge.word()),
            Step::State(name) => format!("state `{name}`"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Journey {
    pub name: String,
    /// A path served as one of `pages.paths` is.
    pub page: String,
    pub steps: Vec<Step>,
    pub timeout_ms: u64,
}

impl Journey {
    /// The names of its states, in order.
    pub fn states(&self) -> Vec<&str> {
        self.steps
            .iter()
            .filter_map(|s| match s {
                Step::State(name) => Some(name.as_str()),
                _ => None,
            })
            .collect()
    }
}

const STEP_KEYS: &[&str] = &["click", "type", "press", "wait", "scroll", "state"];

/// A journey's or a state's name: text, not empty, without `/` (the scope item is
/// `<journey>/<state>`) and without a line break.
fn name_ok(name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("is empty".to_string());
    }
    if name.contains('/') {
        return Err("has a `/`, which separates the journey from the state".to_string());
    }
    if name.contains('\n') || name.contains('\r') {
        return Err("has a line break".to_string());
    }
    Ok(())
}

fn selector(v: &Value, key: &str, at: &str) -> Result<String, String> {
    match v.get(key) {
        Some(Value::Str(css)) if !css.trim().is_empty() => Ok(css.clone()),
        _ => Err(format!("{at}: `{key}` names a CSS selector")),
    }
}

fn step(v: &Value, at: &str) -> Result<Step, String> {
    if !matches!(v, Value::Map(_)) {
        return Err(format!(
            "{at}: a step is a map with one of {}",
            STEP_KEYS.join(", ")
        ));
    }
    for (k, _) in v.entries() {
        if !STEP_KEYS.contains(&k.as_str()) && k != "text" {
            return Err(format!(
                "{at}: unknown key `{k}`; a step is one of {}",
                STEP_KEYS.join(", ")
            ));
        }
    }
    let actions: Vec<&str> = v
        .entries()
        .iter()
        .map(|(k, _)| k.as_str())
        .filter(|k| STEP_KEYS.contains(k))
        .collect();
    let [action] = actions[..] else {
        return Err(format!(
            "{at}: one of {} per step, not {}",
            STEP_KEYS.join(", "),
            if actions.is_empty() {
                "none".to_string()
            } else {
                actions.join(" and ")
            }
        ));
    };
    if action != "type" && v.get("text").is_some() {
        return Err(format!("{at}: `text` goes with `type` only"));
    }
    Ok(match action {
        "click" => Step::Click(selector(v, "click", at)?),
        "type" => {
            let css = selector(v, "type", at)?;
            let text = match v.get("text") {
                Some(t) => t
                    .scalar_text()
                    .ok_or_else(|| format!("{at}: `text` is text"))?,
                None => return Err(format!("{at}: `type` needs the `text` it types")),
            };
            Step::Type { css, text }
        }
        "press" => {
            let key = v.get_str("press").unwrap_or_default();
            if !KEYS.contains(&key) {
                return Err(format!(
                    "{at}: key `{}` is not one of {}",
                    v.get("press")
                        .and_then(Value::scalar_text)
                        .unwrap_or_default(),
                    KEYS.join(", ")
                ));
            }
            Step::Press(key.to_string())
        }
        "wait" => Step::Wait(selector(v, "wait", at)?),
        "scroll" => match v.get_str("scroll") {
            Some("top") => Step::Scroll(Edge::Top),
            Some("bottom") => Step::Scroll(Edge::Bottom),
            _ => return Err(format!("{at}: `scroll` is `top` or `bottom`")),
        },
        _ => {
            let name = v
                .get("state")
                .and_then(Value::scalar_text)
                .ok_or_else(|| format!("{at}: `state` names the state"))?;
            name_ok(&name).map_err(|e| format!("{at}: state `{name}` {e}"))?;
            Step::State(name)
        }
    })
}

/// The `journeys` section: a list of `{name, page, steps, timeout_ms}`.
pub fn config(v: Option<&Value>) -> Result<Vec<Journey>, String> {
    let Some(list) = v else {
        return Ok(Vec::new());
    };
    let Value::List(items) = list else {
        return Err("`journeys` is a list of journeys".to_string());
    };
    let mut out: Vec<Journey> = Vec::new();
    for (i, j) in items.iter().enumerate() {
        if !matches!(j, Value::Map(_)) {
            return Err(format!(
                "journey {}: a map of `name`, `page`, `steps`",
                i + 1
            ));
        }
        let name = j
            .get("name")
            .and_then(Value::scalar_text)
            .ok_or_else(|| format!("journey {}: no `name`", i + 1))?;
        let what = format!("journey `{name}`");
        name_ok(&name).map_err(|e| format!("{what}: the name {e}"))?;
        for (k, _) in j.entries() {
            if !["name", "page", "steps", "timeout_ms"].contains(&k.as_str()) {
                return Err(format!("{what}: unknown key `{k}`"));
            }
        }
        if out.iter().any(|o| o.name == name) {
            return Err(format!("{what} is declared twice"));
        }
        let page = match j.get("page") {
            Some(Value::Str(p)) if p.starts_with('/') => p.clone(),
            Some(Value::Str(p)) => {
                return Err(format!("{what}: page `{p}` does not start with `/`"));
            }
            _ => return Err(format!("{what}: no `page`, the path it starts on")),
        };
        let timeout_ms = match j.get("timeout_ms") {
            None => DEFAULT_TIMEOUT_MS,
            Some(Value::Int(n)) if *n > 0 => *n as u64,
            Some(_) => {
                return Err(format!(
                    "{what}: `timeout_ms` is a whole number of milliseconds above 0"
                ));
            }
        };
        let steps_v = match j.get("steps") {
            Some(Value::List(s)) => s,
            _ => return Err(format!("{what}: `steps` is a list of steps")),
        };
        if steps_v.is_empty() {
            return Err(format!("{what}: no steps"));
        }
        let mut steps: Vec<Step> = Vec::new();
        for (n, s) in steps_v.iter().enumerate() {
            let at = format!("{what}: step {}", n + 1);
            let parsed = step(s, &at)?;
            if let Step::State(state) = &parsed
                && steps
                    .iter()
                    .any(|o| matches!(o, Step::State(x) if x == state))
            {
                return Err(format!("{at}: state `{state}` is declared twice"));
            }
            steps.push(parsed);
        }
        if !steps.iter().any(|s| matches!(s, Step::State(_))) {
            return Err(format!(
                "{what}: no `state`, so no checks run; declare at least one"
            ));
        }
        out.push(Journey {
            name,
            page,
            steps,
            timeout_ms,
        });
    }
    Ok(out)
}

// ---------------------------------------------------------------- results

/// One state at one viewport in one mode: its checks, or why it was not reached.
pub struct StateRun {
    pub viewport: String,
    pub mode: String,
    pub outcome: Result<UnitResult, String>,
}

pub struct StateResult {
    pub name: String,
    pub runs: Vec<StateRun>,
}

pub struct JourneyResult {
    pub name: String,
    pub page: String,
    pub states: Vec<StateResult>,
}

impl StateResult {
    /// The reasons of the runs that did not reach the state, as `<viewport>/<mode>: <reason>`.
    pub fn skipped(&self) -> Vec<String> {
        self.runs
            .iter()
            .filter_map(|r| {
                r.outcome
                    .as_ref()
                    .err()
                    .map(|e| format!("{}/{}: {e}", r.viewport, r.mode))
            })
            .collect()
    }
    pub fn units(&self) -> impl Iterator<Item = &UnitResult> {
        self.runs.iter().filter_map(|r| r.outcome.as_ref().ok())
    }
}

/// Why a run stopped: the page did not load (step 0), or a step failed.
fn failure_reason(journey: &Journey, failed: &Value) -> String {
    let error = failed.get_str("error").unwrap_or("no reason recorded");
    match get_i64(failed, "step") {
        Some(n) if n >= 1 => match journey.steps.get(n as usize - 1) {
            Some(step) => format!("step {n}, {}: {error}", step.describe()),
            None => format!("step {n}: {error}"),
        },
        _ => format!("the page did not load: {error}"),
    }
}

/// The values of one journey from its recorded runs (`measure.rs`, `--capture`).
pub fn journey_result(config: &Config, recorded: &Value) -> Result<JourneyResult, String> {
    let name = recorded
        .get_str("journey")
        .ok_or("journey: no `journey`")?
        .to_string();
    let journey = config
        .journeys
        .iter()
        .find(|j| j.name == name)
        .ok_or_else(|| format!("journey `{name}` is not declared"))?;
    let mut states: Vec<StateResult> = journey
        .states()
        .into_iter()
        .map(|s| StateResult {
            name: s.to_string(),
            runs: Vec::new(),
        })
        .collect();
    for vp_rec in recorded
        .get("viewports")
        .map(Value::list)
        .unwrap_or_default()
    {
        let vp_name = vp_rec
            .get_str("viewport")
            .ok_or("journey: a viewport without a name")?;
        let vp: &NamedViewport = config
            .viewports
            .iter()
            .find(|v| v.name == vp_name)
            .ok_or_else(|| format!("journey `{name}`: viewport `{vp_name}` is not declared"))?;
        for run in vp_rec.get("runs").map(Value::list).unwrap_or_default() {
            let mode = run
                .get_str("mode")
                .ok_or("journey: a run without a mode")?
                .to_string();
            let mut stopped = match run.get("failed") {
                None | Some(Value::Null) => None,
                Some(f) => Some(failure_reason(journey, f)),
            };
            if stopped.is_none() && get_bool(run, "mode_applied") == Some(false) {
                stopped = Some("the page did not apply the mode".to_string());
            }
            let recorded_states = run.get("states").map(Value::list).unwrap_or_default();
            for state in states.iter_mut() {
                let found = recorded_states
                    .iter()
                    .find(|r| r.get_str("state") == Some(state.name.as_str()));
                let outcome = match (found, &stopped) {
                    (Some(r), _) => Ok(state_unit(config, journey, vp, &mode, r)?),
                    (None, Some(reason)) => Err(reason.clone()),
                    (None, None) => {
                        return Err(format!(
                            "journey `{name}` {vp_name} {mode}: no record of state `{}`",
                            state.name
                        ));
                    }
                };
                state.runs.push(StateRun {
                    viewport: vp.name.clone(),
                    mode: mode.clone(),
                    outcome,
                });
            }
        }
    }
    Ok(JourneyResult {
        name,
        page: journey.page.clone(),
        states,
    })
}

fn state_unit(
    config: &Config,
    journey: &Journey,
    vp: &NamedViewport,
    mode: &str,
    recorded: &Value,
) -> Result<UnitResult, String> {
    let at = format!(
        "journey `{}` {} {mode} state `{}`",
        journey.name,
        vp.name,
        recorded.get_str("state").unwrap_or_default()
    );
    let raw = OverflowRaw::from_value(recorded.get("overflow").ok_or("state: no `overflow`")?)?;
    let rule = &config.layout.overflow;
    if raw.elements != rule.elements || raw.exclude != rule.exclude {
        return Err(format!(
            "{at}: the overflow was recorded with other `elements` or `exclude` than the configuration's"
        ));
    }
    let visible = checks::visible_of(config, &journey.page, vp, POSITION, recorded.get("visible"))
        .map_err(|e| format!("{at}: {e}"))?;
    Ok(UnitResult {
        viewport: vp.name.clone(),
        mode: mode.to_string(),
        mode_applied: true,
        page_scroll_px: checks::page_scroll(&raw),
        page_slack_px: config.layout.page_slack_px,
        overflow: checks::overflow_with(&raw, rule),
        visible,
    })
}

/// A capture file name of a journey: `journey-<name in lower case, other characters `-`>`.
pub fn capture_name(name: &str) -> String {
    let slug: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    format!("journey-{slug}")
}
