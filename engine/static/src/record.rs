//! One evidence record, an assessment run in ratchet's draft shape (ratchet `docs/design.md`
//! § 6 "Assessment run", § 1 "Measurement kind", § 3 scope and coverage, § 5 verdicts). Field by
//! field: `docs/ux-static.md`.

use crate::ASSESSOR;
use crate::assess::Input;
use crate::names::Outcome;
use uxui_doc::yaml::Value;

pub struct Metric {
    pub id: &'static str,
    pub unit: &'static str,
}

#[derive(Clone, Copy)]
pub enum Scope<'a> {
    Variant,
    Journey(&'a str),
    Page(&'a str),
}

impl Scope<'_> {
    fn kind(self) -> &'static str {
        match self {
            Scope::Variant => "variant",
            Scope::Journey(_) => "journey",
            Scope::Page(_) => "page",
        }
    }
    fn id(self) -> String {
        match self {
            Scope::Variant => "document".to_string(),
            Scope::Journey(id) | Scope::Page(id) => id.to_string(),
        }
    }
}

/// A value, or insufficient evidence with its cause. Never a pass by default: an assessor that
/// cannot compute a value says so.
pub enum Reading {
    Measured(i64),
    Insufficient {
        cause: &'static str,
        detail: String,
        at_least: Option<i64>,
    },
}

impl Reading {
    pub fn measured(value: i64) -> Self {
        Reading::Measured(value)
    }
    pub fn insufficient(cause: &'static str, detail: String, at_least: Option<i64>) -> Self {
        Reading::Insufficient {
            cause,
            detail,
            at_least,
        }
    }
}

pub struct Finding {
    pub rule: &'static str,
    pub node_path: String,
    pub outcome: Outcome,
    pub reason: String,
}

impl Finding {
    pub fn new(metric: &Metric, path: &str, outcome: Outcome, reason: &str) -> Self {
        Finding {
            rule: metric.id,
            node_path: path.to_string(),
            outcome,
            reason: reason.to_string(),
        }
    }
}

pub struct Evidence<'a> {
    pub document: &'a Input,
    pub journeys: &'a Input,
    pub declarations: &'a Input,
    pub class: &'a str,
}

fn s(text: &str) -> Value {
    Value::Str(text.to_string())
}

fn map(entries: Vec<(&str, Value)>) -> Value {
    Value::Map(
        entries
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    )
}

fn file(input: &Input) -> Value {
    map(vec![("path", s(&input.path)), ("digest", s(&input.digest))])
}

impl Evidence<'_> {
    /// The record. `coverage`: (items executed, items unsupported); `outcomes`: ACT outcome counts.
    #[allow(clippy::too_many_arguments)]
    pub fn record(
        &self,
        metric: &Metric,
        scope: Scope,
        result: Reading,
        coverage: (i64, i64),
        outcomes: Option<&[(&str, i64)]>,
        findings: Vec<Finding>,
        items: Vec<Value>,
    ) -> Value {
        let digest12 = self
            .document
            .digest
            .trim_start_matches("sha256:")
            .get(..12)
            .unwrap_or("");
        let mut resolved = vec![
            ("document", file(self.document)),
            ("journeys", file(self.journeys)),
            ("declarations", file(self.declarations)),
        ];
        if let Scope::Journey(name) = scope {
            resolved.push(("journey", s(name)));
        }
        resolved.push(("tool", s("ux-static")));
        resolved.push(("tool_version", s(env!("CARGO_PKG_VERSION"))));
        let mut entries = vec![
            ("format", s("ratchet-evidence/draft")),
            ("record", s("assessment-run")),
            (
                "id",
                s(&format!(
                    "{ASSESSOR}/{digest12}/{}/{}/{}",
                    metric.id,
                    scope.kind(),
                    scope.id()
                )),
            ),
            ("assessor", s(ASSESSOR)),
            (
                "candidate",
                map(vec![
                    ("kind", s("ui-document")),
                    ("digest", s(&self.document.digest)),
                ]),
            ),
            (
                "target",
                map(vec![("kind", s("ui-variant")), ("class", s(self.class))]),
            ),
            ("resolved_target", map(resolved)),
            (
                "observation",
                map(vec![
                    ("kind", s("snapshot")),
                    ("runs", Value::Int(1)),
                    ("conditions", map(vec![])),
                ]),
            ),
            (
                "measurement",
                map(vec![
                    ("output", s(metric.id)),
                    ("kind", s("exact")),
                    ("unit", s(metric.unit)),
                    ("direction", s("lower")),
                    ("scope", s(scope.kind())),
                    ("aggregation", s("count")),
                ]),
            ),
            (
                "scope",
                map(vec![("kind", s(scope.kind())), ("id", s(&scope.id()))]),
            ),
        ];
        match result {
            Reading::Measured(value) => {
                entries.push(("result", s("measured")));
                entries.push(("value", Value::Int(value)));
            }
            Reading::Insufficient {
                cause,
                detail,
                at_least,
            } => {
                entries.push(("result", s("insufficient")));
                entries.push(("value", Value::Null));
                if let Some(n) = at_least.filter(|n| *n > 0) {
                    entries.push(("value_at_least", Value::Int(n)));
                }
                entries.push((
                    "insufficient",
                    map(vec![("cause", s(cause)), ("detail", s(&detail))]),
                ));
            }
        }
        entries.push(("unit", s(metric.unit)));
        let (executed, unsupported) = coverage;
        entries.push((
            "coverage",
            map(vec![
                ("executed", Value::Int(executed)),
                ("skipped", Value::Int(0)),
                ("not_run", Value::Int(0)),
                ("unsupported", Value::Int(unsupported)),
            ]),
        ));
        if let Some(outcomes) = outcomes {
            entries.push((
                "outcomes",
                Value::Map(
                    outcomes
                        .iter()
                        .map(|(k, n)| (k.to_string(), Value::Int(*n)))
                        .collect(),
                ),
            ));
        }
        entries.push((
            "findings",
            Value::List(
                findings
                    .into_iter()
                    .map(|f| {
                        map(vec![
                            ("rule", s(f.rule)),
                            ("node_path", s(&f.node_path)),
                            ("outcome", s(f.outcome.word())),
                            ("reason", s(&f.reason)),
                        ])
                    })
                    .collect(),
            ),
        ));
        if !items.is_empty() {
            entries.push(("items", Value::List(items)));
        }
        map(entries)
    }
}
