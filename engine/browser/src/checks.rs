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

/// The slack, in CSS pixels, an always-visible item may pass the viewport by.
const VIEWPORT_SLACK: f64 = 0.5;

// ---------------------------------------------------------------- page scroll and overflow

/// The elements the overflow check measures besides the text runs unless `layout.overflow.elements`
/// names others: 0.3.0's, llm's.
pub const OVERFLOW_ELEMENTS: &[&str] = &[
    "img", "svg", "video", "canvas", "iframe", "input", "select", "textarea", "table", "pre",
];

/// The overflow rule (`layout.overflow`): how far an edge may pass its limit, the elements measured
/// besides the text runs, and the CSS selectors under which nothing is measured. The elements and
/// the exclusions are inputs of the probe; the tolerance is applied here.
#[derive(Clone, Debug, PartialEq)]
pub struct OverflowRule {
    pub tolerance_px: f64,
    pub elements: Vec<String>,
    pub exclude: Vec<String>,
}

impl Default for OverflowRule {
    fn default() -> Self {
        OverflowRule {
            tolerance_px: 1.0,
            elements: OVERFLOW_ELEMENTS.iter().map(|e| e.to_string()).collect(),
            exclude: Vec::new(),
        }
    }
}

/// The rules of page scroll and overflow (`layout`); the default is 0.3.0's: a unit counts when the
/// page scrolls sideways at all, an edge overflows when it passes its limit by more than 1 px.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct LayoutRule {
    /// How far, in CSS pixels, the page may scroll sideways before the unit counts.
    pub page_slack_px: f64,
    pub overflow: OverflowRule,
}

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
    /// Index into the recorded `exclude`: the first selector the item's element is inside.
    pub excluded: Option<usize>,
}

pub struct Block {
    pub desc: String,
    pub right: f64,
}

/// `probes/overflow.js`: the viewport's client width, the page's scroll width, the probe inputs
/// it was called with (`elements`, `exclude`; absent in a 0.3.0 capture: the defaults), and every
/// text run and every element `elements` matches, with its block and its first exclusion.
pub struct OverflowRaw {
    pub vw: f64,
    pub scroll_width: f64,
    pub elements: Vec<String>,
    pub exclude: Vec<String>,
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
                let excluded = match l.len() {
                    5 => -1.0,
                    6 => num_at(l, 5)?,
                    _ => return None,
                };
                Some(Item {
                    right: num_at(l, 0)?,
                    width: num_at(l, 1)?,
                    height: num_at(l, 2)?,
                    context,
                    block: (block >= 0.0).then_some(block as usize),
                    excluded: (excluded >= 0.0).then_some(excluded as usize),
                })
            })
            .collect::<Option<Vec<_>>>()
            .ok_or("overflow: an item is not [right, width, height, context, block, excluded]")?;
        if items
            .iter()
            .any(|i| i.block.is_some_and(|b| b >= blocks.len()))
        {
            return Err("overflow: an item names a block that is not listed".to_string());
        }
        let defaults = OverflowRule::default();
        let elements = match v.get("elements") {
            None => defaults.elements,
            Some(_) => get_strs(v, "elements").map_err(|e| format!("overflow: {e}"))?,
        };
        let exclude = get_strs(v, "exclude").map_err(|e| format!("overflow: {e}"))?;
        if items
            .iter()
            .any(|i| i.excluded.is_some_and(|x| x >= exclude.len()))
        {
            return Err("overflow: an item names an exclusion that is not listed".to_string());
        }
        Ok(OverflowRaw {
            vw,
            scroll_width,
            elements,
            exclude,
            blocks,
            items,
        })
    }
}

/// How far the page scrolls sideways, in CSS pixels; a unit counts when it is above the slack
/// ([`UnitResult::scrolls`]).
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

/// [`overflow_with`] the default rule, 0.3.0's.
pub fn overflow(raw: &OverflowRaw) -> Vec<Overflow> {
    overflow_with(raw, &OverflowRule::default())
}

