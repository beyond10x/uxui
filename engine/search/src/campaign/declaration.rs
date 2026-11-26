//! The declaration of a campaign, `ux-search-campaign/1`: the page, the environments, the axes
//! and the terms, read from a file in the YAML subset of `uxui_doc::yaml`.
//!
//! ```yaml
//! format: ux-search-campaign/1
//! page:
//!   path: /orders/list              # the page, under the origin given to `campaign run`
//!   ready: 'main[data-ready]'       # the page is ready when this selector matches
//!   ready_timeout_ms: 5000          # how long to wait for it (default 10000)
//!   settle_ms: 100                  # a wait after it matched (default 0)
//! environments:
//!   viewports: [1366x768, 390x844]
//!   modes:                          # each mode is set in one or more of three ways
//!     - {name: light, media: light} # prefers-color-scheme, emulated
//!     - {name: dark, media: dark, attribute: {name: data-theme, value: dark}, query: theme=dark}
//! axes:
//!   - name: spacing
//!     shipped: stock                # the value the page is built with
//!     values:
//!       - id: tight
//!         rules:                    # custom properties per selector, optionally for one mode
//!           - {selector: ':root', set: {--space: 0.5rem}}
//!           - {selector: ':root[data-theme="dark"]', mode: dark, set: {--ink: '#abcdef'}}
//!       - id: stock                 # no rule: the page as it is
//! primary: 'button.primary'         # the primary action
//! regions: 'header, main > section' # what the term `regions` counts
//! always_visible:                   # what the term `always_visible_misses` looks for
//!   - {name: navigation, selector: nav}
//! small_target_px: 24               # a target under this size is small (default 44)
//! hard: [primary_in_view, no_h_scroll, named]   # of HARD_RULES
//! terms: [low_contrast, primary_px] # of TERMS, in ranking order; `hard` is always first
//! ```
//!
//! A variant is one value of each axis; its id is the values' ids joined by `.`. It is rendered
//! as a style sheet of its values' custom properties, injected after the page's own. A value
//! without rules injects nothing, so the variant of the shipped values is the page as it is.
//!
//! Every refusal names the key it breaks at, as `` `axes[1].values[0].id`: ... ``; a refusal of
//! a file ([`Declaration::read`]) names the file before it.

use std::path::Path;

use ux_browser::target::Site;
use uxui_doc::yaml::{self, Value};

use crate::landing::patch::{self, CssRule};
use crate::landing::record::digest;

/// The format of a declaration.
pub const FORMAT: &str = "ux-search-campaign/1";

/// The hard rules a declaration chooses from. A variant that breaks one of the chosen rules in
/// one environment stands on no frontier.
///
/// - `primary_in_view`: the primary action lies fully in view without scrolling: in the viewport
///   and in every box around it that clips or scrolls.
/// - `no_h_scroll`: the page does not scroll horizontally.
/// - `contrast`: no text run under WCAG 2.x 1.4.3 (4.5:1, 3:1 for large text).
/// - `named`: every visible interactive element has an accessible name.
pub const HARD_RULES: [&str; 4] = ["primary_in_view", "no_h_scroll", "contrast", "named"];

/// The terms a declaration chooses from and orders. Each is a whole number where lower is
/// better, the worst value of the environments; `hard`, the count of hard rules broken, is
/// always the first term and is not named.
///
/// - `primary_px`: the top of the primary action, in page px.
/// - `small_targets`: visible links and buttons under `small_target_px` in width or height.
/// - `regions`: the visible regions (the selector `regions`) that start at or above the
///   primary action.
/// - `text_under_14`: the share of the visible text runs under 14 px, in thousandths.
/// - `low_contrast`: the visible text runs under WCAG 2.x 1.4.3.
/// - `overflow`: visible texts and replaced elements that end to the right of the viewport or
///   of the box that clips or scrolls them.
/// - `always_visible_misses`: the items of `always_visible` that are not fully in the viewport
///   at the top of the page, or not at its end.
pub const TERMS: [&str; 7] = [
    "primary_px",
    "small_targets",
    "regions",
    "text_under_14",
    "low_contrast",
    "overflow",
    "always_visible_misses",
];

/// The most variants one campaign measures: a campaign is exhaustive.
pub const MAX_VARIANTS: usize = 4096;

