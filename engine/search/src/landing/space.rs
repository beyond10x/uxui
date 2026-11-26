//! The design space of the landing page (`docs/targets.md`, the worked example): six axes, 144
//! variants, each a patch to the page's two sources. Every axis value is written here and
//! nowhere else.
//!
//! | axis | source | values (the shipped one marked) |
//! |---|---|---|
//! | theme | `landing.css`, the docs-system colour tokens | `docs-system` (shipped): none set; `studio`: [`STUDIO_PALETTE`] |
//! | hero | `landing.css` `--uxui-hero-align`, `--uxui-hero-lede-inline` | `left` (shipped): `start`, `0`; `centred`: `center`, `auto` |
//! | actions | `product.json` `/product/actions` | `1+3`: two links added; `1+1` (shipped); `1+0`: the link removed |
//! | section order | `product.json` `/sections` | `features-first` (shipped); `status-first`: the status section moved to the top |
//! | density | `landing.css` `--uxui-density` | `0.85`, `1` (shipped), `1.15` |
//! | heading scale | `landing.css` `--uxui-heading-scale` | `0.85` (shipped), `1` |

use super::patch::{self, CssRule, Op, Value};

/// One axis: its values in enumeration order and which of them the page ships.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Axis {
    /// The axis as the report names it.
    pub name: &'static str,
    /// Each value as a variant id writes it.
    pub ids: &'static [&'static str],
    /// Each value as the report's header writes it.
    pub labels: &'static [&'static str],
    /// The index of the value the page ships.
    pub shipped: usize,
}

const THEME: usize = 0;
const HERO: usize = 1;
const ACTIONS: usize = 2;
const SECTION_ORDER: usize = 3;
const DENSITY: usize = 4;
const HEADING_SCALE: usize = 5;

/// The axes, the first the most significant in enumeration order.
pub const AXES: [Axis; 6] = [
    Axis {
        name: "theme",
        ids: &["docs-system", "studio"],
        labels: &["docs-system", "studio"],
        shipped: 0,
    },
    Axis {
        name: "hero",
        ids: &["left", "centred"],
        labels: &["left", "centred"],
        shipped: 0,
    },
    Axis {
        name: "actions",
        ids: &["1+3", "1+1", "1+0"],
        labels: &["1+3", "1+1", "1+0"],
        shipped: 1,
    },
    Axis {
        name: "section order",
        ids: &["features-first", "status-first"],
        labels: &["features-first", "status-first"],
        shipped: 0,
    },
    Axis {
        name: "density",
        ids: &["d85", "d100", "d115"],
        labels: &["0.85", "1", "1.15"],
        shipped: 1,
    },
    Axis {
        name: "heading scale",
        ids: &["h85", "h100"],
        labels: &["0.85", "1"],
        shipped: 0,
    },
];

/// The rule of `landing.css` that holds the page's own custom properties.
pub const ROOT: &str = ":root";
/// The rule the studio palette's light colours are written to: the root element of a page that
/// holds the landing page, above the docs-system's own `html[data-theme=...]` rules.
pub const PALETTE_LIGHT: &str = "html:root:has(.b10x-product)";
/// The rule the studio palette's dark colours are written to, after [`PALETTE_LIGHT`].
pub const PALETTE_DARK: &str = "html[data-theme='dark']:has(.b10x-product)";

/// The hero's two custom properties per value of the hero axis: alignment, the lede's inline
/// margin.
const HERO_VALUES: [(&str, &str); 2] = [("start", "0"), ("center", "auto")];

/// The links beside the primary action that `1+3` adds before the shipped one, as the page's
/// first version had them (commit 1b330f3): label and target.
const ADDED_ACTIONS: [(&str, &str); 2] = [
    ("Run one round", "/docs/guides/run-a-round"),
    ("The search engine design", "/docs/concepts/search-engine"),
];
/// The target of the link the page ships beside the primary action, at `/product/actions/1`.
const SHIPPED_LINK: &str = "/docs/status";
/// Where the status section is in the shipped `/sections`.
const SHIPPED_STATUS_AT: usize = 2;

/// A colour of the studio palette: the docs-system token it is written to, the variable of
/// `themes/studio.css` it comes from, and that variable's value in light and in dark mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Colour {
    pub token: &'static str,
    pub studio: &'static str,
    pub light: &'static str,
    pub dark: &'static str,
}

const fn colour(
    token: &'static str,
    studio: &'static str,
    light: &'static str,
    dark: &'static str,
) -> Colour {
    Colour {
        token,
        studio,
        light,
        dark,
    }
}

