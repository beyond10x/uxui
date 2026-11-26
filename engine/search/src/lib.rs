//! ux-search: the engine's first search, on layout model v0.
//!
//! Layout model v0 is ESS's terminal renderer, driven in process (`ess_ui_tui::App` at the ESS
//! tag [`ESS_TAG`]): a variant document is opened from its text, one page is opened, the screen
//! is drawn into a test backend of a given size, and the cells each node occupies are read back
//! as regions. None of that touches a terminal, a browser or the network.
//!
//! A second backend is the browser adapter (`ux_browser`, `engine/browser`): [`landing`] runs
//! one campaign over the documentation landing page in Chromium and ranks its record with
//! [`rank`], and [`campaign`] runs a campaign whose page, environments, axes and terms a
//! declaration file names, on a server somebody else started. Only `landing::run` and
//! `campaign::run` start a browser.
//!
//! - [`variant`]: the design space as text: a page's sections in another order, the theme
//!   chosen; the section orders as permutations.
//! - [`drive`]: open a document at a page and read the regions of one drawing.
//! - [`rank`]: lexicographic ranking and the Pareto frontier over a variant's terms.
//! - [`climb`]: the exhaustive climb over theme x section order (`ux-search climb`).
//! - [`cost`]: the cost of one evaluation per UI state (`ux-search cost`).
//! - [`landing`]: the campaign over the documentation landing page on the browser adapter
//!   (`ux-search landing run`, `ux-search landing report`).
//! - [`campaign`]: a declared campaign on the browser adapter, over custom properties of any
//!   served page (`ux-search campaign run`, `ux-search campaign report`).

pub mod campaign;
pub mod climb;
pub mod cost;
pub mod drive;
pub mod landing;
pub mod rank;
pub mod variant;

/// The ESS release the terminal renderer is taken from: the git tag of the ESS crates in this
/// crate's `Cargo.toml`. The reports name it in their first line; a test holds the manifest, the
/// lock file and the documents that name the tag to it (`tests/ess_tag.rs`).
pub const ESS_TAG: &str = "0.99.5";
