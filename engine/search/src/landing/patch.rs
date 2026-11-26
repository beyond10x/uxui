//! What a variant's patch is made of: RFC 6902 operations on a JSON value, the JSON text
//! `product.json` is written in, and the custom properties of a style sheet.

pub use uxui_doc::yaml::Value;

/// One RFC 6902 operation. A path is a JSON pointer (RFC 6901).
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    /// Sets a member, or inserts an element before the index (`-`: after the last).
    Add { path: String, value: Value },
    /// Removes the value at the path, which must exist.
    Remove { path: String },
    /// Puts a value in place of the one at the path, which must exist.
    Replace { path: String, value: Value },
    /// Removes the value at `from` and adds it at `path`.
    Move { from: String, path: String },
    /// Adds a copy of the value at `from` at `path`.
    Copy { from: String, path: String },
    /// Refuses the whole patch unless the value at the path equals `value`.
    Test { path: String, value: Value },
}

/// Custom properties of one rule of a style sheet: `<selector> { <property>: <value>; ... }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CssRule {
    /// The rule's selector as the style sheet writes it, at the top level (no at-rule).
    pub selector: String,
    /// The custom properties (`--name`) and their values, in the order written.
    pub values: Vec<(String, String)>,
}

// JSON text.

/// Reads JSON text; objects keep their key order.
pub fn parse(text: &str) -> Result<Value, String> {
    ux_browser::json::parse(text)
}

/// A string as `JSON.stringify` writes it: only `"`, `\` and control characters escaped.
fn string(text: &str, out: &mut String) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

/// A value on one line (`indent` is `None`) or over lines, two spaces per level.
fn put(value: &Value, indent: Option<usize>, out: &mut String) {
    let line = |out: &mut String, level: usize| {
        if indent.is_some() {
            out.push('\n');
            out.push_str(&"  ".repeat(level));
        }
    };
    let inner = indent.map(|level| level + 1);
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Int(n) => out.push_str(&n.to_string()),
        Value::Float(f) if f.is_finite() => out.push_str(&f.to_string()),
        Value::Float(_) => out.push_str("null"),
        Value::Str(s) => string(s, out),
        Value::List(items) if items.is_empty() => out.push_str("[]"),
        Value::Map(entries) if entries.is_empty() => out.push_str("{}"),
        Value::List(items) => {
            out.push('[');
            for (at, item) in items.iter().enumerate() {
                if at > 0 {
                    out.push(',');
                }
                line(out, inner.unwrap_or(0));
                put(item, inner, out);
            }
            line(out, indent.unwrap_or(0));
            out.push(']');
        }
        Value::Map(entries) => {
            out.push('{');
            for (at, (key, item)) in entries.iter().enumerate() {
                if at > 0 {
                    out.push(',');
                }
                line(out, inner.unwrap_or(0));
                string(key, out);
                out.push(':');
                if indent.is_some() {
                    out.push(' ');
                }
                put(item, inner, out);
            }
            line(out, indent.unwrap_or(0));
            out.push('}');
        }
    }
}

/// The value as `JSON.stringify(value, null, 2)` writes it, and a newline. A number is written
/// the shortest way, so `1.0` reads back as `1`.
pub fn pretty(value: &Value) -> String {
    let mut out = String::new();
    put(value, Some(0), &mut out);
    out.push('\n');
    out
}

/// The value on one line, without spaces.
pub fn compact(value: &Value) -> String {
    let mut out = String::new();
    put(value, None, &mut out);
    out
}

// RFC 6902.

