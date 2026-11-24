//! Resolution of one page of an `ess-ui/1` document into the form the page runtime reads: page
//! kinds merged, short forms expanded, defaults filled, tone maps looked up and every node given
//! its canonical node path. ESS does all of this in `ess-ui` but hands no resolved form over
//! (`ess ui load` prints a summary line), so it is re-implemented here for the constructs the
//! page uses. Each part is fenced by `PART:` markers; `docs/own-generator.md` counts them.

use crate::yaml::Value;

pub fn obj(entries: Vec<(&str, Value)>) -> Value {
    Value::Map(entries.into_iter().filter(|(_, v)| !v.is_null()).map(|(k, v)| (k.to_string(), v)).collect())
}

pub fn s(text: &str) -> Value {
    Value::Str(text.to_string())
}

fn opt(text: Option<&str>) -> Value {
    text.map(s).unwrap_or(Value::Null)
}

fn cloned(value: Option<&Value>) -> Value {
    value.cloned().unwrap_or(Value::Null)
}

// PART: defaults and short forms
/// The built-in page kinds of `ess-ui/1` the generator supports: what a page inherits.
fn page_kind(kind: &str) -> Result<Value, String> {
    let sections = |name: &str, component: &str| {
        Value::List(vec![obj(vec![("name", s(name)), ("component", s(component))])])
    };
    match kind {
        "detail_page" => Ok(obj(vec![("sections", sections("summary", "record"))])),
        "list_page" => Ok(obj(vec![
            ("sections", Value::List(vec![
                obj(vec![("name", s("filters")), ("component", s("filter_bar"))]),
                obj(vec![("name", s("list")), ("component", s("collection"))]),
            ])),
            ("header", obj(vec![("title", s("from_page")), ("total", s("list")), ("filters", s("filters"))])),
        ])),
        other => Err(format!("page kind `{other}` is not supported by this generator")),
    }
}

/// Deep merge of maps, the page winning; `null` removes an inherited entry.
fn merge(base: &Value, over: &Value) -> Value {
    match (base, over) {
        (Value::Map(b), Value::Map(o)) => {
            let mut out: Vec<(String, Value)> = Vec::new();
            for (k, v) in b {
                match o.iter().find(|(ok, _)| ok == k) {
                    Some((_, Value::Null)) => {}
                    Some((_, ov)) => out.push((k.clone(), merge(v, ov))),
                    None => out.push((k.clone(), v.clone())),
                }
            }
            for (k, v) in o {
                if !b.iter().any(|(bk, _)| bk == k) && !v.is_null() {
                    out.push((k.clone(), v.clone()));
                }
            }
            Value::Map(out)
        }
        _ => over.clone(),
    }
}

/// Named lists merge by name: inherited entries in their order, then new ones in page order;
/// `{name, remove: true}` drops an inherited one.
fn merge_named(base: &[Value], over: &[Value]) -> Vec<Value> {
    let name = |v: &Value| v.get_str("name").map(str::to_string);
    let mut out = Vec::new();
    for b in base {
        match over.iter().find(|o| name(o) == name(b)) {
            Some(o) if o.get("remove") == Some(&Value::Bool(true)) => {}
            Some(o) => out.push(merge(b, o)),
            None => out.push(b.clone()),
        }
    }
    for o in over {
        if !base.iter().any(|b| name(b) == name(o)) {
            out.push(o.clone());
        }
    }
    out
}

/// A field written as a bare name is `{field: <name>, name: <name>}`; `name` defaults to
/// `field`; the label to the name with `_` read as a space (the stock renderers' rule; the schema
/// says `model.field_label`, which the model does not declare).
fn field(doc: &Value, written: &Value, parent: &str, container: &str) -> Value {
    let (field, rest) = match written {
        Value::Str(name) => (name.clone(), Value::Map(vec![])),
        other => (other.get_str("field").unwrap_or("").to_string(), other.clone()),
    };
    let name = rest.get_str("name").unwrap_or(&field).to_string();
    let label = rest.get_str("label").map(str::to_string).unwrap_or_else(|| name.replace('_', " "));
    obj(vec![
        ("field", s(&field)),
        ("name", s(&name)),
        ("label", s(&label)),
        ("as", s(rest.get_str("as").unwrap_or("text"))),
        ("tones", tone_map(doc, rest.get("tone_by"))),
        ("tone", opt(rest.get_str("tone"))),
        ("visible", opt(rest.get_str("visible"))),
        ("path", s(&format!("{parent}/{container}/{name}"))),
    ])
}

/// An action's name when it declares none: the first of `opens`, the command's last segment in
/// snake case, `navigate.to`, `export.reads`, `upload.does`, the first key `sets`.
fn action_name(action: &Value) -> String {
    uxui_doc::tree::action_name(action).unwrap_or_else(|| "action".to_string())
}