const TOP_KEYS: [&str; 10] = [
    "format",
    "page",
    "environments",
    "axes",
    "primary",
    "regions",
    "always_visible",
    "small_target_px",
    "hard",
    "terms",
];

/// The page a campaign measures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    /// The path under the origin, starting with `/`.
    pub path: String,
    /// The selector that matches once the page is ready to be measured.
    pub ready: String,
    /// How long the ready selector may take to match.
    pub ready_timeout_ms: u64,
    /// A wait after the ready selector matched, before the first measurement.
    pub settle_ms: u64,
}

/// One mode and how it is set: emulated `prefers-color-scheme`, an attribute of the root
/// element, a query of the page's URL, or more than one of them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mode {
    pub name: String,
    /// `light` or `dark`; a mode without it is rendered with `light`.
    pub media: Option<String>,
    /// The attribute's name and value.
    pub attribute: Option<(String, String)>,
    /// The query, without its `?`.
    pub query: Option<String>,
}

/// One environment: a viewport and a mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    pub width: u32,
    pub height: u32,
    pub mode: String,
}

impl Environment {
    /// `1366x768 light`.
    pub fn name(&self) -> String {
        format!("{}x{} {}", self.width, self.height, self.mode)
    }
}

/// Custom properties one value sets for one selector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    /// Where the declaration writes the rule: `axes[0].values[1].rules[0]`.
    pub key: String,
    /// The one mode the rule is injected in; `None`: in every mode.
    pub mode: Option<String>,
    pub css: CssRule,
}

/// One value of an axis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AxisValue {
    pub id: String,
    pub rules: Vec<Rule>,
}

/// One axis: its values, and the one the page is built with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Axis {
    pub name: String,
    pub values: Vec<AxisValue>,
    /// The index of the shipped value.
    pub shipped: usize,
}

/// One point of the space: a value of each axis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variant {
    /// The values' ids in the order of the axes, joined by `.`.
    pub id: String,
    /// The index of each axis's value.
    pub choice: Vec<usize>,
}

/// Something that must stay in view, for the term `always_visible_misses`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub name: String,
    pub selector: String,
}

/// One property an injected sheet sets, to be checked on the page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    /// The rule's key in the declaration.
    pub key: String,
    pub selector: String,
    pub property: String,
    pub value: String,
}

/// A campaign as declared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration {
    /// The text it was read from.
    pub text: String,
    /// `sha256:<hex>` of that text.
    pub digest: String,
    pub page: Page,
    /// Width and height, in the order declared.
    pub viewports: Vec<(u32, u32)>,
    pub modes: Vec<Mode>,
    pub axes: Vec<Axis>,
    /// The selector of the primary action.
    pub primary: String,
    /// The selector of the regions the term `regions` counts.
    pub regions: Option<String>,
    pub always_visible: Vec<Item>,
    pub small_target_px: u64,
    /// The chosen hard rules, of [`HARD_RULES`].
    pub hard: Vec<String>,
    /// The chosen terms in ranking order, of [`TERMS`].
    pub terms: Vec<String>,
}

fn at(key: &str, why: impl std::fmt::Display) -> String {
    format!("`{key}`: {why}")
}

fn join(path: &str, key: &str) -> String {
    if path.is_empty() {
        key.to_owned()
    } else {
        format!("{path}.{key}")
    }
}

/// The value as a map whose keys are all of `allowed`.
fn map<'a>(value: &'a Value, path: &str, allowed: &[&str]) -> Result<&'a Value, String> {
    let Value::Map(entries) = value else {
        return Err(at(
            path,
            format!("not a map with the keys {}", allowed.join(", ")),
        ));
    };
    for (key, _) in entries {
        if !allowed.contains(&key.as_str()) {
            return Err(at(
                &join(path, key),
                format!("unknown key; the keys here are {}", allowed.join(", ")),
            ));
        }
    }
    Ok(value)
}

/// A scalar of the map as text; `None` when the key is absent or null. An empty text and a
/// text with white space at an end are refused.
fn text(value: &Value, path: &str, key: &str) -> Result<Option<String>, String> {
    let key_path = join(path, key);
    match value.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::List(_) | Value::Map(_)) => Err(at(&key_path, "not a text")),
        Some(scalar) => {
            let text = scalar.scalar_text().unwrap_or_default();
            if text.is_empty() || text.trim() != text {
                return Err(at(
                    &key_path,
                    format!("`{text}` is empty or starts or ends with white space"),
                ));
            }
            Ok(Some(text))
        }
    }
}

