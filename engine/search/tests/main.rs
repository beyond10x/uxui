//! The one integration-test binary of ux-search.
//!
//! `fixtures/partner-portal/` is ESS's example `examples/partner-portal` at tag 0.99.4 (its
//! `README.md` gives each file's SHA-256). `fixtures/partner-portal.frontier.txt` is the climb's
//! own output over it, written by `ux-search climb` (the command is in that test's message).
//!
//! Budget: no test runs longer than 60 s; a climb over the fixture is 18 variants, each checked,
//! opened once and drawn four times.

mod climb;
mod cost;
mod rank;
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
