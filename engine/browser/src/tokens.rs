//! Design-token checks (`tokens` of `ux-browser-checks/1`, `docs/browser-adapter.md`, Design
//! tokens): per page, viewport and mode, the font sizes, colours, gaps and paddings, and radii the
//! page uses off a declared token set, and the lowest contrast of its text on its background. The
//! in-page probe (`probes/tokens.js`) returns what the page uses, each distinct value once with
//! how often and the first node; [`judge`] decides, in Rust, against the declared tokens.

use crate::value::{decimal, f64_of, get_strs};
use uxui_doc::yaml::Value;

/// Distinct font sizes of drawn text off the declared scale.
pub const TYPE_SCALE: &str = "proxy.type_scale@1";
/// Distinct colours of text, backgrounds and borders off the declared colours.
pub const COLOURS: &str = "proxy.colours@1";
/// Distinct gaps between sibling boxes and paddings that are not a multiple of the spacing unit.
pub const SPACING: &str = "proxy.spacing_units@1";
/// Distinct corner radii off the declared radii.
pub const RADII: &str = "proxy.radii@1";
/// The lowest contrast ratio of drawn text on its background.
pub const CONTRAST: &str = "a11y.contrast_min@1";

/// The outputs and their units, in the order a document writes them.
pub const OUTPUTS: [(&str, &str); 5] = [
    (TYPE_SCALE, "sizes"),
    (COLOURS, "colours"),
    (SPACING, "gaps"),
    (RADII, "radii"),
    (CONTRAST, "ratio"),
];

/// How far, in CSS pixels, a font size may lie from a size of the scale.
pub const SIZE_TOLERANCE_PX: f64 = 0.5;
/// How far each of red, green and blue (0 to 255) may lie from a declared colour.
pub const CHANNEL_TOLERANCE: f64 = 2.0;
/// How far the alpha (0 to 1) may lie from a declared colour's.
pub const ALPHA_TOLERANCE: f64 = 0.02;
/// How far, in CSS pixels, a gap or padding may lie from a multiple of the spacing unit.
pub const GAP_TOLERANCE_PX: f64 = 0.5;
/// How far, in CSS pixels, a radius may lie from a declared radius; a radius below it is none.
pub const RADIUS_TOLERANCE_PX: f64 = 0.5;
/// The most offenders one result names in its details; `details.more` counts the rest.
pub const OFFENDERS_CAP: usize = 20;
/// The contrast text needs (WCAG 1.4.3): normal text, and large text (24 px, or 18.66 px bold).
pub const CONTRAST_NORMAL: f64 = 4.5;
pub const CONTRAST_LARGE: f64 = 3.0;

/// The in-page probe: what the page uses, per distinct value.
pub const PROBE: &str = include_str!("probes/tokens.js");

// ---------------------------------------------------------------- configuration

#[derive(Clone, Debug, PartialEq)]
pub enum FontSizes {
    /// The sizes of the scale, in CSS pixels.
    List(Vec<f64>),
    /// Every size `base * ratio^k` for a whole number `k`.
    Scale { base: f64, ratio: f64 },
}

/// A colour: red, green and blue from 0 to 255, alpha from 0 to 1 (sRGB).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgba {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Rgba {
    pub fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Rgba { r, g, b, a }
    }

    /// `#rrggbb`, or `#rrggbbaa` when it is not opaque.
    pub fn hex(&self) -> String {
        let byte = |x: f64| x.round().clamp(0.0, 255.0) as u8;
        let mut out = format!(
            "#{:02x}{:02x}{:02x}",
            byte(self.r),
            byte(self.g),
            byte(self.b)
        );
        if self.a < 1.0 {
            out.push_str(&format!("{:02x}", byte(self.a * 255.0)));
        }
        out
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Colours {
    /// Custom properties read from the page's root in each mode; a trailing `*` takes every one
    /// whose name starts with the part before it and holds a colour.
    pub properties: Vec<String>,
    /// Colours declared as values.
    pub values: Vec<Rgba>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Radius {
    Px(f64),
    /// A corner rounded to at least half the box's shorter side: a pill or a circle.
    Full,
}

/// The `tokens` section: each kind left out is not declared, and its output is not run.
#[derive(Clone, Debug, PartialEq)]
pub struct Tokens {
    pub font_sizes: Option<FontSizes>,
    pub colours: Option<Colours>,
    pub spacing_unit_px: Option<f64>,
    pub radii: Option<Vec<Radius>>,
}

const KEYS: &[&str] = &["font_sizes", "colours", "spacing_unit_px", "radii"];

fn only(v: &Value, keys: &[&str], what: &str) -> Result<(), String> {
    for (k, _) in v.entries() {
        if !keys.contains(&k.as_str()) {
            return Err(format!("{what}: unknown key `{k}`"));
        }
    }
    Ok(())
}

fn positive(v: &Value, what: &str) -> Result<f64, String> {
    match f64_of(v) {
        Some(x) if x > 0.0 && x.is_finite() => Ok(x),
        _ => Err(format!("{what} is a number of CSS pixels above 0")),
    }
}

fn nonempty_list<'a>(v: &'a Value, what: &str) -> Result<&'a [Value], String> {
    match v {
        Value::List(items) if !items.is_empty() => Ok(items),
        _ => Err(format!("{what} is a list that is not empty")),
    }
}