/// Items whose right edge passes the viewport or their block by more than the tolerance, at most
/// one per block; items in a fixed layer, a scroll container or a clipping box, and items the probe
/// found inside an excluded selector, are not overflow. The elements and exclusions of `rule` are
/// the probe's inputs; [`page_result`] refuses a recording made with others.
pub fn overflow_with(raw: &OverflowRaw, rule: &OverflowRule) -> Vec<Overflow> {
    let tolerance = rule.tolerance_px;
    let mut failed = vec![false; raw.blocks.len()];
    let mut out = Vec::new();
    for item in &raw.items {
        if item.width < 1.0
            || item.height < 1.0
            || item.context != Context::Flow
            || item.excluded.is_some()
        {
            continue;
        }
        let Some(b) = item.block else { continue };
        let block = &raw.blocks[b];
        let (against, limit) = if item.right > raw.vw + tolerance {
            ("viewport", raw.vw)
        } else if item.right > block.right + tolerance {
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
    /// The element at each of five points of its box, the centre and the four corners, is it or
    /// inside it (`docs/browser-adapter.md`, The cover test); a record of 0.9.1 and before holds the
    /// centre only.
    pub hit: bool,
    /// Its `aria-current` attribute, if it has one; a record of 0.3.0 has none.
    pub current: Option<String>,
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
            current: v.get_str("current").map(str::to_string),
        })
    }

    /// Whether it is the current element of a set (`aria-current` other than `false`): an empty
    /// value is `false`, as is no attribute; any other value, ARIA's tokens or not, is current.
    pub fn is_current(&self) -> bool {
        self.current.as_deref().is_some_and(|v| {
            let v = v.trim();
            !v.is_empty() && !v.eq_ignore_ascii_case("false")
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

/// How an item is found: by role and name, else by CSS, inside the first match of `scope`; with
/// `current`, only an element whose `aria-current` is not `false`, and without a role or CSS any
/// such element in the scope.
#[derive(Clone, Debug)]
pub struct Locate {
    /// `None`: the accessibility tree is not read, and no name is matched.
    pub role: Option<String>,
    pub name: Name,
    pub css: Option<String>,
    pub scope: Option<String>,
    pub current: bool,
}

/// The selector of the elements that may be current: any with `aria-current`; the candidate probe
/// reads its value.
pub const CURRENT_CSS: &str = "[aria-current]";

impl Locate {
    /// The CSS selector whose matches inside the scope are candidates after the role matches:
    /// `css`, else [`CURRENT_CSS`] for an item with `current` and no role.
    pub fn selector(&self) -> Option<&str> {
        match (&self.css, &self.role, self.current) {
            (Some(css), _, _) => Some(css),
            (None, None, true) => Some(CURRENT_CSS),
            _ => None,
        }
    }
}

/// The candidate that is the item: the first visible one in scope, and current when the item
/// declares `current`.
pub fn located(locate: &Locate, candidates: &[Candidate]) -> Option<usize> {
    candidates
        .iter()
        .position(|c| c.in_scope && c.visible && (!locate.current || c.is_current()))
}

#[derive(Clone, Debug)]
pub struct AlwaysVisible {
    pub name: String,
    pub locate: Locate,
    pub min_width: Option<i64>,
    /// The widest viewport the item is checked at: it is checked at `min_width <= width <=
    /// max_width`. A whole number above 0, not below `min_width`.
    pub max_width: Option<i64>,
    pub path_prefix: Option<String>,
}

pub fn applies(item: &AlwaysVisible, path: &str, width: i64) -> bool {
    item.min_width.is_none_or(|m| width >= m)
        && item.max_width.is_none_or(|m| width <= m)
        && item
            .path_prefix
            .as_deref()
            .is_none_or(|p| path.starts_with(p))
}

// ---------------------------------------------------------------- keyboard

/// `probes/focus-mark.js` on the focused element: whether it matches `:focus-visible`, its computed
/// outline and box-shadow, and a short description of it.
#[derive(Clone, Debug, PartialEq)]
pub struct FocusStyle {
    pub element: String,
    pub focus_visible: bool,
    pub outline_style: String,
    pub outline_width: f64,
    pub outline_color: String,
    pub box_shadow: String,
}

impl FocusStyle {
    pub fn from_value(v: &Value) -> Result<Self, String> {
        let text = |key: &str| {
            v.get_str(key)
                .map(str::to_string)
                .ok_or_else(|| format!("focus mark: no `{key}`"))
        };
        Ok(FocusStyle {
            element: text("element")?,
            focus_visible: get_bool(v, "focus_visible").ok_or("focus mark: no `focus_visible`")?,
            outline_style: text("outline_style")?,
            outline_width: get_f64(v, "outline_width").ok_or("focus mark: no `outline_width`")?,
            outline_color: text("outline_color")?,
            box_shadow: text("box_shadow")?,
        })
    }
}

/// Whether a computed colour is fully transparent: `transparent`, or a colour function whose alpha
/// (its fourth part, or the part after `/`) is 0.
fn transparent(color: &str) -> bool {
    let c = color.trim();
    if c.eq_ignore_ascii_case("transparent") {
        return true;
    }
    let Some(inner) = c
        .split_once('(')
        .and_then(|(_, rest)| rest.strip_suffix(')'))
    else {
        return false;
    };
    let alpha = match inner.split_once('/') {
        Some((_, a)) => Some(a),
        None => {
            let parts: Vec<&str> = inner.split(',').collect();
            (parts.len() == 4).then(|| parts[3])
        }
    };
    alpha.is_some_and(|a| {
        let a = a.trim();
        let number = a.strip_suffix('%').unwrap_or(a);
        number.parse::<f64>().is_ok_and(|x| x == 0.0)
    })
}

/// A focus mark: the element matches `:focus-visible`, and its outline is drawn (a style, a width
/// above 0, a colour that is not fully transparent) or it has a box-shadow.
pub fn focus_marked(style: &FocusStyle) -> bool {
    let outline = style.outline_style != "none"
        && style.outline_width > 0.0
        && !transparent(&style.outline_color);
    style.focus_visible && (outline || style.box_shadow != "none")
}

impl FocusStyle {
    /// What a miss says: the focused element and what it showed.
    pub fn detail(&self) -> String {
        let visible = if self.focus_visible {
            ""
        } else {
            ", not :focus-visible"
        };
        format!(
            "focused {}: outline {} {}px {}, box-shadow {}{visible}",
            self.element,
            self.outline_style,
            crate::value::decimal(self.outline_width, 2),
            self.outline_color,
            self.box_shadow
        )
    }
}

/// What one key did within its wait (`probes/activate.js` and the browser's navigation events).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyFacts {
    pub navigated: bool,
    pub focus_moved: bool,
    pub state_changed: bool,
    pub dom_changed: bool,
}

impl KeyFacts {
    pub fn from_value(v: &Value) -> Result<Self, String> {
        let fact = |key: &str| get_bool(v, key).ok_or_else(|| format!("activation: no `{key}`"));
        Ok(KeyFacts {
            navigated: fact("navigated")?,
            focus_moved: fact("focus_moved")?,
            state_changed: fact("state_changed")?,
            dom_changed: fact("dom_changed")?,
        })
    }
}

/// The activation of one target: whether focus came back to it on a fresh load, and what Enter and
/// Space did (`None`: not pressed).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Activation {
    pub refocused: bool,
    pub enter: Option<KeyFacts>,
    pub space: Option<KeyFacts>,
}

impl Activation {
    pub fn from_value(v: &Value) -> Result<Self, String> {
        let key = |name: &str| match v.get(name) {
            None | Some(Value::Null) => Ok(None),
            Some(f) => KeyFacts::from_value(f).map(Some),
        };
        Ok(Activation {
            refocused: get_bool(v, "refocused").ok_or("activation: no `refocused`")?,
            enter: key("enter")?,
            space: key("space")?,
        })
    }
}

impl KeyFacts {
    pub fn any(&self) -> bool {
        self.navigated || self.focus_moved || self.state_changed || self.dom_changed
    }
}

/// Enter or Space acted on a target focused again: the page navigated, focus moved, the focused
/// element changed its state, or the document changed where it had been quiet.
pub fn acted(a: &Activation) -> bool {
    a.refocused && (a.enter.is_some_and(|k| k.any()) || a.space.is_some_and(|k| k.any()))
}

impl Activation {
    /// What a miss says.
    pub fn detail(&self) -> String {
        if !self.refocused {
            "focus did not come back to it on a fresh load".to_string()
        } else {
            "neither Enter nor Space changed the page".to_string()
        }
    }
}

/// `measure.rs`'s keyboard walk: the label of the first Tab stop, whether Enter (or one more Tab)
/// put focus into the skip link's target (only with a skip link), which targets were found, per Tab
/// the targets focus was in, and with them the focus marks and the activations of the targets.
pub struct KeyboardRaw {
    pub first_label: Option<String>,
    pub after_enter: bool,
    pub after_tab: Option<bool>,
    pub located: Vec<(String, bool)>,
    pub tabs: Vec<Vec<String>>,
    pub marks: Vec<(String, FocusStyle)>,
    pub activations: Vec<(String, Activation)>,
}

impl KeyboardRaw {
    pub fn from_value(v: &Value) -> Result<Self, String> {
        let first_label = v.get_str("first_label").map(str::to_string);
        let after_enter = match first_label {
            Some(_) => get_bool(v, "after_enter").ok_or("keyboard: no `after_enter`")?,
            None => false,
        };
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
        let marks = v
            .get("marks")
            .map(Value::entries)
            .unwrap_or_default()
            .iter()
            .map(|(k, m)| Ok((k.clone(), FocusStyle::from_value(m)?)))
            .collect::<Result<_, String>>()?;
        let activations = v
            .get("activations")
            .map(Value::entries)
            .unwrap_or_default()
            .iter()
            .map(|(k, a)| Ok((k.clone(), Activation::from_value(a)?)))
            .collect::<Result<_, String>>()?;
        Ok(KeyboardRaw {
            first_label,
            after_enter,
            after_tab,
            located,
            tabs,
            marks,
            activations,
        })
    }
}

/// One judged check of a target: whether it held, and what was seen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Judged {
    pub target: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyboardResult {
    /// Whether the skip-link checks ran: a skip link is declared.
    pub skip_link: bool,
    pub skip_link_first: bool,
    pub skip_moves_focus: bool,
    pub reached: Vec<(String, bool)>,
    /// With `focus_mark`, per reached target.
    pub focus_marks: Vec<Judged>,
    /// Per reached target of `activates`.
    pub activated: Vec<Judged>,
}

impl KeyboardResult {
    /// With a skip link: the skip link not first, its target not focused (also when it is not
    /// first); then each target not reached, each focus mark not shown, each activation that did
    /// not act.
    pub fn misses(&self) -> i64 {
        let skip = if self.skip_link {
            i64::from(!self.skip_link_first) + i64::from(!self.skip_moves_focus)
        } else {
            0
        };
        skip + self.reached.iter().filter(|(_, r)| !r).count() as i64
            + self.focus_marks.iter().filter(|j| !j.ok).count() as i64
            + self.activated.iter().filter(|j| !j.ok).count() as i64
    }
}

pub fn keyboard(raw: &KeyboardRaw, skip_link: &Name, max_tabs: usize) -> KeyboardResult {
    let skip_link_first = raw
        .first_label
        .as_deref()
        .is_some_and(|l| skip_link.matches(l));
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
        skip_link: true,
        skip_link_first,
        skip_moves_focus,
        reached,
        focus_marks: Vec::new(),
        activated: Vec::new(),
    }
}

