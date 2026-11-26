//! A declared campaign on the browser adapter: the page, the environments, the axes and the
//! terms come from a declaration file, so the page may be any page a server serves, of this
//! repository or of another.
//!
//! - [`declaration`]: the declaration `ux-search-campaign/1` and what it refuses; the variants
//!   it spans and the style sheet of custom properties each one injects.
//! - [`run`]: every variant in every environment in Chromium, and the checks that keep a
//!   render honest (`ux-search campaign run`).
//! - [`record`]: the measurements file `ux-search-campaign-measurements/2`, which holds its
//!   declaration and the digest of its content.
//! - [`report`]: the terms, the ranking, the frontier and the page as shipped among them
//!   (`ux-search campaign report`), ranked with [`crate::rank`].
//!
//! Only `run::run` starts a browser. The landing campaign ([`crate::landing`]) stays what it
//! is: its space and its probe are compiled in for one page.

pub mod declaration;
pub mod record;
pub mod report;
pub mod run;
