//! The journeys of an `ess-ui-test/1` file and the actions each takes.

use uxui_doc::yaml::Value;

pub const FORMAT: &str = "ess-ui-test/1";

/// Steps a person performs: one action each (`docs/ux-static.md`).
const ACTS: &[&str] = &[
    "act", "select", "mark", "type", "choose", "move", "upload", "page",
];
/// Steps that check, or that the environment performs: no action.
const CHECKS: &[&str] = &[
    "expect",
    "expect_command",
    "expect_export",
    "play",
    "advance",
];

#[derive(Debug)]
pub struct Journey {
    pub name: String,
    pub steps: Vec<Step>,
}

#[derive(Debug)]
pub struct Step {
    /// 1-based, as a person counts the steps in the file.
    pub number: usize,
    pub keyword: String,
    /// The node path the step names, if it names one (`open` names its page).
    pub at: Option<String>,
    /// Actions the step takes; `None` for a keyword ux-static@1 does not know.
    pub actions: Option<i64>,
}

/// The file's `document` and its journeys.
pub fn parse(doc: &Value) -> Result<(String, Vec<Journey>), String> {
    if doc.get_str("format") != Some(FORMAT) {
        return Err(format!("not an {FORMAT} file (`format: {FORMAT}`)"));
    }
    let document = doc
        .get_str("document")
        .ok_or("`document` names no file")?
        .to_string();
    let mut journeys = Vec::new();
    for test in doc.get("tests").map(Value::list).unwrap_or(&[]) {
        let name = test
            .get_str("name")
            .ok_or("a test without a name")?
            .to_string();
        let steps = test
            .get("steps")
            .map(Value::list)
            .unwrap_or(&[])
            .iter()
            .enumerate()
            .map(|(i, s)| step(i, s))
            .collect();
        journeys.push(Journey { name, steps });
    }
    Ok((document, journeys))
}

fn step(index: usize, written: &Value) -> Step {
    let (keyword, body) = match written.entries() {
        [(keyword, body)] => (keyword.clone(), body),
        _ => (String::from("(not one keyword)"), &Value::Null),
    };
    // The node a step names: `open` its page; `act`, `select` and `mark` a path written as the
    // step's value; the others that name one, their `at`. `play`, `advance` and `expect_command`
    // name none (an `advance` value is a duration, not a path).
    let at = match keyword.as_str() {
        "open" => body
            .str()
            .or_else(|| body.get_str("page"))
            .map(|page| format!("pages/{page}")),
        "act" | "select" | "mark" => body.str().map(str::to_string),
        "type" | "choose" | "move" | "upload" | "page" | "expect" | "expect_export" => {
            body.get_str("at").map(str::to_string)
        }
        _ => None,
    };
    let actions = match keyword.as_str() {
        // The first `open` is where the journey starts; a later one is a navigation.
        "open" => Some(i64::from(index > 0)),
        k if ACTS.contains(&k) => Some(1),
        k if CHECKS.contains(&k) => Some(0),
        _ => None,
    };
    Step {
        number: index + 1,
        keyword,
        at,
        actions,
    }
}