/// The checks of one recorded walk under the configuration's keyboard section: the skip link's
/// two when one is declared, each target's reach, and of each reached target its focus mark (with
/// `focus_mark`) and its activation (when `activates` names it). A reached target whose mark or
/// activation the walk did not record is refused: the record is not of this configuration.
pub fn keyboard_of(raw: &KeyboardRaw, k: &KeyboardConfig) -> Result<KeyboardResult, String> {
    let mut r = match &k.skip_link {
        Some(name) => keyboard(raw, name, k.max_tabs),
        None => KeyboardResult {
            skip_link: false,
            skip_link_first: false,
            skip_moves_focus: false,
            ..keyboard(raw, &Name::Any, k.max_tabs)
        },
    };
    for (target, reached) in &r.reached {
        if !reached {
            continue;
        }
        if k.focus_mark {
            let (_, style) = raw
                .marks
                .iter()
                .find(|(n, _)| n == target)
                .ok_or_else(|| format!("keyboard: no focus mark recorded for `{target}`"))?;
            r.focus_marks.push(Judged {
                target: target.clone(),
                ok: focus_marked(style),
                detail: style.detail(),
            });
        }
        if k.activates.contains(target) {
            let (_, a) = raw
                .activations
                .iter()
                .find(|(n, _)| n == target)
                .ok_or_else(|| format!("keyboard: no activation recorded for `{target}`"))?;
            r.activated.push(Judged {
                target: target.clone(),
                ok: acted(a),
                detail: a.detail(),
            });
        }
    }
    Ok(r)
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
    /// The skip link's name; `None`: the walk has no skip-link checks.
    pub skip_link: Option<Name>,
    pub must_reach: Vec<String>,
    pub max_tabs: usize,
    /// Each reached target shows a focus mark (`focus_marked`).
    pub focus_mark: bool,
    /// The targets on which Enter or Space must act (`acted`).
    pub activates: Vec<String>,
    /// The viewports (names) the walk runs at; `None`: every viewport.
    pub viewports: Option<Vec<String>>,
    pub modes: Vec<String>,
    /// The wait after Enter on the skip link, and the quiet watch and the waits of an activation.
    pub focus_settle_ms: u64,
}

