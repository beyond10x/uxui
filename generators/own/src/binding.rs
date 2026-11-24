//! Model binding of the resolved page: every view, event and command it names resolves in the ESS
//! model, read params are params of the view, fields are fields of the view, bound inputs are
//! inputs of the command, and an event that inserts rows carries the fields the view shows.

// PART: model binding
use crate::model::Model;
use crate::yaml::Value;

#[derive(Debug, Default)]
pub struct Findings {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn check(page: &Value, doc: &Value, model: &Model) -> Findings {
    let mut out = Findings::default();
    for section in page.get("sections").map(Value::list).unwrap_or(&[]) {
        let path = section.get_str("path").unwrap_or("");
        let Some(reads) = section.get("reads").filter(|r| !r.is_null()) else { continue };
        let Some(view_name) = reads.get_str("view") else { continue };
        let Some(view) = model.views.get(view_name) else {
            out.errors.push(format!("{path}/reads: view `{view_name}` is not in the model"));
            continue;
        };
        for (param, _) in reads.get("params").map(Value::entries).unwrap_or(&[]) {
            if !view.params.iter().any(|p| p == param) {
                out.errors.push(format!("{path}/reads: `{param}` is not a param of `{view_name}`"));
            }
        }
        let mut shown: Vec<&str> = Vec::new();
        for field in section.get("fields").map(Value::list).unwrap_or(&[]) {
            let name = field.get_str("field").unwrap_or("");
            shown.push(name);
            if view.field(name).is_none() {
                out.errors.push(format!("{}: `{name}` is not a field of `{view_name}`", field.get_str("path").unwrap_or(path)));
            }
        }
        for key in ["x"] {
            if let Some(name) = section.get_str(key) {
                shown.push(name);
            }
        }
        for name in section.get("series").map(Value::list).unwrap_or(&[]).iter().filter_map(Value::str) {
            shown.push(name);
            if view.field(name).is_none() {
                out.errors.push(format!("{path}: series `{name}` is not a field of `{view_name}`"));
            }
        }
        if let Some(live) = section.get("live").filter(|l| !l.is_null()) {
            let channel = live.get_str("channel").unwrap_or("");
            let carried: Vec<&str> = doc
                .get("channels")
                .and_then(|c| c.get(channel))
                .and_then(|c| c.get("carries"))
                .and_then(|c| c.get("events"))
                .map(Value::list)
                .unwrap_or(&[])
                .iter()
                .filter_map(Value::str)
                .collect();
            let on: Vec<&str> = match live.get("on") {
                Some(Value::List(list)) => list.iter().filter_map(Value::str).collect(),
                _ => carried.clone(),
            };
            let inserts = matches!(live.get_str("effect"), Some("insert_or_patch" | "insert_top" | "patch_row" | "replace"));
            for event in on {
                if !carried.contains(&event) {
                    out.errors.push(format!("{path}/live: channel `{channel}` does not carry `{event}`"));
                }
                let Some(fields) = model.events.get(event) else {
                    out.errors.push(format!("{path}/live: event `{event}` is not in the model"));
                    continue;
                };
                if inserts {
                    let missing: Vec<&str> =
                        shown.iter().copied().filter(|f| !fields.iter().any(|e| e.name == *f)).collect();
                    if !missing.is_empty() {
                        out.warnings.push(format!(
                            "{path}/live: `{event}` carries no {} of `{view_name}`; rows it inserts show them empty",
                            missing.iter().map(|m| format!("`{m}`")).collect::<Vec<_>>().join(", ")
                        ));
                    }
                }
            }
        }
        actions(section.get("row_actions"), model, doc, &mut out);
    }
    actions(page.get("header").and_then(|h| h.get("actions")), model, doc, &mut out);
    out
}

fn actions(list: Option<&Value>, model: &Model, doc: &Value, out: &mut Findings) {
    for action in list.map(Value::list).unwrap_or(&[]) {
        let path = action.get_str("path").unwrap_or("");
        if let Some(command) = action.get_str("does") {
            match model.commands.get(command) {
                None => out.errors.push(format!("{path}: command `{command}` is not in the model")),
                Some(input) => {
                    for (name, _) in action.get("bind").map(Value::entries).unwrap_or(&[]) {
                        if !input.iter().any(|f| f.name == *name) {
                            out.errors.push(format!("{path}: `{name}` is not an input of `{command}`"));
                        }
                    }
                }
            }
        }
        if let Some(to) = action.get("navigate").and_then(|n| n.get_str("to")) {
            if doc.get("pages").and_then(|p| p.get(to)).is_none() {
                out.errors.push(format!("{path}: page `{to}` is not in the document"));
            }
        }
    }
}
// END PART
