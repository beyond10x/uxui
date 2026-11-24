//! The three metrics of ux-static@1 over one set of inputs, one evidence record per metric and
//! scope.

use crate::declarations::{self, Declarations};
use crate::journeys::{self, Journey};
use crate::names::{self, Outcome};
use crate::record::{Evidence, Finding, Metric, Reading, Scope};
use std::path::Path;
use uxui_doc::tree::{self, Tree};
use uxui_doc::yaml::{self, Value};

pub const STRUCTURE: Metric = Metric {
    id: "structure.required_present@1",
    unit: "nodes",
};
pub const ACTIONS: Metric = Metric {
    id: "effort.actions_per_journey@1",
    unit: "actions",
};
pub const NAMES: Metric = Metric {
    id: "a11y.accessible_names@1",
    unit: "nodes",
};

/// One input file, read once: its path as given, its bytes' digest and its parsed form.
pub struct Input {
    pub path: String,
    pub digest: String,
    pub value: Value,
}

pub fn read(path: &Path) -> Result<Input, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let text = String::from_utf8(bytes.clone()).map_err(|e| format!("{}: {e}", path.display()))?;
    let value = yaml::parse(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Input {
        path: path.display().to_string(),
        digest: uxui_doc::sha256::hex(&bytes),
        value,
    })
}

/// Fails unless `named`, written in the file at `from`, is the file at `document`.
pub fn same_document(from: &Path, named: &str, document: &Path) -> Result<(), String> {
    let base = from.parent().unwrap_or(Path::new("."));
    let canon = |p: &Path| {
        p.canonicalize()
            .map_err(|e| format!("{}: {e}", p.display()))
    };
    let named_path = base.join(named);
    let named_canon = canon(&named_path)
        .map_err(|e| format!("{}: names document `{named}`: {e}", from.display()))?;
    if named_canon != canon(document)? {
        return Err(format!(
            "{}: names document `{named}`, not {}",
            from.display(),
            document.display()
        ));
    }
    Ok(())
}

pub struct Run<'a> {
    pub evidence: Evidence<'a>,
    pub tree: Tree,
    pub declarations: Declarations,
    pub journeys: Vec<Journey>,
}

