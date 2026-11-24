//! The render record `ux-browser-render/1`: for one URL at one viewport and mode, each declared node's
//! box, computed styles and accessibility-tree entry, and a screenshot, naming the adapter and its
//! version. What the browser returns is read here; driving it is `session.rs`.

use crate::value::{f64_of, get_f64, get_i64, map, num, s};
use uxui_doc::yaml::{self, Value};

pub const FORMAT: &str = "ux-browser-render/1";
/// The adapter kind of the specification (`uxsearch.design.AdapterKind`).
pub const ADAPTER: &str = "browser";
pub const TOOL: &str = "ux-browser";
/// The colour schemes a mode can name; the browser emulates `prefers-color-scheme` with it.
pub const MODES: &[&str] = &["light", "dark"];
/// The settle time when the declarations name none: the settle time of the first consumer's page checks.
pub const DEFAULT_SETTLE_MS: u64 = 250;

#[derive(Clone, Debug)]
pub struct Viewport {
    pub width: i64,
    pub height: i64,
}

pub struct NodeDecl {
    pub name: String,
    pub selector: String,
}

pub struct Declarations {
    pub viewport: Viewport,
    pub mode: String,
    pub styles: Vec<String>,
    pub nodes: Vec<NodeDecl>,
    /// The wait after fonts are loaded and two frames drawn, before anything is read.
    pub settle_ms: u64,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AxEntry {
    pub role: String,
    pub name: String,
    pub ignored: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn from_value(v: &Value) -> Option<Rect> {
        Some(Rect {
            x: get_f64(v, "x")?,
            y: get_f64(v, "y")?,
            width: get_f64(v, "width")?,
            height: get_f64(v, "height")?,
        })
    }
    pub fn value(&self) -> Value {
        map(vec![
            ("x", num(self.x)),
            ("y", num(self.y)),
            ("width", num(self.width)),
            ("height", num(self.height)),
        ])
    }
}

pub struct NodeRender {
    pub name: String,
    pub selector: String,
    /// The elements the selector matched; the first is the one recorded.
    pub matches: i64,
    pub rect: Option<Rect>,
    pub styles: Vec<(String, String)>,
    pub ax: Option<AxEntry>,
}

/// A file the run read: its path when it lies under the root, and the digest of its bytes.
pub struct FileRef {
    pub path: Option<String>,
    pub digest: String,
}

pub struct Shot {
    pub file: String,
    pub digest: String,
    pub width: u32,
    pub height: u32,
}

pub struct Record<'a> {
    pub url: String,
    pub declarations: &'a Declarations,
    pub declarations_file: FileRef,
    /// The browser's product string (`Browser.getVersion`).
    pub browser: String,
    pub nodes: Vec<NodeRender>,
    pub screenshot: Option<Shot>,
}

pub fn viewport(v: &Value, what: &str) -> Result<Viewport, String> {
    let (Some(width), Some(height)) = (get_i64(v, "width"), get_i64(v, "height")) else {
        return Err(format!("{what}: `width` and `height` are whole numbers"));
    };
    if width <= 0 || height <= 0 {
        return Err(format!("{what}: `width` and `height` are above 0"));
    }
    Ok(Viewport { width, height })
}

pub fn declarations(text: &str) -> Result<Declarations, String> {
    let v = yaml::parse(text).map_err(|e| e.to_string())?;
    if v.get_str("format") != Some(FORMAT) {
        return Err(format!("`format` is not `{FORMAT}`"));
    }
    for (key, _) in v.entries() {
        if !["format", "viewport", "mode", "styles", "nodes", "settle_ms"].contains(&key.as_str()) {
            return Err(format!("unknown key `{key}`"));
        }
    }
    let viewport = viewport(v.get("viewport").ok_or("no `viewport`")?, "viewport")?;
    let mode = v.get_str("mode").ok_or("no `mode`")?.to_string();
    if !MODES.contains(&mode.as_str()) {
        return Err(format!("mode `{mode}` is not one of {MODES:?}"));
    }
    let styles = crate::value::get_strs(&v, "styles")?;
    let mut nodes = Vec::new();
    for (i, n) in v
        .get("nodes")
        .map(Value::list)
        .unwrap_or_default()
        .iter()
        .enumerate()
    {
        let (Some(name), Some(selector)) = (n.get_str("name"), n.get_str("selector")) else {
            return Err(format!("nodes[{i}]: `name` and `selector` are required"));
        };
        if nodes.iter().any(|d: &NodeDecl| d.name == name) {
            return Err(format!("nodes[{i}]: `{name}` is declared twice"));
        }
        nodes.push(NodeDecl {
            name: name.to_string(),
            selector: selector.to_string(),
        });
    }
    if nodes.is_empty() {
        return Err("`nodes` declares no node".to_string());
    }
    let settle_ms = match v.get("settle_ms") {
        None => DEFAULT_SETTLE_MS,
        Some(Value::Int(n)) if *n >= 0 => *n as u64,
        Some(_) => return Err("`settle_ms` is a whole number of milliseconds".to_string()),
    };
    Ok(Declarations {
        viewport,
        mode,
        styles,
        nodes,
        settle_ms,
    })
}

