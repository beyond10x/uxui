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
//! `docs/campaign-2026-11-26-docs-landing-engine.md`). `fixtures/campaign/` is the declared
//! campaign: `small.campaign.yaml` uses every key of a declaration, and `studio/` is the campaign
//! over the frontier page of the generated studio app, its declaration, its recorded measurements
//! and the report over them (`docs/campaign-2026-11-26-studio-engine.md`), and `live/` is a page
//! of this repository's own with the campaign the browser lane runs on it, and a list page with
//! rows and its campaign (`rows.campaign.yaml`, `ux-search-campaign/2`).
//!
//! Budget: no test runs longer than 60 s; a climb over the fixture is 18 variants, each checked,
//! opened once and drawn four times; the studio climb is 120 such variants, run once. The module
//! `landing` starts no browser and runs no campaign: it reads the recorded one. So does the
//! module `campaign`: `tools/studio-campaign.sh` records, the tests read.
//!
//! The browser lane is the module `campaign_live`: it runs `ux-search campaign run` in Chromium
//! on `fixtures/campaign/live/`, twelve campaigns for all of its tests. The module `campaign_events` holds the live view (`--events`) without a browser, over the recorded studio campaign. Each of its tests is
//! ignored, so a plain `cargo test` and the gate, which has no browser, start none; the lane is
//! `UXUI_CHROME=<chrome> cargo test --locked --manifest-path engine/search/Cargo.toml -- --ignored campaign_live`.

mod assessor;
mod campaign;
mod campaign_events;
mod campaign_live;
mod climb;
mod cost;
mod ess_tag;
mod landing;
mod landing_adversary;
mod landing_promote;
mod promote_adversary;
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

/// Whether `text` is a bare SemVer: three numbers joined by dots, none with a leading zero.
pub fn is_semver(text: &str) -> bool {
    let parts: Vec<&str> = text.split('.').collect();
    parts.len() == 3
        && parts.iter().all(|part| {
            !part.is_empty()
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && (part.len() == 1 || !part.starts_with('0'))
        })
}

/// The version in a record's adapter line, `ux-browser <version>`: the adapter the record was
/// made with. No test compares it with the version of this build: a record says what it was
/// recorded with, and a release records nothing again.
pub fn adapter_version(line: &str) -> Option<&str> {
    line.strip_prefix("ux-browser ")
        .filter(|version| is_semver(version))
}

/// A state directory of the test's own under cargo's scratch directory for integration tests;
/// nothing is written to it unless a key sets file-placed state.
pub fn state_dir(test: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ux-search")
        .join(test)
}

/// Writes `text` as an executable at `path`, for a test that runs it. The test process never
/// opens `path` for writing: the text goes to `path.src`, and `install` copies it in a process
/// of its own. A file this process held open for writing could be inherited by a child another
/// test thread forks at that moment, and running the file then fails with "Text file busy"
/// (the same helper and reason as in `engine/browser/tests/main.rs`).
pub fn executable(path: &Path, text: &str) {
    let mut source = path.as_os_str().to_owned();
    source.push(".src");
    let source = PathBuf::from(source);
    std::fs::write(&source, text).expect("the source of an executable");
    let status = std::process::Command::new("install")
        .args(["-m", "0755"])
        .arg(&source)
        .arg(path)
        .status()
        .expect("install runs");
    assert!(
        status.success(),
        "install {} exited {status}",
        path.display()
    );
}