pub enum Pages {
    Sitemap(String),
    Paths(Vec<String>),
}

/// A condition the page sets when its content is drawn: an element that matches a CSS selector, or
/// a property of `window` the page sets to `true`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ready {
    Selector(String),
    Global(String),
}

/// How long a page may take to set its ready condition, by default.
pub const DEFAULT_READY_TIMEOUT_MS: u64 = 10_000;

impl Ready {
    /// `selector <css>` or `global <name>`.
    pub fn condition(&self) -> String {
        match self {
            Ready::Selector(css) => format!("selector {css}"),
            Ready::Global(name) => format!("global {name}"),
        }
    }
}

/// The ready condition as the document names it (`conditions.other.ready`): `load` without one.
pub fn ready_condition(config: &Config) -> String {
    config
        .ready
        .as_ref()
        .map_or_else(|| "load".to_string(), Ready::condition)
}

/// The ready instant (`performance.now()` once the condition held, fonts were loaded and two frames
/// drawn) from the answer of `measure::READY_PROBE`, or why the page is not ready: the page's
/// failure, which makes its results incomplete.
pub fn ready_time(config: &Config, answer: &Value) -> Result<f64, String> {
    let condition = ready_condition(config);
    if get_bool(answer, "invalid") == Some(true) {
        let css = match &config.ready {
            Some(Ready::Selector(css)) => css.as_str(),
            _ => condition.as_str(),
        };
        return Err(format!("ready: `{css}` is not a valid selector"));
    }
    match get_bool(answer, "met") {
        Some(true) => get_f64(answer, "ready")
            .ok_or_else(|| format!("ready: `{condition}` held and the page gave no ready time")),
        Some(false) => Err(format!(
            "ready: the page did not set `{condition}` within {} ms",
            config.ready_timeout_ms
        )),
        None => Err(format!("ready: no answer for `{condition}`")),
    }
}