impl Op {
    /// The operation as RFC 6902 writes it: `op`, then `from`, `path` and `value` as it has
    /// them.
    pub fn value(&self) -> Value {
        let text = |text: &str| Value::Str(text.to_owned());
        let entries: Vec<(&str, Value)> = match self {
            Op::Add { path, value } => vec![
                ("op", text("add")),
                ("path", text(path)),
                ("value", value.clone()),
            ],
            Op::Remove { path } => vec![("op", text("remove")), ("path", text(path))],
            Op::Replace { path, value } => vec![
                ("op", text("replace")),
                ("path", text(path)),
                ("value", value.clone()),
            ],
            Op::Move { from, path } => vec![
                ("op", text("move")),
                ("from", text(from)),
                ("path", text(path)),
            ],
            Op::Copy { from, path } => vec![
                ("op", text("copy")),
                ("from", text(from)),
                ("path", text(path)),
            ],
            Op::Test { path, value } => vec![
                ("op", text("test")),
                ("path", text(path)),
                ("value", value.clone()),
            ],
        };
        Value::Map(
            entries
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value))
                .collect(),
        )
    }
}

/// The operations as one JSON list on one line.
pub fn ops_json(ops: &[Op]) -> String {
    compact(&Value::List(ops.iter().map(Op::value).collect()))
}

/// The reference tokens of a JSON pointer: `/a~1b/0` is `a/b`, `0`.
fn tokens(pointer: &str) -> Result<Vec<String>, String> {
    if pointer.is_empty() {
        return Ok(Vec::new());
    }
    let Some(rest) = pointer.strip_prefix('/') else {
        return Err(format!(
            "`{pointer}` is not a JSON pointer: it does not start with `/`"
        ));
    };
    Ok(rest
        .split('/')
        .map(|token| token.replace("~1", "/").replace("~0", "~"))
        .collect())
}

/// A list index as RFC 6901 writes it: digits, no leading zero.
fn index(token: &str) -> Option<usize> {
    let plain = token == "0" || (!token.starts_with('0') && !token.is_empty());
    (plain && token.bytes().all(|b| b.is_ascii_digit()))
        .then(|| token.parse().ok())
        .flatten()
}

fn at<'a>(document: &'a Value, path: &[String]) -> Option<&'a Value> {
    path.iter().try_fold(document, |value, token| match value {
        Value::Map(entries) => entries.iter().find(|(key, _)| key == token).map(|(_, v)| v),
        Value::List(items) => items.get(index(token)?),
        _ => None,
    })
}

fn at_mut<'a>(document: &'a mut Value, path: &[String]) -> Option<&'a mut Value> {
    path.iter().try_fold(document, |value, token| match value {
        Value::Map(entries) => entries
            .iter_mut()
            .find(|(key, _)| key == token)
            .map(|(_, v)| v),
        Value::List(items) => items.get_mut(index(token)?),
        _ => None,
    })
}

fn add(document: &mut Value, pointer: &str, value: Value) -> Result<(), String> {
    let path = tokens(pointer)?;
    let Some((last, parent)) = path.split_last() else {
        *document = value;
        return Ok(());
    };
    match at_mut(document, parent) {
        Some(Value::Map(entries)) => {
            match entries.iter_mut().find(|(key, _)| key == last) {
                Some((_, held)) => *held = value,
                None => entries.push((last.clone(), value)),
            }
            Ok(())
        }
        Some(Value::List(items)) => {
            let place = if last == "-" {
                Some(items.len())
            } else {
                index(last).filter(|&place| place <= items.len())
            };
            let place = place.ok_or_else(|| {
                format!("`{pointer}`: no such place in a list of {}", items.len())
            })?;
            items.insert(place, value);
            Ok(())
        }
        _ => Err(format!("`{pointer}`: nothing to add into")),
    }
}

fn remove(document: &mut Value, pointer: &str) -> Result<Value, String> {
    let path = tokens(pointer)?;
    let missing = || format!("`{pointer}`: no such value");
    let Some((last, parent)) = path.split_last() else {
        return Err("the whole document cannot be removed".to_owned());
    };
    match at_mut(document, parent) {
        Some(Value::Map(entries)) => {
            let place = entries
                .iter()
                .position(|(key, _)| key == last)
                .ok_or_else(missing)?;
            Ok(entries.remove(place).1)
        }
        Some(Value::List(items)) => {
            let place = index(last)
                .filter(|&place| place < items.len())
                .ok_or_else(missing)?;
            Ok(items.remove(place))
        }
        _ => Err(missing()),
    }
}