/// The studio palette on the docs-system colour tokens: every `--b10x-color-*` token the
/// docs-system theme sets, each from the nearest role of `themes/studio.css`. The studio has
/// one line colour, one accent and no strong accent, so `line` and `hairline` share one value,
/// and `link`, `focus`, `accent` and `accent-strong` another.
pub const STUDIO_PALETTE: [Colour; 22] = [
    colour("--b10x-color-canvas", "--ui-color-bg", "#f4f5f9", "#0e1020"),
    colour(
        "--b10x-color-surface",
        "--ui-color-surface",
        "#ffffff",
        "#171a2e",
    ),
    colour(
        "--b10x-color-surface-muted",
        "--ui-color-focus",
        "#eeeefc",
        "#23264a",
    ),
    colour(
        "--b10x-color-panel",
        "--ui-color-surface",
        "#ffffff",
        "#171a2e",
    ),
    colour(
        "--b10x-color-heading",
        "--ui-color-text",
        "#161a2b",
        "#e7e9f5",
    ),
    colour("--b10x-color-text", "--ui-color-text", "#161a2b", "#e7e9f5"),
    colour(
        "--b10x-color-muted",
        "--ui-color-muted",
        "#646b82",
        "#9aa0bd",
    ),
    colour("--b10x-color-line", "--ui-color-line", "#e3e6ef", "#2a2e48"),
    colour(
        "--b10x-color-hairline",
        "--ui-color-line",
        "#e3e6ef",
        "#2a2e48",
    ),
    colour(
        "--b10x-color-link",
        "--ui-color-accent",
        "#5b5bd6",
        "#8b8bff",
    ),
    colour(
        "--b10x-color-focus",
        "--ui-color-accent",
        "#5b5bd6",
        "#8b8bff",
    ),
    colour(
        "--b10x-color-accent",
        "--ui-color-accent",
        "#5b5bd6",
        "#8b8bff",
    ),
    colour(
        "--b10x-color-accent-strong",
        "--ui-color-accent",
        "#5b5bd6",
        "#8b8bff",
    ),
    colour(
        "--b10x-color-on-accent",
        "--ui-color-on_accent",
        "#ffffff",
        "#0e1020",
    ),
    colour(
        "--b10x-color-amber",
        "--ui-color-warning",
        "#b26b00",
        "#f0b55a",
    ),
    colour(
        "--b10x-color-rose",
        "--ui-color-danger",
        "#c63d4b",
        "#ff7a85",
    ),
    colour("--b10x-color-sky", "--ui-color-info", "#3a6fd8", "#7aa5ff"),
    colour(
        "--b10x-color-success",
        "--ui-color-success",
        "#1f8a5b",
        "#4cc38a",
    ),
    colour(
        "--b10x-color-warning",
        "--ui-color-warning",
        "#b26b00",
        "#f0b55a",
    ),
    colour(
        "--b10x-color-danger",
        "--ui-color-danger",
        "#c63d4b",
        "#ff7a85",
    ),
    colour("--b10x-color-cyan", "--ui-color-info", "#3a6fd8", "#7aa5ff"),
    colour(
        "--b10x-color-lime",
        "--ui-color-success",
        "#1f8a5b",
        "#4cc38a",
    ),
];

/// One variant: a value of every axis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variant {
    /// The axis values' ids joined by `.`, `docs-system.left.1+1.features-first.d100.h85`.
    pub id: String,
    /// The index of the value chosen on each axis, in the order of [`AXES`].
    pub choice: [usize; 6],
}

/// A variant's change to the two sources.
#[derive(Debug, Clone, PartialEq)]
pub struct Patch {
    /// RFC 6902 operations on `website/product.json`; none for the values the page ships.
    pub product: Vec<Op>,
    /// The custom properties of `website/src/css/landing.css`, every one the variant sets.
    pub css: Vec<CssRule>,
}

/// Every variant, in enumeration order: the last axis changes fastest.
pub fn variants() -> Vec<Variant> {
    let mut choices = vec![[0usize; 6]];
    for (at, axis) in AXES.iter().enumerate() {
        choices = choices
            .into_iter()
            .flat_map(|choice| {
                (0..axis.ids.len()).map(move |value| {
                    let mut next = choice;
                    next[at] = value;
                    next
                })
            })
            .collect();
    }
    choices.into_iter().map(variant).collect()
}

/// The variant the page ships.
pub fn shipped() -> Variant {
    variant(std::array::from_fn(|at| AXES[at].shipped))
}

