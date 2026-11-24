//! `uxui-declarations/1`: what an `ess-ui/1` document does not say about UX and a campaign needs,
//! by node path. Reference: `docs/uxui-declarations.md`.

use uxui_doc::yaml::Value;

pub const FORMAT: &str = "uxui-declarations/1";

#[derive(Debug)]
pub struct Declarations {
    /// The document the file declares for, relative to the file.
    pub document: String,
    /// Nodes visible in every state of their page.
    pub always: Vec<String>,
    /// The node that tells a person what to do next, if the document has one.
    pub cue: Option<String>,
    /// Per journey of the `ess-ui-test/1` file, by test name, in file order.
    pub journeys: Vec<Journey>,
}

#[derive(Debug)]
pub struct Journey {
    pub name: String,
    /// Nodes the journey needs filled or shown at its end.
    pub required: Vec<String>,
    /// The node that shows the journey is complete.
    pub signal: Option<String>,
}

pub fn parse(doc: &Value) -> Result<Declarations, String> {
    if doc.get_str("format") != Some(FORMAT) {
        return Err(format!("not a {FORMAT} file (`format: {FORMAT}`)"));
    }
    known(
        doc,
        &["format", "document", "always", "cue", "journeys"],
        "the file",
    )?;
    let document = doc
        .get_str("document")
        .ok_or("`document` names no file")?
        .to_string();
    let mut journeys = Vec::new();
    for (name, journey) in doc.get("journeys").map(Value::entries).unwrap_or(&[]) {
        known(
            journey,
            &["required", "signal"],
            &format!("journey `{name}`"),
        )?;
        journeys.push(Journey {
            name: name.clone(),
            required: paths(
                journey.get("required"),
                &format!("journey `{name}`: required"),
            )?,
            signal: path(journey.get("signal"), &format!("journey `{name}`: signal"))?,
        });
    }
    Ok(Declarations {
        document,
        always: paths(doc.get("always"), "always")?,
        cue: path(doc.get("cue"), "cue")?,
        journeys,
    })
}

fn known(map: &Value, keys: &[&str], at: &str) -> Result<(), String> {
    if !matches!(map, Value::Map(_)) {
        return Err(format!("{at}: expected a map"));
    }
    match map
        .entries()
        .iter()
        .find(|(k, _)| !keys.contains(&k.as_str()))
    {
        Some((key, _)) => Err(format!(
            "{at}: unknown key `{key}` (known: {})",
            keys.join(", ")
        )),
        None => Ok(()),
    }
}

fn paths(value: Option<&Value>, at: &str) -> Result<Vec<String>, String> {
    match value {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(Value::List(items)) => items
            .iter()
            .map(|item| {
                item.str()
                    .map(str::to_string)
                    .ok_or(format!("{at}: a node path is a string"))
            })
            .collect(),
        Some(_) => Err(format!("{at}: expected a list of node paths")),
    }
}

fn path(value: Option<&Value>, at: &str) -> Result<Option<String>, String> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Str(path)) => Ok(Some(path.clone())),
        Some(_) => Err(format!("{at}: expected a node path")),
    }
}