fn get(document: &Value, pointer: &str) -> Result<Value, String> {
    at(document, &tokens(pointer)?)
        .cloned()
        .ok_or_else(|| format!("`{pointer}`: no such value"))
}

/// Whether two JSON values are equal as RFC 6902 section 4.6 has it: objects by their members
/// whatever the order, arrays element by element, numbers by their value.
fn equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(a), Value::Float(b)) | (Value::Float(b), Value::Int(a)) => {
            *b == *a as f64 && *b as i64 == *a
        }
        (Value::List(a), Value::List(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| equal(a, b))
        }
        (Value::Map(a), Value::Map(b)) => {
            let within = |a: &[(String, Value)], b: &[(String, Value)]| {
                a.iter().all(|(key, value)| {
                    b.iter()
                        .find(|(other, _)| other == key)
                        .is_some_and(|(_, other)| equal(value, other))
                })
            };
            a.len() == b.len() && within(a, b) && within(b, a)
        }
        _ => a == b,
    }
}

fn apply_op(document: &mut Value, op: &Op) -> Result<(), String> {
    match op {
        Op::Add { path, value } => add(document, path, value.clone()),
        Op::Remove { path } => remove(document, path).map(|_| ()),
        Op::Replace { path, value } => {
            let held = at_mut(document, &tokens(path)?)
                .ok_or_else(|| format!("`{path}`: no such value"))?;
            *held = value.clone();
            Ok(())
        }
        Op::Move { from, path } => {
            if path
                .strip_prefix(from.as_str())
                .is_some_and(|rest| rest.starts_with('/'))
            {
                return Err(format!("`{from}` cannot move into itself, `{path}`"));
            }
            let value = remove(document, from)?;
            add(document, path, value)
        }
        Op::Copy { from, path } => {
            let value = get(document, from)?;
            add(document, path, value)
        }
        Op::Test { path, value } => {
            let held = get(document, path)?;
            if equal(&held, value) {
                Ok(())
            } else {
                Err(format!(
                    "`{path}` is {}, not {}",
                    compact(&held),
                    compact(value)
                ))
            }
        }
    }
}

/// Applies the operations in order; the first that fails refuses the patch, naming itself, and
/// leaves the document as it was.
pub fn apply_ops(document: &mut Value, ops: &[Op]) -> Result<(), String> {
    let mut patched = document.clone();
    for op in ops {
        apply_op(&mut patched, op).map_err(|why| format!("{}: {why}", compact(&op.value())))?;
    }
    *document = patched;
    Ok(())
}

// Custom properties.

type Span = std::ops::Range<usize>;

/// A rule of a style sheet: its selector (an at-rule's starts with `@`) and the bytes of its
/// body, between the braces.
struct Block<'a> {
    selector: &'a str,
    body: Span,
}

/// One thing a rule body, or the sheet itself, holds.
enum Item<'a> {
    /// `<name>: <value>;`. `value` is the bytes of the value, without the white space and the
    /// comments around it; `closed` says whether a semicolon ends the declaration.
    Declaration {
        name: &'a str,
        value: Span,
        closed: bool,
    },
    /// A rule, with what it holds unread.
    Rule(Block<'a>),
}

/// Where the comment that starts at `at` ends, if one starts there.
fn comment(css: &str, at: usize, end: usize) -> Result<Option<usize>, String> {
    if !css.as_bytes()[at..end].starts_with(b"/*") {
        return Ok(None);
    }
    css[at + 2..end]
        .find("*/")
        .map(|close| Some(at + close + 4))
        .ok_or_else(|| "the style sheet ends inside a comment".to_owned())
}

/// Where the string that starts with the quote at `at` ends.
fn quoted(css: &str, at: usize, end: usize) -> Result<usize, String> {
    let bytes = css.as_bytes();
    let mut next = at + 1;
    while next < end {
        match bytes[next] {
            b'\\' => next += 1,
            b'\n' => break,
            quote if quote == bytes[at] => return Ok(next + 1),
            _ => {}
        }
        next += 1;
    }
    Err("the style sheet has a string that does not end on its line".to_owned())
}

