//! What a declared campaign records: the measurements file
//! `ux-search-campaign-measurements/1`, one line of numbers per variant and environment.
//!
//! The file holds the declaration it was measured for, as its text and that text's digest, so
//! a report needs no other file; and the digest of everything else it holds, so a file whose
//! numbers were changed after the run is refused.

use super::declaration::{Declaration, Environment};
use crate::landing::patch::{self, Value};

pub use crate::landing::record::digest;

/// The format of the measurements file.
pub const FORMAT: &str = "ux-search-campaign-measurements/1";

/// What one render of one variant in one environment measured: every number of the catalogue,
/// whether or not the declaration ranks by it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Render {
    /// The primary action lies fully in view without scrolling: in the viewport and in every box
    /// around it that clips or scrolls.
    pub primary_in_view: bool,
    /// The page scrolls horizontally.
    pub h_scroll: bool,
    /// Visible interactive elements without an accessible name.
    pub unnamed: u64,
    /// The top of the primary action, in page px.
    pub primary_px: u64,
    /// Visible links and buttons under `small_target_px` in width or height.
    pub small_targets: u64,
    /// The visible regions that start at or above the primary action.
    pub regions: u64,
    /// The visible text runs: text nodes with something else than white space.
    pub text_runs: u64,
    /// Those of them under 14 px.
    pub text_under_14: u64,
    /// Those of them under WCAG 2.x 1.4.3: 4.5:1, 3:1 for large text.
    pub low_contrast: u64,
    /// Visible texts and replaced elements that end to the right of the viewport or of the box
    /// that clips or scrolls them.
    pub overflow: u64,
    /// The items of `always_visible` not fully in the viewport at the top of the page, or not
    /// at its end.
    pub always_visible_misses: u64,
}

/// The names of a render's numbers, in the order the measurements file writes them.
pub const COLUMNS: [&str; 11] = [
    "primary_in_view",
    "h_scroll",
    "unnamed",
    "primary_px",
    "small_targets",
    "regions",
    "text_runs",
    "text_under_14",
    "low_contrast",
    "overflow",
    "always_visible_misses",
];

impl Render {
    /// The render's numbers in the order of [`COLUMNS`].
    pub fn row(&self) -> [u64; 11] {
        [
            u64::from(self.primary_in_view),
            u64::from(self.h_scroll),
            self.unnamed,
            self.primary_px,
            self.small_targets,
            self.regions,
            self.text_runs,
            self.text_under_14,
            self.low_contrast,
            self.overflow,
            self.always_visible_misses,
        ]
    }

    fn from_row(row: &[u64]) -> Result<Render, String> {
        let row: &[u64; 11] = row
            .try_into()
            .map_err(|_| format!("a render has {} numbers, not {}", row.len(), COLUMNS.len()))?;
        let flag = |at: usize| match row[at] {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(format!("`{}` is not 0 or 1", COLUMNS[at])),
        };
        Ok(Render {
            primary_in_view: flag(0)?,
            h_scroll: flag(1)?,
            unnamed: row[2],
            primary_px: row[3],
            small_targets: row[4],
            regions: row[5],
            text_runs: row[6],
            text_under_14: row[7],
            low_contrast: row[8],
            overflow: row[9],
            always_visible_misses: row[10],
        })
    }
}

/// A recorded campaign.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recorded {
    /// The adapter and its version, `ux-browser 0.10.0`.
    pub adapter: String,
    /// The browser's product, `HeadlessChrome/149.0.7827.55`.
    pub browser: String,
    /// The declaration the campaign was measured for.
    pub declaration: Declaration,
    /// Each variant's id and its renders, one per environment in the order of
    /// [`Declaration::environments`].
    pub variants: Vec<(String, Vec<Render>)>,
}

fn quoted(text: &str) -> String {
    patch::compact(&Value::Str(text.to_owned()))
}

fn list(names: &[String]) -> String {
    let quoted: Vec<String> = names.iter().map(|name| quoted(name)).collect();
    format!("[{}]", quoted.join(", "))
}

fn names(value: &Value, key: &str) -> Result<Vec<String>, String> {
    match value.get(key) {
        Some(Value::List(items)) => items
            .iter()
            .map(|item| {
                item.str()
                    .map(str::to_owned)
                    .ok_or_else(|| format!("`{key}` holds a value that is not text"))
            })
            .collect(),
        _ => Err(format!("no list `{key}`")),
    }
}

fn environment_names(declaration: &Declaration) -> Vec<String> {
    declaration
        .environments()
        .iter()
        .map(Environment::name)
        .collect()
}

fn column_names() -> Vec<String> {
    COLUMNS.iter().map(|&column| column.to_owned()).collect()
}

impl Recorded {
    /// Whether the record is of its declaration: every variant of the space, in its order,
    /// each with one render per environment.
    pub fn check(&self) -> Result<(), String> {
        let variants = self.declaration.variants();
        if self.variants.len() != variants.len() {
            return Err(format!(
                "the record holds {} variants, the declaration {}",
                self.variants.len(),
                variants.len()
            ));
        }
        let environments = self.declaration.environments().len();
        for (variant, (id, renders)) in variants.iter().zip(&self.variants) {
            if *id != variant.id {
                return Err(format!(
                    "the record holds `{id}` where the declaration has `{}`",
                    variant.id
                ));
            }
            if renders.len() != environments {
                return Err(format!(
                    "{id}: {} renders, not one per environment ({environments})",
                    renders.len()
                ));
            }
        }
        Ok(())
    }