/// The entry of the node itself in an `Accessibility.getPartialAXTree` result fetched without
/// relatives: its first node.
pub fn ax_entry(partial: &Value) -> Option<AxEntry> {
    let node = partial.get("nodes")?.list().first()?;
    let text = |key: &str| {
        node.get(key)
            .and_then(|p| p.get("value"))
            .and_then(Value::scalar_text)
            .unwrap_or_default()
    };
    Some(AxEntry {
        role: text("role"),
        name: text("name"),
        ignored: matches!(node.get("ignored"), Some(Value::Bool(true))),
    })
}

/// Width and height from a PNG's IHDR chunk.
pub fn png_size(bytes: &[u8]) -> Option<(u32, u32)> {
    const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    if bytes.get(..8)? != SIGNATURE || bytes.get(12..16)? != b"IHDR" {
        return None;
    }
    let word = |at: usize| -> Option<u32> {
        let b = bytes.get(at..at + 4)?;
        Some(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    };
    Some((word(16)?, word(20)?))
}

/// One node from the in-page probe (`probes/node.js`): `{matches, box, styles}`.
pub fn node_render(
    decl: &NodeDecl,
    styles_declared: &[String],
    probe: &Value,
    ax: Option<AxEntry>,
) -> Result<NodeRender, String> {
    let matches = get_i64(probe, "matches")
        .ok_or_else(|| format!("{}: probe without `matches`", decl.name))?;
    if matches == 0 {
        return Ok(NodeRender {
            name: decl.name.clone(),
            selector: decl.selector.clone(),
            matches,
            rect: None,
            styles: Vec::new(),
            ax: None,
        });
    }
    let rect = probe
        .get("box")
        .and_then(Rect::from_value)
        .ok_or_else(|| format!("{}: probe without a box", decl.name))?;
    let styles = probe
        .get("styles")
        .map(|m| {
            m.entries()
                .iter()
                .map(|(k, v)| (k.clone(), v.scalar_text().unwrap_or_default()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for style in styles_declared {
        if !styles.iter().any(|(k, _)| k == style) {
            return Err(format!("{}: no computed value for `{style}`", decl.name));
        }
    }
    Ok(NodeRender {
        name: decl.name.clone(),
        selector: decl.selector.clone(),
        matches,
        rect: Some(rect),
        styles,
        ax,
    })
}

pub fn record_value(record: &Record) -> Value {
    let d = record.declarations;
    let nodes = record
        .nodes
        .iter()
        .map(|n| {
            let mut entries = vec![
                ("name", s(&n.name)),
                ("selector", s(&n.selector)),
                ("matches", Value::Int(n.matches)),
            ];
            if let Some(rect) = &n.rect {
                entries.push(("box", rect.value()));
                entries.push((
                    "styles",
                    Value::Map(n.styles.iter().map(|(k, v)| (k.clone(), s(v))).collect()),
                ));
            }
            if let Some(ax) = &n.ax {
                entries.push((
                    "accessibility",
                    map(vec![
                        ("role", s(&ax.role)),
                        ("name", s(&ax.name)),
                        ("ignored", Value::Bool(ax.ignored)),
                    ]),
                ));
            }
            map(entries)
        })
        .collect();
    let mut file = Vec::new();
    if let Some(path) = &record.declarations_file.path {
        file.push(("path", s(path)));
    }
    file.push(("digest", s(&record.declarations_file.digest)));
    file.push(("digest_form", s("bytes")));
    let mut entries = vec![
        ("format", s(FORMAT)),
        (
            "adapter",
            map(vec![
                ("name", s(ADAPTER)),
                ("tool", s(TOOL)),
                ("version", s(env!("CARGO_PKG_VERSION"))),
            ]),
        ),
        ("browser", s(&record.browser)),
        ("url", s(&record.url)),
        (
            "viewport",
            map(vec![
                ("width", Value::Int(d.viewport.width)),
                ("height", Value::Int(d.viewport.height)),
            ]),
        ),
        ("mode", s(&d.mode)),
        ("declarations", map(file)),
        ("nodes", Value::List(nodes)),
    ];
    if let Some(shot) = &record.screenshot {
        entries.push((
            "screenshot",
            map(vec![
                ("file", s(&shot.file)),
                ("digest", s(&shot.digest)),
                ("width", Value::Int(i64::from(shot.width))),
                ("height", Value::Int(i64::from(shot.height))),
            ]),
        ));
    }
    map(entries)
}

/// The record as JSON: one node per line, a newline at the end.
pub fn record_text(record: &Value) -> String {
    let mut out = String::from("{");
    for (i, (key, item)) in record.entries().iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&uxui_doc::json::string(key));
        out.push(':');
        match item {
            Value::List(nodes) if key == "nodes" && !nodes.is_empty() => {
                out.push('[');
                for (j, node) in nodes.iter().enumerate() {
                    out.push_str(if j > 0 { ",\n" } else { "\n" });
                    out.push_str(&uxui_doc::json::write(node));
                }
                out.push_str("\n]");
            }
            _ => out.push_str(&uxui_doc::json::write(item)),
        }
    }
    out.push_str("}\n");
    out
}

/// Reads a number that the browser may send as an integer or a float.
pub fn number(v: &Value) -> Option<f64> {
    f64_of(v)
}