/// Whether `name` is a JavaScript identifier of plain characters: `[A-Za-z_$][A-Za-z0-9_$]*`.
fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '$')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

fn ready_of(v: &Value) -> Result<(Option<Ready>, u64), String> {
    let ready = match v.get("ready") {
        None => None,
        Some(r @ Value::Map(_)) => {
            only(r, &["selector", "global"], "ready")?;
            match (r.get("selector"), r.get("global")) {
                (Some(css), None) => {
                    let css = css
                        .str()
                        .filter(|c| !c.trim().is_empty())
                        .ok_or("ready: `selector` is a CSS selector")?;
                    Some(Ready::Selector(css.to_string()))
                }
                (None, Some(name)) => {
                    let name = name
                        .str()
                        .filter(|n| is_identifier(n))
                        .ok_or("ready: `global` is the name of a property of `window`")?;
                    Some(Ready::Global(name.to_string()))
                }
                _ => return Err("ready: one of `selector` or `global`".to_string()),
            }
        }
        Some(_) => return Err("ready: `{selector: <css>}` or `{global: <name>}`".to_string()),
    };
    let timeout = match (v.get("ready_timeout_ms"), &ready) {
        (None, _) => DEFAULT_READY_TIMEOUT_MS,
        (Some(_), None) => return Err("`ready_timeout_ms` without `ready`".to_string()),
        (Some(Value::Int(n)), Some(_)) if *n > 0 => *n as u64,
        (Some(_), Some(_)) => {
            return Err("`ready_timeout_ms` is a whole number of milliseconds above 0".to_string());
        }
    };
    Ok((ready, timeout))
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
    /// The condition the page sets when it is ready (`ready`); `None`: its `load` event.
    pub ready: Option<Ready>,
    /// How long a page may take to set `ready` after its `load` event.
    pub ready_timeout_ms: u64,
    /// How the values file is made from the document (`ux-browser values`).
    pub values: Option<crate::values::ValuesConfig>,
    /// The visual ratchet: the screenshots of each page, viewport and mode against a baseline
    /// (`visual`, `visual::config`); none: no screenshot is taken.
    pub visual: Option<crate::visual::Visual>,
    /// The HTTP methods a page may send to the measured server (`request_methods`); default
    /// [`READ_METHODS`]. A request with another method fails, as one to another origin does.
    pub request_methods: Vec<String>,
    /// The page-scroll slack and the overflow rule (`layout`).
    pub layout: LayoutRule,
    /// The page clock (`clock`); `None`: the host's.
    pub clock: Option<Clock>,
    /// Switches Chromium starts with after the defaults (`chrome_args`, `chrome::switches`).
    pub chrome_args: Vec<String>,
    /// The Chrome build the run uses (`chrome`, `chrome::find`); `None`: the newest.
    pub chrome: Option<crate::chrome::Pin>,
    /// The journeys whose states are checked (`journeys`, `journey::config`); none by default.
    pub journeys: Vec<crate::journey::Journey>,
    /// The design tokens the token checks hold the page to (`tokens`, `tokens::config`); none:
    /// no token check runs.
    pub tokens: Option<crate::tokens::Tokens>,
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
    "ready",
    "ready_timeout_ms",
    "always_visible",
    "keyboard",
    "opt_in",
    "values",
    "visual",
    "request_methods",
    "layout",
    "clock",
    "chrome_args",
    "chrome",
    "journeys",
    "tokens",
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
                "current",
                "min_width",
                "max_width",
                "path_prefix",
            ],
            "always_visible",
        )?;
        let name = item
            .get_str("name")
            .ok_or("always_visible: an item has no `name`")?;
        let what = format!("always_visible `{name}`");
        let role = item.get_str("role").map(str::to_string);
        let css = item.get_str("css").map(str::to_string);
        let current = match item.get("current") {
            None => false,
            Some(Value::Bool(b)) => *b,
            Some(_) => return Err(format!("{what}: `current` is true or false")),
        };
        if role.is_none() && css.is_none() && !current {
            return Err(format!(
                "{what}: no `role`, `css` or `current: true` to locate it by"
            ));
        }
        let name_match = name_of(item, "name_is", "name_contains", &what)?;
        if role.is_none() && name_match != Name::Any {
            return Err(format!(
                "{what}: a name is matched on a `role`'s accessible name; the item has no `role`"
            ));
        }
        if always_visible.iter().any(|a| a.name == name) {
            return Err(format!("{what} is declared twice"));
        }
        let min_width = get_i64(item, "min_width");
        let max_width = match item.get("max_width") {
            None => None,
            Some(Value::Int(n)) if *n > 0 => Some(*n),
            Some(_) => {
                return Err(format!(
                    "{what}: `max_width` is a whole number of pixels above 0"
                ));
            }
        };
        if let (Some(min), Some(max)) = (min_width, max_width)
            && max < min
        {
            return Err(format!(
                "{what}: `max_width` {max} is below `min_width` {min}"
            ));
        }
        always_visible.push(AlwaysVisible {
            name: name.to_string(),
            locate: Locate {
                role,
                name: name_match,
                css,
                scope: item.get_str("scope").map(str::to_string),
                current,
            },
            min_width,
            max_width,
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
                    "focus_mark",
                    "activates",
                    "modes",
                    "viewports",
                    "focus_settle_ms",
                ],
                "keyboard",
            )?;
            let skip_link = match name_of(k, "skip_link_is", "skip_link_contains", "keyboard")? {
                Name::Any => None,
                name => Some(name),
            };
            let must_reach = get_strs(k, "must_reach")?;
            for target in &must_reach {
                if !always_visible.iter().any(|a| &a.name == target) {
                    return Err(format!(
                        "keyboard: `{target}` is not an always-visible item"
                    ));
                }
            }
            let focus_mark = match k.get("focus_mark") {
                None => false,
                Some(Value::Bool(b)) => *b,
                Some(_) => return Err("keyboard: `focus_mark` is true or false".to_string()),
            };
            if k.get("activates")
                .is_some_and(|a| !matches!(a, Value::List(_)))
            {
                return Err("keyboard: `activates` is a list of targets".to_string());
            }
            let activates = get_strs(k, "activates")?;
            for (i, target) in activates.iter().enumerate() {
                if !must_reach.contains(target) {
                    return Err(format!(
                        "keyboard: `activates` names `{target}`, which is not in `must_reach`"
                    ));
                }
                if activates[..i].contains(target) {
                    return Err(format!("keyboard: `activates` names `{target}` twice"));
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
                focus_mark,
                activates,
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
    let (ready, ready_timeout_ms) = ready_of(&v)?;
    let chrome_args = get_strs(&v, "chrome_args")?;
    crate::chrome::switches(&chrome_args).map_err(|e| format!("chrome_args: {e}"))?;
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
        ready,
        ready_timeout_ms,
        values,
        visual: crate::visual::config(v.get("visual"), &names)?,
        request_methods: request_methods(&v)?,
        layout: layout_rule(&v)?,
        clock: clock_of(&v)?,
        chrome_args,
        chrome: v.get("chrome").map(crate::chrome::pin).transpose()?,
        journeys: crate::journey::config(v.get("journeys"))?,
        tokens: crate::tokens::config(v.get("tokens"))?,
    })
}