    /// Reads a measurements file. Refused: another format, a declaration that is not the text
    /// its digest names or that does not read, other environments or columns than the
    /// declaration's and the catalogue's, other variants than the declaration's, and content
    /// that is not what the file's digest names.
    pub fn parse(text: &str) -> Result<Recorded, String> {
        let value = patch::parse(text)?;
        let field = |key: &str| {
            value
                .get_str(key)
                .map(str::to_owned)
                .ok_or_else(|| format!("no text `{key}`"))
        };
        if field("format")? != FORMAT {
            return Err(format!("`format` is not `{FORMAT}`"));
        }
        let declared = |key: &str| {
            value
                .get("declaration")
                .and_then(|declaration| declaration.get_str(key))
                .ok_or_else(|| format!("no text `declaration.{key}`"))
        };
        let source = declared("text")?;
        if declared("digest")? != digest(source.as_bytes()) {
            return Err(
                "`declaration.digest` is not the digest of `declaration.text`: the declaration \
                 was changed after the campaign ran"
                    .to_owned(),
            );
        }
        let declaration = Declaration::parse(source)
            .map_err(|why| format!("`declaration.text` is no declaration: {why}"))?;
        for (key, expected) in [
            ("environments", environment_names(&declaration)),
            ("columns", column_names()),
        ] {
            if names(&value, key)? != expected {
                return Err(format!("`{key}` is not {}", list(&expected)));
            }
        }
        let Some(Value::List(items)) = value.get("variants") else {
            return Err("no list `variants`".to_owned());
        };
        let mut variants = Vec::with_capacity(items.len());
        for item in items {
            let id = item.get_str("id").ok_or("a variant has no text `id`")?;
            let Some(Value::List(rows)) = item.get("renders") else {
                return Err(format!("{id}: no list `renders`"));
            };
            let mut renders = Vec::with_capacity(rows.len());
            for row in rows {
                let numbers: Vec<u64> = row
                    .list()
                    .iter()
                    .map(|number| match number {
                        Value::Int(n) => {
                            u64::try_from(*n).map_err(|_| format!("{id}: {n} is below 0"))
                        }
                        other => Err(format!(
                            "{id}: {} is not a whole number",
                            patch::compact(other)
                        )),
                    })
                    .collect::<Result<_, _>>()?;
                renders.push(Render::from_row(&numbers).map_err(|why| format!("{id}: {why}"))?);
            }
            variants.push((id.to_owned(), renders));
        }
        let recorded = Recorded {
            adapter: field("adapter")?,
            browser: field("browser")?,
            declaration,
            variants,
        };
        recorded.check()?;
        if field("digest")? != recorded.digest() {
            return Err(
                "`digest` is not the digest of what the file holds: it was changed after the \
                 campaign wrote it"
                    .to_owned(),
            );
        }
        Ok(recorded)
    }

    /// The file without its last member, the digest: what the digest is taken of.
    fn body(&self) -> String {
        let mut out = String::from("{\n");
        for (key, value) in [
            ("format", FORMAT),
            ("adapter", self.adapter.as_str()),
            ("browser", self.browser.as_str()),
        ] {
            out.push_str(&format!("  {}: {},\n", quoted(key), quoted(value)));
        }
        out.push_str(&format!(
            "  \"declaration\": {{\"digest\": {}, \"text\": {}}},\n",
            quoted(&self.declaration.digest),
            quoted(&self.declaration.text)
        ));
        for (key, value) in [
            ("environments", environment_names(&self.declaration)),
            ("columns", column_names()),
        ] {
            out.push_str(&format!("  {}: {},\n", quoted(key), list(&value)));
        }
        out.push_str("  \"variants\": [\n");
        for (at, (id, renders)) in self.variants.iter().enumerate() {
            out.push_str(&format!("    {{\"id\": {}, \"renders\": [\n", quoted(id)));
            for (row, render) in renders.iter().enumerate() {
                let numbers: Vec<String> = render.row().iter().map(u64::to_string).collect();
                let comma = if row + 1 < renders.len() { "," } else { "" };
                out.push_str(&format!("      [{}]{comma}\n", numbers.join(",")));
            }
            let comma = if at + 1 < self.variants.len() {
                ","
            } else {
                ""
            };
            out.push_str(&format!("    ]}}{comma}\n"));
        }
        out.push_str("  ],\n");
        out
    }

    /// `sha256:<hex>` of the file's content before its `digest` member.
    pub fn digest(&self) -> String {
        digest(self.body().as_bytes())
    }

    /// The measurements file: one line per render, the numbers in the order of [`COLUMNS`],
    /// and the digest of all of it as the last member. The same record gives the same bytes.
    pub fn text(&self) -> String {
        format!(
            "{}  \"digest\": {}\n}}\n",
            self.body(),
            quoted(&self.digest())
        )
    }
}
