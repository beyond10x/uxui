//! The one `ratchet-evidence/1` document of a checks run (ratchet `docs/evidence-format.md`,
//! revision 2), in the shape `ux-static@1` writes: one number per check and page, then the failed
//! checks of each page as an identity set. Field by field: `docs/browser-adapter.md`.
//!
//! The target is of the kind `build`: the served build, identified by its artifact digest
//! ([`build_digest`]), which is also the candidate.

use crate::checks::{self, Config, PageResult};
use crate::render::FileRef;
use crate::value::{decimal, map, s};
use std::path::Path;
use uxui_doc::yaml::Value;

pub const FORMAT: &str = "ratchet-evidence/1";
pub const ASSESSOR: &str = "ux-browser@1";
pub const FINDINGS: &str = "findings";
/// The target kind: a served build.
pub const TARGET_KIND: &str = "build";

/// What the document says about the run beside its results.
pub struct Header<'a> {
    pub project: &'a str,
    pub assessment: &'a str,
    pub class: &'a str,
    pub config: &'a FileRef,
    /// The served build: its digest ([`build_digest`]), and its path when it lies under the root.
    pub build: &'a FileRef,
    /// The `--pages` value, when the run measured a list of pages and not the whole site.
    pub pages: Option<&'a str>,
    /// The local inputs of the run (`jobs`, `capture`, `profile`) with the values it used:
    /// `run.local_inputs`, outside the evidence.
    pub local_inputs: Vec<(&'a str, String)>,
    /// `Chrome/<version>`.
    pub browser: &'a str,
    pub started_at: String,
    pub ended_at: String,
}

fn walk(dir: &Path, rel: &str, out: &mut Vec<(String, std::path::PathBuf)>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        let rel = if rel.is_empty() {
            name
        } else {
            format!("{rel}/{name}")
        };
        if path.is_dir() {
            walk(&path, &rel, out)?;
        } else if path.is_file() {
            out.push((rel, path));
        }
    }
    Ok(())
}

/// The digest of a served build, its artifact digest: SHA-256 over one line `<hex>  <path>\n` per
/// file under `dir` (`<hex>` the SHA-256 of the file's bytes, `<path>` relative to `dir` with `/`),
/// sorted by the bytes of the path, as `sha256sum` prints them. Directories, empty or not, add
/// nothing; a symbolic link counts as what it points to.
pub fn build_digest(dir: &Path) -> Result<String, String> {
    let mut files = Vec::new();
    walk(dir, "", &mut files)?;
    files.sort();
    let mut lines = String::new();
    for (rel, path) in files {
        let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let hex = uxui_doc::sha256::hex(&bytes);
        lines.push_str(hex.trim_start_matches("sha256:"));
        lines.push_str("  ");
        lines.push_str(&rel);
        lines.push('\n');
    }
    Ok(uxui_doc::sha256::hex(lines.as_bytes()))
}

/// Seconds since 1970 as `YYYY-MM-DDTHH:MM:SSZ`.
pub fn utc(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    // Days to a civil date, proleptic Gregorian (H. Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        rem / 60 % 60,
        rem % 60
    )
}

pub fn now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    utc(secs)
}

fn file(f: &FileRef, key: &str, form: &str) -> Value {
    let mut entries = Vec::new();
    if let Some(path) = &f.path {
        entries.push((key, s(path)));
    }
    entries.push(("digest", s(&f.digest)));
    entries.push(("digest_form", s(form)));
    map(entries)
}

#[derive(Default)]
struct Coverage {
    executed: i64,
    not_run: i64,
    unsupported: i64,
    passed: i64,
    failed: i64,
    not_executed: Vec<Value>,
}

impl Coverage {
    fn judged(&mut self, ok: bool) {
        self.executed += 1;
        if ok {
            self.passed += 1;
        } else {
            self.failed += 1;
        }
    }
    fn skipped(&mut self, id: &str, state: &str, reason: &str) {
        if state == "not_run" {
            self.not_run += 1;
        } else {
            self.unsupported += 1;
        }
        self.not_executed.push(map(vec![
            ("id", s(id)),
            ("state", s(state)),
            ("reason", s(reason)),
        ]));
    }
}