/// The `tokens` section of a checks configuration; `None` without one.
pub fn config(v: Option<&Value>) -> Result<Option<Tokens>, String> {
    let Some(v) = v else { return Ok(None) };
    if !matches!(v, Value::Map(_)) {
        return Err("`tokens` is a map".to_string());
    }
    only(v, KEYS, "tokens")?;
    if v.entries().is_empty() {
        return Err("`tokens` declares no token".to_string());
    }
    let font_sizes = match v.get("font_sizes") {
        None => None,
        Some(Value::Map(_)) => {
            let fs = v.get("font_sizes").unwrap();
            only(fs, &["base", "ratio"], "tokens.font_sizes")?;
            let base = positive(
                fs.get("base").ok_or("tokens.font_sizes: no `base`")?,
                "tokens.font_sizes.base",
            )?;
            let ratio = match fs.get("ratio").and_then(f64_of) {
                Some(r) if r > 1.0 && r.is_finite() => r,
                _ => return Err("tokens.font_sizes.ratio is a number above 1".to_string()),
            };
            Some(FontSizes::Scale { base, ratio })
        }
        Some(list) => {
            let items = nonempty_list(list, "tokens.font_sizes (or {base, ratio})")?;
            Some(FontSizes::List(
                items
                    .iter()
                    .map(|i| positive(i, "tokens.font_sizes: each size"))
                    .collect::<Result<_, _>>()?,
            ))
        }
    };
    let colours = match v.get("colours") {
        None => None,
        Some(c) => {
            if !matches!(c, Value::Map(_)) {
                return Err("tokens.colours is a map of `properties` and `values`".to_string());
            }
            only(c, &["properties", "values"], "tokens.colours")?;
            for key in ["properties", "values"] {
                if let Some(l) = c.get(key) {
                    nonempty_list(l, &format!("tokens.colours.{key}"))?;
                }
            }
            let properties =
                get_strs(c, "properties").map_err(|e| format!("tokens.colours: {e}"))?;
            for p in &properties {
                let stem = p.strip_suffix('*').unwrap_or(p);
                if !stem.starts_with("--") || stem.contains('*') || stem.contains(' ') {
                    return Err(format!(
                        "tokens.colours.properties: `{p}` is not a custom property (`--name`, or `--prefix*`)"
                    ));
                }
            }
            let values = get_strs(c, "values")
                .map_err(|e| format!("tokens.colours: {e}"))?
                .iter()
                .map(|t| {
                    parse_colour(t).ok_or_else(|| {
                        format!(
                            "tokens.colours.values: `{t}` is not `#rgb`, `#rrggbb`, `#rrggbbaa`, `rgb()` or `rgba()`"
                        )
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            if properties.is_empty() && values.is_empty() {
                return Err("tokens.colours declares no `properties` and no `values`".to_string());
            }
            Some(Colours { properties, values })
        }
    };
    let spacing_unit_px = v
        .get("spacing_unit_px")
        .map(|s| positive(s, "tokens.spacing_unit_px"))
        .transpose()?;
    let radii = match v.get("radii") {
        None => None,
        Some(list) => Some(
            nonempty_list(list, "tokens.radii")?
                .iter()
                .map(|r| match r {
                    Value::Str(s) if s == "full" => Ok(Radius::Full),
                    other => match f64_of(other) {
                        Some(x) if x >= 0.0 && x.is_finite() => Ok(Radius::Px(x)),
                        _ => Err(
                            "tokens.radii: each radius is a number of CSS pixels, 0 or more, or `full`"
                                .to_string(),
                        ),
                    },
                })
                .collect::<Result<_, _>>()?,
        ),
    };
    Ok(Some(Tokens {
        font_sizes,
        colours,
        spacing_unit_px,
        radii,
    }))
}

/// A colour written `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, `rgb(r, g, b)`, `rgba(r, g, b, a)` or
/// `rgb(r g b / a)` (alpha a number or a percentage).
pub fn parse_colour(text: &str) -> Option<Rgba> {
    let t = text.trim().to_ascii_lowercase();
    if let Some(hex) = t.strip_prefix('#') {
        if !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let digit = |i: usize| u8::from_str_radix(&hex[i..i + 1], 16).ok().map(f64::from);
        let pair = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok().map(f64::from);
        return match hex.len() {
            3 | 4 => {
                let c = |i| digit(i).map(|d| d * 17.0);
                let a = if hex.len() == 4 { c(3)? / 255.0 } else { 1.0 };
                Some(Rgba::new(c(0)?, c(1)?, c(2)?, a))
            }
            6 | 8 => {
                let a = if hex.len() == 8 {
                    pair(6)? / 255.0
                } else {
                    1.0
                };
                Some(Rgba::new(pair(0)?, pair(2)?, pair(4)?, a))
            }
            _ => None,
        };
    }
    let inner = t
        .strip_prefix("rgba(")
        .or_else(|| t.strip_prefix("rgb("))?
        .strip_suffix(')')?;
    let (channels, alpha) = match inner.split_once('/') {
        Some((c, a)) => (c.to_string(), Some(a.trim().to_string())),
        None => (inner.to_string(), None),
    };
    let mut parts: Vec<String> = channels
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|p| !p.is_empty())
        .map(str::to_string)
        .collect();
    let alpha = match alpha {
        Some(a) => Some(a),
        None if parts.len() == 4 => parts.pop(),
        None => None,
    };
    if parts.len() != 3 {
        return None;
    }
    let channel = |p: &str| p.parse::<f64>().ok().filter(|x| (0.0..=255.0).contains(x));
    let a = match alpha {
        None => 1.0,
        Some(a) => match a.strip_suffix('%') {
            Some(pct) => pct.trim().parse::<f64>().ok()? / 100.0,
            None => a.parse::<f64>().ok()?,
        },
    };
    if !(0.0..=1.0).contains(&a) {
        return None;
    }
    Some(Rgba::new(
        channel(&parts[0])?,
        channel(&parts[1])?,
        channel(&parts[2])?,
        a,
    ))
}

// ---------------------------------------------------------------- the rules

/// Whether a font size lies on the scale, within [`SIZE_TOLERANCE_PX`].
pub fn size_on(scale: &FontSizes, px: f64) -> bool {
    match scale {
        FontSizes::List(sizes) => sizes.iter().any(|s| (s - px).abs() <= SIZE_TOLERANCE_PX),
        FontSizes::Scale { base, ratio } => {
            if px <= 0.0 {
                return false;
            }
            let k = ((px / base).ln() / ratio.ln()).round();
            [k - 1.0, k, k + 1.0]
                .iter()
                .any(|k| (base * ratio.powf(*k) - px).abs() <= SIZE_TOLERANCE_PX)
        }
    }
}

/// Whether two colours are the same within [`CHANNEL_TOLERANCE`] and [`ALPHA_TOLERANCE`].
pub fn colour_matches(a: Rgba, b: Rgba) -> bool {
    (a.r - b.r).abs() <= CHANNEL_TOLERANCE
        && (a.g - b.g).abs() <= CHANNEL_TOLERANCE
        && (a.b - b.b).abs() <= CHANNEL_TOLERANCE
        && (a.a - b.a).abs() <= ALPHA_TOLERANCE
}

/// Whether a gap is a multiple of the unit, within [`GAP_TOLERANCE_PX`].
pub fn multiple_of(px: f64, unit: f64) -> bool {
    (px - (px / unit).round() * unit).abs() <= GAP_TOLERANCE_PX
}

/// `top` over an opaque `under`.
pub fn composite(top: Rgba, under: Rgba) -> Rgba {
    let a = top.a;
    Rgba::new(
        top.r * a + under.r * (1.0 - a),
        top.g * a + under.g * (1.0 - a),
        top.b * a + under.b * (1.0 - a),
        1.0,
    )
}

fn luminance(c: Rgba) -> f64 {
    let lin = |x: f64| {
        let x = x / 255.0;
        if x <= 0.04045 {
            x / 12.92
        } else {
            ((x + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(c.r) + 0.7152 * lin(c.g) + 0.0722 * lin(c.b)
}

/// WCAG's contrast ratio of text of colour `fg` on the opaque background `bg`, the text's alpha
/// composited on the background first.
pub fn contrast(fg: Rgba, bg: Rgba) -> f64 {
    let (l1, l2) = (luminance(composite(fg, bg)), luminance(bg));
    let (hi, lo) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (hi + 0.05) / (lo + 0.05)
}

// ---------------------------------------------------------------- results

/// One value the page uses off its tokens.
#[derive(Clone, Debug, PartialEq)]
pub struct Offender {
    /// The first node that uses it (`tag#id.class "text"`).
    pub node: String,
    /// The value: `18px`, `#2e7d32`, a ratio.
    pub value: String,
    /// Where it is used: the properties of a colour, `gap` or `padding`, `full` for a radius, the
    /// colours and limit of a contrast; empty when there is nothing to add.
    pub what: String,
}

/// An output that ran in one viewport and mode.
#[derive(Clone, Debug, PartialEq)]
pub struct Judged {
    /// The count of distinct values off the tokens, or the lowest contrast ratio.
    pub value: f64,
    /// The values off the tokens, or the text below its contrast limit; none when it passes.
    pub offenders: Vec<Offender>,
    /// Text whose background is not known (contrast only), not judged.
    pub unjudged: i64,
}

/// An output of one viewport and mode: judged, or not run and why.
pub type Outcome = Result<Judged, String>;

/// The token outputs of one viewport and mode, in the order of [`OUTPUTS`].
#[derive(Clone, Debug)]
pub struct TokenUnit {
    pub viewport: String,
    pub mode: String,
    pub outcomes: Vec<Outcome>,
}

// ---------------------------------------------------------------- the probe's answer

struct Use {
    node: usize,
    count: i64,
}

/// A custom property as the root holds it: its name, its colour (an index into `colours`), its text.
type Property = (String, Option<usize>, String);

struct Raw {
    properties: Vec<(String, Vec<Property>)>,
    colours: Vec<Rgba>,
    nodes: Vec<String>,
    sizes: Vec<(f64, Use)>,
    usage: Vec<(usize, String, Use)>,
    text: Vec<(usize, Option<usize>, f64, f64, Use)>,
    gaps: Vec<(f64, String, Use)>,
    radii: Vec<(f64, bool, Use)>,
}

fn rows<'a>(v: &'a Value, key: &str) -> Result<&'a [Value], String> {
    match v.get(key) {
        Some(Value::List(l)) => Ok(l),
        _ => Err(format!("tokens: no `{key}` list in the recording")),
    }
}

fn num(row: &[Value], i: usize, what: &str) -> Result<f64, String> {
    row.get(i)
        .and_then(f64_of)
        .ok_or_else(|| format!("tokens: a row of `{what}` has no number at {i}"))
}

fn index(row: &[Value], i: usize, len: usize, what: &str) -> Result<Option<usize>, String> {
    let n = num(row, i, what)?;
    if n < 0.0 {
        return Ok(None);
    }
    let n = n as usize;
    if n >= len {
        return Err(format!(
            "tokens: a row of `{what}` names an entry that is not listed"
        ));
    }
    Ok(Some(n))
}

fn text_at(row: &[Value], i: usize, what: &str) -> Result<String, String> {
    row.get(i)
        .and_then(Value::str)
        .map(str::to_string)
        .ok_or_else(|| format!("tokens: a row of `{what}` has no text at {i}"))
}

impl Raw {
    fn from_value(v: &Value) -> Result<Raw, String> {
        let colours = rows(v, "colours")?
            .iter()
            .map(|c| {
                let l = c.list();
                Ok(Rgba::new(
                    num(l, 0, "colours")?,
                    num(l, 1, "colours")?,
                    num(l, 2, "colours")?,
                    num(l, 3, "colours")?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let nodes = rows(v, "nodes")?
            .iter()
            .map(|n| {
                n.str()
                    .map(str::to_string)
                    .ok_or_else(|| "tokens: a node is not text".to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let (nc, nn) = (colours.len(), nodes.len());
        let used = |l: &[Value], at: usize, what: &str| -> Result<Use, String> {
            Ok(Use {
                count: num(l, at, what)? as i64,
                node: index(l, at + 1, nn, what)?
                    .ok_or_else(|| format!("tokens: a row of `{what}` names no node"))?,
            })
        };
        let mut properties = Vec::new();
        for p in rows(v, "properties")? {
            let l = p.list();
            let pattern = text_at(l, 0, "properties")?;
            let mut found = Vec::new();
            for e in l.get(1).map(Value::list).unwrap_or_default() {
                let e = e.list();
                found.push((
                    text_at(e, 0, "properties")?,
                    index(e, 1, nc, "properties")?,
                    text_at(e, 2, "properties")?,
                ));
            }
            properties.push((pattern, found));
        }
        let sizes = rows(v, "sizes")?
            .iter()
            .map(|r| {
                let l = r.list();
                Ok((num(l, 0, "sizes")?, used(l, 1, "sizes")?))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let usage = rows(v, "usage")?
            .iter()
            .map(|r| {
                let l = r.list();
                Ok((
                    index(l, 0, nc, "usage")?.ok_or("tokens: a use names no colour")?,
                    text_at(l, 1, "usage")?,
                    used(l, 2, "usage")?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let text = rows(v, "text")?
            .iter()
            .map(|r| {
                let l = r.list();
                Ok((
                    index(l, 0, nc, "text")?.ok_or("tokens: a text names no colour")?,
                    index(l, 1, nc, "text")?,
                    num(l, 2, "text")?,
                    num(l, 3, "text")?,
                    used(l, 4, "text")?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let gaps = rows(v, "gaps")?
            .iter()
            .map(|r| {
                let l = r.list();
                Ok((
                    num(l, 0, "gaps")?,
                    text_at(l, 1, "gaps")?,
                    used(l, 2, "gaps")?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let radii = rows(v, "radii")?
            .iter()
            .map(|r| {
                let l = r.list();
                let full = matches!(l.get(1), Some(Value::Bool(true)));
                Ok((num(l, 0, "radii")?, full, used(l, 2, "radii")?))
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Raw {
            properties,
            colours,
            nodes,
            sizes,
            usage,
            text,
            gaps,
            radii,
        })
    }
}

/// A length as text: `18px`, `12.8px`.
fn px(x: f64) -> String {
    format!("{}px", decimal(x, 2))
}

/// The distinct values of `items` (by `key`, in the order first seen) that `on` refuses, each with
/// the first node that uses it and the `what` of every use joined by `, `.
fn distinct_off<T>(
    items: &[T],
    nodes: &[String],
    key: impl Fn(&T) -> String,
    on: impl Fn(&T) -> bool,
    what: impl Fn(&T) -> String,
    node: impl Fn(&T) -> usize,
) -> Judged {
    let mut off: Vec<(String, Offender)> = Vec::new();
    for item in items.iter().filter(|i| !on(i)) {
        let k = key(item);
        let w = what(item);
        match off.iter_mut().find(|(seen, _)| *seen == k) {
            Some((_, o)) => {
                if !w.is_empty() && !o.what.split(", ").any(|x| x == w) {
                    if !o.what.is_empty() {
                        o.what.push_str(", ");
                    }
                    o.what.push_str(&w);
                }
            }
            None => off.push((
                k.clone(),
                Offender {
                    node: nodes[node(item)].clone(),
                    value: k,
                    what: w,
                },
            )),
        }
    }
    Judged {
        value: off.len() as f64,
        offenders: off.into_iter().map(|(_, o)| o).collect(),
        unjudged: 0,
    }
}

/// The colours the tokens name in this mode: the custom properties as the page's root holds them,
/// and the declared values. An explicit property that is not a colour there, or a `--prefix*` that
/// matches no colour, leaves the colours unknown.
fn colour_tokens(c: &Colours, raw: &Raw) -> Result<Vec<Rgba>, String> {
    let mut out = c.values.clone();
    for pattern in &c.properties {
        let found = raw
            .properties
            .iter()
            .find(|(p, _)| p == pattern)
            .map(|(_, f)| f.as_slice())
            .ok_or_else(|| format!("`{pattern}` was not read"))?;
        if pattern.ends_with('*') {
            let colours: Vec<Rgba> = found
                .iter()
                .filter_map(|(_, c, _)| c.map(|i| raw.colours[i]))
                .collect();
            if colours.is_empty() {
                return Err(format!(
                    "no custom property `{pattern}` on the page's root holds a colour"
                ));
            }
            out.extend(colours);
        } else {
            match found.first() {
                Some((_, Some(i), _)) => out.push(raw.colours[*i]),
                Some((_, None, text)) if !text.is_empty() => {
                    return Err(format!(
                        "`{pattern}` on the page's root is `{text}`, not a colour"
                    ));
                }
                _ => {
                    return Err(format!("the page's root does not define `{pattern}`"));
                }
            }
        }
    }
    Ok(out)
}

/// The token outputs of one viewport and mode from the probe's answer, in the order of
/// [`OUTPUTS`]; an output whose tokens are undeclared or unknown is not run.
pub fn judge(t: &Tokens, recorded: &Value) -> Result<Vec<Outcome>, String> {
    let raw = Raw::from_value(recorded)?;
    let nodes = &raw.nodes;
    let type_scale = match &t.font_sizes {
        None => Err("the tokens declare no font sizes".to_string()),
        Some(scale) => Ok(distinct_off(
            &raw.sizes,
            nodes,
            |(s, _)| px(*s),
            |(s, _)| size_on(scale, *s),
            |_| String::new(),
            |(_, u)| u.node,
        )),
    };
    let colours = match &t.colours {
        None => Err("the tokens declare no colours".to_string()),
        Some(c) => colour_tokens(c, &raw).map(|declared| {
            distinct_off(
                &raw.usage,
                nodes,
                |(c, _, _)| raw.colours[*c].hex(),
                |(c, _, _)| declared.iter().any(|d| colour_matches(raw.colours[*c], *d)),
                |(_, p, _)| p.clone(),
                |(_, _, u)| u.node,
            )
        }),
    };
    let spacing = match t.spacing_unit_px {
        None => Err("the tokens declare no spacing unit".to_string()),
        Some(unit) => Ok(distinct_off(
            &raw.gaps,
            nodes,
            |(g, _, _)| px(*g),
            |(g, _, _)| multiple_of(*g, unit),
            |(_, k, _)| k.clone(),
            |(_, _, u)| u.node,
        )),
    };
    let radii = match &t.radii {
        None => Err("the tokens declare no radii".to_string()),
        Some(list) => Ok(distinct_off(
            &raw.radii,
            nodes,
            |(r, _, _)| px(*r),
            |(r, full, _)| {
                *r < RADIUS_TOLERANCE_PX
                    || list.iter().any(|d| match d {
                        Radius::Full => *full,
                        Radius::Px(x) => (x - r).abs() <= RADIUS_TOLERANCE_PX,
                    })
            },
            |(_, full, _)| {
                if *full {
                    "full".to_string()
                } else {
                    String::new()
                }
            },
            |(_, _, u)| u.node,
        )),
    };
    let mut lowest: Option<f64> = None;
    let mut unjudged = 0;
    let mut offenders = Vec::new();
    for (fg, bg, size, weight, u) in &raw.text {
        let Some(bg) = bg else {
            unjudged += u.count;
            continue;
        };
        let (fg, bg) = (raw.colours[*fg], raw.colours[*bg]);
        let ratio = (contrast(fg, bg) * 100.0).floor() / 100.0;
        let large = *size >= 24.0 || (*size >= 18.66 && *weight >= 700.0);
        let limit = if large {
            CONTRAST_LARGE
        } else {
            CONTRAST_NORMAL
        };
        lowest = Some(lowest.map_or(ratio, |l| l.min(ratio)));
        if ratio < limit {
            offenders.push(Offender {
                node: nodes[u.node].clone(),
                value: decimal(ratio, 2),
                what: format!(
                    "{} on {}, limit {}",
                    composite(fg, bg).hex(),
                    bg.hex(),
                    decimal(limit, 1)
                ),
            });
        }
    }
    let contrast = match lowest {
        Some(value) => Ok(Judged {
            value,
            offenders,
            unjudged,
        }),
        None if unjudged > 0 => Err(format!(
            "no text with a known background ({unjudged} on a background image or on no background colour)"
        )),
        None => Err("the page draws no text".to_string()),
    };
    Ok(vec![type_scale, colours, spacing, radii, contrast])
}

/// The custom properties the probe reads: the declared ones, or none.
pub fn properties(t: &Tokens) -> Vec<String> {
    t.colours
        .as_ref()
        .map(|c| c.properties.clone())
        .unwrap_or_default()
}