fn required(value: &Value, path: &str, key: &str, what: &str) -> Result<String, String> {
    text(value, path, key)?.ok_or_else(|| at(&join(path, key), format!("missing: {what}")))
}

/// A whole number of the map from `from` to `to`; `None` when the key is absent.
fn number(
    value: &Value,
    path: &str,
    key: &str,
    (from, to): (u64, u64),
) -> Result<Option<u64>, String> {
    match value.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Int(n)) if u64::try_from(*n).is_ok_and(|n| (from..=to).contains(&n)) => {
            Ok(u64::try_from(*n).ok())
        }
        Some(other) => Err(at(
            &join(path, key),
            format!(
                "`{}` is not a whole number from {from} to {to}",
                other
                    .scalar_text()
                    .unwrap_or_else(|| "a list or map".to_owned())
            ),
        )),
    }
}

/// The items of a list of the map; a key that is absent or null has none.
fn items<'a>(value: &'a Value, path: &str, key: &str) -> Result<&'a [Value], String> {
    match value.get(key) {
        None | Some(Value::Null) => Ok(&[]),
        Some(Value::List(items)) => Ok(items),
        Some(_) => Err(at(&join(path, key), "not a list")),
    }
}

/// A name that stands in an id, an environment's name and a file name: letters, digits, `-`,
/// `_` and `+`.
fn name_ok(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_+".contains(c))
}

fn name(value: &Value, path: &str, key: &str, what: &str) -> Result<String, String> {
    let name = required(value, path, key, what)?;
    if !name_ok(&name) {
        return Err(at(
            &join(path, key),
            format!("`{name}` holds something else than letters, digits, `-`, `_` and `+`"),
        ));
    }
    Ok(name)
}

fn twice(key: &str, name: &str, of: &str) -> String {
    at(key, format!("`{name}` is named twice{of}"))
}

fn page(root: &Value) -> Result<Page, String> {
    let Some(value) = root.get("page") else {
        return Err(at("page", "missing: the page the campaign measures"));
    };
    let value = map(
        value,
        "page",
        &["path", "ready", "ready_timeout_ms", "settle_ms"],
    )?;
    let path = required(value, "page", "path", "the page's path under the origin")?;
    if !path.starts_with('/') {
        return Err(at("page.path", format!("`{path}` does not start with `/`")));
    }
    if path.contains(['?', '#']) || path.contains(char::is_whitespace) {
        return Err(at(
            "page.path",
            format!(
                "`{path}` holds a query, a fragment or white space; a query is a mode's `query`"
            ),
        ));
    }
    Ok(Page {
        path,
        ready: required(
            value,
            "page",
            "ready",
            "the selector that matches once the page is ready",
        )?,
        ready_timeout_ms: number(value, "page", "ready_timeout_ms", (1, 120_000))?
            .unwrap_or(10_000),
        settle_ms: number(value, "page", "settle_ms", (0, 60_000))?.unwrap_or(0),
    })
}

fn viewport(text: &str) -> Option<(u32, u32)> {
    let (width, height) = text.split_once('x')?;
    let side = |digits: &str| {
        digits
            .chars()
            .all(|c| c.is_ascii_digit())
            .then(|| digits.parse::<u32>().ok())
            .flatten()
            .filter(|side| (1..=10_000).contains(side))
    };
    Some((side(width)?, side(height)?))
}

fn viewports(value: &Value) -> Result<Vec<(u32, u32)>, String> {
    let list = items(value, "environments", "viewports")?;
    if list.is_empty() {
        return Err(at(
            "environments.viewports",
            "no viewport; one is <W>x<H>, as 1366x768",
        ));
    }
    let mut viewports = Vec::with_capacity(list.len());
    for (index, item) in list.iter().enumerate() {
        let key = format!("environments.viewports[{index}]");
        let text = item.scalar_text().unwrap_or_default();
        let Some(size) = viewport(&text) else {
            return Err(at(
                &key,
                format!("`{text}` is not <W>x<H>, each side from 1 to 10000"),
            ));
        };
        if viewports.contains(&size) {
            return Err(twice(&key, &text, ""));
        }
        viewports.push(size);
    }
    Ok(viewports)
}

fn attribute_name_ok(name: &str) -> bool {
    name.chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_.:".contains(c))
}