// ---------------------------------------------------------------- configuration: clock

/// A fixed page clock (`clock: {at, advance}`): `Date` reads the declared instant, plus the time
/// since the document's time origin unless the clock is frozen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Clock {
    /// The declared instant, in milliseconds since 1970-01-01T00:00:00Z.
    pub at_ms: i64,
    /// Whether the clock runs on from the instant (`advance`, default true) or stands still.
    pub advance: bool,
}

impl Clock {
    /// The instant in UTC, RFC 3339: `2026-01-01T12:00:20Z`, with milliseconds when it has any.
    pub fn instant(&self) -> String {
        let ms = self.at_ms.rem_euclid(1000);
        let secs = self.at_ms.div_euclid(1000);
        let (days, day_secs) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
        let (y, m, d) = civil_from_days(days);
        let fraction = if ms == 0 {
            String::new()
        } else {
            format!(".{ms:03}")
        };
        format!(
            "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}{fraction}Z",
            day_secs / 3600,
            day_secs % 3600 / 60,
            day_secs % 60
        )
    }

    /// `from <instant>`, or `frozen at <instant>`.
    pub fn condition(&self) -> String {
        if self.advance {
            format!("from {}", self.instant())
        } else {
            format!("frozen at {}", self.instant())
        }
    }
}

