//! `a11y.accessible_names@1`: every node a person operates, or that needs a text alternative, has
//! an accessible name. One ACT outcome per node: passed, failed or cantTell; a page with no such
//! node is inapplicable. The name comes from the document, else from the default the `ess-ui/1`
//! schema or ESS's renderers give (`docs/ux-static.md`).

use uxui_doc::tree::{Node, Role};
use uxui_doc::yaml::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Passed,
    Failed,
    CantTell,
}

impl Outcome {
    pub fn word(self) -> &'static str {
        match self {
            Outcome::Passed => "passed",
            Outcome::Failed => "failed",
            Outcome::CantTell => "cantTell",
        }
    }
}

#[derive(Debug)]
pub struct Verdict {
    pub outcome: Outcome,
    /// `declared`, `derived` (from the node's name, as ESS's renderers do) or `default` (the schema's
    /// or the renderers' fixed text).
    pub source: &'static str,
    pub name: Option<String>,
    pub reason: Option<String>,
}

fn passed(source: &'static str, name: String) -> Verdict {
    Verdict {
        outcome: Outcome::Passed,
        source,
        name: Some(name),
        reason: None,
    }
}

fn failed(reason: &str) -> Verdict {
    Verdict {
        outcome: Outcome::Failed,
        source: "none",
        name: None,
        reason: Some(reason.to_string()),
    }
}

/// A text written in the document: `Some(Ok)` non-blank, `Some(Err)` declared but empty.
fn written(value: &Value, key: &str) -> Option<Result<String, ()>> {
    match value.get(key)? {
        Value::Str(s) if !s.trim().is_empty() => Some(Ok(s.clone())),
        _ => Some(Err(())),
    }
}

fn last_segment(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}

/// The verdict on one node with a role.
pub fn judge(node: &Node) -> Verdict {
    let v = &node.value;
    let label = |key: &str, otherwise: Verdict| match written(v, key) {
        Some(Ok(text)) => passed("declared", text),
        Some(Err(())) => failed(&format!("`{key}` is declared empty")),
        None => otherwise,
    };
    match node.role {
        Some(Role::Action) => {
            let otherwise = if v.get_str("as") == Some("icon") {
                failed("an icon action shows a glyph and needs a `label`")
            } else {
                // ESS's renderers show an action's name when it has no label.
                passed("derived", last_segment(&node.path))
            };
            label("label", otherwise)
        }
        // The schema's default `confirm_label` is `Delete`.
        Some(Role::InlineConfirm | Role::Confirm) => {
            label("confirm_label", passed("default", "Delete".to_string()))
        }
        // A field's caption defaults to its name with `_` read as a space (ESS's renderers).
        Some(Role::Input | Role::Choice) => label(
            "label",
            passed("derived", last_segment(&node.path).replace('_', " ")),
        ),
        // ESS's renderers write `Submit` on a submit button without a label.
        Some(Role::Submit) => label("label", passed("default", "Submit".to_string())),
        Some(Role::Primitive) => primitive(v),
        None => Verdict {
            outcome: Outcome::CantTell,
            source: "none",
            name: None,
            reason: Some("no role".to_string()),
        },
    }
}

fn primitive(v: &Value) -> Verdict {
    let required = |key: &str| match written(v, key) {
        Some(Ok(text)) => passed("declared", text),
        Some(Err(())) => failed(&format!("`{key}` is declared empty")),
        None => failed(&format!("`{key}` is missing")),
    };
    match v.get_str("primitive").unwrap_or("") {
        "button" | "icon" | "toggle" => required("label"),
        "image" => required("alt"),
        "link" => match written(v, "text") {
            Some(Ok(text)) if is_expression(&text) => Verdict {
                outcome: Outcome::CantTell,
                source: "expression",
                name: None,
                reason: Some(format!(
                    "the link text `{text}` is an expression read at run time"
                )),
            },
            Some(Ok(text)) => passed("declared", text),
            Some(Err(())) => failed("`text` is declared empty"),
            None => failed("`text` is missing"),
        },
        // An input primitive has no label; its placeholder is the accessible name of last resort.
        "input" => match written(v, "placeholder") {
            Some(Ok(text)) => passed("placeholder", text),
            _ => failed("an input primitive has no label and no placeholder"),
        },
        _ => Verdict {
            outcome: Outcome::Passed,
            source: "not-interactive",
            name: None,
            reason: None,
        },
    }
}

fn is_expression(text: &str) -> bool {
    ["row.", "params.", "state.", "args.", "shell.", "selection."]
        .iter()
        .any(|p| text.starts_with(p))
}

/// Whether the node counts for this rule: a role, and drawn. A primitive that is neither operated
/// nor an image (text, badge, divider) is not applicable.
pub fn applicable(node: &Node) -> bool {
    node.drawn
        && match node.role {
            Some(Role::Primitive) => {
                matches!(
                    node.value.get_str("primitive"),
                    Some("button" | "icon" | "toggle" | "image" | "link" | "input")
                )
            }
            Some(_) => true,
            None => false,
        }
}