fn modes(value: &Value) -> Result<Vec<Mode>, String> {
    let list = items(value, "environments", "modes")?;
    if list.is_empty() {
        return Err(at(
            "environments.modes",
            "no mode; one is {name: light, media: light}",
        ));
    }
    let mut modes: Vec<Mode> = Vec::with_capacity(list.len());
    for (index, item) in list.iter().enumerate() {
        let path = format!("environments.modes[{index}]");
        let item = map(item, &path, &["name", "media", "attribute", "query"])?;
        let name = name(item, &path, "name", "the mode's name")?;
        if modes.iter().any(|mode| mode.name == name) {
            return Err(twice(&join(&path, "name"), &name, ""));
        }
        let media = text(item, &path, "media")?;
        if let Some(media) = &media
            && media != "light"
            && media != "dark"
        {
            return Err(at(
                &join(&path, "media"),
                format!(
                    "`{media}` is not `light` or `dark`, the values of prefers-color-scheme \
                     the browser emulates"
                ),
            ));
        }
        let attribute = match item.get("attribute") {
            None | Some(Value::Null) => None,
            Some(attribute) => {
                let path = join(&path, "attribute");
                let attribute = map(attribute, &path, &["name", "value"])?;
                let name = required(attribute, &path, "name", "the attribute's name")?;
                if !attribute_name_ok(&name) {
                    return Err(at(
                        &join(&path, "name"),
                        format!("`{name}` is not the name of an attribute"),
                    ));
                }
                let value = required(attribute, &path, "value", "the attribute's value")?;
                Some((name, value))
            }
        };
        let query = text(item, &path, "query")?;
        if let Some(query) = &query
            && (query.starts_with('?')
                || query.contains('#')
                || query.contains(char::is_whitespace))
        {
            return Err(at(
                &join(&path, "query"),
                format!("`{query}` starts with `?`, or holds `#` or white space"),
            ));
        }
        if media.is_none() && attribute.is_none() && query.is_none() {
            return Err(at(
                &path,
                format!("the mode `{name}` has no `media`, `attribute` or `query` that sets it"),
            ));
        }
        modes.push(Mode {
            name,
            media,
            attribute,
            query,
        });
    }
    Ok(modes)
}

/// The rules of one value: custom properties per selector, each rule for every mode or one.
fn rules(value: &Value, path: &str, modes: &[Mode]) -> Result<Vec<Rule>, String> {
    let list = items(value, path, "rules")?;
    let mut rules = Vec::with_capacity(list.len());
    for (index, item) in list.iter().enumerate() {
        let key = format!("{path}.rules[{index}]");
        let item = map(item, &key, &["selector", "mode", "set"])?;
        let selector = required(
            item,
            &key,
            "selector",
            "the selector the properties are set for",
        )?;
        let mut css = CssRule {
            selector,
            values: Vec::new(),
        };
        patch::set_custom_properties("", std::slice::from_ref(&css))
            .map_err(|why| at(&join(&key, "selector"), why))?;
        let mode = text(item, &key, "mode")?;
        if let Some(mode) = &mode
            && !modes.iter().any(|known| known.name == *mode)
        {
            let known: Vec<&str> = modes.iter().map(|mode| mode.name.as_str()).collect();
            return Err(at(
                &join(&key, "mode"),
                format!(
                    "`{mode}` is no mode of `environments.modes` ({})",
                    known.join(", ")
                ),
            ));
        }
        let set_key = join(&key, "set");
        let set = match item.get("set") {
            Some(Value::Map(entries)) if !entries.is_empty() => entries,
            None | Some(Value::Null) | Some(Value::Map(_)) => {
                return Err(at(
                    &set_key,
                    "no property set; a rule sets custom properties, as {--space: 0.5rem}",
                ));
            }
            Some(_) => return Err(at(&set_key, "not a map of custom properties")),
        };
        for (property, value) in set {
            let Some(value) = value.scalar_text() else {
                return Err(at(
                    &set_key,
                    format!("the value of `{property}` is not a text"),
                ));
            };
            css.values.push((property.clone(), value));
        }
        patch::set_custom_properties("", std::slice::from_ref(&css))
            .map_err(|why| at(&set_key, why))?;
        rules.push(Rule { key, mode, css });
    }
    Ok(rules)
}

