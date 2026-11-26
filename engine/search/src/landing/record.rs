//! What a campaign records: the environments, the journeys, and the measurements file
//! `ux-search-landing/1`, one line of numbers per variant and environment.

use super::patch::{self, Value};

/// The format of the measurements file.
pub const FORMAT: &str = "ux-search-landing/1";

/// The page the campaign measures, as the built site serves it.
pub const PAGE: &str = "/uxui/";

/// One environment: a viewport and a colour scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Environment {
    pub width: u32,
    pub height: u32,
    pub mode: &'static str,
}

impl Environment {
    /// `1440x900 light`.
    pub fn name(&self) -> String {
        format!("{}x{} {}", self.width, self.height, self.mode)
    }
}

const fn environment(width: u32, height: u32, mode: &'static str) -> Environment {
    Environment {
        width,
        height,
        mode,
    }
}

/// The six environments, in the order a variant's renders are recorded.
pub const ENVIRONMENTS: [Environment; 6] = [
    environment(1440, 900, "light"),
    environment(1440, 900, "dark"),
    environment(1024, 768, "light"),
    environment(1024, 768, "dark"),
    environment(390, 844, "light"),
    environment(390, 844, "dark"),
];

/// The width of the viewport tap targets are counted at.
pub const TAP_WIDTH: u32 = 390;

/// One journey: the pages that end it, and the pages whose sidebar links to them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Journey {
    pub name: &'static str,
    /// The site paths that end the journey.
    pub targets: &'static [&'static str],
    /// Site paths of pages whose sidebar links to a target: a route through one costs one
    /// action more, and on a narrow viewport one more to open the sidebar.
    pub via: &'static [&'static str],
}

/// The three journeys of the worked example.
pub const JOURNEYS: [Journey; 3] = [
    Journey {
        name: "install the plugin",
        targets: &["/docs/getting-started"],
        via: &[],
    },
    Journey {
        name: "what exists today",
        targets: &["/docs/status"],
        via: &[],
    },
    Journey {
        name: "find the metric catalogue",
        targets: &["/docs/reference/metrics"],
        via: &[
            "/docs/reference/skill",
            "/docs/reference/specification",
            "/docs/reference/glossary",
        ],
    },
];

/// The cheapest route of one journey in one render.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Route {
    /// One per click, one per screenful scrolled to reach the link.
    pub actions: u64,
    /// The regions (navbar, hero, each section, footer) that start at or above the link.
    pub regions: u64,
}

/// What one render of one variant in one environment measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Render {
    /// The primary action lies fully in the viewport without scrolling.
    pub primary_in_view: bool,
    /// The page scrolls horizontally.
    pub h_scroll: bool,
    /// Visible texts under WCAG 2.x 1.4.3: 4.5:1, 3:1 for large text.
    pub contrast_failures: u64,
    /// Visible interactive elements without an accessible name.
    pub unnamed: u64,
    /// Sections and cards of the built page that the variant does not show.
    pub content_missing: u64,
    /// Words of visible text above the primary action.
    pub words: u64,
    /// The top of the primary action, in page px.
    pub primary_px: u64,
    /// Visible links and buttons of the landing body under 44 px in width or height.
    pub small_targets: u64,
    /// The cheapest route of each of [`JOURNEYS`]; `None` when there is none.
    pub journeys: [Option<Route>; 3],
}

/// The names of a render's numbers, in the order the measurements file writes them.
pub const COLUMNS: [&str; 14] = [
    "primary_in_view",
    "h_scroll",
    "contrast_failures",
    "unnamed",
    "content_missing",
    "words",
    "primary_px",
    "small_targets",
    "install_actions",
    "install_regions",
    "status_actions",
    "status_regions",
    "metrics_actions",
    "metrics_regions",
];

/// A recorded campaign.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recorded {
    /// The adapter and its version, `ux-browser 0.10.0`.
    pub adapter: String,
    /// The browser's product, `Chrome/149.0.7827.55`.
    pub browser: String,
    /// `sha256:<hex>` of the `product.json` the variants patch.
    pub product_digest: String,
    /// `sha256:<hex>` of the `landing.css` the variants patch.
    pub css_digest: String,
    /// Each variant's id and its renders, one per environment in the order of
    /// [`ENVIRONMENTS`].
    pub variants: Vec<(String, Vec<Render>)>,
}

impl Render {
    /// The render's numbers in the order of [`COLUMNS`]; a journey without a route is two
    /// `None`.
    fn row(&self) -> [Option<u64>; 14] {
        let mut row = [
            Some(u64::from(self.primary_in_view)),
            Some(u64::from(self.h_scroll)),
            Some(self.contrast_failures),
            Some(self.unnamed),
            Some(self.content_missing),
            Some(self.words),
            Some(self.primary_px),
            Some(self.small_targets),
            None,
            None,
            None,
            None,
            None,
            None,
        ];
        for (at, route) in self.journeys.iter().enumerate() {
            row[8 + 2 * at] = route.map(|route| route.actions);
            row[9 + 2 * at] = route.map(|route| route.regions);
        }
        row
    }