struct Finding {
    rule: &'static str,
    node_path: String,
    reason: String,
}

fn head(output: &str, kind: &str, item: &str, value_type: &str) -> Vec<(&'static str, Value)> {
    vec![
        ("output", s(output)),
        ("kind", s(kind)),
        ("scope", s("page")),
        ("item", s(item)),
        ("value_type", s(value_type)),
    ]
}

fn incomplete(cause: &str, detail: &str) -> Value {
    map(vec![("cause", s(cause)), ("detail", s(detail))])
}

fn number(output: &str, item: &str, value: i64, unit: &str, details: Option<Value>) -> Value {
    let mut e = head(output, "exact", item, "number");
    e.push(("value", s(&value.to_string())));
    e.push(("unit", s(unit)));
    if let Some(d) = details {
        e.push(("details", d));
    }
    map(e)
}

const UNITS: [(&str, &str); 5] = [
    (checks::PAGE_SCROLL, "units"),
    (checks::OVERFLOW, "blocks"),
    (checks::ALWAYS_VISIBLE, "items"),
    (checks::KEYBOARD, "checks"),
    (checks::LAYOUT_SHIFT, "score"),
];

fn unit_of(output: &str) -> &'static str {
    UNITS
        .iter()
        .find(|(o, _)| *o == output)
        .map(|(_, u)| *u)
        .unwrap_or("count")
}

