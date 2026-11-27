//! The items of `docs/ess-gaps.md` that the ESS release `ux-search` builds on closes, each held
//! by a test on the crates of [`ux_search::ESS_TAG`] (`story:track-ess-gaps`): an item is marked
//! closed there only with a test here that fails on a release without it.

use std::path::{Path, PathBuf};

use ux_search::drive::{self, Target};

/// The studio's document, in the repository.
fn studio() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ess/ui/studio.ui.yaml")
}

/// The x axis of every chart the terminal draws on the studio's live-climb page, from `text` as
/// the document, in drawing order: `<first> … <last>` of its line `<kind> · seq: <first> … <last>`.
fn chart_axes(test: &str, text: &str) -> Vec<String> {
    let path = studio();
    let base = path.parent().expect("the document's directory");
    let target = Target {
        page: "campaigns.climb".to_owned(),
        params: vec![("campaign_id".to_owned(), "cp-01".to_owned())],
    };
    let app = drive::open(text, base, &crate::state_dir(test), &target)
        .unwrap_or_else(|error| panic!("{error}"));
    app.render_text(120, 200)
        .lines()
        .filter_map(|line| line.split_once(" · seq: "))
        .map(|(_, axis)| axis.split('│').next().unwrap_or_default().trim().to_owned())
        .collect()
}

/// The studio's document with `sort` on both of its charts (`hill`, then `progress`).
fn sorted(text: &str, sort: &str) -> String {
    let x = "\n        x: seq\n";
    assert_eq!(text.matches(x).count(), 2, "the two charts' `x: seq`");
    text.replace(x, &format!("\n        x: seq\n        sort: {sort}\n"))
}

/// Item 6, chart order (F1): a chart takes `sort: {by, dir}` and a live insert keeps the order.
/// The page's two line charts read rows with `seq` 1, 2, 3; at time zero the climb channel
/// inserts `seq` 4 into `progress` (`insert_or_patch`), which the terminal draws first as
/// written, and last under `sort: {by: seq, dir: asc}`.
#[test]
fn item_6_a_chart_draws_its_rows_in_the_order_its_sort_declares() {
    let text = std::fs::read_to_string(studio()).expect("the studio document");
    assert_eq!(
        chart_axes("ess-gaps-chart-as-written", &text),
        ["1 … 3", "4 … 3"],
        "hill and progress as written: the live insert drawn first"
    );
    assert_eq!(
        chart_axes("ess-gaps-chart-asc", &sorted(&text, "{by: seq, dir: asc}")),
        ["1 … 3", "1 … 4"],
        "both charts with `sort: {{by: seq, dir: asc}}`"
    );
    assert_eq!(
        chart_axes(
            "ess-gaps-chart-desc",
            &sorted(&text, "{by: seq, dir: desc}")
        ),
        ["3 … 1", "4 … 1"],
        "both charts with `sort: {{by: seq, dir: desc}}`"
    );
}