/// The clock as the document names it (`conditions.other.clock`): `real` without one.
pub fn clock_condition(config: &Config) -> String {
    config
        .clock
        .as_ref()
        .map_or_else(|| "real".to_string(), Clock::condition)
}

/// Days since 1970-01-01 of a date of the proleptic Gregorian calendar (H. Hinnant's
/// `days_from_civil`).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The date of a day since 1970-01-01 (`civil_from_days`).
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

fn days_in_month(y: i64, m: i64) -> i64 {
    match m {
        2 if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// An RFC 3339 instant (`date-time`: `2026-01-01T12:00:20Z`, `2026-01-01T13:00:20.25+01:00`; `T`
/// and `Z` in either case) in milliseconds since 1970-01-01T00:00:00Z. A second fraction has at
/// most 3 digits, as a JavaScript `Date` holds no finer time; a leap second (`:60`) is refused, as
/// `Date` has none.
pub fn rfc3339_ms(text: &str) -> Result<i64, String> {
    let bad = || {
        format!(
            "clock: `at` is an RFC 3339 instant, `2026-01-01T12:00:20Z` or `2026-01-01T13:00:20+01:00`; `{text}` is not one"
        )
    };
    let b = text.as_bytes();
    let digits = |from: usize, n: usize| -> Option<i64> {
        let part = b.get(from..from + n)?;
        part.iter()
            .all(u8::is_ascii_digit)
            .then(|| part.iter().fold(0i64, |a, c| a * 10 + i64::from(c - b'0')))
    };
    let at = |i: usize, c: u8| b.get(i).is_some_and(|x| x.eq_ignore_ascii_case(&c));
    if !(at(4, b'-') && at(7, b'-') && at(10, b'T') && at(13, b':') && at(16, b':')) {
        return Err(bad());
    }
    let (Some(y), Some(mo), Some(d), Some(h), Some(mi), Some(s)) = (
        digits(0, 4),
        digits(5, 2),
        digits(8, 2),
        digits(11, 2),
        digits(14, 2),
        digits(17, 2),
    ) else {
        return Err(bad());
    };
    if !(1..=12).contains(&mo) || d < 1 || d > days_in_month(y, mo) || h > 23 || mi > 59 || s > 59 {
        return Err(bad());
    }
    let mut i = 19;
    let mut ms = 0;
    if at(i, b'.') {
        let n = b[i + 1..].iter().take_while(|c| c.is_ascii_digit()).count();
        if !(1..=3).contains(&n) {
            return Err(bad());
        }
        ms = digits(i + 1, n).ok_or_else(bad)? * 10i64.pow(3 - n as u32);
        i += 1 + n;
    }
    let offset_min = if at(i, b'Z') && b.len() == i + 1 {
        0
    } else if (at(i, b'+') || at(i, b'-')) && b.len() == i + 6 && at(i + 3, b':') {
        let (Some(oh), Some(om)) = (digits(i + 1, 2), digits(i + 4, 2)) else {
            return Err(bad());
        };
        if oh > 23 || om > 59 {
            return Err(bad());
        }
        let sign = if b[i] == b'-' { -1 } else { 1 };
        sign * (oh * 60 + om)
    } else {
        return Err(bad());
    };
    let secs = days_from_civil(y, mo, d) * 86_400 + h * 3600 + mi * 60 + s - offset_min * 60;
    Ok(secs * 1000 + ms)
}

/// `clock`: `{at: <RFC 3339 instant>, advance: true | false}`; `advance` defaults to true.
fn clock_of(v: &Value) -> Result<Option<Clock>, String> {
    let Some(section) = v.get("clock") else {
        return Ok(None);
    };
    if !matches!(section, Value::Map(_)) {
        return Err("`clock` is a section: `{at: <RFC 3339 instant>, advance}`".to_string());
    }
    only(section, &["at", "advance"], "clock")?;
    let at = match section.get("at") {
        Some(Value::Str(text)) => rfc3339_ms(text)?,
        Some(_) => return Err("clock: `at` is an RFC 3339 instant, as text".to_string()),
        None => return Err("clock: no `at`, the instant the page's clock starts at".to_string()),
    };
    let advance = match section.get("advance") {
        None => true,
        Some(Value::Bool(b)) => *b,
        Some(_) => return Err("clock: `advance` is true or false".to_string()),
    };
    Ok(Some(Clock { at_ms: at, advance }))
}

// ---------------------------------------------------------------- configuration: layout

/// `layout`: `page_slack_px` and `overflow: {tolerance_px, elements, exclude}`; a key left out keeps
/// its default ([`LayoutRule::default`], 0.3.0's rule).
fn layout_rule(v: &Value) -> Result<LayoutRule, String> {
    let mut rule = LayoutRule::default();
    let Some(section) = v.get("layout") else {
        return Ok(rule);
    };
    if !matches!(section, Value::Map(_)) {
        return Err("`layout` is a section: `page_slack_px`, `overflow`".to_string());
    }
    only(section, &["page_slack_px", "overflow"], "layout")?;
    if let Some(px) = pixels(section, "page_slack_px")? {
        rule.page_slack_px = px;
    }
    let Some(o) = section.get("overflow") else {
        return Ok(rule);
    };
    if !matches!(o, Value::Map(_)) {
        return Err(
            "`layout.overflow` is a section: `tolerance_px`, `elements`, `exclude`".to_string(),
        );
    }
    only(
        o,
        &["tolerance_px", "elements", "exclude"],
        "layout.overflow",
    )?;
    if let Some(px) = pixels(o, "tolerance_px")? {
        rule.overflow.tolerance_px = px;
    }
    if let Some(elements) = selectors(o, "elements")? {
        if elements.is_empty() {
            return Err("`layout.overflow.elements` names none".to_string());
        }
        rule.overflow.elements = elements;
    }
    if let Some(exclude) = selectors(o, "exclude")? {
        rule.overflow.exclude = exclude;
    }
    Ok(rule)
}

/// A length in CSS pixels, 0 or more.
fn pixels(v: &Value, key: &str) -> Result<Option<f64>, String> {
    match v.get(key) {
        None => Ok(None),
        Some(n) => match crate::value::f64_of(n) {
            Some(px) if px.is_finite() && px >= 0.0 => Ok(Some(px)),
            _ => Err(format!(
                "`layout`: `{key}` is a number of pixels, 0 or more"
            )),
        },
    }
}

/// A list of CSS selectors, each text and named once.
fn selectors(v: &Value, key: &str) -> Result<Option<Vec<String>>, String> {
    match v.get(key) {
        None => Ok(None),
        Some(Value::List(_)) => {
            let list = get_strs(v, key).map_err(|e| format!("`layout.overflow`: {e}"))?;
            for (i, sel) in list.iter().enumerate() {
                if sel.trim().is_empty() {
                    return Err(format!("`layout.overflow.{key}`: a selector is empty"));
                }
                if list[..i].contains(sel) {
                    return Err(format!("`layout.overflow.{key}`: `{sel}` is named twice"));
                }
            }
            Ok(Some(list))
        }
        Some(_) => Err(format!(
            "`layout.overflow.{key}` is a list of CSS selectors"
        )),
    }
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
    pub page_slack_px: f64,
    pub overflow: Vec<Overflow>,
    pub visible: Vec<Visible>,
}

impl UnitResult {
    /// The unit counts for `layout.page_scroll`: the page scrolls sideways by more than the slack.
    pub fn scrolls(&self) -> bool {
        self.page_scroll_px > self.page_slack_px
    }
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
    /// The token outputs per viewport and mode (`tokens`); none without a `tokens` section.
    pub tokens: Vec<crate::tokens::TokenUnit>,
}

impl PageResult {
    pub fn page_scroll(&self) -> i64 {
        self.units.iter().filter(|u| u.scrolls()).count() as i64
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

/// The always-visible items of a page at `vp` from their recorded candidates at `position`.
pub fn visible_of(
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
            let first = located(&item.locate, &candidates).map(|i| &candidates[i]);
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
        tokens: Vec::new(),
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
            let rule = &config.layout.overflow;
            if raw.elements != rule.elements || raw.exclude != rule.exclude {
                return Err(format!(
                    "{path} {} {mode}: the overflow was recorded with other `elements` or `exclude` than the configuration's",
                    vp.name
                ));
            }
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
            if let Some(tokens) = &config.tokens {
                let recorded = unit.get("tokens").ok_or_else(|| {
                    format!(
                        "{path} {} {mode}: recorded without `tokens`; the configuration declares them",
                        vp.name
                    )
                })?;
                result.tokens.push(crate::tokens::TokenUnit {
                    viewport: vp.name.clone(),
                    mode: mode.clone(),
                    outcomes: crate::tokens::judge(tokens, recorded)
                        .map_err(|e| format!("{path} {} {mode}: {e}", vp.name))?,
                });
            }
            result.units.push(UnitResult {
                viewport: vp.name.clone(),
                mode,
                mode_applied,
                page_scroll_px: page_scroll(&raw),
                page_slack_px: config.layout.page_slack_px,
                overflow: overflow_with(&raw, rule),
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
                    result: keyboard_of(&raw, k)?,
                });
            }
        }
    }
    Ok(result)
}
