//! The document's fixtures: view rows by view, derived views and channel scripts, read from the
//! fixture index and the files it names, durations turned into milliseconds.

// PART: fixture semantics
use crate::resolve::{millis, obj, s};
use crate::yaml::{self, Value};
use std::path::Path;

pub fn load(doc: &Value, doc_dir: &Path) -> Result<Value, String> {
    let Some(index) = doc.get("fixtures") else {
        return Ok(obj(vec![
            ("views", Value::Map(vec![])),
            ("derived", Value::Map(vec![])),
            ("scripts", Value::Map(vec![])),
        ]));
    };
    let dir = doc_dir.join(index.get_str("dir").unwrap_or("."));
    // `index` holds views, derived and scripts; keys written on the document itself add to it.
    let mut table = match index.get_str("index") {
        Some(file) => read(&doc_dir.join(file))?,
        None => Value::Map(vec![]),
    };
    for key in ["views", "derived", "scripts"] {
        if let (Some(extra), Value::Map(entries)) = (index.get(key), &mut table) {
            entries.push((key.to_string(), extra.clone()));
        }
    }
    let mut views: Vec<(String, Value)> = Vec::new();
    let mut files: Vec<(String, Value)> = Vec::new();
    for (view, file) in table.get("views").map(Value::entries).unwrap_or(&[]) {
        let file = file
            .str()
            .ok_or(format!("fixture of `{view}` is not a file name"))?;
        if !files.iter().any(|(f, _)| f == file) {
            files.push((file.to_string(), read(&dir.join(file))?));
        }
        let held = &files.iter().find(|(f, _)| f == file).expect("read above").1;
        // A file holds one view (`view:` and `rows:`) or several (`views:`).
        let answer = match held.get("views") {
            Some(map) => map.get(view).cloned(),
            None if held.get_str("view") == Some(view) => Some(held.clone()),
            None => None,
        }
        .ok_or(format!("fixture file `{file}` holds no rows for `{view}`"))?;
        views.push((
            view.clone(),
            obj(vec![
                (
                    "rows",
                    answer.get("rows").cloned().unwrap_or(Value::List(vec![])),
                ),
                (
                    "by_params",
                    answer.get("by_params").cloned().unwrap_or(Value::Null),
                ),
            ]),
        ));
    }
    let mut scripts: Vec<(String, Value)> = Vec::new();
    for (channel, file) in table.get("scripts").map(Value::entries).unwrap_or(&[]) {
        let file = file
            .str()
            .ok_or(format!("script of `{channel}` is not a file name"))?;
        let script = read(&dir.join(file))?;
        let mut events = Vec::new();
        for entry in script.get("events").map(Value::list).unwrap_or(&[]) {
            let at = entry
                .get_str("at")
                .and_then(millis)
                .ok_or(format!("{file}: an entry without a valid `at`"))?;
            events.push(obj(vec![
                ("at", Value::Int(at)),
                ("event", entry.get("event").cloned().unwrap_or(Value::Null)),
                (
                    "lifecycle",
                    entry.get("lifecycle").cloned().unwrap_or(Value::Null),
                ),
                (
                    "payload",
                    entry.get("payload").cloned().unwrap_or(Value::Null),
                ),
            ]));
        }
        scripts.push((
            channel.clone(),
            obj(vec![
                (
                    "loop",
                    Value::Bool(script.get("loop") == Some(&Value::Bool(true))),
                ),
                ("events", Value::List(events)),
            ]),
        ));
    }
    let derived = table.get("derived").cloned().unwrap_or(Value::Map(vec![]));
    Ok(obj(vec![
        ("views", Value::Map(views)),
        ("derived", derived),
        ("scripts", Value::Map(scripts)),
        ("source", s("fixtures")),
    ]))
}

fn read(path: &Path) -> Result<Value, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    yaml::parse(&text).map_err(|e| format!("{}: {e}", path.display()))
}
// END PART
