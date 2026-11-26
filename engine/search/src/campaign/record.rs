//! What a declared campaign records: the measurements file
//! `ux-search-campaign-measurements/2`, one line of numbers per variant and environment.
//!
//! The file holds the declaration it was measured for, as its text and that text's digest, so
//! a report needs no other file; and the digest of everything else it holds, so a file whose
//! numbers were changed after the run is refused. Because it holds the declaration's text, the
//! file of a page that is private is as private as the declaration.
//!
//! Beside the numbers it holds what a reader needs to judge them: where the pointer rested in
//! each environment (`pointer`), per axis the number of values that change the page at all
//! (`axes`), and, for the page as shipped and for the frontier, the first elements behind each
//! count (`notes` of a variant).

use super::declaration::{Declaration, Environment};
use crate::landing::patch::{self, Value};

pub use crate::landing::record::digest;

/// The format of the measurements file.
pub const FORMAT: &str = "ux-search-campaign-measurements/2";

/// The format before the pointer was parked: a file of it is refused, with the reason.
const FORMAT_BEFORE_THE_POINTER: &str = "ux-search-campaign-measurements/1";

/// The file of a run restricted to one variant or one environment (`campaign run --variant`,
/// `--environment`): its renders, which are no campaign. The report refuses it.
pub const RENDERS_FORMAT: &str = "ux-search-campaign-renders/1";

/// The counts whose first elements a run names, in the order a variant's notes are written in.
pub const NOTED: [&str; 4] = [
    "low_contrast",
    "overflow",
    "small_targets",
    "always_visible_misses",
];

/// The most elements one note names.
pub const NOTE_ELEMENTS: usize = 3;

/// The first elements behind one count of one render, in document order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    /// The index of the environment among [`Declaration::environments`].
    pub environment: usize,
    /// The count, one of [`NOTED`].
    pub term: String,
    /// What the count counts first: at most [`NOTE_ELEMENTS`] elements, each as a short path
    /// with the start of its text and what was measured on it.
    pub elements: Vec<String>,
}

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
    /// Per axis, in the declaration's order: how many of its values beside the shipped one
    /// change a box or a computed style on the page (the ones the probe's digest reads,
    /// `probes/campaign.js`), or a measured number, in any environment and beside any choice of
    /// the other axes. 0: no value of the axis changes this page.
    pub changing: Vec<u64>,
    /// Per variant, in the order of `variants`: the first elements behind its counts. A run
    /// keeps them for the page as shipped and for the frontier; every other variant has none.
    pub notes: Vec<Vec<Note>>,
}

