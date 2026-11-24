//! The node tree of an `ess-ui/1` document: every node with its canonical node path, the
//! sections and overlays a page inherits from its kind included, and the places the document alone
//! cannot say what is drawn (a host slot, a widget use, a component or page kind this reader does
//! not know). The path grammar is ESS's `NodePath` (`ess-ui.md`, § NodePath): map containers name
//! a child by its key, lists by its `name` (fields by `name` or `field`, actions by `name` or the
//! derived action name), any other key holding a map is a single-valued child, and runtime rows sit
//! under `rows/<key>`.

use crate::yaml::Value;
use std::collections::BTreeSet;

const MAP_CONTAINERS: &[&str] = &[
    "shells",
    "page_kinds",
    "widgets",
    "types",
    "pages",
    "channels",
    "regions",
    "overlays",
    "state",
    "params",
    "board_widgets",
];
const FIELD_LISTS: &[&str] = &["columns", "inputs", "fields"];
const ACTION_LISTS: &[&str] = &[
    "row_actions",
    "bulk_actions",
    "actions",
    "item_actions",
    "node_actions",
    "edge_actions",
    "alternatives",
];
const NAMED_LISTS: &[&str] = &[
    "sections",
    "item",
    "choices",
    "parts",
    "metrics",
    "toolbar",
    "guards",
    "tabs",
    "groups",
    "children",
    "body",
    "navigation_sections",
];

/// The composites of `ess-ui/1`. A `component` naming anything else is a widget use or unknown.
pub const COMPONENTS: &[&str] = &[
    "collection",
    "record",
    "form",
    "choice",
    "filter_bar",
    "header",
    "overlay",
    "confirm",
    "metric",
    "chart",
    "board",
    "graph_editor",
    "rich_text",
    "references",
];

/// What a person operates or reads a name from, where the document says so.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// An entry of an action list, or a section's empty-state action.
    Action,
    /// The overlay an inline `confirm: {title}` expands to, at `<action>/confirm/overlay`.
    InlineConfirm,
    /// An overlay or section with `component: confirm`.
    Confirm,
    /// A field of a form: an input.
    Input,
    /// A form's submit button, at `<form>/submit`.
    Submit,
    /// A choice of a filter bar, or a section that is a choice.
    Choice,
    /// A primitive the person operates or that needs a text alternative.
    Primitive,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub path: String,
    /// The key of the container the node sits in (`sections`, `row_actions`, ...), or the field key.
    pub container: String,
    pub value: Value,
    pub role: Option<Role>,
    /// Drawn on a page; false inside the declaration of a widget or a page kind.
    pub drawn: bool,
}

/// A node whose drawn content the document alone does not give.
#[derive(Debug, Clone)]
pub struct Opaque {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Default)]
pub struct Tree {
    pub nodes: Vec<Node>,
    pub opaque: Vec<Opaque>,
    paths: BTreeSet<String>,
}

impl Tree {
    /// Whether the document declares a node at `path` (runtime `rows/<key>` segments dropped).
    pub fn has(&self, path: &str) -> bool {
        self.paths.contains(&normalize(path))
    }

    /// The opaque node at or above `path`, if any.
    pub fn opaque_over(&self, path: &str) -> Option<&Opaque> {
        let path = normalize(path);
        self.opaque
            .iter()
            .find(|o| path == o.path || path.starts_with(&format!("{}/", o.path)))
    }

    fn add(&mut self, node: Node) -> bool {
        if !self.paths.insert(node.path.clone()) {
            return false;
        }
        self.nodes.push(node);
        true
    }
}

/// A node path with its runtime row segments (`rows/<key>`) dropped: the declared node it names.
pub fn normalize(path: &str) -> String {
    let segments: Vec<&str> = path.split('/').collect();
    let mut out: Vec<&str> = Vec::new();
    let mut i = 0;
    while i < segments.len() {
        let container = out.last().is_some_and(|prev| is_container(prev));
        if segments[i] == "rows" && !container && i + 1 < segments.len() {
            i += 2;
            continue;
        }
        out.push(segments[i]);
        i += 1;
    }
    out.join("/")
}

fn is_container(key: &str) -> bool {
    [MAP_CONTAINERS, FIELD_LISTS, ACTION_LISTS, NAMED_LISTS]
        .iter()
        .any(|set| set.contains(&key))
}