fn axes(root: &Value, modes: &[Mode]) -> Result<Vec<Axis>, String> {
    let list = items(root, "", "axes")?;
    if list.is_empty() {
        return Err(at("axes", "no axis; a campaign varies at least one thing"));
    }
    let mut axes: Vec<Axis> = Vec::with_capacity(list.len());
    for (index, item) in list.iter().enumerate() {
        let path = format!("axes[{index}]");
        let item = map(item, &path, &["name", "shipped", "values"])?;
        let name = name(item, &path, "name", "the axis's name")?;
        if axes.iter().any(|axis| axis.name == name) {
            return Err(twice(&join(&path, "name"), &name, ""));
        }
        let list = items(item, &path, "values")?;
        if list.is_empty() {
            return Err(at(
                &join(&path, "values"),
                format!("the axis `{name}` has no value"),
            ));
        }
        let mut values: Vec<AxisValue> = Vec::with_capacity(list.len());
        for (at_value, value) in list.iter().enumerate() {
            let path = format!("{path}.values[{at_value}]");
            let value = map(value, &path, &["id", "rules"])?;
            let id = self::name(value, &path, "id", "the value's name")?;
            if values.iter().any(|value| value.id == id) {
                return Err(twice(
                    &join(&path, "id"),
                    &id,
                    &format!(" in the axis `{name}`"),
                ));
            }
            values.push(AxisValue {
                id,
                rules: rules(value, &path, modes)?,
            });
        }
        let shipped = required(
            item,
            &path,
            "shipped",
            &format!("the value of the axis `{name}` the page is built with"),
        )?;
        let Some(shipped) = values.iter().position(|value| value.id == shipped) else {
            let known: Vec<&str> = values.iter().map(|value| value.id.as_str()).collect();
            return Err(at(
                &join(&path, "shipped"),
                format!(
                    "`{shipped}` is no value of the axis `{name}` ({})",
                    known.join(", ")
                ),
            ));
        };
        axes.push(Axis {
            name,
            values,
            shipped,
        });
    }
    let variants = axes
        .iter()
        .try_fold(1usize, |count, axis| count.checked_mul(axis.values.len()))
        .unwrap_or(usize::MAX);
    if variants > MAX_VARIANTS {
        return Err(at(
            "axes",
            format!(
                "{variants} variants, more than the {MAX_VARIANTS} one campaign measures: every \
                 variant is rendered in every environment"
            ),
        ));
    }
    set_once(&axes)?;
    Ok(axes)
}

/// Refuses a property of a selector that two rules of one variant set in one mode: two rules of
/// one value, or rules of two axes. One of the two would not be in force.
fn set_once(axes: &[Axis]) -> Result<(), String> {
    struct Set<'a> {
        axis: usize,
        value: usize,
        rule: &'a Rule,
        property: &'a str,
    }
    let mut seen: Vec<Set> = Vec::new();
    for (axis, declared) in axes.iter().enumerate() {
        for (value, declared) in declared.values.iter().enumerate() {
            for rule in &declared.rules {
                for (property, _) in &rule.css.values {
                    let earlier = seen.iter().find(|earlier| {
                        (earlier.axis != axis || earlier.value == value)
                            && earlier.rule.css.selector == rule.css.selector
                            && earlier.property == property
                            && (earlier.rule.mode.is_none()
                                || rule.mode.is_none()
                                || earlier.rule.mode == rule.mode)
                    });
                    if let Some(earlier) = earlier {
                        return Err(at(
                            &join(&rule.key, "set"),
                            format!(
                                "`{property}` of `{}` is also set by `{}` for a mode they share; \
                                 one variant would set it twice",
                                rule.css.selector,
                                join(&earlier.rule.key, "set")
                            ),
                        ));
                    }
                    seen.push(Set {
                        axis,
                        value,
                        rule,
                        property,
                    });
                }
            }
        }
    }
    Ok(())
}

fn always_visible(root: &Value) -> Result<Vec<Item>, String> {
    let list = items(root, "", "always_visible")?;
    let mut found: Vec<Item> = Vec::with_capacity(list.len());
    for (index, item) in list.iter().enumerate() {
        let path = format!("always_visible[{index}]");
        let item = map(item, &path, &["name", "selector"])?;
        let name = required(item, &path, "name", "what the item is called")?;
        if found.iter().any(|item| item.name == name) {
            return Err(twice(&join(&path, "name"), &name, ""));
        }
        found.push(Item {
            name,
            selector: required(item, &path, "selector", "the selector of the item")?,
        });
    }
    Ok(found)
}