/// The numbers and the findings of one page, and its checks counted into `coverage`.
fn page(p: &PageResult, coverage: &mut Coverage) -> (Vec<Value>, Value) {
    let item = p.path.as_str();
    if let Some(error) = &p.error {
        coverage.skipped(item, "not_run", error);
        let why = incomplete("incompatible_target", error);
        let mut numbers = Vec::new();
        for (output, unit) in UNITS {
            let kind = if output == checks::LAYOUT_SHIFT {
                "sampled"
            } else {
                "exact"
            };
            let mut e = head(output, kind, item, "number");
            e.push(("unit", s(unit)));
            e.push(("incomplete", why.clone()));
            numbers.push(map(e));
        }
        let mut set = head(FINDINGS, "exact", item, "identity_set");
        set.push(("key", Value::List(vec![s("rule"), s("node_path")])));
        set.push(("items", Value::List(Vec::new())));
        set.push(("incomplete", why));
        return (numbers, map(set));
    }
    let mut findings: Vec<Finding> = Vec::new();
    for u in &p.units {
        let at = format!("{}/{}", u.viewport, u.mode);
        coverage.judged(u.page_scroll_px <= 0.0);
        if u.page_scroll_px > 0.0 {
            findings.push(Finding {
                rule: checks::PAGE_SCROLL,
                node_path: format!("{at}/page"),
                reason: format!(
                    "the page scrolls {} px sideways",
                    decimal(u.page_scroll_px, 2)
                ),
            });
        }
        coverage.judged(u.overflow.is_empty());
        for o in &u.overflow {
            findings.push(Finding {
                rule: checks::OVERFLOW,
                node_path: format!("{at}/{}", o.desc),
                reason: format!(
                    "right edge {} past its {} at {}",
                    o.right, o.against, o.limit
                ),
            });
        }
        for v in u
            .visible
            .iter()
            .filter(|v| v.status != checks::Status::NotApplicable)
        {
            coverage.judged(!v.status.is_miss());
            if v.status.is_miss() {
                findings.push(Finding {
                    rule: checks::ALWAYS_VISIBLE,
                    node_path: format!("{at}/{}@{}", v.item, v.position),
                    reason: v.status.word().to_string(),
                });
            }
        }
    }
    let mut walks = Vec::new();
    for k in &p.keyboard {
        let at = format!("{}/{}", k.viewport, k.mode);
        let r = &k.result;
        let mut checks_of = vec![
            ("skip-link-first".to_string(), r.skip_link_first),
            ("skip-link-moves-focus".to_string(), r.skip_moves_focus),
        ];
        checks_of.extend(
            r.reached
                .iter()
                .map(|(n, ok)| (format!("tab-reaches-{n}"), *ok)),
        );
        for (name, ok) in checks_of {
            coverage.judged(ok);
            if !ok {
                findings.push(Finding {
                    rule: checks::KEYBOARD,
                    node_path: format!("{at}/{name}"),
                    reason: "missed".to_string(),
                });
            }
        }
        walks.push(map(vec![
            ("viewport", s(&k.viewport)),
            ("mode", s(&k.mode)),
            ("skip_link_first", Value::Bool(r.skip_link_first)),
            ("skip_moves_focus", Value::Bool(r.skip_moves_focus)),
            (
                "reached",
                Value::Map(
                    r.reached
                        .iter()
                        .map(|(n, ok)| (n.clone(), Value::Bool(*ok)))
                        .collect(),
                ),
            ),
        ]));
    }
    let mut shift_details = Vec::new();
    let mut shift_missing = Vec::new();
    for sh in &p.shifts {
        match sh.after_ready {
            Some(v) => {
                coverage.executed += 1;
                shift_details.push((sh.viewport.clone(), s(&decimal(v, 4))));
            }
            None => {
                let id = format!("{item}/{}/layout-shift", sh.viewport);
                coverage.skipped(
                    &id,
                    "unsupported",
                    "the browser has no layout-shift observer",
                );
                shift_missing.push(sh.viewport.clone());
            }
        }
    }
    let numbers = vec![
        number(
            checks::PAGE_SCROLL,
            item,
            p.page_scroll(),
            unit_of(checks::PAGE_SCROLL),
            None,
        ),
        number(
            checks::OVERFLOW,
            item,
            p.overflow(),
            unit_of(checks::OVERFLOW),
            None,
        ),
        number(
            checks::ALWAYS_VISIBLE,
            item,
            p.always_visible_misses(),
            unit_of(checks::ALWAYS_VISIBLE),
            None,
        ),
        number(
            checks::KEYBOARD,
            item,
            p.keyboard_misses(),
            unit_of(checks::KEYBOARD),
            (!walks.is_empty()).then(|| map(vec![("walks", Value::List(walks))])),
        ),
        {
            let mut e = head(checks::LAYOUT_SHIFT, "sampled", item, "number");
            match p.shift_after_ready() {
                Some(v) if shift_missing.is_empty() => e.push(("value", s(&decimal(v, 4)))),
                _ => {}
            }
            e.push(("unit", s(unit_of(checks::LAYOUT_SHIFT))));
            e.push(("samples", Value::Int(shift_details.len() as i64)));
            if !shift_missing.is_empty() {
                e.push((
                    "incomplete",
                    incomplete(
                        "coverage_below_plan",
                        &format!("no layout-shift observer at {}", shift_missing.join(", ")),
                    ),
                ));
            }
            if !shift_details.is_empty() {
                e.push((
                    "details",
                    map(vec![("viewports", Value::Map(shift_details))]),
                ));
            }
            map(e)
        },
    ];
    let mut seen = std::collections::HashSet::new();
    let items = findings
        .into_iter()
        .filter(|f| seen.insert((f.rule, f.node_path.clone())))
        .map(|f| {
            map(vec![
                (
                    "identity",
                    map(vec![("rule", s(f.rule)), ("node_path", s(&f.node_path))]),
                ),
                ("fields", map(vec![("reason", s(&f.reason))])),
            ])
        })
        .collect();
    let mut set = head(FINDINGS, "exact", item, "identity_set");
    set.push(("key", Value::List(vec![s("rule"), s("node_path")])));
    set.push(("items", Value::List(items)));
    (numbers, map(set))
}

