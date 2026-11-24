//! A reader for the YAML subset ESS documents use: block maps and lists, flow maps and lists,
//! plain, single- and double-quoted scalars, `|` and `>` block scalars, comments. No anchors, tags
//! or multi-document streams. ESS hands no resolved form over, so an own generator reads the YAML.

use std::fmt;

/// A YAML value. Maps keep their key order.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(Vec<Value>),
    Map(Vec<(String, Value)>),
}

impl Value {
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Map(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
    pub fn str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(Value::str)
    }
    pub fn list(&self) -> &[Value] {
        match self {
            Value::List(items) => items,
            _ => &[],
        }
    }
    pub fn entries(&self) -> &[(String, Value)] {
        match self {
            Value::Map(entries) => entries,
            _ => &[],
        }
    }
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
    /// The value as text the way a scalar prints, for keys and labels.
    pub fn scalar_text(&self) -> Option<String> {
        match self {
            Value::Str(s) => Some(s.clone()),
            Value::Int(n) => Some(n.to_string()),
            Value::Float(f) => Some(f.to_string()),
            Value::Bool(b) => Some(b.to_string()),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Error {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

fn err<T>(line: usize, message: impl Into<String>) -> Result<T, Error> {
    Err(Error { line, message: message.into() })
}

#[derive(Debug, Clone)]
struct Line {
    no: usize,
    indent: usize,
    text: String,
    raw: String,
}

/// Parses one YAML document.
pub fn parse(source: &str) -> Result<Value, Error> {
    let mut lines = Vec::new();
    for (index, raw) in source.lines().enumerate() {
        let indent = raw.len() - raw.trim_start_matches(' ').len();
        let text = strip_comment(&raw[indent..]).trim_end().to_string();
        lines.push(Line { no: index + 1, indent, text, raw: raw.to_string() });
    }
    let mut p = Parser { lines, at: 0 };
    p.skip_blank();
    if p.at < p.lines.len() && p.lines[p.at].text == "---" {
        p.at += 1;
        p.skip_blank();
    }
    if p.at >= p.lines.len() {
        return Ok(Value::Null);
    }
    let indent = p.lines[p.at].indent;
    p.block(indent)
}

/// Drops a `#` comment that is outside quotes and starts the line or follows a space.
fn strip_comment(text: &str) -> &str {
    let bytes = text.as_bytes();
    let (mut single, mut double) = (false, false);
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        match c {
            b'\\' if double => i += 1,
            b'"' if !single => double = !double,
            b'\'' if !double => single = !single,
            b'#' if !single && !double && (i == 0 || bytes[i - 1] == b' ') => return &text[..i],
            _ => {}
        }
        i += 1;
    }
    text
}

struct Parser {
    lines: Vec<Line>,
    at: usize,
}

impl Parser {
    fn skip_blank(&mut self) {
        while self.at < self.lines.len() && self.lines[self.at].text.is_empty() {
            self.at += 1;
        }
    }

    fn peek(&mut self) -> Option<&Line> {
        self.skip_blank();
        self.lines.get(self.at)
    }

    fn block(&mut self, indent: usize) -> Result<Value, Error> {
        let line = self.peek().cloned().expect("a line");
        if is_item(&line.text) {
            self.sequence(indent)
        } else if split_key(&line.text).is_some() {
            self.mapping(indent)
        } else {
            self.at += 1;
            let text = self.continued_flow(line.text.clone(), line.no)?;
            scalar_or_flow(&text, line.no)
        }
    }

    fn sequence(&mut self, indent: usize) -> Result<Value, Error> {
        let mut items = Vec::new();
        while let Some(line) = self.peek().cloned() {
            if line.indent != indent || !is_item(&line.text) {
                if line.indent > indent {
                    return err(line.no, "unexpected indentation in a list");
                }
                break;
            }
            let rest = line.text[1..].trim_start();
            if rest.is_empty() {
                self.at += 1;
                match self.peek().cloned() {
                    Some(next) if next.indent > indent => items.push(self.block(next.indent)?),
                    _ => items.push(Value::Null),
                }
            } else {
                // The item's content is read as a block that starts at its own column.
                let offset = line.text.len() - rest.len();
                let entry = &mut self.lines[self.at];
                entry.indent = indent + offset;
                entry.text = rest.to_string();
                items.push(self.block(indent + offset)?);
            }
        }
        Ok(Value::List(items))
    }

    fn mapping(&mut self, indent: usize) -> Result<Value, Error> {
        let mut entries: Vec<(String, Value)> = Vec::new();
        while let Some(line) = self.peek().cloned() {
            if line.indent != indent || is_item(&line.text) {
                if line.indent > indent {
                    return err(line.no, "unexpected indentation in a map");
                }
                break;
            }
            let Some((key, rest)) = split_key(&line.text) else {
                return err(line.no, format!("expected `key: value`, found `{}`", line.text));
            };
            let key = unquote_key(key, line.no)?;
            self.at += 1;
            let rest = rest.trim();
            let value = if rest.is_empty() {
                match self.peek().cloned() {
                    Some(next) if next.indent > indent => self.block(next.indent)?,
                    Some(next) if next.indent == indent && is_item(&next.text) => self.sequence(indent)?,
                    _ => Value::Null,
                }
            } else if rest.starts_with('|') || rest.starts_with('>') {
                self.block_scalar(rest, indent)?
            } else {
                let text = self.continued_flow(rest.to_string(), line.no)?;
                scalar_or_flow(&text, line.no)?
            };
            if entries.iter().any(|(k, _)| *k == key) {
                return err(line.no, format!("duplicate key `{key}`"));
            }
            entries.push((key, value));
        }
        Ok(Value::Map(entries))
    }

    /// A flow collection left open at the end of a line continues on the next lines.
    fn continued_flow(&mut self, mut text: String, no: usize) -> Result<String, Error> {
        if !(text.starts_with('{') || text.starts_with('[')) {
            return Ok(text);
        }
        while depth(&text) > 0 {
            let Some(next) = self.peek().cloned() else {
                return err(no, "unclosed flow collection");
            };
            self.at += 1;
            text.push(' ');
            text.push_str(next.text.trim());
        }
        Ok(text)
    }

    fn block_scalar(&mut self, header: &str, indent: usize) -> Result<Value, Error> {
        let folded = header.starts_with('>');
        let chomp = header[1..].trim();
        let mut body: Vec<String> = Vec::new();
        let mut inner: Option<usize> = None;
        while self.at < self.lines.len() {
            let line = &self.lines[self.at];
            let blank = line.raw.trim().is_empty();
            if !blank && line.indent <= indent {
                break;
            }
            if blank {
                body.push(String::new());
            } else {
                let width = *inner.get_or_insert(line.indent);
                body.push(line.raw.get(width..).unwrap_or("").to_string());
            }
            self.at += 1;
        }
        while body.last().is_some_and(String::is_empty) {
            body.pop();
        }
        let mut text = if folded {
            let mut out = String::new();
            for (i, part) in body.iter().enumerate() {
                if i > 0 {
                    out.push(if part.is_empty() || body[i - 1].is_empty() { '\n' } else { ' ' });
                }
                out.push_str(part);
            }
            out
        } else {
            body.join("\n")
        };
        if chomp != "-" {
            text.push('\n');
        }
        Ok(Value::Str(text))
    }
}

fn is_item(text: &str) -> bool {
    text == "-" || text.starts_with("- ")
}

/// The open bracket depth of a flow text, outside quotes.
fn depth(text: &str) -> i32 {
    let (mut d, mut single, mut double, mut escape) = (0, false, false, false);
    for c in text.chars() {
        if escape {
            escape = false;
            continue;
        }
        match c {
            '\\' if double => escape = true,
            '"' if !single => double = !double,
            '\'' if !double => single = !single,
            '{' | '[' if !single && !double => d += 1,
            '}' | ']' if !single && !double => d -= 1,
            _ => {}
        }
    }
    d
}

/// Splits `key: rest` at the first `: ` (or a trailing `:`) outside quotes and brackets.
fn split_key(text: &str) -> Option<(&str, &str)> {
    if text.starts_with('{') || text.starts_with('[') {
        return None;
    }
    let bytes = text.as_bytes();
    let (mut single, mut double) = (false, false);
    for i in 0..bytes.len() {
        match bytes[i] {
            b'"' if !single => double = !double,
            b'\'' if !double => single = !single,
            b':' if !single && !double && (i + 1 == bytes.len() || bytes[i + 1] == b' ') => {
                return Some((&text[..i], &text[i + 1..]));
            }
            _ => {}
        }
    }
    None
}

fn unquote_key(key: &str, no: usize) -> Result<String, Error> {
    let key = key.trim();
    if key.starts_with('"') || key.starts_with('\'') {
        let mut flow = Flow { chars: key.chars().collect(), at: 0, no };
        match flow.scalar()? {
            Value::Str(s) => Ok(s),
            other => Ok(other.scalar_text().unwrap_or_default()),
        }
    } else {
        Ok(key.to_string())
    }
}

fn scalar_or_flow(text: &str, no: usize) -> Result<Value, Error> {
    let mut flow = Flow { chars: text.chars().collect(), at: 0, no };
    let value = flow.value(false)?;
    flow.space();
    if flow.at < flow.chars.len() {
        return err(no, format!("unexpected `{}` after a value", flow.rest()));
    }
    Ok(value)
}

struct Flow {
    chars: Vec<char>,
    at: usize,
    no: usize,
}

impl Flow {
    fn rest(&self) -> String {
        self.chars[self.at..].iter().collect()
    }
    fn space(&mut self) {
        while self.chars.get(self.at).is_some_and(|c| c.is_whitespace()) {
            self.at += 1;
        }
    }
    fn eat(&mut self, c: char) -> bool {
        self.space();
        if self.chars.get(self.at) == Some(&c) {
            self.at += 1;
            true
        } else {
            false
        }
    }

    fn value(&mut self, in_flow: bool) -> Result<Value, Error> {
        self.space();
        match self.chars.get(self.at) {
            Some('{') => self.map(),
            Some('[') => self.list(),
            Some('"') | Some('\'') => self.scalar(),
            _ => Ok(plain(&self.plain_text(in_flow))),
        }
    }

    fn plain_text(&mut self, in_flow: bool) -> String {
        let start = self.at;
        while let Some(&c) = self.chars.get(self.at) {
            if in_flow && (c == ',' || c == '}' || c == ']') {
                break;
            }
            if in_flow && c == ':' && self.chars.get(self.at + 1).is_none_or(|n| n.is_whitespace()) {
                break;
            }
            self.at += 1;
        }
        self.chars[start..self.at].iter().collect::<String>().trim().to_string()
    }

    fn scalar(&mut self) -> Result<Value, Error> {
        let quote = self.chars[self.at];
        self.at += 1;
        let mut out = String::new();
        loop {
            let Some(&c) = self.chars.get(self.at) else {
                return err(self.no, "unclosed quote");
            };
            self.at += 1;
            if c == quote {
                if quote == '\'' && self.chars.get(self.at) == Some(&'\'') {
                    out.push('\'');
                    self.at += 1;
                    continue;
                }
                return Ok(Value::Str(out));
            }
            if c == '\\' && quote == '"' {
                let Some(&e) = self.chars.get(self.at) else {
                    return err(self.no, "unfinished escape");
                };
                self.at += 1;
                match e {
                    'n' => out.push('\n'),
                    't' => out.push('\t'),
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    '/' => out.push('/'),
                    'u' => {
                        let hex: String = self.chars.get(self.at..self.at + 4).unwrap_or(&[]).iter().collect();
                        self.at += 4;
                        let code = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32);
                        match code {
                            Some(ch) => out.push(ch),
                            None => return err(self.no, format!("bad escape \\u{hex}")),
                        }
                    }
                    other => return err(self.no, format!("unknown escape \\{other}")),
                }
                continue;
            }
            out.push(c);
        }
    }

    fn map(&mut self) -> Result<Value, Error> {
        self.at += 1;
        let mut entries: Vec<(String, Value)> = Vec::new();
        if self.eat('}') {
            return Ok(Value::Map(entries));
        }
        loop {
            self.space();
            let key = match self.chars.get(self.at) {
                Some('"') | Some('\'') => self.scalar()?.scalar_text().unwrap_or_default(),
                _ => self.plain_text(true),
            };
            let value = if self.eat(':') {
                self.space();
                match self.chars.get(self.at) {
                    Some(',') | Some('}') => Value::Null,
                    _ => self.value(true)?,
                }
            } else {
                Value::Null
            };
            if entries.iter().any(|(k, _)| *k == key) {
                return err(self.no, format!("duplicate key `{key}`"));
            }
            entries.push((key, value));
            if self.eat(',') {
                continue;
            }
            if self.eat('}') {
                return Ok(Value::Map(entries));
            }
            return err(self.no, format!("expected `,` or `}}` at `{}`", self.rest()));
        }
    }

    fn list(&mut self) -> Result<Value, Error> {
        self.at += 1;
        let mut items = Vec::new();
        if self.eat(']') {
            return Ok(Value::List(items));
        }
        loop {
            let start = self.at;
            let item = self.value(true)?;
            // A `key: value` pair inside a flow list is a one-entry map.
            let item = if self.eat(':') {
                let key = item.scalar_text().unwrap_or_default();
                let value = self.value(true)?;
                Value::Map(vec![(key, value)])
            } else {
                item
            };
            if self.at == start {
                return err(self.no, "empty flow list entry");
            }
            items.push(item);
            if self.eat(',') {
                continue;
            }
            if self.eat(']') {
                return Ok(Value::List(items));
            }
            return err(self.no, format!("expected `,` or `]` at `{}`", self.rest()));
        }
    }
}

/// A plain scalar: null, a boolean, a number or text (YAML 1.2 core schema).
fn plain(text: &str) -> Value {
    match text {
        "" | "~" | "null" | "Null" | "NULL" => return Value::Null,
        "true" | "True" | "TRUE" => return Value::Bool(true),
        "false" | "False" | "FALSE" => return Value::Bool(false),
        _ => {}
    }
    let numeric = text.chars().all(|c| c.is_ascii_digit() || "+-.eE".contains(c))
        && text.chars().any(|c| c.is_ascii_digit());
    if numeric {
        if let Ok(n) = text.parse::<i64>() {
            return Value::Int(n);
        }
        if let Ok(f) = text.parse::<f64>() {
            return Value::Float(f);
        }
    }
    Value::Str(text.to_string())
}
