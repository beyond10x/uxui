//! The climb: every theme x every section order of one page, each variant checked, opened fresh
//! and drawn on layout model v0, ranked, and printed as a frontier and a full ranking.
//!
//! The section orders of a page without `layout: columns:` are the orders of its `sections:`
//! (the first `cap` permuted, [`SECTION_CAP`] by default). On a page with columns they are the
//! product of each column's orders (the first `cap` of each column permuted): a variant writes
//! each column's `sections: [...]` in its order and the page's `sections:` column by column,
//! the sections no column lists after them as written. The terminal renderer draws no columns
//! (ESS 0.99.9 lacks `no_columns` and stacks the page's `sections:` in the order written), so
//! the stack it draws is the columns read one after another, as the variant lists them.
//!
//! The terms, in ranking order, each a count where lower is better:
//!
//! 1. `errors`: the error findings of `ess_ui_check::check_source` on the variant's text.
//! 2. `below_fold`: at the narrow size (the smallest width given), the regions of the page that
//!    the drawing at that size leaves out while the reference drawing shows them. The reference
//!    drawing is the same width at [`REFERENCE_HEIGHT`] rows, where nothing is cut by the frame.
//! 3. `cut`: summed over every size, the regions drawn smaller than in the reference drawing of
//!    the same width: a section box or a child the frame's bottom edge cuts through.
//! 4. `primary_rows`: summed over every size, the rows above the page's primary action (its
//!    region's `y`); where the size does not draw it, its `y` in the reference drawing; where
//!    that does not draw it either, [`REFERENCE_HEIGHT`].
//!
//! The primary action ([`primary_action`]) is read from the base document once: the first
//! `button` child of tone `primary` among the page's sections in the order written, else the
//! page header's first action, else none (the term is then 0 for every variant).
//!
//! No theme term: at ESS 0.99.9 the terminal renderer reads no theme (`ess-ui-tui/src` names no
//! theme), so the drawing of two variants that differ only in theme is the same, and they tie
//! on every term; the id orders them.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::drive::{self, Cells, Placed, Target};
use crate::rank::{self, Scored};
use crate::variant::{self, Column, Columns, Sections};

/// How many of a page's sections are permuted at most: 5 sections give 120 orders.
pub const SECTION_CAP: usize = 5;

/// The height of the reference drawing, ESS's largest screen side.
pub const REFERENCE_HEIGHT: u16 = drive::MAX_SIDE;

/// The terms' names, in ranking order.
pub const TERMS: [&str; 4] = ["errors", "below_fold", "cut", "primary_rows"];

/// A term's value for a variant that does not open: worse than any drawing.
pub const UNDRAWN: u64 = u64::MAX;

/// What to climb.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// The page and its params.
    pub target: Target,
    /// The sizes each variant is drawn at, `(width, height)`.
    pub sizes: Vec<(u16, u16)>,
    /// Where file-placed state is kept; nothing the climb does sets any.
    pub state_dir: PathBuf,
    /// How many sections are permuted at most ([`SECTION_CAP`]).
    pub cap: usize,
}

/// A variant's terms, lower is better.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Terms {
    /// Error findings of the checker.
    pub errors: u64,
    /// Regions left out at the narrow size.
    pub below_fold: u64,
    /// Regions cut by the frame, over every size.
    pub cut: u64,
    /// Rows above the primary action, over every size.
    pub primary_rows: u64,
}

impl Terms {
    /// The terms in ranking order ([`TERMS`]).
    pub fn values(&self) -> Vec<u64> {
        vec![self.errors, self.below_fold, self.cut, self.primary_rows]
    }

    fn text(&self) -> String {
        TERMS
            .iter()
            .zip(self.values())
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// One evaluated variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variant {
    /// `<theme>:<section>,<section>,...`; the theme is `-` when the document has none. On a
    /// page with columns, `|` separates the columns (and the sections no column lists, last).
    pub id: String,
    /// The theme chosen, `None` when the document declares no theme.
    pub theme: Option<String>,
    /// The page's sections in the variant's order, as its `sections:` writes them.
    pub order: Vec<String>,
    /// Its terms.
    pub terms: Terms,
    /// Why it did not open, when it did not.
    pub refused: Option<String>,
}

/// A finished climb.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Climb {
    /// The document's file name.
    pub file: String,
    /// The document's `app`.
    pub app: String,
    /// The page and its params.
    pub target: Target,
    /// The sizes drawn.
    pub sizes: Vec<(u16, u16)>,
    /// The size `below_fold` is counted at.
    pub narrow: (u16, u16),
    /// The primary action's node path.
    pub primary: Option<String>,
    /// The theme axis: every key of `themes` and the `theme.default`, sorted.
    pub themes: Vec<Option<String>>,
    /// The page's sections, in the order written.
    pub sections: Vec<String>,
    /// The page's layout columns, as written; empty when it has none.
    pub columns: Vec<Column>,
    /// How many sections are permuted at most, in each column on a page with columns.
    pub cap: usize,
    /// How many of them are permuted, summed over the columns on a page with columns.
    pub permuted: usize,
    /// The number of section orders.
    pub orders: usize,
    /// Every variant, theme-major in enumeration order.
    pub variants: Vec<Variant>,
}