/// An action's node name: its `name`, else ESS's derived name, the first present of `opens`,
/// `does` (its last segment in snake case), `navigate.to`, `export.reads`, `upload.does` and the
/// first key of `sets`. A `copy` action derives none.
pub fn action_name(action: &Value) -> Option<String> {
    if let Some(name) = action.get_str("name") {
        return Some(name.to_string());
    }
    if let Some(opens) = action.get_str("opens") {
        return Some(opens.to_string());
    }
    if let Some(does) = action.get_str("does") {
        let last = does.rsplit('.').next().unwrap_or(does);
        let mut out = String::new();
        for (i, c) in last.chars().enumerate() {
            if c.is_uppercase() && i > 0 {
                out.push('_');
            }
            out.extend(c.to_lowercase());
        }
        return Some(out);
    }
    let nested = |key: &str, field: &str| {
        action
            .get(key)
            .and_then(|v| v.get_str(field))
            .map(str::to_string)
    };
    nested("navigate", "to")
        .or_else(|| nested("export", "reads"))
        .or_else(|| nested("upload", "does"))
        .or_else(|| {
            action
                .get("sets")
                .and_then(|s| s.entries().first())
                .map(|(k, _)| k.clone())
        })
}

/// The sections a built-in page kind gives every page of that kind (`ess-ui.md`, § PageKind).
fn builtin_sections(kind: &str) -> Option<Vec<(&'static str, &'static str)>> {
    Some(match kind {
        "list_page" | "report_page" => vec![("filters", "filter_bar"), ("list", "collection")],
        "detail_page" | "static_page" => vec![(
            if kind == "detail_page" {
                "summary"
            } else {
                "body"
            },
            "record",
        )],
        "settings_page" => vec![("settings", "form")],
        "dashboard_page" => vec![("board", "board")],
        "editor_page" => vec![("editor", "graph_editor")],
        "form_page" => vec![("form", "form")],
        _ => return None,
    })
}

/// The kind's own sections, overlays, header and state, `extends` followed; `None` for a kind
/// neither built in nor declared under `page_kinds`.
fn kind_parts(doc: &Value, kind: &str, depth: usize) -> Option<Value> {
    if let Some(sections) = builtin_sections(kind) {
        let sections = sections
            .into_iter()
            .map(|(name, component)| {
                Value::Map(vec![
                    ("name".to_string(), Value::Str(name.to_string())),
                    ("component".to_string(), Value::Str(component.to_string())),
                ])
            })
            .collect();
        return Some(Value::Map(vec![(
            "sections".to_string(),
            Value::List(sections),
        )]));
    }
    let declared = doc.get("page_kinds")?.get(kind)?;
    let base = match declared.get_str("extends") {
        Some(parent) if depth < 16 => kind_parts(doc, parent, depth + 1)?,
        Some(_) => return None,
        None => Value::Map(vec![]),
    };
    Some(merge_page(&base, declared))
}

/// A page over what it inherits: sections merge by name (inherited first, `remove: true` drops
/// one), the maps `overlays` and `state` by key, `header` key by key; the page wins.
fn merge_page(base: &Value, page: &Value) -> Value {
    let mut out: Vec<(String, Value)> = Vec::new();
    for (key, inherited) in base.entries() {
        if page.get(key).is_none() {
            out.push((key.clone(), inherited.clone()));
        }
    }
    for (key, own) in page.entries() {
        let merged = match (key.as_str(), base.get(key)) {
            ("sections", Some(inherited)) => Value::List(merge_named(inherited.list(), own.list())),
            ("overlays" | "state" | "header", Some(inherited)) => merge_keys(inherited, own),
            _ => own.clone(),
        };
        out.push((key.clone(), merged));
    }
    Value::Map(out)
}

fn merge_keys(base: &Value, over: &Value) -> Value {
    let mut out: Vec<(String, Value)> = base
        .entries()
        .iter()
        .filter(|(k, _)| over.get(k).is_none())
        .cloned()
        .collect();
    out.extend(over.entries().iter().filter(|(_, v)| !v.is_null()).cloned());
    Value::Map(out)
}

fn merge_named(base: &[Value], over: &[Value]) -> Vec<Value> {
    let name = |v: &Value| v.get_str("name").map(str::to_string);
    let mut out = Vec::new();
    for b in base {
        match over.iter().find(|o| name(o) == name(b)) {
            Some(o) if o.get("remove") == Some(&Value::Bool(true)) => {}
            Some(o) => out.push(merge_keys(b, o)),
            None => out.push(b.clone()),
        }
    }
    out.extend(
        over.iter()
            .filter(|o| !base.iter().any(|b| name(b) == name(o)))
            .cloned(),
    );
    out
}

/// Every node of `doc`.
pub fn tree(doc: &Value) -> Tree {
    let mut walker = Walker {
        doc,
        tree: Tree::default(),
    };
    for (key, value) in doc.entries() {
        let drawn = key != "widgets" && key != "page_kinds";
        match (key.as_str(), value) {
            ("pages", Value::Map(pages)) => {
                for (name, page) in pages {
                    let path = format!("pages/{name}");
                    let kind = page.get_str("kind");
                    let resolved = match kind.map(|k| (k, kind_parts(doc, k, 0))) {
                        Some((_, Some(parts))) => merge_page(&parts, page),
                        Some((k, None)) => {
                            walker.opaque(
                                &path,
                                format!("page kind `{k}` is neither built in nor declared"),
                            );
                            page.clone()
                        }
                        None if page.get("host").is_some() => page.clone(),
                        None => {
                            walker.opaque(&path, "the page names no kind".to_string());
                            page.clone()
                        }
                    };
                    walker.walk(&resolved, path, "pages", false, None, true);
                }
            }
            _ => walker.child(key, value, "", false, drawn),
        }
    }
    walker.tree
}