/// An action: its name, label, effect and, for an inline `confirm: {title}`, the confirm overlay
/// the short form expands to, at `<action path>/confirm/overlay`.
fn action(written: &Value, parent: &str, container: &str) -> Value {
    let name = action_name(written);
    let path = format!("{parent}/{container}/{name}");
    let confirm = written.get("confirm").map(|c| match c {
        Value::Str(overlay) => obj(vec![("overlay", s(overlay)), ("path", s(&format!("{path}/confirm/overlay")))]),
        record => obj(vec![
            ("kind", s("dialog")),
            ("title", cloned(record.get("title"))),
            ("body", cloned(record.get("body"))),
            // The schema's default is `Delete`; the stock renderers show `Confirm`.
            ("confirm_label", s(record.get_str("confirm_label").unwrap_or("Confirm"))),
            ("path", s(&format!("{path}/confirm/overlay"))),
        ]),
    });
    obj(vec![
        ("name", s(&name)),
        ("label", s(&written.get_str("label").map(str::to_string).unwrap_or_else(|| name.replace('_', " ")))),
        ("does", cloned(written.get("does"))),
        ("bind", cloned(written.get("bind"))),
        ("opens", cloned(written.get("opens"))),
        ("navigate", cloned(written.get("navigate"))),
        ("visible", cloned(written.get("visible"))),
        ("confirm", confirm.unwrap_or(Value::Null)),
        ("path", s(&path)),
    ])
}

/// A duration (`500ms`, `30s`, `1m`, `1h`) in milliseconds.
pub fn millis(text: &str) -> Option<i64> {
    let (number, unit) = text.find(|c: char| !c.is_ascii_digit()).map(|at| text.split_at(at))?;
    let n: i64 = number.parse().ok()?;
    Some(match unit {
        "ms" => n,
        "s" => n * 1000,
        "m" => n * 60_000,
        "h" => n * 3_600_000,
        _ => return None,
    })
}
// END PART

// PART: tone maps
/// `tone_by: <name>` resolves to the document's tone map of that name.
fn tone_map(doc: &Value, tone_by: Option<&Value>) -> Value {
    match tone_by {
        Some(Value::Str(name)) => doc.get("tone_maps").and_then(|m| m.get(name)).cloned().unwrap_or(Value::Null),
        Some(Value::Map(_)) => tone_by.and_then(|t| t.get("map")).cloned().unwrap_or(Value::Null),
        _ => Value::Null,
    }
}
// END PART

// PART: live and channel semantics
/// A section's `live`: `match` defaults to the read's `key`, else `id`; durations in ms.
fn live(written: Option<&Value>, key: &str) -> Value {
    let Some(live) = written else { return Value::Null };
    obj(vec![
        ("channel", cloned(live.get("channel"))),
        ("on", cloned(live.get("on"))),
        ("effect", cloned(live.get("effect"))),
        ("match", s(live.get_str("match").unwrap_or(key))),
        ("only_if", cloned(live.get("only_if"))),
        ("coalesce", live.get_str("coalesce").and_then(millis).map(Value::Int).unwrap_or(Value::Null)),
        ("max_rows", cloned(live.get("max_rows"))),
        ("when_paged_away", s(live.get_str("when_paged_away").unwrap_or("count_new"))),
    ])
}

/// The document's channels, as the runtime plays them.
pub fn channels(doc: &Value) -> Value {
    Value::Map(
        doc.get("channels")
            .map(Value::entries)
            .unwrap_or(&[])
            .iter()
            .map(|(name, ch)| {
                let events = ch.get("carries").and_then(|c| c.get("events")).cloned().unwrap_or(Value::List(vec![]));
                let backoff = ch.get("reconnect").and_then(|r| r.get("backoff"));
                let ms = |v: Option<&Value>| v.and_then(Value::str).and_then(millis).map(Value::Int).unwrap_or(Value::Null);
                (
                    name.clone(),
                    obj(vec![
                        ("events", events),
                        ("delivery", cloned(ch.get("delivery"))),
                        ("resume", cloned(ch.get("resume"))),
                        ("stale_after", ms(ch.get("stale_after"))),
                        ("backoff_from", ms(backoff.and_then(|b| b.get("from")))),
                        ("backoff_to", ms(backoff.and_then(|b| b.get("to")))),
                    ]),
                )
            })
            .collect(),
    )
}
// END PART