/// A render's count by its column's name.
fn counted(render: &Render, column: &str) -> u64 {
    COLUMNS
        .iter()
        .position(|&name| name == column)
        .map_or(0, |at| render.row()[at])
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

/// Where the pointer rested in each environment, as the file writes it: `[[x,y],[x,y]]`.
fn pointers(declaration: &Declaration) -> String {
    let each: Vec<String> = declaration
        .environments()
        .iter()
        .map(|environment| {
            let (x, y) = declaration.pointer(environment);
            format!("[{x},{y}]")
        })
        .collect();
    format!("[{}]", each.join(","))
}

fn whole(value: Option<&Value>, what: &str) -> Result<u64, String> {
    match value {
        Some(Value::Int(n)) => u64::try_from(*n).map_err(|_| format!("{what}: {n} is below 0")),
        _ => Err(format!("{what}: not a whole number")),
    }
}

/// The notes of one variant of the file.
fn notes_of(item: &Value, id: &str) -> Result<Vec<Note>, String> {
    let items = match item.get("notes") {
        None => return Ok(Vec::new()),
        Some(Value::List(items)) => items,
        Some(_) => return Err(format!("{id}: `notes` is not a list")),
    };
    items
        .iter()
        .map(|note| {
            let environment = whole(note.get("environment"), "a note's `environment`")
                .and_then(|at| usize::try_from(at).map_err(|error| error.to_string()))
                .map_err(|why| format!("{id}: {why}"))?;
            let term = note
                .get_str("term")
                .ok_or_else(|| format!("{id}: a note has no text `term`"))?;
            Ok(Note {
                environment,
                term: term.to_owned(),
                elements: names(note, "elements").map_err(|why| format!("{id}: a note: {why}"))?,
            })
        })
        .collect()
}

impl Recorded {
    /// Whether the record is of its declaration: every variant of the space, in its order,
    /// each with one render per environment; one number per axis, at most the values the axis
    /// has beside the shipped one; and notes only of counts a render has.
    pub fn check(&self) -> Result<(), String> {
        self.check_variants()?;
        let axes = &self.declaration.axes;
        if self.changing.len() != axes.len() {
            return Err(format!(
                "the record holds {} axes, the declaration {}",
                self.changing.len(),
                axes.len()
            ));
        }
        for (axis, &changing) in axes.iter().zip(&self.changing) {
            let others = axis.values.len().saturating_sub(1);
            if usize::try_from(changing).map_or(true, |changing| changing > others) {
                return Err(format!(
                    "axis `{}`: {changing} values change the page, of {others} beside the \
                     shipped one",
                    axis.name
                ));
            }
        }
        if self.notes.len() != self.variants.len() {
            return Err(format!(
                "the record holds the notes of {} variants, and {} variants",
                self.notes.len(),
                self.variants.len()
            ));
        }
        for ((id, renders), notes) in self.variants.iter().zip(&self.notes) {
            for note in notes {
                let Some(render) = renders.get(note.environment) else {
                    return Err(format!(
                        "{id}: a note of environment {}, of {}",
                        note.environment,
                        renders.len()
                    ));
                };
                if !NOTED.contains(&note.term.as_str()) {
                    return Err(format!(
                        "{id}: a note of `{}`, which is none of {}",
                        note.term,
                        NOTED.join(", ")
                    ));
                }
                let count = counted(render, &note.term);
                let named = note.elements.len();
                if named == 0
                    || named > NOTE_ELEMENTS
                    || u64::try_from(named).map_or(true, |named| named > count)
                {
                    return Err(format!(
                        "{id}: a note names {named} elements of `{}`, which counts {count} in \
                         that environment; a note names 1 to {NOTE_ELEMENTS} of them",
                        note.term
                    ));
                }
            }
        }
        Ok(())
    }

    fn check_variants(&self) -> Result<(), String> {
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
    /// its digest names or that does not read, other environments, pointer positions, columns
    /// or axes than the declaration's and the catalogue's, other variants than the
    /// declaration's, and content that is not what the file's digest names.
    pub fn parse(text: &str) -> Result<Recorded, String> {
        let value = patch::parse(text)?;
        let field = |key: &str| {
            value
                .get_str(key)
                .map(str::to_owned)
                .ok_or_else(|| format!("no text `{key}`"))
        };
        let format = field("format")?;
        if format == FORMAT_BEFORE_THE_POINTER {
            return Err(format!(
                "`format` is `{FORMAT_BEFORE_THE_POINTER}`: a file of it was measured with the \
                 pointer wherever the browser left it, and says nothing of the axes; run the \
                 campaign again for `{FORMAT}`"
            ));
        }
        if format == RENDERS_FORMAT {
            return Err(format!(
                "`format` is `{RENDERS_FORMAT}`: the renders of a restricted run (`--variant`, \
                 `--environment`), not a campaign; the report ranks the measurements of a whole \
                 one, `{FORMAT}`"
            ));
        }
        if format != FORMAT {
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
        let pointer = pointers(&declaration);
        if value.get("pointer").map(patch::compact).as_deref() != Some(pointer.as_str()) {
            return Err(format!(
                "`pointer` is not {pointer}, where the declaration rests the pointer in each \
                 environment"
            ));
        }
        let Some(Value::List(axes)) = value.get("axes") else {
            return Err("no list `axes`".to_owned());
        };
        let named: Vec<&str> = axes
            .iter()
            .map(|axis| axis.get_str("name").unwrap_or_default())
            .collect();
        let declared: Vec<&str> = declaration
            .axes
            .iter()
            .map(|axis| axis.name.as_str())
            .collect();
        if named != declared {
            return Err(format!(
                "`axes` does not name the declaration's axes, {}",
                declared.join(", ")
            ));
        }
        let changing = axes
            .iter()
            .zip(&declared)
            .map(|(axis, name)| whole(axis.get("changing"), &format!("`axes`: `{name}`")))
            .collect::<Result<Vec<u64>, String>>()?;
        let Some(Value::List(items)) = value.get("variants") else {
            return Err("no list `variants`".to_owned());
        };
        let mut variants = Vec::with_capacity(items.len());
        let mut notes = Vec::with_capacity(items.len());
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
            notes.push(notes_of(item, id)?);
        }
        let recorded = Recorded {
            adapter: field("adapter")?,
            browser: field("browser")?,
            declaration,
            variants,
            changing,
            notes,
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
        let axes: Vec<String> = self
            .declaration
            .axes
            .iter()
            .zip(&self.changing)
            .map(|(axis, changing)| {
                format!(
                    "{{\"name\": {}, \"changing\": {changing}}}",
                    quoted(&axis.name)
                )
            })
            .collect();
        let environments: Vec<usize> = (0..self.declaration.environments().len()).collect();
        body_of(
            &Head {
                format: FORMAT,
                adapter: &self.adapter,
                browser: &self.browser,
                declaration: &self.declaration,
                environments: &environments,
                between: format!("  \"axes\": [{}],\n", axes.join(", ")),
            },
            &self.variants,
            &self.notes,
        )
    }

    /// `sha256:<hex>` of the file's content before its `digest` member.
    pub fn digest(&self) -> String {
        digest(self.body().as_bytes())
    }

    /// The measurements file: one line per render, the numbers in the order of [`COLUMNS`],
    /// and the digest of all of it as the last member. The same record gives the same bytes.
    pub fn text(&self) -> String {
        with_digest(self.body())
    }
}

/// What a file of renders holds before its variants.
struct Head<'a> {
    format: &'a str,
    adapter: &'a str,
    browser: &'a str,
    declaration: &'a Declaration,
    /// The environments measured, as indices of [`Declaration::environments`].
    environments: &'a [usize],
    /// The members between the columns and the variants, each a line of its own.
    between: String,
}

/// A file of renders without its digest: the head, then one line per render and the notes of
/// each variant.
fn body_of(head: &Head, variants: &[(String, Vec<Render>)], notes: &[Vec<Note>]) -> String {
    let all = head.declaration.environments();
    let environments: Vec<&Environment> = head.environments.iter().map(|&at| &all[at]).collect();
    let mut out = String::from("{\n");
    for (key, value) in [
        ("format", head.format),
        ("adapter", head.adapter),
        ("browser", head.browser),
    ] {
        out.push_str(&format!("  {}: {},\n", quoted(key), quoted(value)));
    }
    out.push_str(&format!(
        "  \"declaration\": {{\"digest\": {}, \"text\": {}}},\n",
        quoted(&head.declaration.digest),
        quoted(&head.declaration.text)
    ));
    let names: Vec<String> = environments
        .iter()
        .map(|environment| environment.name())
        .collect();
    out.push_str(&format!("  \"environments\": {},\n", list(&names)));
    let pointer: Vec<String> = environments
        .iter()
        .map(|environment| {
            let (x, y) = head.declaration.pointer(environment);
            format!("[{x},{y}]")
        })
        .collect();
    out.push_str(&format!("  \"pointer\": [{}],\n", pointer.join(",")));
    out.push_str(&format!("  \"columns\": {},\n", list(&column_names())));
    out.push_str(&head.between);
    out.push_str("  \"variants\": [\n");
    for (at, (id, renders)) in variants.iter().enumerate() {
        out.push_str(&format!("    {{\"id\": {}, \"renders\": [\n", quoted(id)));
        for (row, render) in renders.iter().enumerate() {
            let numbers: Vec<String> = render.row().iter().map(u64::to_string).collect();
            let comma = if row + 1 < renders.len() { "," } else { "" };
            out.push_str(&format!("      [{}]{comma}\n", numbers.join(",")));
        }
        let notes = notes.get(at).map(Vec::as_slice).unwrap_or_default();
        if !notes.is_empty() {
            out.push_str("    ], \"notes\": [\n");
            for (row, note) in notes.iter().enumerate() {
                let comma = if row + 1 < notes.len() { "," } else { "" };
                out.push_str(&format!(
                    "      {{\"environment\": {}, \"term\": {}, \"elements\": {}}}{comma}\n",
                    note.environment,
                    quoted(&note.term),
                    list(&note.elements)
                ));
            }
        }
        let comma = if at + 1 < variants.len() { "," } else { "" };
        out.push_str(&format!("    ]}}{comma}\n"));
    }
    out.push_str("  ],\n");
    out
}

/// A file's body with the digest of it as the last member.
fn with_digest(body: String) -> String {
    let digest = digest(body.as_bytes());
    format!("{body}  \"digest\": {}\n}}\n", quoted(&digest))
}

/// The renders of a run restricted to one variant, one environment or both
/// ([`RENDERS_FORMAT`]): the measurements file's members for the renders it made, and what it was
/// restricted to. No report ranks it: a campaign is every variant in every environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Renders {
    pub adapter: String,
    pub browser: String,
    pub declaration: Declaration,
    /// The variant the run was restricted to, by id; `None`: every variant.
    pub variant: Option<String>,
    /// The environment the run was restricted to, by name; `None`: every environment.
    pub environment: Option<String>,
    /// The environments measured, as indices of [`Declaration::environments`].
    pub environments: Vec<usize>,
    /// Each variant measured: its id and its renders, one per measured environment.
    pub variants: Vec<(String, Vec<Render>)>,
    /// Per variant: the first elements behind its counts; `environment` is an index of
    /// [`Declaration::environments`].
    pub notes: Vec<Vec<Note>>,
}

impl Renders {
    /// The file: like the measurements file, with `restricted` in place of `axes`.
    pub fn text(&self) -> String {
        let named = |name: &Option<String>| name.as_deref().map_or("null".to_owned(), quoted);
        with_digest(body_of(
            &Head {
                format: RENDERS_FORMAT,
                adapter: &self.adapter,
                browser: &self.browser,
                declaration: &self.declaration,
                environments: &self.environments,
                between: format!(
                    "  \"restricted\": {{\"variant\": {}, \"environment\": {}}},\n",
                    named(&self.variant),
                    named(&self.environment)
                ),
            },
            &self.variants,
            &self.notes,
        ))
    }
}
