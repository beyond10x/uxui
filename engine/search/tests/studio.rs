//! The studio climb (`story:studio-climb`): the live-climb page of the studio,
//! `ess/ui/studio.ui.yaml`, climbed over theme x section order within each layout column on
//! layout model v0 and printed as a ranked frontier.
//!
//! The page's layout has two columns, `side` (`summary`) and `main` (`hill_kpis`, `hill`,
//! `progress`, `points`, `frontier`). The terminal renderer draws no columns (ESS 0.99.9 lacks
//! `no_columns` and falls back to `stack`): it stacks the page's `sections:` in the order
//! written. A variant therefore reorders the sections inside each column and writes the page's
//! `sections:` column by column, so the stack is the columns read top to bottom, left to right.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use ux_search::climb::{self, Climb, Options};
use ux_search::drive::Target;

/// The first line of the golden file until it is generated from a run.
const PLACEHOLDER: &str = "# to be generated from a run of ux-search climb on the studio";

/// The command that writes the golden file, from the repository root. Run it again whenever
/// `ess/ui/studio.ui.yaml` or its fixtures change.
const GENERATE: &str = "engine/search/target/debug/ux-search climb --doc ess/ui/studio.ui.yaml --page campaigns.climb --param campaign_id=cp-01 --sizes 120x40,48x40 > engine/search/tests/fixtures/studio.frontier.txt";

/// The studio's document, in the repository.
fn studio() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ess/ui/studio.ui.yaml")
}

fn options(test: &str, cap: usize) -> Options {
    Options {
        target: Target {
            page: "campaigns.climb".to_owned(),
            params: vec![("campaign_id".to_owned(), "cp-01".to_owned())],
        },
        sizes: vec![(120, 40), (48, 40)],
        state_dir: crate::state_dir(test),
        cap,
    }
}

fn run(test: &str, cap: usize) -> Climb {
    climb::climb(&studio(), &options(test, cap)).unwrap_or_else(|error| panic!("{error}"))
}

/// One full climb shared by the tests that only read it.
fn shared() -> &'static Climb {
    static CLIMB: OnceLock<Climb> = OnceLock::new();
    CLIMB.get_or_init(|| run("studio-shared", climb::SECTION_CAP))
}

fn ids(climb: &Climb) -> Vec<&str> {
    climb
        .variants
        .iter()
        .map(|variant| variant.id.as_str())
        .collect()
}

/// The climb runs on the studio: every variant opens the live-climb page of campaign `cp-01`,
/// draws, and passes the checker.
#[test]
fn the_studio_climb_opens_every_variant_on_the_live_climb_page() {
    let climb = shared();
    assert_eq!(climb.app, "studio");
    assert_eq!(climb.file, "studio.ui.yaml");
    assert_eq!(
        climb.primary.as_deref(),
        Some("pages/campaigns.climb/header/actions/stop"),
        "the page has no primary button; its header's first action stands in"
    );
    for variant in &climb.variants {
        assert!(
            variant.refused.is_none(),
            "{} did not open:\n{}",
            variant.id,
            climb.render()
        );
        assert_eq!(
            variant.terms.errors,
            0,
            "{}:\n{}",
            variant.id,
            climb.render()
        );
    }
}

/// The variant count is the product of the axes: theme (1: `light`) x the orders of `side`
/// (1! = 1) x the orders of `main` (5! = 120).
#[test]
fn the_studio_variant_count_is_the_product_of_its_axes() {
    let climb = shared();
    let themes: Vec<Option<&str>> = climb.themes.iter().map(Option::as_deref).collect();
    assert_eq!(themes, [Some("light")]);
    assert_eq!(
        climb.sections,
        [
            "summary",
            "hill_kpis",
            "hill",
            "progress",
            "points",
            "frontier"
        ]
    );
    assert_eq!(climb.orders, 120, "side 1! x main 5!");
    assert_eq!(climb.variants.len(), climb.themes.len() * climb.orders);
    assert_eq!(climb.variants.len(), 120);
    let mut unique = ids(climb);
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), 120, "{:?}", ids(climb));
    for variant in &climb.variants {
        assert!(
            variant.id.starts_with("light:summary|"),
            "the side column holds `summary` alone and comes first: {}",
            variant.id
        );
        assert_eq!(variant.order.first().map(String::as_str), Some("summary"));
        assert_eq!(variant.order.len(), 6, "{}", variant.id);
    }
    assert!(
        climb.render().contains(
            "# axes: theme 1 (light) x section order 120 (per column: side 1 of 1, main 5 of 5 \
             sections permuted) = 120 variants\n"
        ),
        "{}",
        climb.render()
    );
}

/// The search is deterministic on the studio: two climbs in one process print the same bytes.
/// Both permute 3 of `main`'s 5 sections (6 variants) to keep the step inside its budget; the
/// full climb is held to the golden file, written by another process.
#[test]
fn two_studio_climbs_print_the_same_bytes() {
    let first = run("studio-twice-first", 3);
    let second = run("studio-twice-second", 3);
    assert_eq!(first.variants.len(), 6, "{}", first.render());
    assert_eq!(first.render(), second.render());
}

/// The output is the committed golden file, byte for byte.
#[test]
fn the_studio_climb_prints_the_committed_frontier() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/studio.frontier.txt");
    let golden = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    assert!(
        !golden.starts_with(PLACEHOLDER),
        "{} is still the placeholder; generate it from the repository root with:\n{GENERATE}",
        path.display()
    );
    assert_eq!(shared().render(), golden);
}

/// The ranking separates variants on a term: `below_fold`, the regions the 48x40 drawing leaves
/// out. The top variant is no worse than the document as written on every other term and better
/// on `below_fold`, so it dominates it and the document as written is off the frontier.
#[test]
fn the_studio_ranking_separates_variants_on_below_fold() {
    let climb = shared();
    let ranking = climb.ranking();
    let as_written_id = "light:summary|hill_kpis,hill,progress,points,frontier";
    let as_written = climb
        .variants
        .iter()
        .position(|variant| variant.id == as_written_id)
        .unwrap_or_else(|| panic!("no variant {as_written_id}:\n{}", climb.render()));
    let top = &climb.variants[ranking[0]];
    let written = &climb.variants[as_written];
    assert_eq!(top.terms.errors, written.terms.errors, "{}", climb.render());
    assert!(top.terms.cut <= written.terms.cut, "{}", climb.render());
    assert_eq!(
        top.terms.primary_rows,
        written.terms.primary_rows,
        "{}",
        climb.render()
    );
    assert!(
        top.terms.below_fold < written.terms.below_fold,
        "{} and {as_written_id} tie on below_fold:\n{}",
        top.id,
        climb.render()
    );
    assert!(
        !climb.frontier().contains(&as_written),
        "the document as written is on the frontier:\n{}",
        climb.render()
    );
}
