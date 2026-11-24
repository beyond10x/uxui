//! The three metrics of ux-static@1 over one set of inputs: a number per metric and scope item, an
//! identity set of findings per scope item of a rule, and one coverage count over all checks.

use crate::declarations::{self, Declarations};
use crate::journeys::{self, Journey};
use crate::names::{self, Outcome};
use crate::record::{
    self, CLASSES, Cause, Coverage, Finding, Header, Incomplete, Metric, Number, Reading, Scope,
    Set,
};
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

/// One input file, read once: its path relative to the repository root, with `/`, its bytes'
/// digest and its parsed form.
pub struct Input {
    /// Relative to the root, with `/`; `None` for a file outside the root, which the evidence
    /// identifies by its digest alone.
    pub path: Option<String>,
    /// The path as the caller gave it, for messages only: never written into the evidence.
    pub given: String,
    pub digest: String,
    pub value: Value,
}

impl Input {
    /// The name of the file in a message: its path in the tree, else the path as given.
    pub fn name(&self) -> &str {
        self.path.as_deref().unwrap_or(&self.given)
    }
}

/// `path` relative to `root`, with `/`; `None` when the file lies outside the root, since the
/// evidence holds no absolute path.
fn relative(path: &Path, root: &Path) -> Result<Option<String>, String> {
    let canon = |p: &Path| {
        p.canonicalize()
            .map_err(|e| format!("{}: {e}", p.display()))
    };
    let (file, root_canon) = (canon(path)?, canon(root)?);
    let Ok(rest) = file.strip_prefix(&root_canon) else {
        return Ok(None);
    };
    let parts: Option<Vec<&str>> = rest.components().map(|c| c.as_os_str().to_str()).collect();
    parts
        .map(|p| Some(p.join("/")))
        .ok_or_else(|| format!("{}: the path is not UTF-8", path.display()))
}

/// One input file. A file outside the root (a generated variant in a scratch directory) is read
/// and has no path: its digest identifies it (ratchet § 10).
pub fn read(path: &Path, root: &Path) -> Result<Input, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let text = String::from_utf8(bytes.clone()).map_err(|e| format!("{}: {e}", path.display()))?;
    let value = yaml::parse(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Input {
        path: relative(path, root)?,
        given: path.display().to_string(),
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
    pub header: Header<'a>,
    pub tree: Tree,
    pub declarations: Declarations,
    pub journeys: Vec<Journey>,
}

/// One scope item of a rule: its number and its findings.
struct Judged<'a> {
    number: Number<'a>,
    set: Set<'a>,
}

fn outcomes(counts: &[(&str, i64)]) -> Value {
    Value::Map(
        counts
            .iter()
            .map(|(k, n)| (k.to_string(), Value::Int(*n)))
            .collect(),
    )
}

