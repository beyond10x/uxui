//! The part of the ESS model the page binds to: views (params, fields), events (fields) and
//! command names, read from the specification files `ess-inputs.yaml` lists.

use crate::yaml::{self, Value};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Default)]
pub struct Model {
    pub views: BTreeMap<String, View>,
    pub events: BTreeMap<String, Vec<Field>>,
    pub commands: BTreeMap<String, Vec<Field>>,
}

#[derive(Debug, Default)]
pub struct View {
    pub params: Vec<String>,
    pub fields: Vec<Field>,
    pub display: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: String,
}

impl View {
    pub fn field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.name == name)
    }
}

pub fn load(dir: &Path) -> Result<Model, String> {
    let inputs = dir.join("ess-inputs.yaml");
    let text =
        std::fs::read_to_string(&inputs).map_err(|e| format!("{}: {e}", inputs.display()))?;
    let doc = yaml::parse(&text).map_err(|e| format!("{}: {e}", inputs.display()))?;
    let mut model = Model::default();
    for file in doc.get("specification").map(Value::list).unwrap_or(&[]) {
        let Some(file) = file.str() else { continue };
        let path = dir.join(file);
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let spec = yaml::parse(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        if spec.get("domain").is_none() {
            continue;
        }
        for view in spec.get("views").map(Value::list).unwrap_or(&[]) {
            let Some(name) = view.get_str("name") else {
                continue;
            };
            let params = view
                .get("params")
                .map(Value::list)
                .unwrap_or(&[])
                .iter()
                .filter_map(|p| p.get_str("name").map(str::to_string))
                .collect();
            let display = view
                .get("naming")
                .and_then(|n| n.get_str("display"))
                .map(str::to_string);
            model.views.insert(
                name.to_string(),
                View {
                    params,
                    fields: fields(view.get("fields")),
                    display,
                },
            );
        }
        for event in spec.get("events").map(Value::list).unwrap_or(&[]) {
            if let Some(name) = event.get_str("name") {
                model
                    .events
                    .insert(name.to_string(), fields(event.get("fields")));
            }
        }
        for command in spec.get("commands").map(Value::list).unwrap_or(&[]) {
            if let Some(name) = command.get_str("name") {
                model
                    .commands
                    .insert(name.to_string(), fields(command.get("input")));
            }
        }
    }
    Ok(model)
}

fn fields(list: Option<&Value>) -> Vec<Field> {
    list.map(Value::list)
        .unwrap_or(&[])
        .iter()
        .filter_map(|f| {
            Some(Field {
                name: f.get_str("name")?.to_string(),
                ty: f.get_str("type").unwrap_or("").to_string(),
            })
        })
        .collect()
}