/// The names of a list, each of the catalogue and named once.
fn chosen(root: &Value, key: &str, what: &str, catalogue: &[&str]) -> Result<Vec<String>, String> {
    let list = items(root, "", key)?;
    let mut names: Vec<String> = Vec::with_capacity(list.len());
    for (index, item) in list.iter().enumerate() {
        let at_key = format!("{key}[{index}]");
        let name = item.scalar_text().unwrap_or_default();
        if key == "terms" && name == "hard" {
            return Err(at(
                &at_key,
                "`hard` is always the first term and is not named",
            ));
        }
        if !catalogue.contains(&name.as_str()) {
            return Err(at(
                &at_key,
                format!(
                    "`{name}` is no {what}; the catalogue: {}",
                    catalogue.join(", ")
                ),
            ));
        }
        if names.contains(&name) {
            return Err(twice(&at_key, &name, ""));
        }
        names.push(name);
    }
    Ok(names)
}

impl Declaration {
    /// Reads a declaration from its text.
    pub fn parse(source: &str) -> Result<Declaration, String> {
        let root = yaml::parse(source).map_err(|error| error.to_string())?;
        if !matches!(root, Value::Map(_)) {
            return Err(format!(
                "the declaration is not a map; it starts with `format: {FORMAT}`"
            ));
        }
        match root.get("format").and_then(Value::scalar_text) {
            Some(format) if format == FORMAT => {}
            Some(format) => return Err(at("format", format!("`{format}` is not `{FORMAT}`"))),
            None => return Err(at("format", format!("missing: `{FORMAT}`"))),
        }
        map(&root, "", &TOP_KEYS)?;
        let page = page(&root)?;
        let Some(environments) = root.get("environments") else {
            return Err(at(
                "environments",
                "missing: the viewports and the modes the page is measured in",
            ));
        };
        let environments = map(environments, "environments", &["viewports", "modes"])?;
        let viewports = viewports(environments)?;
        let modes = modes(environments)?;
        let axes = axes(&root, &modes)?;
        let primary = required(&root, "", "primary", "the selector of the primary action")?;
        let regions = text(&root, "", "regions")?;
        let always_visible = always_visible(&root)?;
        let small_target_px = number(&root, "", "small_target_px", (1, 1000))?.unwrap_or(44);
        let hard = chosen(&root, "hard", "hard rule", &HARD_RULES)?;
        let terms = chosen(&root, "terms", "term", &TERMS)?;
        if terms.is_empty() {
            return Err(at(
                "terms",
                format!(
                    "no term; the catalogue: {}; `hard` is always the first",
                    TERMS.join(", ")
                ),
            ));
        }
        for (index, term) in terms.iter().enumerate() {
            let needs = match term.as_str() {
                "regions" if regions.is_none() => {
                    "`regions`: the selector of the regions it counts"
                }
                "always_visible_misses" if always_visible.is_empty() => {
                    "`always_visible`: the items it looks for"
                }
                _ => continue,
            };
            return Err(at(
                &format!("terms[{index}]"),
                format!("the term `{term}` needs the key {needs}"),
            ));
        }
        let declaration = Declaration {
            digest: digest(source.as_bytes()),
            text: source.to_owned(),
            page,
            viewports,
            modes,
            axes,
            primary,
            regions,
            always_visible,
            small_target_px,
            hard,
            terms,
        };
        // Every sheet is built once here, so no run finds a variant it cannot write.
        for variant in declaration.variants() {
            for mode in &declaration.modes {
                declaration
                    .built(&variant, &mode.name)
                    .map_err(|why| at("axes", format!("{}: {why}", variant.id)))?;
            }
        }
        Ok(declaration)
    }

