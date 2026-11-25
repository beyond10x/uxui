//! The five page checks the UI-loop copies share (`research/2026-11-24-loop-copies-page-checks.md`),
//! with the first consumer's definitions, computed from what the in-page probes return. The browser
//! run (`measure.rs`) records that raw geometry per page; [`page_result`] turns it into values. The
//! viewports, modes, settle times and declared items are inputs: a `ux-browser-checks/1` file.

use crate::render::{Rect, Viewport};
use crate::value::{get_bool, get_f64, get_i64, get_strs};
use uxui_doc::yaml::{self, Value};

pub const FORMAT: &str = "ux-browser-checks/1";

pub const PAGE_SCROLL: &str = "layout.page_scroll@1";
pub const OVERFLOW: &str = "layout.overflow@1";
pub const ALWAYS_VISIBLE: &str = "always_visible.misses@1";
pub const KEYBOARD: &str = "keyboard.misses@1";
pub const LAYOUT_SHIFT: &str = "layout_shift.after_ready@1";
pub const LAYOUT_SHIFT_LOAD: &str = "layout_shift.load@1";

/// The slack, in CSS pixels, an edge may pass its limit by before it overflows.
const OVERFLOW_SLACK: f64 = 1.0;
/// The slack, in CSS pixels, an always-visible item may pass the viewport by.
const VIEWPORT_SLACK: f64 = 0.5;

// ---------------------------------------------------------------- page scroll and overflow

/// Where an item sits: in the flow (0), under `position: fixed` (1), in a horizontal scroll
/// container (2), or under `overflow-x: hidden | clip` (3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Context {
    Flow,
    Fixed,
    Scroll,
    Clip,
}

pub struct Item {
    pub right: f64,
    pub width: f64,
    pub height: f64,
    pub context: Context,
    /// Index into `blocks`: the nearest ancestor (or self) whose display is not inline.
    pub block: Option<usize>,
}

pub struct Block {
    pub desc: String,
    pub right: f64,
}

/// `probes/overflow.js`: the viewport's client width, the page's scroll width, and every text run
/// and every `img, svg, video, canvas, iframe, input, select, textarea, table, pre` with its block.
pub struct OverflowRaw {
    pub vw: f64,
    pub scroll_width: f64,
    pub blocks: Vec<Block>,
    pub items: Vec<Item>,
}

fn num_at(list: &[Value], i: usize) -> Option<f64> {
    list.get(i).and_then(crate::value::f64_of)
}

impl OverflowRaw {
    pub fn from_value(v: &Value) -> Result<Self, String> {
        let vw = get_f64(v, "vw").ok_or("overflow: no `vw`")?;
        let scroll_width = get_f64(v, "scroll_width").ok_or("overflow: no `scroll_width`")?;
        let blocks = v
            .get("blocks")
            .map(Value::list)
            .unwrap_or_default()
            .iter()
            .map(|b| {
                let l = b.list();
                Some(Block {
                    desc: l.first()?.str()?.to_string(),
                    right: num_at(l, 1)?,
                })
            })
            .collect::<Option<Vec<_>>>()
            .ok_or("overflow: a block is not [desc, right]")?;
        let items = v
            .get("items")
            .map(Value::list)
            .unwrap_or_default()
            .iter()
            .map(|i| {
                let l = i.list();
                let context = match num_at(l, 3)? as i64 {
                    0 => Context::Flow,
                    1 => Context::Fixed,
                    2 => Context::Scroll,
                    3 => Context::Clip,
                    _ => return None,
                };
                let block = num_at(l, 4)?;
                Some(Item {
                    right: num_at(l, 0)?,
                    width: num_at(l, 1)?,
                    height: num_at(l, 2)?,
                    context,
                    block: (block >= 0.0).then_some(block as usize),
                })
            })
            .collect::<Option<Vec<_>>>()
            .ok_or("overflow: an item is not [right, width, height, context, block]")?;
        if items
            .iter()
            .any(|i| i.block.is_some_and(|b| b >= blocks.len()))
        {
            return Err("overflow: an item names a block that is not listed".to_string());
        }
        Ok(OverflowRaw {
            vw,
            scroll_width,
            blocks,
            items,
        })
    }
}

