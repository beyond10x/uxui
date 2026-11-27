//! The values file of a checks document (`docs/browser-adapter.md`, `ux-browser values`): per page
//! scenario the numbers a consumer's ratchet compares, per page, viewport and mode the hard-gate
//! failures with the open finding record that waives each, the incomplete results and the pages
//! without a scenario. Where the scenarios and the finding records lie, which gate each output counts
//! against and which viewports gate are the consumer's, declared in the `values` section of its
//! `ux-browser-checks/1` file; nothing here names a consumer.
//!
//! The file is written as `jq -S` writes JSON: keys sorted by their bytes, two spaces of indent, each
//! number as the document's decimal text.

use crate::checks;
use crate::value::get_strs;
use std::path::{Path, PathBuf};
use uxui_doc::yaml::Value;

/// The outputs that have findings, and so may count against a gate.
pub const GATED_OUTPUTS: [&str; 4] = [
    checks::PAGE_SCROLL,
    checks::OVERFLOW,
    checks::ALWAYS_VISIBLE,
    checks::KEYBOARD,
];

pub struct ScenarioSource {
    /// A path pattern under the root.
    pub files: String,
    /// The key of each file's object that holds the page path.
    pub path_key: String,
}

pub struct WaiverSource {
    /// A path pattern under the root.
    pub files: String,
    /// The statuses of a record that waives nothing.
    pub closed: Vec<String>,
}

pub struct ValuesConfig {
    pub scenarios: ScenarioSource,
    pub waivers: WaiverSource,
    /// Output name and gate number, in the order declared.
    pub gates: Vec<(String, i64)>,
    /// The viewports whose results are values and failures; `None`: every viewport.
    pub gating: Option<Vec<String>>,
}

fn only(v: &Value, keys: &[&str], what: &str) -> Result<(), String> {
    for (k, _) in v.entries() {
        if !keys.contains(&k.as_str()) {
            return Err(format!("{what}: unknown key `{k}`"));
        }
    }
    Ok(())
}

/// The `values` section of a checks configuration, checked against its viewport names.
pub fn config(v: &Value, viewports: &[String]) -> Result<ValuesConfig, String> {
    if !matches!(v, Value::Map(_)) {
        return Err("`values` is a map".to_string());
    }
    only(v, &["scenarios", "waivers", "gates", "gating"], "values")?;
    let s = v.get("scenarios").ok_or("values: no `scenarios`")?;
    only(s, &["files", "path_key"], "values.scenarios")?;
    let scenarios = ScenarioSource {
        files: s
            .get_str("files")
            .ok_or("values.scenarios: no `files`")?
            .to_string(),
        path_key: s.get_str("path_key").unwrap_or("path").to_string(),
    };
    let w = v.get("waivers").ok_or("values: no `waivers`")?;
    only(w, &["files", "closed"], "values.waivers")?;
    let closed = match w.get("closed") {
        None => vec!["closed".to_string()],
        Some(_) => get_strs(w, "closed")?,
    };
    let waivers = WaiverSource {
        files: w
            .get_str("files")
            .ok_or("values.waivers: no `files`")?
            .to_string(),
        closed,
    };
    let mut gates = Vec::new();
    for (output, gate) in v.get("gates").ok_or("values: no `gates`")?.entries() {
        if !GATED_OUTPUTS.contains(&output.as_str()) {
            return Err(format!(
                "values.gates: `{output}` is not an output with findings ({})",
                GATED_OUTPUTS.join(", ")
            ));
        }
        match gate {
            Value::Int(n) if *n >= 1 => gates.push((output.clone(), *n)),
            _ => {
                return Err(format!(
                    "values.gates: the gate of `{output}` is a whole number of at least 1"
                ));
            }
        }
    }
    let gating = match v.get("gating") {
        None => None,
        Some(_) => {
            let names = get_strs(v, "gating")?;
            for n in &names {
                if !viewports.contains(n) {
                    return Err(format!("values.gating: viewport `{n}` is not declared"));
                }
            }
            if names.is_empty() {
                return Err("values.gating: names no viewport".to_string());
            }
            Some(names)
        }
    };
    Ok(ValuesConfig {
        scenarios,
        waivers,
        gates,
        gating,
    })
}

// ---------------------------------------------------------------- patterns