impl Run<'_> {
    pub fn document(&self) -> Result<Value, String> {
        for declared in &self.declarations.journeys {
            if !self.journeys.iter().any(|j| j.name == declared.name) {
                return Err(format!(
                    "{}: declares journey `{}`, which {} does not have",
                    self.header.declarations.name(),
                    declared.name,
                    self.header.journeys.name()
                ));
            }
        }
        let mut coverage = Coverage::default();
        let mut structure = vec![self.always(&mut coverage)];
        for journey in &self.journeys {
            structure.push(self.required(journey, &mut coverage));
        }
        let actions: Vec<Number> = self
            .journeys
            .iter()
            .map(|j| self.actions(j, &mut coverage))
            .collect();
        let pages: Vec<String> = self
            .header
            .document
            .value
            .get("pages")
            .map(Value::entries)
            .unwrap_or(&[])
            .iter()
            .map(|(name, _)| format!("pages/{name}"))
            .collect();
        let names: Vec<Judged> = pages
            .iter()
            .map(|page| self.names(page, &mut coverage))
            .collect();
        // Numbers per metric (structure, actions, names), then the findings in the same order.
        let (mut numbers, mut sets) = (Vec::new(), Vec::new());
        let mut names_numbers = Vec::new();
        for j in structure {
            numbers.push(j.number);
            sets.push(j.set);
        }
        for j in names {
            names_numbers.push(j.number);
            sets.push(j.set);
        }
        numbers.extend(actions);
        numbers.extend(names_numbers);
        Ok(record::document(&self.header, coverage, &numbers, &sets))
    }

    /// Checks declared node paths against the document, each path once.
    fn presence<'s>(
        &self,
        scope: Scope<'s>,
        paths: Vec<String>,
        nothing: &str,
        coverage: &mut Coverage,
    ) -> Judged<'s> {
        let mut declared: Vec<String> = Vec::new();
        for path in paths {
            if !declared.contains(&path) {
                declared.push(path);
            }
        }
        let mut items = Vec::new();
        let (mut passed, mut missing, mut cant_tell) = (0, 0, 0);
        for path in &declared {
            if self.tree.has(path) {
                passed += 1;
                coverage.passed();
                continue;
            }
            match self.tree.opaque_over(path) {
                Some(o) => {
                    cant_tell += 1;
                    coverage.unsupported(
                        format!("{} {} {path}", STRUCTURE.id, scope.item()),
                        &o.reason,
                    );
                }
                None => {
                    missing += 1;
                    coverage.failed();
                    items.push(Finding {
                        rule: STRUCTURE.id,
                        node_path: path.clone(),
                        reason: "no node at this path in the document".to_string(),
                    });
                }
            }
        }
        let why = if declared.is_empty() {
            Some(Incomplete {
                cause: Cause::CoverageBelowPlan,
                detail: nothing.to_string(),
            })
        } else if cant_tell > 0 {
            Some(Incomplete {
                cause: Cause::CoverageBelowPlan,
                detail: format!(
                    "{cant_tell} declared nodes lie where the document alone cannot say what is drawn"
                ),
            })
        } else {
            None
        };
        let reading = match &why {
            None => Reading::Complete(missing),
            Some(why) => Reading::Incomplete {
                why: why.clone(),
                at_least: missing,
            },
        };
        let details = if declared.is_empty() {
            Vec::new()
        } else {
            vec![(
                "outcomes",
                outcomes(&[
                    ("passed", passed),
                    ("failed", missing),
                    ("cantTell", cant_tell),
                ]),
            )]
        };
        Judged {
            number: Number {
                metric: STRUCTURE,
                scope,
                reading,
                details,
            },
            set: Set {
                scope,
                items,
                incomplete: why,
            },
        }
    }

    fn always(&self, coverage: &mut Coverage) -> Judged<'static> {
        let d = &self.declarations;
        let declared: Vec<String> = d.always.iter().chain(d.cue.iter()).cloned().collect();
        self.presence(
            Scope::Variant,
            declared,
            "the declarations name no always-visible node and no cue",
            coverage,
        )
    }

    fn required<'j>(&self, journey: &'j Journey, coverage: &mut Coverage) -> Judged<'j> {
        let declared = self
            .declarations
            .journeys
            .iter()
            .find(|j| j.name == journey.name);
        let paths: Vec<String> = declared
            .map(|j| j.required.iter().chain(j.signal.iter()).cloned().collect())
            .unwrap_or_default();
        self.presence(
            Scope::Journey(&journey.name),
            paths,
            "the declarations name no required node and no signal for this journey",
            coverage,
        )
    }

    /// One check per step: counted when ux-static knows the step and its node; unsupported for an
    /// unknown keyword or a node where the document cannot say; not run for a node it lacks.
    fn actions<'j>(&self, journey: &'j Journey, coverage: &mut Coverage) -> Number<'j> {
        let mut total = 0;
        let mut problems = Vec::new();
        let (mut unsupported, mut not_run) = (0, 0);
        let mut items = Vec::new();
        for step in &journey.steps {
            let id = format!("{} {} step {}", ACTIONS.id, journey.name, step.number);
            let missing = step.at.as_ref().filter(|at| !self.tree.has(at));
            match (step.actions, missing) {
                (None, _) => {
                    let reason = format!("`{}` is not a step ux-static@1 knows", step.keyword);
                    problems.push(format!("step {}: {reason}", step.number));
                    unsupported += 1;
                    coverage.unsupported(id, &reason);
                }
                (Some(_), Some(at)) => match self.tree.opaque_over(at) {
                    Some(o) => {
                        let reason = format!("{at}: {}", o.reason);
                        problems.push(format!("step {}: {reason}", step.number));
                        unsupported += 1;
                        coverage.unsupported(id, &reason);
                    }
                    None => {
                        let reason = format!("no node at {at}");
                        problems.push(format!("step {}: {reason}", step.number));
                        not_run += 1;
                        coverage.not_run(id, &reason);
                    }
                },
                (Some(n), None) => {
                    total += n;
                    coverage.counted();
                }
            }
            if step.actions.unwrap_or(0) > 0 {
                let mut item = vec![
                    ("step".to_string(), Value::Int(step.number as i64)),
                    ("keyword".to_string(), Value::Str(step.keyword.clone())),
                ];
                if let Some(at) = &step.at {
                    item.push(("at".to_string(), Value::Str(at.clone())));
                }
                items.push(Value::Map(item));
            }
        }
        let reading = if unsupported + not_run == 0 {
            Reading::Complete(total)
        } else {
            let cause = if unsupported > 0 {
                Cause::CoverageBelowPlan
            } else {
                Cause::IncompatibleTarget
            };
            // An unknown step can only keep or raise the known steps' sum: that sum is a lower
            // bound. A journey with a step on a missing node cannot be followed and has no bound.
            Reading::Incomplete {
                why: Incomplete {
                    cause,
                    detail: problems.join("; "),
                },
                at_least: if not_run == 0 { total } else { 0 },
            }
        };
        let details = if items.is_empty() {
            Vec::new()
        } else {
            vec![("items", Value::List(items))]
        };
        Number {
            metric: ACTIONS,
            scope: Scope::Journey(&journey.name),
            reading,
            details,
        }
    }

    fn names<'p>(&self, page: &'p str, coverage: &mut Coverage) -> Judged<'p> {
        let under = |path: &str| path == page || path.starts_with(&format!("{page}/"));
        let mut findings = Vec::new();
        let mut items = Vec::new();
        let (mut passed, mut failed, mut cant_tell) = (0, 0, 0);
        let item = |path: &str, outcome: Outcome, source: &str, name: Option<String>| {
            let mut entries = vec![
                ("node_path".to_string(), Value::Str(path.to_string())),
                (
                    "outcome".to_string(),
                    Value::Str(outcome.word().to_string()),
                ),
                ("source".to_string(), Value::Str(source.to_string())),
            ];
            if let Some(name) = name {
                entries.push(("name".to_string(), Value::Str(name)));
            }
            Value::Map(entries)
        };
        for node in self
            .tree
            .nodes
            .iter()
            .filter(|n| under(&n.path) && names::applicable(n))
        {
            let verdict = names::judge(node);
            let reason = verdict.reason.as_deref().unwrap_or("");
            match verdict.outcome {
                Outcome::Passed => {
                    passed += 1;
                    coverage.passed();
                }
                Outcome::Failed => {
                    failed += 1;
                    coverage.failed();
                    findings.push(Finding {
                        rule: NAMES.id,
                        node_path: node.path.clone(),
                        reason: reason.to_string(),
                    });
                }
                Outcome::CantTell => {
                    cant_tell += 1;
                    coverage.unsupported(format!("{} {}", NAMES.id, node.path), reason);
                }
            }
            items.push(item(
                &node.path,
                verdict.outcome,
                verdict.source,
                verdict.name.clone(),
            ));
        }
        for opaque in self.tree.opaque.iter().filter(|o| under(&o.path)) {
            cant_tell += 1;
            coverage.unsupported(format!("{} {}", NAMES.id, opaque.path), &opaque.reason);
            items.push(item(&opaque.path, Outcome::CantTell, "none", None));
        }
        let applicable = passed + failed + cant_tell;
        let why = (cant_tell > 0).then(|| Incomplete {
            cause: Cause::CoverageBelowPlan,
            detail: format!("{cant_tell} nodes whose name the document alone cannot give"),
        });
        let reading = match &why {
            None => Reading::Complete(failed),
            Some(why) => Reading::Incomplete {
                why: why.clone(),
                at_least: failed,
            },
        };
        let mut details = vec![(
            "outcomes",
            outcomes(&[
                ("passed", passed),
                ("failed", failed),
                ("cantTell", cant_tell),
                ("inapplicable", i64::from(applicable == 0)),
            ]),
        )];
        if !items.is_empty() {
            details.push(("items", Value::List(items)));
        }
        let scope = Scope::Page(page);
        Judged {
            number: Number {
                metric: NAMES,
                scope,
                reading,
                details,
            },
            set: Set {
                scope,
                items: findings,
                incomplete: why,
            },
        }
    }
}