// PART: page resolution and node paths
/// One page, resolved.
pub fn page(doc: &Value, name: &str) -> Result<Value, String> {
    let written = doc.get("pages").and_then(|p| p.get(name)).ok_or(format!("no page `{name}`"))?;
    let kind = written.get_str("kind").ok_or(format!("page `{name}` has no kind"))?;
    let inherited = page_kind(kind)?;
    let path = format!("pages/{name}");
    let sections_written = merge_named(
        inherited.get("sections").map(Value::list).unwrap_or(&[]),
        written.get("sections").map(Value::list).unwrap_or(&[]),
    );
    let header = merge(&cloned(inherited.get("header")), &cloned(written.get("header")));
    let title = written.get_str("title").unwrap_or(name);
    let header_path = format!("{path}/header");
    let header_title = match header.get_str("title") {
        Some("from_page") | None => title.to_string(),
        Some(other) => other.to_string(),
    };
    let actions = header.get("actions").map(Value::list).unwrap_or(&[]);
    let switch_to: Vec<Value> = std::iter::once(obj(vec![("page", s(name)), ("label", s(title)), ("current", Value::Bool(true))]))
        .chain(written.get("switch_to").map(Value::list).unwrap_or(&[]).iter().filter_map(Value::str).map(|to| {
            let label = doc.get("pages").and_then(|p| p.get(to)).and_then(|p| p.get_str("title")).unwrap_or(to);
            obj(vec![("page", s(to)), ("label", s(label))])
        }))
        .collect();
    let sections: Vec<Value> =
        sections_written.iter().map(|sec| section(doc, sec, &path)).collect::<Result<_, _>>()?;
    let layout = match written.get("layout").and_then(|l| l.get("columns")) {
        Some(columns) => Value::List(
            columns
                .list()
                .iter()
                .map(|c| {
                    let cname = c.get_str("name").unwrap_or("");
                    obj(vec![
                        ("name", s(cname)),
                        ("width", s(c.get_str("width").unwrap_or("normal"))),
                        ("sections", cloned(c.get("sections"))),
                        ("path", s(&format!("{path}/layout/columns/{cname}"))),
                    ])
                })
                .collect(),
        ),
        // `stack`: one column holding every section in declaration order.
        None => Value::List(vec![obj(vec![
            ("name", s("stack")),
            ("width", s("wide")),
            ("sections", Value::List(sections.iter().map(|x| cloned(x.get("name"))).collect())),
        ])]),
    };
    Ok(obj(vec![
        ("name", s(name)),
        ("path", s(&path)),
        ("title", s(title)),
        ("params", Value::List(written.get("params").map(Value::entries).unwrap_or(&[]).iter().map(|(k, _)| s(k)).collect())),
        (
            "header",
            obj(vec![
                ("title", s(&header_title)),
                ("live", cloned(header.get("live"))),
                ("actions", Value::List(actions.iter().map(|a| action(a, &header_path, "actions")).collect())),
                ("switch", Value::List(switch_to)),
                ("path", s(&header_path)),
            ]),
        ),
        ("layout", layout),
        ("sections", Value::List(sections)),
    ]))
}

fn section(doc: &Value, written: &Value, page_path: &str) -> Result<Value, String> {
    let name = written.get_str("name").ok_or("a section without a name")?;
    let path = format!("{page_path}/sections/{name}");
    let component = written.get_str("component").ok_or(format!("section `{name}` has no component"))?;
    let reads = match written.get("reads") {
        Some(Value::Str(view)) => obj(vec![("view", s(view))]),
        Some(r) => r.clone(),
        None => Value::Null,
    };
    let key = reads.get_str("key").or_else(|| written.get("live").and_then(|l| l.get_str("match"))).unwrap_or("id").to_string();
    let reads = if reads.is_null() {
        reads
    } else {
        obj(vec![
            ("view", cloned(reads.get("view"))),
            ("key", s(&key)),
            ("params", reads.get("params").cloned().unwrap_or(Value::Map(vec![]))),
            ("paging", s(reads.get_str("paging").unwrap_or("none"))),
            ("filter", cloned(reads.get("filter"))),
        ])
    };
    let fields = |key: &str, container: &str| {
        Value::List(written.get(key).map(Value::list).unwrap_or(&[]).iter().map(|f| field(doc, f, &path, container)).collect())
    };
    let fields_value = match component {
        "record" => fields("fields", "fields"),
        "collection" => fields("columns", "columns"),
        "chart" => Value::Null,
        other => return Err(format!("section `{name}`: component `{other}` is not supported by this generator")),
    };
    let empty = written.get("states").and_then(|st| st.get("empty")).and_then(|e| e.get_str("message"));
    Ok(obj(vec![
        ("name", s(name)),
        ("path", s(&path)),
        ("component", s(component)),
        ("title", cloned(written.get("title"))),
        ("reads", reads),
        ("live", live(written.get("live"), &key)),
        ("fields", fields_value),
        ("chart", cloned(written.get("chart"))),
        ("x", cloned(written.get("x"))),
        ("series", cloned(written.get("series"))),
        ("sort", cloned(written.get("sort"))),
        ("row_actions", Value::List(written.get("row_actions").map(Value::list).unwrap_or(&[]).iter().map(|a| action(a, &path, "row_actions")).collect())),
        // Absent, a section with no rows says what the stock renderers say.
        ("empty", s(empty.unwrap_or("Nothing to show yet"))),
        ("degrades", cloned(written.get("degrades"))),
    ]))
}
// END PART
