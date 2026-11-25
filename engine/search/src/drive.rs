//! Opening a document at a page on ESS's terminal renderer, and reading where one drawing put
//! each node.

use std::path::Path;
use std::time::Duration;

use ess_ui_tui::{App, Options};

/// The largest side of a drawing, ESS's `ScreenSize::MAX`.
pub const MAX_SIDE: u16 = 1000;

/// A page to open, with its params.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    /// The page's name under `pages:`.
    pub page: String,
    /// Its params, `(name, value)`, in the order given.
    pub params: Vec<(String, String)>,
}

impl Target {
    /// The params as `name=value`, joined by `&`; `-` when there are none.
    pub fn params_text(&self) -> String {
        if self.params.is_empty() {
            return "-".to_owned();
        }
        self.params
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("&")
    }
}

/// The screen cells a node occupies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cells {
    /// Column of the left edge.
    pub x: u16,
    /// Row of the top edge.
    pub y: u16,
    /// Columns.
    pub width: u16,
    /// Rows.
    pub height: u16,
}

/// One region of a drawing: a node's canonical path, its row key and its cells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placed {
    /// The node's canonical path, as ESS writes it (`pages/<page>/sections/<name>`).
    pub path: String,
    /// The row key, for a row and its cells.
    pub row: Option<String>,
    /// Where it was drawn.
    pub cells: Cells,
}

/// Loads `text` (its fixtures resolved against `base`, file-placed state under `state_dir`),
/// opens `target` as its route does, and answers every read due at time zero.
pub fn open(text: &str, base: &Path, state_dir: &Path, target: &Target) -> Result<App, String> {
    let mut app = App::from_text(text, base, Options::new(state_dir.to_path_buf()))
        .map_err(|error| format!("the document does not open: {error}"))?;
    if !app.page_names().contains(&target.page) {
        return Err(format!("the document has no page `{}`", target.page));
    }
    let params: Vec<(&str, &str)> = target
        .params
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_str()))
        .collect();
    app.open_page(&target.page, &params);
    app.advance(Duration::ZERO);
    if app.page() != target.page {
        return Err(format!(
            "page `{}` did not open; the app shows `{}`",
            target.page,
            app.page()
        ));
    }
    Ok(app)
}

/// Draws the app into a `width` x `height` test backend and returns every region the drawing
/// recorded, in drawing order.
pub fn draw(app: &App, width: u16, height: u16) -> Vec<Placed> {
    let _screen = app.render_text(width, height);
    app.regions()
        .into_iter()
        .map(|region| Placed {
            path: region.path,
            row: region.row,
            cells: Cells {
                x: region.area.x,
                y: region.area.y,
                width: region.area.width,
                height: region.area.height,
            },
        })
        .collect()
}

/// A size written `<width>x<height>`, each side from 1 to [`MAX_SIDE`].
pub fn parse_size(text: &str) -> Option<(u16, u16)> {
    let (width, height) = text.split_once('x')?;
    let side = |text: &str| {
        Some(text)
            .filter(|text| !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit()))
            .and_then(|text| text.parse::<u16>().ok())
            .filter(|side| (1..=MAX_SIDE).contains(side))
    };
    Some((side(width)?, side(height)?))
}

/// Sizes written `<width>x<height>`, separated by commas.
pub fn parse_sizes(text: &str) -> Option<Vec<(u16, u16)>> {
    text.split(',').map(parse_size).collect()
}