/// `*` in `pattern` stands for any run of bytes; everything else is itself; the whole of `text`.
fn star_match(pattern: &[u8], text: &[u8]) -> bool {
    let (mut p, mut t) = (0, 0);
    let mut back: Option<(usize, usize)> = None;
    while t < text.len() {
        if p < pattern.len() && pattern[p] == b'*' {
            back = Some((p, t));
            p += 1;
        } else if p < pattern.len() && pattern[p] == text[t] {
            p += 1;
            t += 1;
        } else if let Some((bp, bt)) = back {
            p = bp + 1;
            t = bt + 1;
            back = Some((bp, bt + 1));
        } else {
            return false;
        }
    }
    pattern[p..].iter().all(|&c| c == b'*')
}

/// Whether a waiver pattern matches a whole failure key; `*` stands for any run of characters.
pub fn waiver_matches(pattern: &str, key: &str) -> bool {
    star_match(pattern.as_bytes(), key.as_bytes())
}

/// The files under `root` that a path pattern matches, in the byte order of their paths: `*` stands
/// for any run of characters inside one segment, never a leading `.`.
pub fn glob(root: &Path, pattern: &str) -> Result<Vec<PathBuf>, String> {
    let mut found = vec![root.to_path_buf()];
    let segments: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    for (i, seg) in segments.iter().enumerate() {
        let last = i + 1 == segments.len();
        let mut next = Vec::new();
        for dir in &found {
            if !seg.contains('*') {
                let p = dir.join(seg);
                if (last && p.is_file()) || (!last && p.is_dir()) {
                    next.push(p);
                }
                continue;
            }
            let Ok(entries) = std::fs::read_dir(dir) else {
                continue;
            };
            for entry in entries {
                let entry = entry.map_err(|e| format!("{}: {e}", dir.display()))?;
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') && !seg.starts_with('.') {
                    continue;
                }
                let p = entry.path();
                if star_match(seg.as_bytes(), name.as_bytes())
                    && ((last && p.is_file()) || (!last && p.is_dir()))
                {
                    next.push(p);
                }
            }
        }
        found = next;
    }
    if segments.is_empty() {
        return Ok(Vec::new());
    }
    found.sort_by(|a, b| {
        a.as_os_str()
            .as_encoded_bytes()
            .cmp(b.as_os_str().as_encoded_bytes())
    });
    Ok(found)
}

// ---------------------------------------------------------------- inputs

pub struct Waiver {
    pub id: String,
    pub pattern: String,
}

/// What the files under the root say: page path to scenario name, and the waivers of the open
/// records in file order.
pub struct Inputs {
    pub scenarios: Vec<(String, String)>,
    pub waivers: Vec<Waiver>,
}

impl Inputs {
    fn scenario(&self, path: &str) -> Option<&str> {
        // The last file read wins, as an object built from them in order.
        self.scenarios
            .iter()
            .rev()
            .find(|(p, _)| p == path)
            .map(|(_, n)| n.as_str())
    }
}

/// The values of the lines `<key>: <value>` of a header, each with the spaces after the colon
/// removed, joined by newlines.
fn header_field(lines: &[&str], key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    let found: Vec<&str> = lines
        .iter()
        .filter_map(|l| l.strip_prefix(prefix.as_str()))
        .map(|v| v.trim_start_matches(' '))
        .collect();
    (!found.is_empty()).then(|| found.join("\n"))
}

/// The front matter of a record: from its second line to the next line `---`, that line included.
fn front_matter(text: &str) -> Vec<&str> {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut out = Vec::new();
    for (i, line) in lines.iter().enumerate().skip(1) {
        out.push(*line);
        if i > 1 && *line == "---" {
            break;
        }
    }
    out
}