    fn from_row(row: &[Option<u64>]) -> Result<Render, String> {
        let row: &[Option<u64>; 14] = row
            .try_into()
            .map_err(|_| format!("a render has {} numbers, not {}", row.len(), COLUMNS.len()))?;
        let number = |at: usize| row[at].ok_or_else(|| format!("`{}` is null", COLUMNS[at]));
        let flag = |at: usize| match row[at] {
            Some(0) => Ok(false),
            Some(1) => Ok(true),
            _ => Err(format!("`{}` is not 0 or 1", COLUMNS[at])),
        };
        let route = |at: usize| match (row[at], row[at + 1]) {
            (Some(actions), Some(regions)) => Ok(Some(Route { actions, regions })),
            (None, None) => Ok(None),
            _ => Err(format!(
                "`{}` and `{}` are not both numbers or both null",
                COLUMNS[at],
                COLUMNS[at + 1]
            )),
        };
        Ok(Render {
            primary_in_view: flag(0)?,
            h_scroll: flag(1)?,
            contrast_failures: number(2)?,
            unnamed: number(3)?,
            content_missing: number(4)?,
            words: number(5)?,
            primary_px: number(6)?,
            small_targets: number(7)?,
            journeys: [route(8)?, route(10)?, route(12)?],
        })
    }
}

fn quoted(text: &str) -> String {
    patch::compact(&Value::Str(text.to_owned()))
}

fn list(names: impl IntoIterator<Item = String>) -> String {
    let quoted: Vec<String> = names.into_iter().map(|name| quoted(&name)).collect();
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

fn environment_names() -> Vec<String> {
    ENVIRONMENTS.iter().map(Environment::name).collect()
}

fn journey_names() -> Vec<String> {
    JOURNEYS
        .iter()
        .map(|journey| journey.name.to_owned())
        .collect()
}

fn column_names() -> Vec<String> {
    COLUMNS.iter().map(|&column| column.to_owned()).collect()
}

impl Recorded {
    /// Reads a measurements file. A file of another format, page, environments, journeys or
    /// columns is refused: its numbers mean something else.
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
        if field("page")? != PAGE {
            return Err(format!("`page` is not `{PAGE}`"));
        }
        for (key, expected) in [
            ("environments", environment_names()),
            ("journeys", journey_names()),
            ("columns", column_names()),
        ] {
            if names(&value, key)? != expected {
                return Err(format!("`{key}` is not {}", list(expected)));
            }
        }
        let source = |key: &str| {
            value
                .get("sources")
                .and_then(|sources| sources.get_str(key))
                .map(str::to_owned)
                .ok_or_else(|| format!("no text `sources.{key}`"))
        };
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
                let numbers: Vec<Option<u64>> = row
                    .list()
                    .iter()
                    .map(|number| match number {
                        Value::Null => Ok(None),
                        Value::Int(n) => u64::try_from(*n)
                            .map(Some)
                            .map_err(|_| format!("{id}: {n} is below 0")),
                        other => Err(format!(
                            "{id}: {} is not a whole number or null",
                            patch::compact(other)
                        )),
                    })
                    .collect::<Result<_, _>>()?;
                renders.push(Render::from_row(&numbers).map_err(|why| format!("{id}: {why}"))?);
            }
            variants.push((id.to_owned(), renders));
        }
        Ok(Recorded {
            adapter: field("adapter")?,
            browser: field("browser")?,
            product_digest: source("product.json")?,
            css_digest: source("landing.css")?,
            variants,
        })
    }

    /// The measurements file: one line per render, the numbers in the order of [`COLUMNS`].
    /// The same record gives the same bytes.
    pub fn text(&self) -> String {
        let mut out = String::from("{\n");
        for (key, value) in [
            ("format", FORMAT),
            ("adapter", self.adapter.as_str()),
            ("browser", self.browser.as_str()),
            ("page", PAGE),
        ] {
            out.push_str(&format!("  {}: {},\n", quoted(key), quoted(value)));
        }
        out.push_str(&format!(
            "  \"sources\": {{\"product.json\": {}, \"landing.css\": {}}},\n",
            quoted(&self.product_digest),
            quoted(&self.css_digest)
        ));
        for (key, value) in [
            ("environments", environment_names()),
            ("journeys", journey_names()),
            ("columns", column_names()),
        ] {
            out.push_str(&format!("  {}: {},\n", quoted(key), list(value)));
        }
        out.push_str("  \"variants\": [\n");
        for (at, (id, renders)) in self.variants.iter().enumerate() {
            out.push_str(&format!("    {{\"id\": {}, \"renders\": [\n", quoted(id)));
            for (row, render) in renders.iter().enumerate() {
                let numbers: Vec<String> = render
                    .row()
                    .iter()
                    .map(|number| number.map_or("null".to_owned(), |n| n.to_string()))
                    .collect();
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
        out.push_str("  ]\n}\n");
        out
    }
}

/// `sha256:<hex>` of the bytes, as the measurements file names a source.
pub fn digest(bytes: &[u8]) -> String {
    uxui_doc::sha256::hex(bytes)
}
