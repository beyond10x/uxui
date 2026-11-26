//! The documentation landing page as a search target on the browser adapter: the worked example
//! of `docs/targets.md`, the page `/uxui/` built from `website/`.
//!
//! A variant is a patch to the page's two sources: RFC 6902 operations on `website/product.json`
//! and values of the custom properties of `website/src/css/landing.css`. The campaign renders
//! every variant in every environment in Chromium through `ux_browser`, records what it
//! measured, and ranks the record. Only [`run`] starts a browser; everything else is text in,
//! text out.
//!
//! - [`patch`]: RFC 6902 on a JSON value, the JSON text `product.json` is written in, and the
//!   custom properties of a style sheet.
//! - [`space`]: the six axes, the 144 variants and each one's patch.
//! - [`record`]: the environments, the journeys and the measurements file
//!   (`ux-search-landing/1`).
//! - [`promote`]: one variant of the frontier written to the two sources, with its evidence
//!   document (`ux-search landing promote`).
//! - [`report`]: the terms of a variant, the ranking, the frontier and the report
//!   (`ux-search landing report`).
//! - [`run`]: the campaign in Chromium (`ux-search landing run`).

pub mod patch;
pub mod promote;
pub mod record;
pub mod report;
pub mod run;
pub mod space;