/// Reads the scenario files and the finding records the section names, under `root`.
pub fn inputs(c: &ValuesConfig, root: &Path) -> Result<Inputs, String> {
    let mut scenarios = Vec::new();
    for file in glob(root, &c.scenarios.files)? {
        let text =
            std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
        let v = crate::json::parse(&text).map_err(|e| format!("{}: {e}", file.display()))?;
        let path = v.get_str(&c.scenarios.path_key).ok_or_else(|| {
            format!(
                "{}: no `{}` that is text",
                file.display(),
                c.scenarios.path_key
            )
        })?;
        let name = file
            .parent()
            .and_then(Path::file_name)
            .map(|n| n.to_string_lossy().to_string())
            .ok_or_else(|| format!("{}: no directory", file.display()))?;
        scenarios.push((path.to_string(), name));
    }
    let mut waivers = Vec::new();
    for file in glob(root, &c.waivers.files)? {
        let text =
            std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
        let header = front_matter(&text);
        let status = header_field(&header, "status").unwrap_or_default();
        if c.waivers.closed.contains(&status) {
            continue;
        }
        let id = header_field(&header, "id").unwrap_or_default();
        for line in header.iter().filter_map(|l| l.strip_prefix("waives:")) {
            for pattern in line.split(',') {
                let pattern = pattern.trim();
                if !pattern.is_empty() {
                    waivers.push(Waiver {
                        id: id.clone(),
                        pattern: pattern.to_string(),
                    });
                }
            }
        }
    }
    Ok(Inputs { scenarios, waivers })
}

// ---------------------------------------------------------------- the file

/// A JSON value as `jq -S` writes it; a number keeps its decimal text.
enum Out {
    Null,
    Bool(bool),
    Num(String),
    Str(String),
    List(Vec<Out>),
    Obj(Vec<(String, Out)>),
}

fn jq_string(text: &str, out: &mut String) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn put(v: &Out, indent: usize, out: &mut String) {
    let pad = |n: usize, out: &mut String| out.extend(std::iter::repeat_n(' ', n));
    match v {
        Out::Null => out.push_str("null"),
        Out::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Out::Num(n) => out.push_str(n),
        Out::Str(s) => jq_string(s, out),
        Out::List(items) if items.is_empty() => out.push_str("[]"),
        Out::List(items) => {
            out.push_str("[\n");
            for (i, item) in items.iter().enumerate() {
                pad(indent + 2, out);
                put(item, indent + 2, out);
                out.push_str(if i + 1 < items.len() { ",\n" } else { "\n" });
            }
            pad(indent, out);
            out.push(']');
        }
        Out::Obj(entries) if entries.is_empty() => out.push_str("{}"),
        Out::Obj(entries) => {
            let mut sorted: Vec<&(String, Out)> = entries.iter().collect();
            sorted.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
            out.push_str("{\n");
            for (i, (k, item)) in sorted.iter().enumerate() {
                pad(indent + 2, out);
                jq_string(k, out);
                out.push_str(": ");
                put(item, indent + 2, out);
                out.push_str(if i + 1 < sorted.len() { ",\n" } else { "\n" });
            }
            pad(indent, out);
            out.push('}');
        }
    }
}

fn obj(entries: Vec<(&str, Out)>) -> Out {
    Out::Obj(
        entries
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    )
}

/// A decimal number's text, `-?[0-9]+(.[0-9]+)?`.
fn is_decimal(text: &str) -> bool {
    let digits = text.strip_prefix('-').unwrap_or(text);
    let (whole, frac) = match digits.split_once('.') {
        Some((w, f)) => (w, Some(f)),
        None => (digits, None),
    };
    let all = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    all(whole) && frac.is_none_or(all)
}

/// A value of the document as a number's text.
fn number_text(v: &Value, what: &str) -> Result<String, String> {
    match v {
        Value::Str(s) if is_decimal(s) => Ok(s.clone()),
        Value::Int(n) => Ok(n.to_string()),
        _ => Err(format!("{what}: the value is not a decimal number")),
    }
}

/// The output without its `@<version>` suffix.
fn metric(output: &str) -> &str {
    match output.rfind('@') {
        Some(i) if i + 1 < output.len() && output[i + 1..].bytes().all(|b| b.is_ascii_digit()) => {
            &output[..i]
        }
        _ => output,
    }
}

fn text_or_null(v: Option<&Value>) -> String {
    match v {
        None | Some(Value::Null) => "null".to_string(),
        Some(Value::Str(s)) => s.clone(),
        Some(other) => uxui_doc::json::write(other),
    }
}