fn variant(choice: [usize; 6]) -> Variant {
    let ids: Vec<&str> = AXES
        .iter()
        .zip(choice)
        .map(|(axis, value)| axis.ids[value])
        .collect();
    Variant {
        id: ids.join("."),
        choice,
    }
}

fn text(text: &str) -> Value {
    Value::Str(text.to_owned())
}

fn test(path: String, value: &str) -> Op {
    Op::Test {
        path,
        value: text(value),
    }
}

impl Variant {
    /// Whether this is the page as shipped.
    pub fn is_shipped(&self) -> bool {
        *self == shipped()
    }

    /// The label of the value chosen on the axis at `at`, as the report's header writes it.
    fn label(&self, at: usize) -> &'static str {
        AXES[at].labels[self.choice[at]]
    }

    /// The variant's patch. The operations on `product.json` are made for the page as shipped:
    /// each group starts with a `test` of what it changes, so other sources refuse it.
    pub fn patch(&self) -> Patch {
        let mut product = Vec::new();
        let link = || test("/product/actions/1/href".to_owned(), SHIPPED_LINK);
        match AXES[ACTIONS].ids[self.choice[ACTIONS]] {
            "1+3" => {
                product.push(link());
                for (at, (label, href)) in ADDED_ACTIONS.iter().enumerate() {
                    product.push(Op::Add {
                        path: format!("/product/actions/{}", at + 1),
                        value: Value::Map(vec![
                            ("label".to_owned(), text(label)),
                            ("href".to_owned(), text(href)),
                        ]),
                    });
                }
            }
            "1+0" => {
                product.push(link());
                product.push(Op::Remove {
                    path: "/product/actions/1".to_owned(),
                });
            }
            _ => {}
        }
        if AXES[SECTION_ORDER].ids[self.choice[SECTION_ORDER]] == "status-first" {
            product.push(test(
                format!("/sections/{SHIPPED_STATUS_AT}/kind"),
                "status",
            ));
            product.push(Op::Move {
                from: format!("/sections/{SHIPPED_STATUS_AT}"),
                path: "/sections/0".to_owned(),
            });
        }

        let (align, lede_inline) = HERO_VALUES[self.choice[HERO]];
        let pairs = |values: &[(&str, &str)]| -> Vec<(String, String)> {
            values
                .iter()
                .map(|(property, value)| ((*property).to_owned(), (*value).to_owned()))
                .collect()
        };
        let mut css = vec![CssRule {
            selector: ROOT.to_owned(),
            values: pairs(&[
                ("--uxui-density", self.label(DENSITY)),
                ("--uxui-heading-scale", self.label(HEADING_SCALE)),
                ("--uxui-hero-align", align),
                ("--uxui-hero-lede-inline", lede_inline),
            ]),
        }];
        if AXES[THEME].ids[self.choice[THEME]] == "studio" {
            let palette = |selector: &str, value: fn(&Colour) -> &'static str| CssRule {
                selector: selector.to_owned(),
                values: STUDIO_PALETTE
                    .iter()
                    .map(|colour| (colour.token.to_owned(), value(colour).to_owned()))
                    .collect(),
            };
            css.push(palette(PALETTE_LIGHT, |colour| colour.light));
            css.push(palette(PALETTE_DARK, |colour| colour.dark));
        }
        Patch { product, css }
    }
}

impl Patch {
    /// The two sources with the patch applied: the text of `product.json` and of `landing.css`.
    /// `product.json` is read and written again, so it must be in the form [`patch::pretty`]
    /// writes; a file in another form is refused, since the patch would rewrite all of it.
    pub fn apply(&self, product: &str, css: &str) -> Result<(String, String), String> {
        let mut document = patch::parse(product).map_err(|why| format!("product.json: {why}"))?;
        if patch::pretty(&document) != product {
            return Err(
                "product.json is not written as JSON.stringify(value, null, 2) writes it, with a \
                 last newline; a patch would rewrite lines it does not change"
                    .to_owned(),
            );
        }
        patch::apply_ops(&mut document, &self.product)
            .map_err(|why| format!("product.json: {why}"))?;
        let css = patch::set_custom_properties(css, &self.css)
            .map_err(|why| format!("landing.css: {why}"))?;
        Ok((patch::pretty(&document), css))
    }

    /// The patched `product.json` as a value, for a renderer that reads it.
    pub fn product(&self, product: &str) -> Result<Value, String> {
        let mut document = patch::parse(product).map_err(|why| format!("product.json: {why}"))?;
        patch::apply_ops(&mut document, &self.product)
            .map_err(|why| format!("product.json: {why}"))?;
        Ok(document)
    }
}
