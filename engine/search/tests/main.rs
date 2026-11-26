//! The one integration-test binary of ux-search.
//!
//! `fixtures/partner-portal/` is ESS's example `examples/partner-portal` at the tag `ux-search`
//! builds on, `ux_search::ESS_TAG` (its `README.md` gives each file's SHA-256; the module
//! `ess_tag` holds every place that names the tag to that one). `fixtures/partner-portal.frontier.txt` is the climb's
//! own output over it, written by `ux-search climb` (the command is in that test's message).
//! `fixtures/studio.frontier.txt` is the same over the studio, `ess/ui/studio.ui.yaml` of this
//! repository (`docs/studio-climb.md`). `fixtures/docs-landing/` is the campaign over the
//! documentation landing page on the browser adapter: its sources, its recorded measurements and
//! the report over them (the module `landing` names each file;
//! `docs/campaign-2026-11-26-docs-landing-engine.md`).
//!
//! Budget: no test runs longer than 60 s; a climb over the fixture is 18 variants, each checked,
//! opened once and drawn four times; the studio climb is 120 such variants, run once. The module
//! `landing` starts no browser and runs no campaign: it reads the recorded one.

mod climb;
mod cost;
mod ess_tag;
mod landing;
mod landing_adversary;
mod rank;
mod studio;
mod studio_adversary;
mod variant;

use std::path::{Path, PathBuf};

/// The example document every test that drives the renderer opens.
pub fn portal() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/partner-portal/ui.yaml")
}

/// The page the tests climb and measure, and the ticket it shows.
pub fn ticket_page() -> ux_search::drive::Target {
    ux_search::drive::Target {
        page: "tickets.detail".to_owned(),
        params: vec![("id".to_owned(), "tk-01".to_owned())],
    }
}

/// A state directory of the test's own under cargo's scratch directory for integration tests;
/// nothing is written to it unless a key sets file-placed state.
pub fn state_dir(test: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ux-search")
        .join(test)
}