/// Sums (`exact`) or the largest (`sampled`) of per-viewport values, as text; the smallest for a
/// metric whose name ends in `_min` (`a11y.contrast_min`, the lowest contrast).
fn fold(kind: &str, metric: &str, values: &[String]) -> Option<String> {
    if values.is_empty() {
        return None;
    }
    let name = metric.split('#').next().unwrap_or(metric);
    if name.ends_with("_min") {
        return values
            .iter()
            .min_by(|a, b| {
                let (x, y) = (
                    a.parse::<f64>().unwrap_or(0.0),
                    b.parse::<f64>().unwrap_or(0.0),
                );
                x.total_cmp(&y)
            })
            .cloned();
    }
    if kind == "sampled" {
        values
            .iter()
            .max_by(|a, b| {
                let (x, y) = (
                    a.parse::<f64>().unwrap_or(0.0),
                    b.parse::<f64>().unwrap_or(0.0),
                );
                x.total_cmp(&y)
            })
            .cloned()
    } else {
        Some(
            values
                .iter()
                .map(|v| v.parse::<i64>().unwrap_or(0))
                .sum::<i64>()
                .to_string(),
        )
    }
}

/// The per-viewport values of a number, `details.viewports`, in the document's order.
fn per_viewport(result: &Value, what: &str) -> Result<Vec<(String, String)>, String> {
    let d = result
        .get("details")
        .and_then(|d| d.get("viewports"))
        .ok_or_else(|| {
            format!("{what}: no per-viewport values (`details.viewports`); `gating` needs a document of ux-browser 0.3.0 or later")
        })?;
    d.entries()
        .iter()
        .map(|(vp, v)| Ok((vp.clone(), number_text(v, what)?)))
        .collect()
}

/// How a number folds over viewports: a share (a metric named `*_share`, `visual.diff_share`) as
/// the largest, as a `sampled` number does; every other number by its kind.
fn fold_kind<'a>(output: &str, kind: &'a str) -> &'a str {
    if metric(output).ends_with("_share") {
        "sampled"
    } else {
        kind
    }
}

struct Failure {
    gate: i64,
    detail: String,
    key: String,
    waived_by: Option<String>,
}

/// What the values of a document are: the file's text, the problems that make the run exit 1, and
/// one line per waived failure.
pub struct Values {
    pub text: String,
    pub problems: Vec<String>,
    pub waived: Vec<String>,
}

/// The detail of a finding of `output` with node path `<what>`; `None` when it is left out.
fn detail(output: &str, what: &str, reason: &str, skip_not_first: bool) -> Option<String> {
    match output {
        checks::PAGE_SCROLL => Some("page-scroll".to_string()),
        checks::OVERFLOW => Some(format!("overflow:{what}")),
        checks::ALWAYS_VISIBLE => {
            let (item, at) = what.rsplit_once('@')?;
            Some(if reason == "missing" {
                format!("{item}:{reason}")
            } else {
                format!("{item}:{reason}@{at}")
            })
        }
        checks::KEYBOARD => {
            if what == "skip-link-moves-focus" && skip_not_first {
                None
            } else {
                Some(format!("keyboard:{what}"))
            }
        }
        _ => None,
    }
}