pub fn document(header: &Header, config: &Config, pages: &[PageResult]) -> Value {
    let mut coverage = Coverage::default();
    let mut numbers = Vec::new();
    let mut sets = Vec::new();
    for p in pages {
        let (n, set) = page(p, &mut coverage);
        numbers.extend(n);
        sets.push(set);
    }
    let c = coverage;
    let mut counts = vec![
        ("unit", s("check")),
        ("executed", Value::Int(c.executed)),
        ("skipped", Value::Int(0)),
        ("not_run", Value::Int(c.not_run)),
        ("unsupported", Value::Int(c.unsupported)),
        ("passed", Value::Int(c.passed)),
        ("failed", Value::Int(c.failed)),
    ];
    if !c.not_executed.is_empty() {
        counts.push(("not_executed", Value::List(c.not_executed)));
    }
    let viewports: Vec<&str> = config.viewports.iter().map(|v| v.name.as_str()).collect();
    let mut inputs = vec![("config", file(header.config, "value", "bytes"))];
    if let Some(path) = &header.build.path {
        inputs.push(("build", map(vec![("value", s(path))])));
    }
    if let Some(pages) = header.pages {
        inputs.push(("pages", map(vec![("value", s(pages))])));
    }
    let mut run = vec![
        ("started_at", s(&header.started_at)),
        ("ended_at", s(&header.ended_at)),
    ];
    if !header.local_inputs.is_empty() {
        run.push((
            "local_inputs",
            map(header
                .local_inputs
                .iter()
                .map(|(name, value)| (*name, s(value)))
                .collect()),
        ));
    }
    let evidence = map(vec![
        ("project", s(header.project)),
        ("assessment", s(header.assessment)),
        ("assessor", s(ASSESSOR)),
        ("candidate", s(&header.build.digest)),
        (
            "target",
            map(vec![
                ("kind", s(TARGET_KIND)),
                ("class", s(header.class)),
                ("selection", s("candidate")),
                (
                    "identifiers",
                    map(vec![("artifact_digest", s(&header.build.digest))]),
                ),
            ]),
        ),
        ("inputs", map(inputs)),
        ("completion", s("completed")),
        (
            "conditions",
            map(vec![
                (
                    "tools",
                    map(vec![
                        ("ux-browser", s(env!("CARGO_PKG_VERSION"))),
                        ("chromium", s(header.browser)),
                    ]),
                ),
                (
                    "other",
                    map(vec![
                        ("viewports", s(&viewports.join(" "))),
                        ("modes", s(&config.modes.join(" "))),
                        ("settle_ms", s(&config.settle_ms.to_string())),
                        ("reduced_motion", s(&config.reduced_motion.to_string())),
                    ]),
                ),
            ]),
        ),
        ("coverage", map(counts)),
        (
            "results",
            Value::List(numbers.into_iter().chain(sets).collect()),
        ),
    ]);
    map(vec![
        ("format", s(FORMAT)),
        ("evidence", evidence),
        ("run", map(run)),
    ])
}

/// The document as JSON: compact, one result per line, a newline at the end.
pub fn text(document: &Value) -> String {
    let mut out = String::new();
    put(document, &mut out);
    out.push('\n');
    out
}

fn put(value: &Value, out: &mut String) {
    let Value::Map(entries) = value else {
        out.push_str(&uxui_doc::json::write(value));
        return;
    };
    out.push('{');
    for (i, (key, item)) in entries.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&uxui_doc::json::string(key));
        out.push(':');
        match item {
            Value::List(results) if key == "results" && !results.is_empty() => {
                out.push('[');
                for (j, result) in results.iter().enumerate() {
                    out.push_str(if j > 0 { ",\n" } else { "\n" });
                    out.push_str(&uxui_doc::json::write(result));
                }
                out.push_str("\n]");
            }
            Value::Map(_) => put(item, out),
            _ => out.push_str(&uxui_doc::json::write(item)),
        }
    }
    out.push('}');
}