    /// Reads a declaration file; a refusal names the file.
    pub fn read(path: &Path) -> Result<Declaration, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        Declaration::parse(&text).map_err(|why| format!("{}: {why}", path.display()))
    }

    /// The environments in the order a variant's renders are recorded: each viewport in each
    /// mode.
    pub fn environments(&self) -> Vec<Environment> {
        let mut environments = Vec::with_capacity(self.viewports.len() * self.modes.len());
        for &(width, height) in &self.viewports {
            for mode in &self.modes {
                environments.push(Environment {
                    width,
                    height,
                    mode: mode.name.clone(),
                });
            }
        }
        environments
    }

    /// The declared mode of this name.
    pub fn mode(&self, name: &str) -> Option<&Mode> {
        self.modes.iter().find(|mode| mode.name == name)
    }

    fn variant(&self, choice: Vec<usize>) -> Variant {
        let ids: Vec<&str> = self
            .axes
            .iter()
            .zip(&choice)
            .map(|(axis, &value)| axis.values[value].id.as_str())
            .collect();
        Variant {
            id: ids.join("."),
            choice,
        }
    }

    /// Every variant once: the values of the last axis change fastest.
    pub fn variants(&self) -> Vec<Variant> {
        let mut choices: Vec<Vec<usize>> = vec![Vec::new()];
        for axis in &self.axes {
            choices = choices
                .into_iter()
                .flat_map(|choice| {
                    (0..axis.values.len()).map(move |value| {
                        let mut choice = choice.clone();
                        choice.push(value);
                        choice
                    })
                })
                .collect();
        }
        choices
            .into_iter()
            .map(|choice| self.variant(choice))
            .collect()
    }

    /// The page as shipped: the shipped value of every axis.
    pub fn shipped(&self) -> Variant {
        self.variant(self.axes.iter().map(|axis| axis.shipped).collect())
    }

    /// The variant's rules in the order of the axes; with a mode, only those injected in it.
    fn rules<'a>(&'a self, variant: &Variant, mode: Option<&str>) -> Vec<&'a Rule> {
        self.axes
            .iter()
            .zip(&variant.choice)
            .flat_map(|(axis, &value)| &axis.values[value].rules)
            .filter(|rule| match (mode, &rule.mode) {
                (Some(mode), Some(only)) => only == mode,
                _ => true,
            })
            .collect()
    }

    fn built(&self, variant: &Variant, mode: &str) -> Result<String, String> {
        let rules: Vec<CssRule> = self
            .rules(variant, Some(mode))
            .into_iter()
            .map(|rule| rule.css.clone())
            .collect();
        let sheet = patch::set_custom_properties("", &rules)?;
        Ok(sheet.strip_prefix('\n').unwrap_or(&sheet).to_owned())
    }

    /// The style sheet a run injects for the variant in a mode: the custom properties of its
    /// rules, one rule per selector. Empty for a variant without rules.
    pub fn sheet(&self, variant: &Variant, mode: &str) -> String {
        // `parse` built every sheet once, so none fails here.
        self.built(variant, mode).unwrap_or_default()
    }

    /// Every property the variant's sheet sets in a mode, with the key of the rule that sets
    /// it.
    pub fn checks(&self, variant: &Variant, mode: &str) -> Vec<Check> {
        self.rules(variant, Some(mode))
            .into_iter()
            .flat_map(|rule| {
                rule.css.values.iter().map(|(property, value)| Check {
                    key: rule.key.clone(),
                    selector: rule.css.selector.clone(),
                    property: property.clone(),
                    value: value.clone(),
                })
            })
            .collect()
    }

    /// The variant's patch as a report prints it, one line per rule:
    /// `<selector> { <property>: <value>; ... }`, with `<mode>: ` before a rule of one mode.
    pub fn patch_lines(&self, variant: &Variant) -> Vec<String> {
        let rules = self.rules(variant, None);
        if rules.is_empty() {
            return vec!["no property set: the page as shipped".to_owned()];
        }
        rules
            .into_iter()
            .map(|rule| {
                let values: Vec<String> = rule
                    .css
                    .values
                    .iter()
                    .map(|(property, value)| format!("{property}: {value}"))
                    .collect();
                let mode = rule
                    .mode
                    .as_ref()
                    .map_or(String::new(), |mode| format!("{mode}: "));
                format!("{mode}{} {{ {} }}", rule.css.selector, values.join("; "))
            })
            .collect()
    }

    /// The URL of the page on a server in a mode: the path under the origin (under the path of
    /// `--url`, when it has one), and the mode's query.
    pub fn url(&self, site: &Site, mode: &str) -> Result<String, String> {
        let url = site.page_url("/", &self.page.path)?;
        Ok(match self.mode(mode).and_then(|mode| mode.query.as_ref()) {
            Some(query) => format!("{url}?{query}"),
            None => url,
        })
    }
}