/// How far the page scrolls sideways, in CSS pixels; a unit counts when it is above 0.
pub fn page_scroll(raw: &OverflowRaw) -> f64 {
    (raw.scroll_width - raw.vw).max(0.0)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Overflow {
    pub desc: String,
    pub right: i64,
    pub limit: i64,
    /// `viewport` or `parent`.
    pub against: &'static str,
}

/// JavaScript's `Math.round`: halves go up.
fn js_round(x: f64) -> i64 {
    (x + 0.5).floor() as i64
}

/// Items whose right edge passes the viewport or their block by more than the slack, at most one
/// per block; items in a fixed layer, a scroll container or a clipping box are not overflow.
pub fn overflow(raw: &OverflowRaw) -> Vec<Overflow> {
    let mut failed = vec![false; raw.blocks.len()];
    let mut out = Vec::new();
    for item in &raw.items {
        if item.width < 1.0 || item.height < 1.0 || item.context != Context::Flow {
            continue;
        }
        let Some(b) = item.block else { continue };
        let block = &raw.blocks[b];
        let (against, limit) = if item.right > raw.vw + OVERFLOW_SLACK {
            ("viewport", raw.vw)
        } else if item.right > block.right + OVERFLOW_SLACK {
            ("parent", block.right)
        } else {
            continue;
        };
        if failed[b] {
            continue;
        }
        failed[b] = true;
        out.push(Overflow {
            desc: block.desc.clone(),
            right: js_round(item.right),
            limit: js_round(limit),
            against,
        });
    }
    out
}

// ---------------------------------------------------------------- always visible

/// How an accessible name is matched: any name, the whole name (white space folded, case kept),
/// or a part of it in any case (one of several).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Name {
    Any,
    Is(String),
    Contains(Vec<String>),
}

