//! The one `ratchet-evidence/1` document of a run (ratchet `docs/evidence-format.md`): the target,
//! the inputs, one set of coverage counts in the unit `check`, and the results. Field by field:
//! `docs/ux-static.md`.

use crate::ASSESSOR;
use crate::assess::Input;
use uxui_doc::yaml::Value;

pub const FORMAT: &str = "ratchet-evidence/1";
/// The output whose results are the failed checks, one identity set per scope item.
pub const FINDINGS: &str = "findings";
/// The target classes ratchet knows (`evidence.target.class`).
pub const CLASSES: &[&str] = &["local", "ci", "ephemeral", "staging", "production"];

#[derive(Clone, Copy)]
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
    /// The scope item, the same spelling in every run.
    pub fn item(self) -> String {
        match self {
            Scope::Variant => "document".to_string(),
            Scope::Journey(id) | Scope::Page(id) => id.to_string(),
        }
    }
}

/// A cause of insufficient evidence, in ratchet's words (design § 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cause {
    /// A check ux-static cannot evaluate, or nothing declared to check.
    CoverageBelowPlan,
    /// A step names a node the document does not have: the journey cannot be followed.
    IncompatibleTarget,
}

impl Cause {
    fn word(self) -> &'static str {
        match self {
            Cause::CoverageBelowPlan => "coverage_below_plan",
            Cause::IncompatibleTarget => "incompatible_target",
        }
    }
}

#[derive(Clone)]
pub struct Incomplete {
    pub cause: Cause,
    pub detail: String,
}

/// A count, or less than the whole count with its cause. Never a pass by default: an assessor that
/// cannot compute a value says so.
pub enum Reading {
    Complete(i64),
    Incomplete {
        why: Incomplete,
        /// A lower bound; 0 says nothing and is left out.
        at_least: i64,
    },
}

/// A result of the value type `number`.
pub struct Number<'a> {
    pub metric: Metric,
    pub scope: Scope<'a>,
    pub reading: Reading,
    /// `details`, in ux-static's own spelling; left out when empty.
    pub details: Vec<(&'static str, Value)>,
}

pub struct Finding {
    pub rule: &'static str,
    pub node_path: String,
    pub reason: String,
}

/// A result of the output `findings`: the failed checks of one scope item, also when there are none.
pub struct Set<'a> {
    pub scope: Scope<'a>,
    pub items: Vec<Finding>,
    pub incomplete: Option<Incomplete>,
}

/// The run's coverage in the unit `check`: one rule applied to one node or one step. The four
/// counts do not overlap; `passed` and `failed` are executed checks with an outcome.
#[derive(Default)]
pub struct Coverage {
    executed: i64,
    not_run: i64,
    unsupported: i64,
    passed: i64,
    failed: i64,
    not_executed: Vec<Value>,
}

impl Coverage {
    pub fn passed(&mut self) {
        self.executed += 1;
        self.passed += 1;
    }
    pub fn failed(&mut self) {
        self.executed += 1;
        self.failed += 1;
    }
    /// Executed and counted, with no outcome of its own (a step).
    pub fn counted(&mut self) {
        self.executed += 1;
    }
    /// A check ux-static cannot evaluate: an ACT cantTell, a step keyword it does not know.
    pub fn unsupported(&mut self, id: String, reason: &str) {
        self.unsupported += 1;
        self.not_executed.push(entry(id, "unsupported", reason));
    }
    /// A check that should have run and did not: a step naming no node of the document.
    pub fn not_run(&mut self, id: String, reason: &str) {
        self.not_run += 1;
        self.not_executed.push(entry(id, "not_run", reason));
    }
}

fn entry(id: String, state: &str, reason: &str) -> Value {
    map(vec![
        ("id", Value::Str(id)),
        ("state", s(state)),
        ("reason", s(reason)),
    ])
}

/// What the document says about the run beside its coverage and results.
pub struct Header<'a> {
    pub project: &'a str,
    pub assessment: &'a str,
    pub class: &'a str,
    pub document: &'a Input,
    pub journeys: &'a Input,
    pub declarations: &'a Input,
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

fn file(input: &Input, key: &str) -> Value {
    map(vec![
        (key, s(&input.path)),
        ("digest", s(&input.digest)),
        ("digest_form", s("bytes")),
    ])
}

fn incomplete(why: &Incomplete) -> Value {
    map(vec![
        ("cause", s(why.cause.word())),
        ("detail", s(&why.detail)),
    ])
}

fn head(output: &str, scope: Scope, value_type: &str) -> Vec<(&'static str, Value)> {
    vec![
        ("output", s(output)),
        ("kind", s("exact")),
        ("scope", s(scope.kind())),
        ("item", s(&scope.item())),
        ("value_type", s(value_type)),
    ]
}

impl Number<'_> {
    fn value(&self) -> Value {
        let mut entries = head(self.metric.id, self.scope, "number");
        let (value, at_least, why) = match &self.reading {
            Reading::Complete(n) => (Some(*n), false, None),
            Reading::Incomplete { why, at_least } if *at_least > 0 => {
                (Some(*at_least), true, Some(why))
            }
            Reading::Incomplete { why, .. } => (None, false, Some(why)),
        };
        if let Some(n) = value {
            entries.push(("value", s(&n.to_string())));
        }
        entries.push(("unit", s(self.metric.unit)));
        if at_least {
            entries.push(("at_least", Value::Bool(true)));
        }
        if let Some(why) = why {
            entries.push(("incomplete", incomplete(why)));
        }
        if !self.details.is_empty() {
            entries.push(("details", map(self.details.clone())));
        }
        map(entries)
    }
}

impl Set<'_> {
    fn value(&self) -> Value {
        let mut entries = head(FINDINGS, self.scope, "identity_set");
        entries.push(("key", Value::List(vec![s("rule"), s("node_path")])));
        let items = self
            .items
            .iter()
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
        entries.push(("items", Value::List(items)));
        if let Some(why) = &self.incomplete {
            entries.push(("incomplete", incomplete(why)));
        }
        map(entries)
    }
}

/// The document: the numbers in the order given, then the identity sets.
pub fn document(header: &Header, coverage: Coverage, numbers: &[Number], sets: &[Set]) -> Value {
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
    let results = numbers
        .iter()
        .map(Number::value)
        .chain(sets.iter().map(Set::value))
        .collect();
    let evidence = map(vec![
        ("project", s(header.project)),
        ("assessment", s(header.assessment)),
        ("assessor", s(ASSESSOR)),
        ("candidate", s(&header.document.digest)),
        (
            "target",
            map(vec![
                ("kind", s("document")),
                ("class", s(header.class)),
                ("selection", s("candidate")),
                ("identifiers", file(header.document, "path")),
            ]),
        ),
        (
            "inputs",
            map(vec![
                ("journeys", file(header.journeys, "value")),
                ("declarations", file(header.declarations, "value")),
            ]),
        ),
        ("completion", s("completed")),
        (
            "conditions",
            map(vec![(
                "tools",
                map(vec![("ux-static", s(env!("CARGO_PKG_VERSION")))]),
            )]),
        ),
        ("coverage", map(counts)),
        ("results", Value::List(results)),
    ]);
    map(vec![("format", s(FORMAT)), ("evidence", evidence)])
}

/// The document as JSON: compact, one result per line, a newline at the end. The same document gives
/// the same bytes.
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