/// Where the brace that closes the one at `open` is; comments and strings are skipped.
fn closing(css: &str, open: usize, end: usize) -> Result<usize, String> {
    let bytes = css.as_bytes();
    let (mut at, mut depth) = (open, 0usize);
    while at < end {
        if let Some(after) = comment(css, at, end)? {
            at = after;
            continue;
        }
        match bytes[at] {
            b'"' | b'\'' => {
                at = quoted(css, at, end)?;
                continue;
            }
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(at);
                }
            }
            _ => {}
        }
        at += 1;
    }
    Err("the style sheet ends inside a rule".to_owned())
}

/// The declarations and the rules the bytes `within` hold directly, in the order written: the
/// whole sheet, or the body of one rule. A comment and a string are read as such, so a brace,
/// a colon or a semicolon inside one is text, as is one between parentheses or brackets. What
/// a rule inside holds is not read.
fn items(css: &str, within: Span) -> Result<Vec<Item<'_>>, String> {
    let bytes = css.as_bytes();
    let mut found = Vec::new();
    let (mut at, mut start, mut round) = (within.start, within.start, 0usize);
    let (mut colon, mut value) = (None::<usize>, None::<Span>);
    loop {
        let ended = at >= within.end;
        if ended || (round == 0 && bytes[at] == b';') {
            if let Some(colon) = colon.take() {
                found.push(Item::Declaration {
                    name: css[start..colon].trim(),
                    value: value.take().unwrap_or(colon + 1..colon + 1),
                    closed: !ended,
                });
            }
            if ended {
                break;
            }
            at += 1;
            start = at;
            continue;
        }
        if let Some(after) = comment(css, at, within.end)? {
            if css[start..at].trim().is_empty() {
                start = after;
            }
            at = after;
            continue;
        }
        let after = match bytes[at] {
            b'"' | b'\'' => quoted(css, at, within.end)?,
            b'(' | b'[' => {
                round += 1;
                at + 1
            }
            b')' | b']' => {
                round = round
                    .checked_sub(1)
                    .ok_or("the style sheet closes a parenthesis it did not open")?;
                at + 1
            }
            b'{' if round == 0 => {
                let close = closing(css, at, within.end)?;
                found.push(Item::Rule(Block {
                    selector: css[start..at].trim(),
                    body: at + 1..close,
                }));
                at = close + 1;
                start = at;
                (colon, value) = (None, None);
                continue;
            }
            b'}' if round == 0 => {
                return Err("the style sheet closes a brace it did not open".to_owned());
            }
            b':' if round == 0 && colon.is_none() => {
                colon = Some(at);
                at += 1;
                continue;
            }
            _ => at + 1,
        };
        let blank = after == at + 1 && bytes[at].is_ascii_whitespace();
        if colon.is_some() && !blank {
            value = Some(value.map_or(at, |so_far| so_far.start)..after);
        }
        at = after;
    }
    if round > 0 {
        return Err("the style sheet ends inside a parenthesis".to_owned());
    }
    Ok(found)
}

/// The top-level rules of a style sheet with exactly this selector.
fn top_rules<'a>(css: &'a str, selector: &str) -> Result<Vec<Block<'a>>, String> {
    Ok(items(css, 0..css.len())?
        .into_iter()
        .filter_map(|item| match item {
            Item::Rule(block) if block.selector == selector => Some(block),
            _ => None,
        })
        .collect())
}

/// The values of `property` among the declarations the rule body at `body` holds directly: a
/// declaration of a rule inside it is another rule's.
fn values(css: &str, body: &Span, property: &str) -> Result<Vec<Span>, String> {
    Ok(items(css, body.clone())?
        .into_iter()
        .filter_map(|item| match item {
            Item::Declaration { name, value, .. } if name == property => Some(value),
            _ => None,
        })
        .collect())
}