fn fold(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

impl Name {
    pub fn matches(&self, name: &str) -> bool {
        match self {
            Name::Any => true,
            Name::Is(want) => fold(name) == fold(want),
            Name::Contains(parts) => {
                let name = name.to_lowercase();
                parts.iter().any(|p| name.contains(&p.to_lowercase()))
            }
        }
    }
}

/// The DOM nodes (backend ids) of the accessibility tree's unignored nodes with `role` and a
/// matching name, in tree order.
pub fn ax_candidates(tree: &Value, role: &str, name: &Name) -> Vec<i64> {
    let prop = |node: &Value, key: &str| {
        node.get(key)
            .and_then(|p| p.get("value"))
            .and_then(Value::scalar_text)
            .unwrap_or_default()
    };
    tree.get("nodes")
        .map(Value::list)
        .unwrap_or_default()
        .iter()
        .filter(|n| !matches!(n.get("ignored"), Some(Value::Bool(true))))
        .filter(|n| prop(n, "role") == role && name.matches(&prop(n, "name")))
        .filter_map(|n| get_i64(n, "backendDOMNodeId"))
        .collect()
}

/// One element that may be the item: `probes/candidate.js` on it.
#[derive(Clone, Debug)]
pub struct Candidate {
    pub in_scope: bool,
    pub visible: bool,
    pub rect: Rect,
    /// The element at the centre of its box is it or inside it.
    pub hit: bool,
}

impl Candidate {
    pub fn from_value(v: &Value) -> Result<Self, String> {
        let (Some(in_scope), Some(visible), Some(hit)) = (
            get_bool(v, "in_scope"),
            get_bool(v, "visible"),
            get_bool(v, "hit"),
        ) else {
            return Err("candidate: `in_scope`, `visible` and `hit` are required".to_string());
        };
        let rect = v
            .get("box")
            .and_then(Rect::from_value)
            .ok_or("candidate: no box")?;
        Ok(Candidate {
            in_scope,
            visible,
            rect,
            hit,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Ok,
    Missing,
    Clipped,
    Covered,
    NotApplicable,
}

impl Status {
    pub fn word(self) -> &'static str {
        match self {
            Status::Ok => "ok",
            Status::Missing => "missing",
            Status::Clipped => "clipped",
            Status::Covered => "covered",
            Status::NotApplicable => "n/a",
        }
    }
    pub fn is_miss(self) -> bool {
        matches!(self, Status::Missing | Status::Clipped | Status::Covered)
    }
}

/// The candidate that is the item: the first visible one in scope, role matches before CSS ones.
pub fn first_visible(candidates: &[Candidate]) -> Option<usize> {
    candidates.iter().position(|c| c.in_scope && c.visible)
}

pub fn visible_status(candidate: Option<&Candidate>, vw: f64, vh: f64) -> Status {
    let Some(c) = candidate else {
        return Status::Missing;
    };
    let r = &c.rect;
    if r.x < -VIEWPORT_SLACK
        || r.y < -VIEWPORT_SLACK
        || r.x + r.width > vw + VIEWPORT_SLACK
        || r.y + r.height > vh + VIEWPORT_SLACK
    {
        return Status::Clipped;
    }
    if c.hit { Status::Ok } else { Status::Covered }
}

/// How an item is found: by role and name, else by CSS, inside the first match of `scope`.
#[derive(Clone, Debug)]
pub struct Locate {
    pub role: String,
    pub name: Name,
    pub css: Option<String>,
    pub scope: Option<String>,
}

#[derive(Clone, Debug)]
pub struct AlwaysVisible {
    pub name: String,
    pub locate: Locate,
    pub min_width: Option<i64>,
    pub path_prefix: Option<String>,
}

pub fn applies(item: &AlwaysVisible, path: &str, width: i64) -> bool {
    item.min_width.is_none_or(|m| width >= m)
        && item
            .path_prefix
            .as_deref()
            .is_none_or(|p| path.starts_with(p))
}

// ---------------------------------------------------------------- keyboard

/// `measure.rs`'s keyboard walk: the label of the first Tab stop, whether Enter (or one more Tab)
/// put focus into the skip link's target, which targets were found, and per Tab the targets focus
/// was in.
pub struct KeyboardRaw {
    pub first_label: String,
    pub after_enter: bool,
    pub after_tab: Option<bool>,
    pub located: Vec<(String, bool)>,
    pub tabs: Vec<Vec<String>>,
}

impl KeyboardRaw {
    pub fn from_value(v: &Value) -> Result<Self, String> {
        let first_label = v
            .get_str("first_label")
            .ok_or("keyboard: no `first_label`")?
            .to_string();
        let after_enter = get_bool(v, "after_enter").ok_or("keyboard: no `after_enter`")?;
        let after_tab = get_bool(v, "after_tab");
        let located = v
            .get("located")
            .map(Value::entries)
            .unwrap_or_default()
            .iter()
            .map(|(k, b)| (k.clone(), matches!(b, Value::Bool(true))))
            .collect();
        let tabs = v
            .get("tabs")
            .map(Value::list)
            .unwrap_or_default()
            .iter()
            .map(|t| {
                t.list()
                    .iter()
                    .filter_map(|n| n.str().map(str::to_string))
                    .collect()
            })
            .collect();
        Ok(KeyboardRaw {
            first_label,
            after_enter,
            after_tab,
            located,
            tabs,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyboardResult {
    pub skip_link_first: bool,
    pub skip_moves_focus: bool,
    pub reached: Vec<(String, bool)>,
}

impl KeyboardResult {
    /// The skip link not first, its target not focused (also when it is not first), and each target
    /// not reached.
    pub fn misses(&self) -> i64 {
        i64::from(!self.skip_link_first)
            + i64::from(!self.skip_moves_focus)
            + self.reached.iter().filter(|(_, r)| !r).count() as i64
    }
}

pub fn keyboard(raw: &KeyboardRaw, skip_link: &Name, max_tabs: usize) -> KeyboardResult {
    let skip_link_first = skip_link.matches(&raw.first_label);
    let skip_moves_focus = skip_link_first && (raw.after_enter || raw.after_tab == Some(true));
    let reached = raw
        .located
        .iter()
        .map(|(name, located)| {
            let hit = *located
                && raw
                    .tabs
                    .iter()
                    .take(max_tabs)
                    .any(|t| t.iter().any(|n| n == name));
            (name.clone(), hit)
        })
        .collect();
    KeyboardResult {
        skip_link_first,
        skip_moves_focus,
        reached,
    }
}

// ---------------------------------------------------------------- layout shift

/// The `layout-shift` entries without recent input, `[startTime, value]`, and the ready time
/// (fonts loaded, two frames drawn). `entries` is `None` where the browser has no such observer.
pub struct Shifts {
    pub ready: f64,
    pub entries: Option<Vec<(f64, f64)>>,
}

impl Shifts {
    pub fn from_value(v: &Value) -> Result<Self, String> {
        let ready = get_f64(v, "ready").ok_or("shifts: no `ready`")?;
        let entries = match v.get("entries") {
            None | Some(Value::Null) => None,
            Some(Value::List(items)) => Some(
                items
                    .iter()
                    .map(|e| Some((num_at(e.list(), 0)?, num_at(e.list(), 1)?)))
                    .collect::<Option<Vec<_>>>()
                    .ok_or("shifts: an entry is not [time, value]")?,
            ),
            Some(_) => return Err("shifts: `entries` is not a list".to_string()),
        };
        Ok(Shifts { ready, entries })
    }
}

/// JavaScript's `Math.round(x * 10000) / 10000`.
pub fn round4(x: f64) -> f64 {
    js_round(x * 10000.0) as f64 / 10000.0
}

pub fn shift_after_ready(s: &Shifts) -> Option<f64> {
    let entries = s.entries.as_ref()?;
    Some(round4(
        entries
            .iter()
            .filter(|(t, _)| *t > s.ready)
            .map(|(_, v)| v)
            .sum(),
    ))
}

pub fn shift_load(s: &Shifts) -> Option<f64> {
    Some(round4(s.entries.as_ref()?.iter().map(|(_, v)| v).sum()))
}

// ---------------------------------------------------------------- pages

/// The URL paths of a sitemap's `<loc>` entries, sorted, each once.
pub fn sitemap_paths(xml: &str) -> Vec<String> {
    let mut out: Vec<String> = xml
        .split("<loc>")
        .skip(1)
        .filter_map(|part| part.split("</loc>").next())
        .map(|url| {
            let url = url.trim();
            match url.find("://") {
                Some(i) => {
                    let rest = &url[i + 3..];
                    rest.find('/')
                        .map_or("/".to_string(), |j| rest[j..].to_string())
                }
                None => url.to_string(),
            }
        })
        .collect();
    out.sort();
    out.dedup();
    out
}

// ---------------------------------------------------------------- configuration

#[derive(Clone, Debug)]
pub struct NamedViewport {
    pub name: String,
    pub width: i64,
    pub height: i64,
}

impl NamedViewport {
    pub fn viewport(&self) -> Viewport {
        Viewport {
            width: self.width,
            height: self.height,
        }
    }
}

pub struct KeyboardConfig {
    pub skip_link: Name,
    pub must_reach: Vec<String>,
    pub max_tabs: usize,
    /// The viewports (names) the walk runs at; `None`: every viewport.
    pub viewports: Option<Vec<String>>,
    pub modes: Vec<String>,
    /// The wait after Enter on the skip link.
    pub focus_settle_ms: u64,
}

pub enum Pages {
    Sitemap(String),
    Paths(Vec<String>),
}

pub struct Config {
    pub base_path: String,
    pub pages: Pages,
    pub viewports: Vec<NamedViewport>,
    pub modes: Vec<String>,
    /// The attribute of `<html>` that names the mode the page applied (`data-theme`).
    pub mode_attribute: Option<String>,
    pub reduced_motion: bool,
    /// After fonts and two frames, before the shifts are read and the checks run.
    pub settle_ms: u64,
    /// After a scroll to the bottom, before the always-visible items are checked again.
    pub scroll_settle_ms: u64,
    /// After a switch of mode.
    pub mode_settle_ms: u64,
    pub always_visible: Vec<AlwaysVisible>,
    pub keyboard: Option<KeyboardConfig>,
    /// The opt-in check `layout_shift.load`.
    pub layout_shift_load: bool,
    /// How the values file is made from the document (`ux-browser values`).
    pub values: Option<crate::values::ValuesConfig>,
    /// The HTTP methods a page may send to the measured server (`request_methods`); default
    /// [`READ_METHODS`]. A request with another method fails, as one to another origin does.
    pub request_methods: Vec<String>,
}

/// The methods a page may send to the measured server unless the configuration names more: reads,
/// so that a measurement does not change the state of a server it did not start.
pub const READ_METHODS: &[&str] = &["GET", "HEAD"];

/// `request_methods`: a list of methods in capitals, each once, `GET` among them.
fn request_methods(v: &Value) -> Result<Vec<String>, String> {
    let Some(list) = v.get("request_methods") else {
        return Ok(READ_METHODS.iter().map(|m| m.to_string()).collect());
    };
    if !matches!(list, Value::List(_)) {
        return Err("`request_methods` is a list of HTTP methods".to_string());
    }
    let methods = get_strs(v, "request_methods")?;
    for (i, m) in methods.iter().enumerate() {
        if m.is_empty() || !m.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(format!(
                "`request_methods`: `{m}` is not an HTTP method in capitals"
            ));
        }
        if methods[..i].contains(m) {
            return Err(format!("`request_methods`: `{m}` is named twice"));
        }
    }
    if !methods.iter().any(|m| m == "GET") {
        return Err("`request_methods` names `GET`: a page loads with it".to_string());
    }
    Ok(methods)
}

const TOP_KEYS: &[&str] = &[
    "format",
    "base_path",
    "pages",
    "viewports",
    "modes",
    "mode_attribute",
    "reduced_motion",
    "settle_ms",
    "scroll_settle_ms",
    "mode_settle_ms",
    "always_visible",
    "keyboard",
    "opt_in",
    "values",
    "request_methods",
];

/// The checks that run only when `opt_in` names them.
pub const OPT_IN: &[&str] = &["layout_shift.load"];

fn only(v: &Value, keys: &[&str], what: &str) -> Result<(), String> {
    for (k, _) in v.entries() {
        if !keys.contains(&k.as_str()) {
            return Err(format!("{what}: unknown key `{k}`"));
        }
    }
    Ok(())
}

fn ms(v: &Value, key: &str, default: u64) -> Result<u64, String> {
    match v.get(key) {
        None => Ok(default),
        Some(Value::Int(n)) if *n >= 0 => Ok(*n as u64),
        Some(_) => Err(format!("`{key}` is a whole number of milliseconds")),
    }
}

fn name_of(v: &Value, is_key: &str, contains_key: &str, what: &str) -> Result<Name, String> {
    match (v.get(is_key), v.get(contains_key)) {
        (Some(_), Some(_)) => Err(format!("{what}: `{is_key}` or `{contains_key}`, not both")),
        (Some(is), None) => Ok(Name::Is(
            is.scalar_text()
                .ok_or_else(|| format!("{what}: `{is_key}` is text"))?,
        )),
        (None, Some(_)) => {
            let parts = get_strs(v, contains_key)?;
            if parts.is_empty() {
                return Err(format!("{what}: `{contains_key}` is empty"));
            }
            Ok(Name::Contains(parts))
        }
        (None, None) => Ok(Name::Any),
    }
}

pub fn config(text: &str) -> Result<Config, String> {
    let v = yaml::parse(text).map_err(|e| e.to_string())?;
    if v.get_str("format") != Some(FORMAT) {
        return Err(format!("`format` is not `{FORMAT}`"));
    }
    only(&v, TOP_KEYS, "checks")?;
    let base_path = v.get_str("base_path").ok_or("no `base_path`")?.to_string();
    if !base_path.starts_with('/') || !base_path.ends_with('/') {
        return Err("`base_path` starts and ends with `/`".to_string());
    }
    let pages_v = v.get("pages").ok_or("no `pages`")?;
    only(pages_v, &["sitemap", "paths"], "pages")?;
    let pages = match (pages_v.get_str("sitemap"), pages_v.get("paths")) {
        (Some(file), None) => Pages::Sitemap(file.to_string()),
        (None, Some(_)) => Pages::Paths(get_strs(pages_v, "paths")?),
        _ => return Err("`pages` names a `sitemap` file or a list of `paths`".to_string()),
    };
    let mut viewports: Vec<NamedViewport> = Vec::new();
    for (i, vp) in v
        .get("viewports")
        .map(Value::list)
        .unwrap_or_default()
        .iter()
        .enumerate()
    {
        only(vp, &["name", "width", "height"], "viewport")?;
        let size = crate::render::viewport(vp, &format!("viewports[{i}]"))?;
        let name = vp
            .get_str("name")
            .map_or_else(|| format!("{}x{}", size.width, size.height), str::to_string);
        if viewports.iter().any(|o| o.name == name) {
            return Err(format!("viewport `{name}` is declared twice"));
        }
        viewports.push(NamedViewport {
            name,
            width: size.width,
            height: size.height,
        });
    }
    if viewports.is_empty() {
        return Err("`viewports` declares none".to_string());
    }
    let modes = get_strs(&v, "modes")?;
    if modes.is_empty() {
        return Err("`modes` declares none".to_string());
    }
    for m in &modes {
        if !crate::render::MODES.contains(&m.as_str()) {
            return Err(format!(
                "mode `{m}` is not one of {:?}",
                crate::render::MODES
            ));
        }
    }
    let mut always_visible: Vec<AlwaysVisible> = Vec::new();
    for item in v.get("always_visible").map(Value::list).unwrap_or_default() {
        only(
            item,
            &[
                "name",
                "role",
                "name_is",
                "name_contains",
                "css",
                "scope",
                "min_width",
                "path_prefix",
            ],
            "always_visible",
        )?;
        let name = item
            .get_str("name")
            .ok_or("always_visible: an item has no `name`")?;
        let what = format!("always_visible `{name}`");
        let role = item
            .get_str("role")
            .ok_or_else(|| format!("{what}: no `role`"))?;
        if always_visible.iter().any(|a| a.name == name) {
            return Err(format!("{what} is declared twice"));
        }
        always_visible.push(AlwaysVisible {
            name: name.to_string(),
            locate: Locate {
                role: role.to_string(),
                name: name_of(item, "name_is", "name_contains", &what)?,
                css: item.get_str("css").map(str::to_string),
                scope: item.get_str("scope").map(str::to_string),
            },
            min_width: get_i64(item, "min_width"),
            path_prefix: item.get_str("path_prefix").map(str::to_string),
        });
    }
    let keyboard = match v.get("keyboard") {
        None => None,
        Some(k) => {
            only(
                k,
                &[
                    "skip_link_is",
                    "skip_link_contains",
                    "must_reach",
                    "max_tabs",
                    "modes",
                    "viewports",
                    "focus_settle_ms",
                ],
                "keyboard",
            )?;
            let skip_link = name_of(k, "skip_link_is", "skip_link_contains", "keyboard")?;
            if skip_link == Name::Any {
                return Err(
                    "keyboard: the skip link is named by `skip_link_is` or `skip_link_contains`"
                        .to_string(),
                );
            }
            let must_reach = get_strs(k, "must_reach")?;
            for target in &must_reach {
                if !always_visible.iter().any(|a| &a.name == target) {
                    return Err(format!(
                        "keyboard: `{target}` is not an always-visible item"
                    ));
                }
            }
            let max_tabs = match k.get("max_tabs") {
                Some(Value::Int(n)) if *n > 0 => *n as usize,
                _ => return Err("keyboard: `max_tabs` is a whole number above 0".to_string()),
            };
            let mut walk_modes = get_strs(k, "modes")?;
            if walk_modes.is_empty() {
                walk_modes.push(modes.last().expect("modes is not empty").clone());
            }
            for m in &walk_modes {
                if !modes.contains(m) {
                    return Err(format!("keyboard: mode `{m}` is not in `modes`"));
                }
            }
            let walk_viewports = match k.get("viewports") {
                None => None,
                Some(_) => {
                    let names = get_strs(k, "viewports")?;
                    for n in &names {
                        if !viewports.iter().any(|v| &v.name == n) {
                            return Err(format!("keyboard: viewport `{n}` is not declared"));
                        }
                    }
                    Some(names)
                }
            };
            Some(KeyboardConfig {
                skip_link,
                must_reach,
                max_tabs,
                modes: walk_modes,
                viewports: walk_viewports,
                focus_settle_ms: ms(k, "focus_settle_ms", 100)?,
            })
        }
    };
    let opt_in = get_strs(&v, "opt_in")?;
    for check in &opt_in {
        if !OPT_IN.contains(&check.as_str()) {
            return Err(format!("opt_in: `{check}` is not one of {OPT_IN:?}"));
        }
    }
    let layout_shift_load = opt_in.iter().any(|c| c == "layout_shift.load");
    let names: Vec<String> = viewports.iter().map(|v| v.name.clone()).collect();
    let values = match v.get("values") {
        None => None,
        Some(section) => Some(crate::values::config(section, &names)?),
    };
    let reduced_motion = match v.get("reduced_motion") {
        None => true,
        Some(Value::Bool(b)) => *b,
        Some(_) => return Err("`reduced_motion` is true or false".to_string()),
    };
    Ok(Config {
        base_path,
        pages,
        viewports,
        modes,
        mode_attribute: v.get_str("mode_attribute").map(str::to_string),
        reduced_motion,
        settle_ms: ms(&v, "settle_ms", crate::render::DEFAULT_SETTLE_MS)?,
        scroll_settle_ms: ms(&v, "scroll_settle_ms", 200)?,
        mode_settle_ms: ms(&v, "mode_settle_ms", 100)?,
        always_visible,
        keyboard,
        layout_shift_load,
        values,
        request_methods: request_methods(&v)?,
    })
}

// ---------------------------------------------------------------- results of a page

#[derive(Clone, Debug)]
pub struct Visible {
    pub item: String,
    /// `top` or `bottom`: the scroll position the item was checked at.
    pub position: &'static str,
    pub status: Status,
}

/// One viewport in one mode.
pub struct UnitResult {
    pub viewport: String,
    pub mode: String,
    pub mode_applied: bool,
    pub page_scroll_px: f64,
    pub overflow: Vec<Overflow>,
    pub visible: Vec<Visible>,
}

pub struct KeyboardUnit {
    pub viewport: String,
    pub mode: String,
    pub result: KeyboardResult,
}

/// The shifts of one load: one per viewport, read before the first mode is switched.
pub struct ShiftUnit {
    pub viewport: String,
    pub after_ready: Option<f64>,
    pub load: Option<f64>,
}

pub struct PageResult {
    pub path: String,
    /// Why the page could not be measured: not served, or a mode it did not apply.
    pub error: Option<String>,
    pub units: Vec<UnitResult>,
    pub keyboard: Vec<KeyboardUnit>,
    pub shifts: Vec<ShiftUnit>,
}

impl PageResult {
    pub fn page_scroll(&self) -> i64 {
        self.units.iter().filter(|u| u.page_scroll_px > 0.0).count() as i64
    }
    pub fn overflow(&self) -> i64 {
        self.units.iter().map(|u| u.overflow.len() as i64).sum()
    }
    pub fn always_visible_misses(&self) -> i64 {
        self.units
            .iter()
            .flat_map(|u| &u.visible)
            .filter(|v| v.status.is_miss())
            .count() as i64
    }
    pub fn keyboard_misses(&self) -> i64 {
        self.keyboard.iter().map(|k| k.result.misses()).sum()
    }
    /// The largest load-time shift over the viewports; `None` when one has no value.
    pub fn shift_load(&self) -> Option<f64> {
        self.shifts
            .iter()
            .map(|s| s.load)
            .try_fold(0.0f64, |m, v| v.map(|v| m.max(v)))
    }
    /// The largest over the viewports; `None` when one has no value.
    pub fn shift_after_ready(&self) -> Option<f64> {
        self.shifts
            .iter()
            .map(|s| s.after_ready)
            .try_fold(0.0f64, |m, v| v.map(|v| m.max(v)))
    }
}

fn visible_of(
    config: &Config,
    path: &str,
    vp: &NamedViewport,
    position: &'static str,
    recorded: Option<&Value>,
) -> Result<Vec<Visible>, String> {
    let mut out = Vec::new();
    for item in &config.always_visible {
        let status = if !applies(item, path, vp.width) {
            Status::NotApplicable
        } else {
            let entry = recorded
                .map(Value::list)
                .unwrap_or_default()
                .iter()
                .find(|e| e.get_str("item") == Some(item.name.as_str()))
                .ok_or_else(|| {
                    format!(
                        "{path} {}: no record of `{}` at the {position}",
                        vp.name, item.name
                    )
                })?;
            let candidates = entry
                .get("candidates")
                .map(Value::list)
                .unwrap_or_default()
                .iter()
                .map(Candidate::from_value)
                .collect::<Result<Vec<_>, _>>()?;
            let first = first_visible(&candidates).map(|i| &candidates[i]);
            visible_status(first, vp.width as f64, vp.height as f64)
        };
        out.push(Visible {
            item: item.name.clone(),
            position,
            status,
        });
    }
    Ok(out)
}

/// The values of one page from its recorded geometry (`measure.rs`, `--capture`).
pub fn page_result(config: &Config, page: &Value) -> Result<PageResult, String> {
    let path = page.get_str("path").ok_or("page: no `path`")?.to_string();
    let mut result = PageResult {
        path: path.clone(),
        error: page.get_str("error").map(str::to_string),
        units: Vec::new(),
        keyboard: Vec::new(),
        shifts: Vec::new(),
    };
    if result.error.is_some() {
        return Ok(result);
    }
    for recorded in page.get("viewports").map(Value::list).unwrap_or_default() {
        let name = recorded
            .get_str("viewport")
            .ok_or("page: a viewport without a name")?;
        let vp = config
            .viewports
            .iter()
            .find(|v| v.name == name)
            .ok_or_else(|| format!("{path}: viewport `{name}` is not declared"))?;
        let shifts = Shifts::from_value(recorded.get("shifts").ok_or("page: no `shifts`")?)?;
        result.shifts.push(ShiftUnit {
            viewport: vp.name.clone(),
            after_ready: shift_after_ready(&shifts),
            load: shift_load(&shifts),
        });
        for unit in recorded.get("units").map(Value::list).unwrap_or_default() {
            let mode = unit
                .get_str("mode")
                .ok_or("page: a unit without a mode")?
                .to_string();
            let mode_applied = get_bool(unit, "mode_applied").unwrap_or(false);
            if !mode_applied {
                result.error = Some(format!(
                    "{} {mode}: the page did not apply the mode",
                    vp.name
                ));
            }
            let raw = OverflowRaw::from_value(unit.get("overflow").ok_or("page: no `overflow`")?)?;
            let visible = unit.get("visible");
            let mut seen =
                visible_of(config, &path, vp, "top", visible.and_then(|v| v.get("top")))?;
            seen.extend(visible_of(
                config,
                &path,
                vp,
                "bottom",
                visible.and_then(|v| v.get("bottom")),
            )?);
            result.units.push(UnitResult {
                viewport: vp.name.clone(),
                mode,
                mode_applied,
                page_scroll_px: page_scroll(&raw),
                overflow: overflow(&raw),
                visible: seen,
            });
        }
        if let Some(k) = &config.keyboard {
            for walk in recorded
                .get("keyboard")
                .map(Value::list)
                .unwrap_or_default()
            {
                let raw = KeyboardRaw::from_value(
                    walk.get("walk").ok_or("page: a walk without `walk`")?,
                )?;
                result.keyboard.push(KeyboardUnit {
                    viewport: vp.name.clone(),
                    mode: walk.get_str("mode").unwrap_or_default().to_string(),
                    result: keyboard(&raw, &k.skip_link, k.max_tabs),
                });
            }
        }
    }
    Ok(result)
}