impl Run<'_> {
    pub fn records(&self) -> Result<Vec<Value>, String> {
        for declared in &self.declarations.journeys {
            if !self.journeys.iter().any(|j| j.name == declared.name) {
                return Err(format!(
                    "{}: declares journey `{}`, which {} does not have",
                    self.evidence.declarations.path, declared.name, self.evidence.journeys.path
                ));
            }
        }
        let mut out = vec![self.always()];
        out.extend(self.journeys.iter().map(|j| self.required(j)));
        out.extend(self.journeys.iter().map(|j| self.actions(j)));
        let pages = self
            .evidence
            .document
            .value
            .get("pages")
            .map(Value::entries)
            .unwrap_or(&[]);
        out.extend(
            pages
                .iter()
                .map(|(name, _)| self.names(&format!("pages/{name}"))),
        );
        Ok(out)
    }

    /// Checks declared node paths against the document: a finding per path absent, cantTell where
    /// the document cannot say.
    fn presence(&self, paths: &[String], metric: &Metric) -> (i64, Vec<Finding>) {
        let mut findings = Vec::new();
        let mut missing = 0;
        for path in paths {
            if self.tree.has(path) {
                continue;
            }
            match self.tree.opaque_over(path) {
                Some(o) => findings.push(Finding::new(metric, path, Outcome::CantTell, &o.reason)),
                None => {
                    missing += 1;
                    findings.push(Finding::new(
                        metric,
                        path,
                        Outcome::Failed,
                        "no node at this path in the document",
                    ));
                }
            }
        }
        (missing, findings)
    }

    fn presence_record(&self, scope: Scope, declared: Vec<String>, nothing: &str) -> Value {
        let (missing, findings) = self.presence(&declared, &STRUCTURE);
        let cant_tell = findings
            .iter()
            .filter(|f| f.outcome == Outcome::CantTell)
            .count() as i64;
        let result = if declared.is_empty() {
            Reading::insufficient("declaration", nothing.to_string(), None)
        } else if cant_tell > 0 {
            Reading::insufficient(
                "coverage",
                format!(
                    "{cant_tell} declared nodes lie where the document alone cannot say what is drawn"
                ),
                Some(missing),
            )
        } else {
            Reading::measured(missing)
        };
        let executed = declared.len() as i64;
        let outcomes = [
            ("passed", executed - missing - cant_tell),
            ("failed", missing),
            ("cantTell", cant_tell),
        ];
        self.evidence.record(
            &STRUCTURE,
            scope,
            result,
            (executed, cant_tell),
            Some(&outcomes[..]),
            findings,
            Vec::new(),
        )
    }

    fn always(&self) -> Value {
        let d = &self.declarations;
        let declared: Vec<String> = d.always.iter().chain(d.cue.iter()).cloned().collect();
        self.presence_record(
            Scope::Variant,
            declared,
            "the declarations name no always-visible node and no cue",
        )
    }

    fn required(&self, journey: &Journey) -> Value {
        let declared = self
            .declarations
            .journeys
            .iter()
            .find(|j| j.name == journey.name);
        let paths: Vec<String> = declared
            .map(|j| j.required.iter().chain(j.signal.iter()).cloned().collect())
            .unwrap_or_default();
        self.presence_record(
            Scope::Journey(&journey.name),
            paths,
            "the declarations name no required node and no signal for this journey",
        )
    }

    fn actions(&self, journey: &Journey) -> Value {
        let mut total = 0;
        let mut problems = Vec::new();
        let mut unsupported = 0;
        let mut items = Vec::new();
        for step in &journey.steps {
            match step.actions {
                Some(n) => total += n,
                None => {
                    unsupported += 1;
                    problems.push(format!(
                        "step {}: `{}` is not a step ux-static@1 knows",
                        step.number, step.keyword
                    ));
                }
            }
            if let Some(at) = &step.at
                && !self.tree.has(at)
            {
                match self.tree.opaque_over(at) {
                    Some(o) => problems.push(format!("step {}: {at}: {}", step.number, o.reason)),
                    None => problems.push(format!("step {}: no node at {at}", step.number)),
                }
            }
            if step.actions.unwrap_or(0) > 0 {
                items.push(Value::Map(vec![
                    ("step".to_string(), Value::Int(step.number as i64)),
                    ("keyword".to_string(), Value::Str(step.keyword.clone())),
                    (
                        "at".to_string(),
                        step.at.clone().map(Value::Str).unwrap_or(Value::Null),
                    ),
                ]));
            }
        }
        let result = if problems.is_empty() {
            Reading::measured(total)
        } else {
            let cause = if unsupported > 0 { "coverage" } else { "input" };
            Reading::insufficient(cause, problems.join("; "), None)
        };
        let executed = journey.steps.len() as i64 - unsupported;
        self.evidence.record(
            &ACTIONS,
            Scope::Journey(&journey.name),
            result,
            (executed, unsupported),
            None,
            Vec::new(),
            items,
        )
    }

    fn names(&self, page: &str) -> Value {
        let under = |path: &str| path == page || path.starts_with(&format!("{page}/"));
        let mut findings = Vec::new();
        let mut items = Vec::new();
        let (mut passed, mut failed, mut cant_tell) = (0, 0, 0);
        for node in self
            .tree
            .nodes
            .iter()
            .filter(|n| under(&n.path) && names::applicable(n))
        {
            let verdict = names::judge(node);
            match verdict.outcome {
                Outcome::Passed => passed += 1,
                Outcome::Failed => failed += 1,
                Outcome::CantTell => cant_tell += 1,
            }
            if verdict.outcome != Outcome::Passed {
                findings.push(Finding::new(
                    &NAMES,
                    &node.path,
                    verdict.outcome,
                    verdict.reason.as_deref().unwrap_or(""),
                ));
            }
            items.push(Value::Map(vec![
                ("node_path".to_string(), Value::Str(node.path.clone())),
                (
                    "outcome".to_string(),
                    Value::Str(verdict.outcome.word().to_string()),
                ),
                ("source".to_string(), Value::Str(verdict.source.to_string())),
                (
                    "name".to_string(),
                    verdict.name.map(Value::Str).unwrap_or(Value::Null),
                ),
            ]));
        }
        for opaque in self.tree.opaque.iter().filter(|o| under(&o.path)) {
            cant_tell += 1;
            findings.push(Finding::new(
                &NAMES,
                &opaque.path,
                Outcome::CantTell,
                &opaque.reason,
            ));
            items.push(Value::Map(vec![
                ("node_path".to_string(), Value::Str(opaque.path.clone())),
                (
                    "outcome".to_string(),
                    Value::Str(Outcome::CantTell.word().to_string()),
                ),
                ("source".to_string(), Value::Str("none".to_string())),
                ("name".to_string(), Value::Null),
            ]));
        }
        let applicable = passed + failed + cant_tell;
        let result = if cant_tell > 0 {
            Reading::insufficient(
                "coverage",
                format!("{cant_tell} nodes whose name the document alone cannot give"),
                Some(failed),
            )
        } else {
            Reading::measured(failed)
        };
        let outcomes = [
            ("passed", passed),
            ("failed", failed),
            ("cantTell", cant_tell),
            ("inapplicable", i64::from(applicable == 0)),
        ];
        self.evidence.record(
            &NAMES,
            Scope::Page(page),
            result,
            (applicable, cant_tell),
            Some(&outcomes[..]),
            findings,
            items,
        )
    }
}

/// Reads the three inputs, checks they belong together and builds the run.
pub fn run<'a>(
    document: &'a Input,
    journeys_file: &'a Input,
    declarations_file: &'a Input,
    paths: [&Path; 3],
    class: &'a str,
) -> Result<Run<'a>, String> {
    let [document_path, journeys_path, declarations_path] = paths;
    if document.value.get_str("format") != Some("ess-ui/1") {
        return Err(format!("{}: not an ess-ui/1 document", document.path));
    }
    let (named, journeys) = journeys::parse(&journeys_file.value)
        .map_err(|e| format!("{}: {e}", journeys_file.path))?;
    same_document(journeys_path, &named, document_path)?;
    let declarations = declarations::parse(&declarations_file.value)
        .map_err(|e| format!("{}: {e}", declarations_file.path))?;
    same_document(declarations_path, &declarations.document, document_path)?;
    Ok(Run {
        evidence: Evidence {
            document,
            journeys: journeys_file,
            declarations: declarations_file,
            class,
        },
        tree: tree::tree(&document.value),
        declarations,
        journeys,
    })
}