/// How often the bytes `within` declare `property` for exactly this selector, at any depth: in
/// a rule with the selector, and in an at-rule or a `&` rule inside one. `inside` says whether
/// `within` is the body of such a rule.
fn declared(
    css: &str,
    within: Span,
    inside: bool,
    selector: &str,
    property: &str,
) -> Result<usize, String> {
    let mut count = 0;
    for item in items(css, within)? {
        match item {
            Item::Declaration { name, .. } => count += usize::from(inside && name == property),
            Item::Rule(block) => {
                let same = block.selector == selector
                    || (inside && (block.selector.starts_with('@') || block.selector == "&"));
                count += declared(css, block.body, same, selector, property)?;
            }
        }
    }
    Ok(count)
}

/// The value of a custom property in the top-level rule with exactly this selector.
pub fn custom_property(css: &str, selector: &str, property: &str) -> Option<String> {
    top_rules(css, selector)
        .ok()?
        .iter()
        .find_map(|block| values(css, &block.body, property).ok()?.into_iter().next())
        .map(|value| css[value].to_owned())
}

fn plain(what: &str, text: &str) -> Result<(), String> {
    if text.trim().is_empty() || text.trim() != text {
        return Err(format!(
            "{what} `{text}` is empty or starts or ends with white space"
        ));
    }
    if text.contains(['{', '}', ';', '\n', '\r']) || text.contains("/*") {
        return Err(format!(
            "{what} `{text}` holds a brace, a semicolon, a line break or a comment"
        ));
    }
    Ok(())
}

/// The style sheet with `property` of the rule with this selector set to `value`.
fn set_custom_property(
    css: &mut String,
    selector: &str,
    property: &str,
    value: &str,
) -> Result<(), String> {
    let rules = top_rules(css, selector)?;
    let mut own = Vec::new();
    for block in &rules {
        own.extend(values(css, &block.body, property)?);
    }
    let all = declared(css, 0..css.len(), false, selector, property)?;
    if all > 1 {
        return Err(format!(
            "the style sheet declares `{property}` for `{selector}` {all} times: a value set in one place would leave the others as they are"
        ));
    }
    if all > own.len() {
        return Err(format!(
            "the style sheet declares `{property}` for `{selector}` inside another rule, where it is not set"
        ));
    }
    if let Some(held) = own.pop() {
        css.replace_range(held, value);
        return Ok(());
    }
    let Some(body) = rules.into_iter().next().map(|block| block.body) else {
        if !css.is_empty() && !css.ends_with('\n') {
            css.push('\n');
        }
        css.push_str(&format!("\n{selector} {{\n  {property}: {value};\n}}\n"));
        return Ok(());
    };
    // The last declaration of the body gets the semicolon it lacks, then the new one follows
    // what the body holds.
    let open = match items(css, body.clone())?.last() {
        Some(Item::Declaration {
            value,
            closed: false,
            ..
        }) => Some(value.end),
        _ => None,
    };
    let end = body.start + css[body].trim_end().len();
    css.insert_str(end, &format!("\n  {property}: {value};"));
    if let Some(open) = open {
        css.insert(open, ';');
    }
    Ok(())
}

/// The style sheet with each custom property set: a value written in place of the one the rule
/// has, a property the rule lacks added as its last declaration, a rule the sheet lacks added
/// at its end. Nothing else changes, so the values the sheet already has give its bytes back.
/// Comments, strings and the rules inside a rule are not read as declarations. A property the
/// sheet declares for the selector more than once, or only inside another rule (a second rule
/// with the selector, one inside `@media`), is refused: setting one place would leave the
/// other value in force.
pub fn set_custom_properties(css: &str, rules: &[CssRule]) -> Result<String, String> {
    let mut css = css.to_owned();
    for rule in rules {
        plain("the selector", &rule.selector)?;
        for (property, value) in &rule.values {
            plain("the property", property)?;
            plain("the value", value)?;
            if property.len() < 3 || !property.starts_with("--") || property.contains(':') {
                return Err(format!("`{property}` is not a custom property"));
            }
            set_custom_property(&mut css, &rule.selector, property, value)?;
        }
    }
    Ok(css)
}