struct Walker<'a> {
    doc: &'a Value,
    tree: Tree,
}

impl Walker<'_> {
    fn opaque(&mut self, path: &str, reason: String) {
        self.tree.opaque.push(Opaque {
            path: path.to_string(),
            reason,
        });
    }

    /// One node and everything below it. `form`: the node sits in a form, so its fields are inputs.
    fn walk(
        &mut self,
        value: &Value,
        path: String,
        container: &str,
        form: bool,
        role: Option<Role>,
        drawn: bool,
    ) {
        let component = value.get_str("component");
        let role = role.or(match (container, component) {
            (_, Some("confirm")) => Some(Role::Confirm),
            ("sections", Some("choice")) | ("choices", _) => Some(Role::Choice),
            _ if value.get("primitive").is_some() => Some(Role::Primitive),
            _ => None,
        });
        if !self.tree.add(Node {
            path: path.clone(),
            container: container.to_string(),
            value: value.clone(),
            role,
            drawn,
        }) {
            return;
        }
        let Value::Map(entries) = value else { return };
        if let Some(host) = value.get("host") {
            let host = host.scalar_text().unwrap_or_default();
            self.opaque(
                &path,
                format!("host slot `{host}`: the host draws it, not the document"),
            );
        }
        let form = match component {
            Some("form") => true,
            Some(_) => false,
            None => form,
        };
        if let Some(name) = component.filter(|c| !COMPONENTS.contains(c)) {
            let widget = self.doc.get("widgets").and_then(|w| w.get(name)).is_some();
            let reason = if widget {
                format!("widget use `{name}`: ESS expands its body, this reader does not")
            } else {
                format!("component `{name}` is not an ess-ui/1 composite")
            };
            if drawn {
                self.opaque(&path, reason);
            }
        }
        if component == Some("form") && value.get_str("save") != Some("on_change") {
            let submit = value.get("submit").cloned().unwrap_or(Value::Null);
            self.walk(
                &submit,
                format!("{path}/submit"),
                "submit",
                false,
                Some(Role::Submit),
                drawn,
            );
        }
        for (key, child) in entries {
            self.child(key, child, &path, form, drawn);
        }
    }

    fn child(&mut self, key: &str, child: &Value, parent: &str, form: bool, drawn: bool) {
        let at = |name: &str| {
            if parent.is_empty() {
                format!("{key}/{name}")
            } else {
                format!("{parent}/{key}/{name}")
            }
        };
        let field_key = if parent.is_empty() {
            key.to_string()
        } else {
            format!("{parent}/{key}")
        };
        match child {
            Value::Map(entries) if MAP_CONTAINERS.contains(&key) => {
                for (name, node) in entries {
                    self.walk(node, at(name), key, false, None, drawn);
                }
            }
            Value::List(items) if FIELD_LISTS.contains(&key) => {
                for item in items {
                    let name = item
                        .str()
                        .or_else(|| item.get_str("name"))
                        .or_else(|| item.get_str("field"));
                    if let Some(name) = name {
                        let role = (form && key != "columns").then_some(Role::Input);
                        self.walk(item, at(name), key, form, role, drawn);
                    }
                }
            }
            Value::List(items) if ACTION_LISTS.contains(&key) => {
                for item in items {
                    if let Some(name) = action_name(item) {
                        self.walk(item, at(&name), key, false, Some(Role::Action), drawn);
                    }
                }
            }
            Value::List(items) if NAMED_LISTS.contains(&key) => {
                for item in items {
                    if let Some(name) = item.get_str("name") {
                        self.walk(item, at(name), key, form, None, drawn);
                    }
                }
            }
            Value::List(items) if key == "preload_views" => {
                for item in items {
                    if let Some(view) = item.get_str("view") {
                        self.walk(item, at(view), key, false, None, drawn);
                    }
                }
            }
            Value::Map(_) if key == "confirm" => {
                self.walk(child, field_key.clone(), key, false, None, drawn);
                self.walk(
                    child,
                    format!("{field_key}/overlay"),
                    "confirm",
                    false,
                    Some(Role::InlineConfirm),
                    drawn,
                );
            }
            // A section's empty state may offer one action (`states: {empty: {action}}`); a
            // primitive's `action` is named by its primitive, so it is not a node of its own role.
            Value::Map(_) if key == "action" && parent.ends_with("/states/empty") => {
                self.walk(child, field_key, key, false, Some(Role::Action), drawn);
            }
            Value::Map(_) if key != "rows" => self.walk(child, field_key, key, form, None, drawn),
            _ => {}
        }
    }
}