/// The values file of `document` (a `ratchet-evidence/1` document of `ux-browser checks`).
pub fn values(c: &ValuesConfig, inputs: &Inputs, document: &Value) -> Result<Values, String> {
    let results = document
        .get("evidence")
        .and_then(|e| e.get("results"))
        .ok_or("the document has no `evidence.results`")?
        .list();
    let gating = |vp: &str| c.gating.as_ref().is_none_or(|g| g.iter().any(|n| n == vp));

    let mut incomplete = Vec::new();
    let mut unmapped: Vec<String> = Vec::new();
    // Scenario name -> (metric, number text), in document order.
    let mut values: Vec<(String, Vec<(String, String)>)> = Vec::new();
    // Viewport -> (pages, metric -> collected texts and kind).
    #[allow(clippy::type_complexity)]
    let mut by_viewport: Vec<(String, Vec<String>, Vec<(String, String, Vec<String>)>)> =
        Vec::new();
    // (path, viewport, mode) -> findings (rule, what, reason).
    #[allow(clippy::type_complexity)]
    let mut groups: Vec<((String, String, String), Vec<(String, String, String)>)> = Vec::new();

    for r in results {
        let output = r.get_str("output").unwrap_or_default();
        // A journey's state (scope `state`, item `<journey>/<state>`) counts under the scenario of
        // its page; it is shown as `<page>#<journey>/<state>`, and its numbers are its metrics with
        // `#<journey>/<state>`.
        let state = r.get_str("scope") == Some(crate::evidence::SCOPE_STATE);
        let (item, shown) = match (r.get_str("item"), state) {
            (Some(i), false) => (Some(i.to_string()), Some(i.to_string())),
            (Some(i), true) => match r.get("details").and_then(|d| d.get_str("page")) {
                Some(page) => (Some(page.to_string()), Some(format!("{page}#{i}"))),
                None => (None, Some(i.to_string())),
            },
            (None, _) => (None, None),
        };
        if r.get("incomplete").is_some_and(|i| !i.is_null())
            && !matches!(r.get("incomplete"), Some(Value::Bool(false)))
        {
            incomplete.push(format!(
                "{} {}: {}",
                match (state, &shown) {
                    (true, Some(shown)) => shown.clone(),
                    _ => text_or_null(r.get("item")),
                },
                text_or_null(r.get("output")),
                text_or_null(r.get("incomplete").and_then(|i| i.get("cause")))
            ));
        }
        let (Some(item), Some(shown)) = (item.as_deref(), shown.as_deref()) else {
            continue;
        };
        let state_of = state.then(|| r.get_str("item").unwrap_or_default());
        let metric_of = |output: &str| match state_of {
            Some(s) => format!("{}#{s}", metric(output)),
            None => metric(output).to_string(),
        };
        let scenario = inputs.scenario(item);
        if scenario.is_none() && !unmapped.iter().any(|u| u == item) {
            unmapped.push(item.to_string());
        }
        let what = format!("{shown} {output}");
        if output == crate::evidence::FINDINGS {
            let prefix = state_of.map(|s| format!("journey/{s}/"));
            for f in r.get("items").map(Value::list).unwrap_or_default() {
                let id = f.get("identity");
                let rule = id.and_then(|i| i.get_str("rule")).unwrap_or_default();
                let node = id.and_then(|i| i.get_str("node_path")).unwrap_or_default();
                let node = match &prefix {
                    Some(p) => match node.strip_prefix(p.as_str()) {
                        Some(rest) => rest,
                        None => continue,
                    },
                    None => node,
                };
                let mut parts = node.splitn(3, '/');
                let (Some(vp), Some(mode), Some(rest)) = (parts.next(), parts.next(), parts.next())
                else {
                    continue;
                };
                if vp.is_empty() || mode.is_empty() || rest.contains('\n') || !gating(vp) {
                    continue;
                }
                let reason = f
                    .get("fields")
                    .and_then(|x| x.get_str("reason"))
                    .unwrap_or_default();
                let key = (shown.to_string(), vp.to_string(), mode.to_string());
                let entry = (rule.to_string(), rest.to_string(), reason.to_string());
                match groups.iter_mut().find(|(k, _)| *k == key) {
                    Some((_, list)) => list.push(entry),
                    None => groups.push((key, vec![entry])),
                }
            }
            continue;
        }
        let Some(value) = r.get("value").filter(|v| !v.is_null()) else {
            continue;
        };
        // A state some runs did not reach has at most a lower bound: no value.
        if state && r.get("incomplete").is_some() {
            continue;
        }
        let kind = fold_kind(output, r.get_str("kind").unwrap_or("exact"));
        let number = if c.gating.is_some() {
            let per = per_viewport(r, &what)?;
            for (vp, text) in &per {
                let slot = match by_viewport.iter().position(|(v, _, _)| v == vp) {
                    Some(i) => i,
                    None => {
                        by_viewport.push((vp.clone(), Vec::new(), Vec::new()));
                        by_viewport.len() - 1
                    }
                };
                let (_, pages, metrics) = &mut by_viewport[slot];
                if !pages.iter().any(|p| p == item) {
                    pages.push(item.to_string());
                }
                match metrics.iter_mut().find(|(m, _, _)| *m == metric_of(output)) {
                    Some((_, _, texts)) => texts.push(text.clone()),
                    None => metrics.push((metric_of(output), kind.to_string(), vec![text.clone()])),
                }
            }
            let at_gating: Vec<String> = per
                .into_iter()
                .filter(|(vp, _)| gating(vp))
                .map(|(_, t)| t)
                .collect();
            fold(kind, &metric_of(output), &at_gating)
        } else {
            Some(number_text(value, &what)?)
        };
        let (Some(scenario), Some(number)) = (scenario, number) else {
            continue;
        };
        let entry = (metric_of(output), number);
        match values.iter_mut().find(|(s, _)| s == scenario) {
            Some((_, list)) => {
                list.retain(|(m, _)| *m != entry.0);
                list.push(entry);
            }
            None => values.push((scenario.to_string(), vec![entry])),
        }
    }

    groups.sort_by(|a, b| a.0.cmp(&b.0));
    let mut problems: Vec<String> = incomplete
        .iter()
        .map(|i| format!("incomplete: {i}"))
        .collect();
    unmapped.sort();
    problems.extend(unmapped.iter().map(|u| format!("no page scenario for {u}")));
    let mut waived = Vec::new();
    let mut units = Vec::new();
    for ((path, vp, mode), findings) in &groups {
        let skip_not_first = findings
            .iter()
            .any(|(rule, what, _)| rule == checks::KEYBOARD && what == "skip-link-first");
        let mut failures: Vec<Failure> = Vec::new();
        for (rule, what, reason) in findings {
            let Some((_, gate)) = c.gates.iter().find(|(o, _)| o == rule) else {
                continue;
            };
            let Some(detail) = detail(rule, what, reason, skip_not_first) else {
                continue;
            };
            let clean: String = detail
                .chars()
                .map(|ch| if ch == '|' || ch == ',' { ' ' } else { ch })
                .collect();
            let key = format!("gate{gate}|{path}|{vp}|{mode}|{clean}");
            let waived_by = inputs
                .waivers
                .iter()
                .find(|w| waiver_matches(&w.pattern, &key))
                .map(|w| w.id.clone());
            failures.push(Failure {
                gate: *gate,
                detail,
                key,
                waived_by,
            });
        }
        // One per key, sorted by key; the first of equal keys stays.
        failures.sort_by(|a, b| a.key.as_bytes().cmp(b.key.as_bytes()));
        failures.dedup_by(|b, a| a.key == b.key);
        if failures.is_empty() {
            continue;
        }
        for f in &failures {
            match &f.waived_by {
                Some(id) => waived.push(format!("{} by {id}", f.key)),
                None => problems.push(format!(
                    "hard gate failure not waived by an open finding: {}",
                    f.key
                )),
            }
        }
        units.push(obj(vec![
            ("path", Out::Str(path.clone())),
            ("viewport", Out::Str(vp.clone())),
            ("theme", Out::Str(mode.clone())),
            (
                "failures",
                Out::List(
                    failures
                        .into_iter()
                        .map(|f| {
                            obj(vec![
                                ("gate", Out::Num(f.gate.to_string())),
                                ("detail", Out::Str(f.detail)),
                                ("key", Out::Str(f.key)),
                                ("waivedBy", f.waived_by.map_or(Out::Null, Out::Str)),
                            ])
                        })
                        .collect(),
                ),
            ),
        ]));
    }
    // Problems in the order of the file: incomplete, unmapped, then failures.
    let mut top = vec![
        (
            "incomplete",
            Out::List(incomplete.into_iter().map(Out::Str).collect()),
        ),
        (
            "unmapped",
            Out::List(unmapped.into_iter().map(Out::Str).collect()),
        ),
        (
            "values",
            Out::Obj(
                values
                    .into_iter()
                    .map(|(s, list)| {
                        (
                            s,
                            Out::Obj(list.into_iter().map(|(m, n)| (m, Out::Num(n))).collect()),
                        )
                    })
                    .collect(),
            ),
        ),
        ("units", Out::List(units)),
    ];
    if let Some(g) = &c.gating {
        top.push((
            "viewports",
            Out::Obj(
                by_viewport
                    .into_iter()
                    .map(|(vp, pages, metrics)| {
                        let mut e = vec![
                            ("gating".to_string(), Out::Bool(g.contains(&vp))),
                            ("pages".to_string(), Out::Num(pages.len().to_string())),
                        ];
                        for (m, kind, texts) in metrics {
                            if let Some(n) = fold(&kind, &m, &texts) {
                                e.push((m, Out::Num(n)));
                            }
                        }
                        (vp, Out::Obj(e))
                    })
                    .collect(),
            ),
        ));
    }
    let mut text = String::new();
    put(&obj(top), 0, &mut text);
    text.push('\n');
    Ok(Values {
        text,
        problems,
        waived,
    })
}