impl Climb {
    fn scored(&self) -> Vec<Scored> {
        self.variants
            .iter()
            .map(|variant| Scored {
                id: variant.id.clone(),
                terms: variant.terms.values(),
            })
            .collect()
    }

    /// How the section orders are made: `<n> of <m> sections permuted`, or per column.
    fn order_axis(&self) -> String {
        if self.columns.is_empty() {
            return format!(
                "{} of {} sections permuted",
                self.permuted,
                self.sections.len()
            );
        }
        let columns = self
            .columns
            .iter()
            .map(|column| {
                let count = column.sections.len();
                format!("{} {} of {count}", column.name, count.min(self.cap))
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!("per column: {columns} sections permuted")
    }

    /// The indices of [`Climb::variants`], best first.
    pub fn ranking(&self) -> Vec<usize> {
        rank::rank(&self.scored())
    }

    /// The indices of the Pareto-optimal variants, in rank order.
    pub fn frontier(&self) -> Vec<usize> {
        rank::frontier(&self.scored())
    }

    /// The climb as text: a header, the frontier, then every variant, one line each with its
    /// rank and its terms. The same climb gives the same bytes.
    pub fn render(&self) -> String {
        let sizes = self
            .sizes
            .iter()
            .map(|&size| size_text(size))
            .collect::<Vec<_>>()
            .join(" ");
        let themes = self
            .themes
            .iter()
            .map(|theme| theme.as_deref().unwrap_or("-"))
            .collect::<Vec<_>>()
            .join(", ");
        let mut out = vec![
            format!(
                "# ux-search climb: layout model v0, the terminal renderer of ESS {}, in process",
                crate::ESS_TAG
            ),
            format!(
                "# document {} (app {}), page {}, params {}",
                self.file,
                self.app,
                self.target.page,
                self.target.params_text()
            ),
            format!(
                "# sizes {sizes}; narrow {}; reference height {REFERENCE_HEIGHT}",
                size_text(self.narrow)
            ),
            format!(
                "# primary action {}",
                self.primary.as_deref().unwrap_or("none")
            ),
            format!(
                "# axes: theme {} ({themes}) x section order {} ({}) = {} variants",
                self.themes.len(),
                self.orders,
                self.order_axis(),
                self.variants.len()
            ),
            format!(
                "# terms, lexicographic, lower is better: {}; ties by variant id",
                TERMS.join(" ")
            ),
        ];
        let ranking = self.ranking();
        let place: BTreeMap<usize, usize> = ranking
            .iter()
            .enumerate()
            .map(|(position, &index)| (index, position + 1))
            .collect();
        let line = |index: usize| {
            let variant = &self.variants[index];
            let refused = variant
                .refused
                .as_ref()
                .map(|reason| format!(" refused: {reason}"))
                .unwrap_or_default();
            format!(
                "{:>4} {} {}{refused}",
                place[&index],
                variant.id,
                variant.terms.text()
            )
        };
        let frontier = self.frontier();
        out.push(format!("frontier {}", frontier.len()));
        out.extend(frontier.into_iter().map(line));
        out.push(format!("ranking {}", ranking.len()));
        out.extend(ranking.into_iter().map(line));
        let mut text = out.join("\n");
        text.push('\n');
        text
    }
}

/// Climbs `document` exhaustively over theme x section order.
pub fn climb(document: &Path, options: &Options) -> Result<Climb, String> {
    let narrow = options
        .sizes
        .iter()
        .copied()
        .min_by_key(|&(width, _)| width)
        .ok_or("no size to draw at")?;
    let text = std::fs::read_to_string(document)
        .map_err(|error| format!("cannot read {}: {error}", document.display()))?;
    let base = document.parent().unwrap_or_else(|| Path::new("."));
    let file = document
        .file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().into_owned());
    let loaded = ess_ui::load_str(&text)
        .map_err(|error| format!("{file}: the document does not load: {error}"))?;
    let page = loaded
        .pages
        .get(&options.target.page)
        .ok_or_else(|| format!("{file}: no page `{}`", options.target.page))?;
    let primary = primary_action(&options.target.page, page);
    let themes = theme_axis(&loaded);
    let page_name = &options.target.page;
    let sections = Sections::find(&text, page_name)?;
    let names: Vec<String> = sections.names().into_iter().map(str::to_owned).collect();
    let layout = Columns::find(&text, page_name)?;
    let columns = layout
        .as_ref()
        .map_or_else(Vec::new, |layout| layout.columns.clone());
    let arrangements = arrangements(&names, &columns, options.cap, page_name)?;

    let mut variants = Vec::with_capacity(themes.len() * arrangements.len());
    for theme in &themes {
        for arrangement in &arrangements {
            let mut variant_text = match &layout {
                Some(layout) => {
                    Sections::find(&layout.with_orders(&arrangement.columns), page_name)?
                        .with_order(&arrangement.order)
                }
                None => sections.with_order(&arrangement.order),
            };
            if let Some(theme) = theme {
                variant_text = variant::with_theme(&variant_text, theme)?;
            }
            let order: Vec<String> = arrangement
                .order
                .iter()
                .map(|&index| names[index].clone())
                .collect();
            let (terms, refused) = evaluate(
                &variant_text,
                &file,
                base,
                options,
                primary.as_deref(),
                narrow,
            );
            variants.push(Variant {
                id: format!("{}:{}", theme.as_deref().unwrap_or("-"), arrangement.label),
                theme: theme.clone(),
                order,
                terms,
                refused,
            });
        }
    }
    Ok(Climb {
        file,
        app: loaded.app.clone(),
        target: options.target.clone(),
        sizes: options.sizes.clone(),
        narrow,
        primary,
        themes,
        permuted: if columns.is_empty() {
            names.len().min(options.cap)
        } else {
            columns
                .iter()
                .map(|column| column.sections.len().min(options.cap))
                .sum()
        },
        sections: names,
        columns,
        cap: options.cap,
        orders: arrangements.len(),
        variants,
    })
}

/// One section order: the page's `sections:` as indices, each column's order (empty on a page
/// without columns), and the order as the variant id shows it.
struct Arrangement {
    order: Vec<usize>,
    columns: Vec<Vec<usize>>,
    label: String,
}

/// Every section order of a page whose sections are `names`: the orders of `sections:` on a
/// page without `columns`, else the product of each column's orders, the sections stacked
/// column by column and those no column lists after them as written.
fn arrangements(
    names: &[String],
    columns: &[Column],
    cap: usize,
    page: &str,
) -> Result<Vec<Arrangement>, String> {
    let label = |order: &[usize]| {
        order
            .iter()
            .map(|&index| names[index].as_str())
            .collect::<Vec<_>>()
            .join(",")
    };
    if columns.is_empty() {
        return Ok(variant::orders(names.len(), cap)
            .into_iter()
            .map(|order| Arrangement {
                label: label(&order),
                order,
                columns: Vec::new(),
            })
            .collect());
    }
    let mut placed = BTreeSet::new();
    let mut members = Vec::with_capacity(columns.len());
    for column in columns {
        let mut indices = Vec::with_capacity(column.sections.len());
        for name in &column.sections {
            let index = names
                .iter()
                .position(|written| written == name)
                .ok_or_else(|| {
                    format!(
                        "column `{}` of page `{page}` lists `{name}`, which the page's sections do \
                     not write",
                        column.name
                    )
                })?;
            if !placed.insert(index) {
                return Err(format!(
                    "page `{page}` lists section `{name}` in more than one place of its columns"
                ));
            }
            indices.push(index);
        }
        members.push(indices);
    }
    let rest: Vec<usize> = (0..names.len())
        .filter(|index| !placed.contains(index))
        .collect();
    let per_column: Vec<Vec<Vec<usize>>> = columns
        .iter()
        .map(|column| variant::orders(column.sections.len(), cap))
        .collect();
    Ok(variant::product(&per_column)
        .into_iter()
        .map(|choice| {
            let mut order = Vec::with_capacity(names.len());
            let mut parts = Vec::with_capacity(columns.len() + 1);
            for (indices, chosen) in members.iter().zip(&choice) {
                let stacked: Vec<usize> = chosen.iter().map(|&at| indices[at]).collect();
                parts.push(label(&stacked));
                order.extend(stacked);
            }
            if !rest.is_empty() {
                parts.push(label(&rest));
                order.extend(&rest);
            }
            Arrangement {
                order,
                columns: choice,
                label: parts.join("|"),
            }
        })
        .collect())
}

/// The page's primary action, as the node path its region carries: the first `button` child of
/// tone `primary` among the page's sections, else the header's first action.
pub fn primary_action(page_name: &str, page: &ess_ui::Page) -> Option<String> {
    for section in &page.sections {
        for child in &section.children {
            if let ess_ui::Body::Primitive(ess_ui::Primitive::Button(button)) = &child.body
                && button.tone == Some(ess_ui::ButtonTone::Primary)
                && let Some(name) = &child.common.name
            {
                return Some(format!(
                    "pages/{page_name}/sections/{}/children/{name}",
                    section.name
                ));
            }
        }
    }
    page.header
        .as_ref()
        .and_then(|header| header.actions.first())
        .map(|action| format!("pages/{page_name}/header/actions/{}", action.name))
}

/// Every key of `themes` and the `theme.default`, sorted; one `None` when there is neither.
fn theme_axis(document: &ess_ui::Document) -> Vec<Option<String>> {
    let mut names: BTreeSet<String> = document.themes.keys().cloned().collect();
    if let Some(choice) = &document.theme {
        names.insert(choice.default.clone());
    }
    if names.is_empty() {
        vec![None]
    } else {
        names.into_iter().map(Some).collect()
    }
}

fn evaluate(
    text: &str,
    file: &str,
    base: &Path,
    options: &Options,
    primary: Option<&str>,
    narrow: (u16, u16),
) -> (Terms, Option<String>) {
    let report =
        ess_ui_check::check_source(text, file, base, None, &ess_ui_check::Options::default());
    let errors = count(report.errors());
    let app = match drive::open(text, base, &options.state_dir, &options.target) {
        Ok(app) => app,
        Err(reason) => {
            let terms = Terms {
                errors: errors.max(1),
                below_fold: UNDRAWN,
                cut: UNDRAWN,
                primary_rows: UNDRAWN,
            };
            return (terms, Some(reason));
        }
    };
    let mut terms = Terms {
        errors,
        ..Terms::default()
    };
    for &(width, height) in &options.sizes {
        let reference = drive::draw(&app, width, REFERENCE_HEIGHT);
        let drawn = drive::draw(&app, width, height);
        let full = index(&reference);
        let shown = index(&drawn);
        terms.cut += count(
            shown
                .iter()
                .filter(|(key, cells)| {
                    full.get(*key).is_some_and(|whole| {
                        cells.height < whole.height || cells.width < whole.width
                    })
                })
                .count(),
        );
        if (width, height) == narrow {
            terms.below_fold = count(full.keys().filter(|key| !shown.contains_key(*key)).count());
        }
        if let Some(path) = primary {
            terms.primary_rows += row_of(path, &drawn)
                .or_else(|| row_of(path, &reference))
                .map_or(u64::from(REFERENCE_HEIGHT), u64::from);
        }
    }
    (terms, None)
}

/// A region's key in one drawing: its path, its row key, and which of the regions with that
/// path and row key it is, in drawing order.
type Key = (String, Option<String>, usize);

fn index(placed: &[Placed]) -> BTreeMap<Key, Cells> {
    let mut seen: BTreeMap<(String, Option<String>), usize> = BTreeMap::new();
    let mut indexed = BTreeMap::new();
    for region in placed {
        let nth = seen
            .entry((region.path.clone(), region.row.clone()))
            .or_insert(0);
        indexed.insert(
            (region.path.clone(), region.row.clone(), *nth),
            region.cells,
        );
        *nth += 1;
    }
    indexed
}

/// The row the region of `path` (no row key) starts at.
fn row_of(path: &str, placed: &[Placed]) -> Option<u16> {
    placed
        .iter()
        .find(|region| region.path == path && region.row.is_none())
        .map(|region| region.cells.y)
}

fn size_text((width, height): (u16, u16)) -> String {
    format!("{width}x{height}")
}

fn count(n: usize) -> u64 {
    u64::try_from(n).unwrap_or(u64::MAX)
}