/// What the caller names about the run: the plan it executes and where it runs.
pub struct Plan<'a> {
    pub project: &'a str,
    pub assessment: &'a str,
    pub class: &'a str,
}

/// Fails on a value of a reserved argument that it cannot have: a target class ratchet does not
/// know, an empty project or assessment. A wrong invocation, not a refused input.
pub fn invocation(plan: &Plan) -> Result<(), String> {
    if !CLASSES.contains(&plan.class) {
        return Err(format!(
            "target class `{}` is not one of {}",
            plan.class,
            CLASSES.join(", ")
        ));
    }
    for (what, value) in [("project", plan.project), ("assessment", plan.assessment)] {
        if value.trim().is_empty() {
            return Err(format!("the {what} is empty"));
        }
    }
    Ok(())
}

/// Checks the inputs belong together and builds the run.
pub fn run<'a>(
    plan: Plan<'a>,
    files: [&'a Input; 3],
    paths: [&Path; 3],
) -> Result<Run<'a>, String> {
    let [document, journeys_file, declarations_file] = files;
    let [document_path, journeys_path, declarations_path] = paths;
    invocation(&plan)?;
    if document.value.get_str("format") != Some("ess-ui/1") {
        return Err(format!("{}: not an ess-ui/1 document", document.name()));
    }
    let (named, journeys) = journeys::parse(&journeys_file.value)
        .map_err(|e| format!("{}: {e}", journeys_file.name()))?;
    same_document(journeys_path, &named, document_path)?;
    let declarations = declarations::parse(&declarations_file.value)
        .map_err(|e| format!("{}: {e}", declarations_file.name()))?;
    same_document(declarations_path, &declarations.document, document_path)?;
    Ok(Run {
        header: Header {
            project: plan.project,
            assessment: plan.assessment,
            class: plan.class,
            document,
            journeys: journeys_file,
            declarations: declarations_file,
        },
        tree: tree::tree(&document.value),
        declarations,
        journeys,
    })
}
