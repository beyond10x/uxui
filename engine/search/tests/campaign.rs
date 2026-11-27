//! The declared browser campaign (`story:trial-target`): a declaration `ux-search-campaign/1`
//! names the page, the environments, the axes and the terms; `ux-search campaign run` measures
//! every variant in Chromium and `ux-search campaign report` ranks the record. No test here
//! starts a browser or runs a campaign: the tests that do are the browser lane, the module
//! `campaign_live`, which a plain `cargo test` and the gate leave out.
//!
//! `fixtures/campaign/small.campaign.yaml` is a declaration no page is behind: the tests read
//! it, break it, and rank records they make up for it. `fixtures/campaign/studio/` is the
//! campaign over the frontier page of the studio app this repository generates
//! (`docs/campaign-2026-11-26-studio-engine.md`): the declaration (`frontier.campaign.yaml`),
//! the measurements `ux-search campaign run` wrote (`measurements.json`) and the report over
//! them (`report.txt`), all three recorded by `tools/studio-campaign.sh`.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use ux_search::campaign::declaration::{self, Declaration};
use ux_search::campaign::record::{self, Note, Recorded, Render};
use ux_search::campaign::{report, run};
use ux_search::landing::patch;

/// The command that records the studio campaign, from the repository root.
const RECORD: &str = "sh tools/studio-campaign.sh";

/// Where the screenshots of the studio campaign's frontier are kept, from the repository root.
const SHOTS: &str = "docs/campaign-2026-11-26-studio-engine";

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/campaign")
        .join(name)
}

fn repository(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn small_text() -> String {
    read(&fixture("small.campaign.yaml"))
}

fn small() -> Declaration {
    Declaration::parse(&small_text()).unwrap_or_else(|error| panic!("{error}"))
}

fn variant(declaration: &Declaration, id: &str) -> declaration::Variant {
    declaration
        .variants()
        .into_iter()
        .find(|variant| variant.id == id)
        .unwrap_or_else(|| panic!("no variant {id}"))
}

// The declaration.

#[test]
fn a_declaration_names_its_page_its_environments_its_axes_and_its_terms() {
    let declaration = small();
    assert_eq!(declaration.digest, record::digest(small_text().as_bytes()));
    assert_eq!(declaration.text, small_text());
    assert_eq!(declaration.page.path, "/orders/list");
    assert_eq!(declaration.page.ready, "main[data-ready]");
    assert_eq!(declaration.page.ready_timeout_ms, 5000);
    assert_eq!(declaration.page.settle_ms, 100);
    assert_eq!(declaration.page.pointer, Some((380, 760)));
    for environment in declaration.environments() {
        assert_eq!(declaration.pointer(&environment), (380, 760));
    }
    let environments: Vec<String> = declaration
        .environments()
        .iter()
        .map(declaration::Environment::name)
        .collect();
    assert_eq!(
        environments,
        [
            "1366x768 light",
            "1366x768 dark",
            "390x844 light",
            "390x844 dark"
        ]
    );
    let dark = &declaration.modes[1];
    assert_eq!(dark.name, "dark");
    assert_eq!(dark.media.as_deref(), Some("dark"));
    assert_eq!(
        dark.attribute,
        Some(("data-theme".to_owned(), "dark".to_owned()))
    );
    assert_eq!(dark.query.as_deref(), Some("theme=dark"));
    assert_eq!(
        dark.expect,
        Some(declaration::Expect {
            attribute: Some(("data-theme".to_owned(), "dark".to_owned())),
            selector: Some("body.dark".to_owned()),
        })
    );
    assert_eq!(
        dark.set_by(),
        "the emulated colour scheme `dark` and the attribute `data-theme` `dark` and the query `theme=dark`"
    );
    assert_eq!(declaration.modes[0].attribute, None);
    assert_eq!(declaration.modes[0].expect, None);
    let axes: Vec<(&str, Vec<&str>, usize)> = declaration
        .axes
        .iter()
        .map(|axis| {
            (
                axis.name.as_str(),
                axis.values.iter().map(|value| value.id.as_str()).collect(),
                axis.shipped,
            )
        })
        .collect();
    assert_eq!(
        axes,
        [
            ("spacing", vec!["tight", "stock"], 1),
            ("ink", vec!["stock", "deep"], 0)
        ]
    );
    assert_eq!(declaration.primary, "button.primary");
    assert_eq!(
        declaration.regions.as_deref(),
        Some("header, main > section, footer")
    );
    assert_eq!(declaration.always_visible.len(), 2);
    assert_eq!(declaration.always_visible[1].name, "save");
    assert_eq!(declaration.small_target_px, 24);
    assert_eq!(
        declaration.hard,
        ["primary_in_view", "no_h_scroll", "named"]
    );
    assert_eq!(declaration.terms.len(), 7);
}

/// What a declaration may leave out: the waits, the pointer, a mode's expectation, the regions,
/// the always-visible items, the hard rules and the size of a small target. A pointer that is
/// not named rests in the bottom-right corner of each viewport.
#[test]
fn a_declaration_has_defaults_for_what_it_leaves_out() {
    let text = small_text()
        .replace(
            "  ready_timeout_ms: 5000\n  settle_ms: 100\n  pointer: [380, 760]\n",
            "",
        )
        .replace(
            ", expect: {attribute: {name: data-theme, value: dark}, selector: 'body.dark'}",
            "",
        )
        .replace("regions: 'header, main > section, footer'\n", "")
        .replace(
            "always_visible:\n  - {name: navigation, selector: nav}\n  - {name: save, selector: 'button.primary'}\n",
            "",
        )
        .replace("small_target_px: 24\n", "")
        .replace("hard: [primary_in_view, no_h_scroll, named]\n", "")
        .replace(
            "terms: [always_visible_misses, low_contrast, text_under_14, overflow, primary_px, regions, small_targets]",
            "terms: [low_contrast, primary_px]",
        );
    let declaration = Declaration::parse(&text).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(declaration.page.ready_timeout_ms, 10_000);
    assert_eq!(declaration.page.settle_ms, 0);
    assert_eq!(declaration.page.pointer, None);
    let rests: Vec<(u32, u32)> = declaration
        .environments()
        .iter()
        .map(|environment| declaration.pointer(environment))
        .collect();
    assert_eq!(rests, [(1365, 767), (1365, 767), (389, 843), (389, 843)]);
    assert_eq!(declaration.modes[1].expect, None);
    assert_eq!(declaration.regions, None);
    assert!(declaration.always_visible.is_empty());
    assert_eq!(declaration.small_target_px, 44);
    assert!(declaration.hard.is_empty());
    assert_eq!(declaration.terms, ["low_contrast", "primary_px"]);
}

/// One way to break the small declaration: its name, the texts to put in place of others, and
/// what the refusal must say (the key it breaks at, and the name it stumbles on).
type Broken = (
    &'static str,
    &'static [(&'static str, &'static str)],
    &'static [&'static str],
);

const VALUES_OF_INK: &str = "      - id: stock\n      - id: deep\n        rules:\n          - {selector: ':root', mode: light, set: {--ink: '#123456'}}\n          - {selector: ':root[data-theme=\"dark\"]', mode: dark, set: {--ink: '#abcdef'}}\n          - {selector: .badge, set: {--badge-ink: 'var(--ink)'}}\n";

const BROKEN: &[Broken] = &[
    (
        "another format",
        &[(
            "format: ux-search-campaign/1",
            "format: ux-search-campaign/9",
        )],
        &["`format`", "ux-search-campaign/2", "ux-search-campaign/1"],
    ),
    (
        "a key of the next format",
        &[(
            "small_target_px: 24\n",
            "small_target_px: 24\nrows: 'tbody tr'\n",
        )],
        &["`rows`", "a key of `ux-search-campaign/2`"],
    ),
    (
        "a term of the next format",
        &[(
            "terms: [always_visible_misses,",
            "terms: [primary_bottom_px, always_visible_misses,",
        )],
        &["`terms[0]`", "a term of `ux-search-campaign/2`"],
    ),
    (
        "a container in the format before it",
        &[(
            "{name: navigation, selector: nav}",
            "{name: navigation, selector: nav, container: aside}",
        )],
        &[
            "`always_visible[0].container`",
            "a key of `ux-search-campaign/2`",
        ],
    ),
    (
        "an unknown key",
        &[(
            "small_target_px: 24\n",
            "small_target_px: 24\ncolour: red\n",
        )],
        &["`colour`", "unknown key"],
    ),
    (
        "an unknown key of the page",
        &[("  settle_ms: 100", "  settle: 100")],
        &["`page.settle`", "unknown key"],
    ),
    (
        "no path",
        &[("  path: /orders/list\n", "")],
        &["`page.path`"],
    ),
    (
        "a path that does not start at the root",
        &[("path: /orders/list", "path: orders/list")],
        &["`page.path`", "orders/list"],
    ),
    (
        "a path with a query",
        &[("path: /orders/list", "path: /orders/list?theme=dark")],
        &["`page.path`", "query"],
    ),
    (
        "no ready selector",
        &[("  ready: 'main[data-ready]'\n", "")],
        &["`page.ready`"],
    ),
    (
        "no time to get ready in",
        &[("ready_timeout_ms: 5000", "ready_timeout_ms: 0")],
        &["`page.ready_timeout_ms`"],
    ),
    (
        "a wait that is no number",
        &[("settle_ms: 100", "settle_ms: soon")],
        &["`page.settle_ms`"],
    ),
    (
        "an unknown key of the environments",
        &[("  viewports: [", "  viewport: [")],
        &["`environments.viewport`", "unknown key"],
    ),
    (
        "no viewport",
        &[("viewports: [1366x768, 390x844]", "viewports: []")],
        &["`environments.viewports`"],
    ),
    (
        "a viewport that is no size",
        &[("390x844", "390 by 844")],
        &["`environments.viewports[1]`", "390 by 844"],
    ),
    (
        "one viewport twice",
        &[("390x844", "1366x768")],
        &["`environments.viewports[1]`", "1366x768", "twice"],
    ),
    (
        "no mode",
        &[
            ("    - {name: light, media: light}\n", ""),
            (
                "  modes:\n    - {name: dark, media: dark, attribute: {name: data-theme, value: dark}, query: theme=dark, expect: {attribute: {name: data-theme, value: dark}, selector: 'body.dark'}}\n",
                "  modes: []\n",
            ),
        ],
        &["`environments.modes`"],
    ),
    (
        "a mode nothing sets",
        &[("{name: light, media: light}", "{name: light}")],
        &["`environments.modes[0]`", "media", "attribute", "query"],
    ),
    (
        "a colour scheme the browser does not emulate",
        &[("media: light}", "media: sepia}")],
        &["`environments.modes[0].media`", "sepia"],
    ),
    (
        "two modes with one name",
        &[("{name: dark, media: dark", "{name: light, media: dark")],
        &["`environments.modes[1].name`", "light", "twice"],
    ),
    (
        "an unknown key of a mode",
        &[(
            "{name: light, media: light}",
            "{name: light, media: light, class: bright}",
        )],
        &["`environments.modes[0].class`", "unknown key"],
    ),
    (
        "an attribute without a value",
        &[(
            "attribute: {name: data-theme, value: dark}",
            "attribute: {name: data-theme}",
        )],
        &["`environments.modes[1].attribute.value`"],
    ),
    (
        "a query that starts with its mark",
        &[("query: theme=dark", "query: '?theme=dark'")],
        &["`environments.modes[1].query`"],
    ),
    (
        "a pointer that is no position",
        &[("pointer: [380, 760]", "pointer: [380]")],
        &["`page.pointer`", "[380]", "[<x>, <y>]"],
    ),
    (
        "a pointer that is no pair of whole numbers",
        &[("pointer: [380, 760]", "pointer: [left, 760]")],
        &["`page.pointer`", "left", "[<x>, <y>]"],
    ),
    (
        "a pointer that is a text",
        &[("pointer: [380, 760]", "pointer: corner")],
        &["`page.pointer`", "[<x>, <y>]"],
    ),
    (
        "a pointer to the right of the narrowest viewport",
        &[("pointer: [380, 760]", "pointer: [390, 760]")],
        &[
            "`page.pointer`",
            "[390, 760]",
            "outside",
            "390x844",
            "`environments.viewports[1]`",
        ],
    ),
    (
        "a pointer under the lowest viewport",
        &[("pointer: [380, 760]", "pointer: [380, 768]")],
        &[
            "`page.pointer`",
            "[380, 768]",
            "outside",
            "1366x768",
            "`environments.viewports[0]`",
        ],
    ),
    (
        "an unknown key of an expectation",
        &[("selector: 'body.dark'}}", "css: 'body.dark'}}")],
        &["`environments.modes[1].expect.css`", "unknown key"],
    ),
    (
        "an expectation of nothing",
        &[(
            "expect: {attribute: {name: data-theme, value: dark}, selector: 'body.dark'}",
            "expect: {}",
        )],
        &["`environments.modes[1].expect`", "attribute", "selector"],
    ),
    (
        "an expectation that is no map",
        &[(
            "expect: {attribute: {name: data-theme, value: dark}, selector: 'body.dark'}",
            "expect: dark",
        )],
        &["`environments.modes[1].expect`"],
    ),
    (
        "an expected attribute without a value",
        &[(
            "expect: {attribute: {name: data-theme, value: dark}",
            "expect: {attribute: {name: data-theme}",
        )],
        &["`environments.modes[1].expect.attribute.value`"],
    ),
    (
        "an expected attribute whose name is none",
        &[(
            "expect: {attribute: {name: data-theme, value: dark}",
            "expect: {attribute: {name: 'data theme', value: dark}",
        )],
        &[
            "`environments.modes[1].expect.attribute.name`",
            "data theme",
        ],
    ),
    (
        "an expected selector that is empty",
        &[("selector: 'body.dark'}}", "selector: ''}}")],
        &["`environments.modes[1].expect.selector`"],
    ),
    (
        "an unknown key of an axis",
        &[(
            "  - name: spacing\n    shipped: stock",
            "  - name: spacing\n    shiped: stock",
        )],
        &["`axes[0].shiped`", "unknown key"],
    ),
    (
        "an axis without a name",
        &[(
            "  - name: spacing\n    shipped: stock",
            "  - shipped: stock",
        )],
        &["`axes[0].name`"],
    ),
    (
        "two axes with one name",
        &[("  - name: ink", "  - name: spacing")],
        &["`axes[1].name`", "spacing", "twice"],
    ),
    (
        "an empty axis",
        &[
            (VALUES_OF_INK, ""),
            (
                "    shipped: stock\n    values:\nprimary",
                "    shipped: stock\n    values: []\nprimary",
            ),
        ],
        &["`axes[1].values`", "ink", "no value"],
    ),
    (
        "two values with one name",
        &[("      - id: deep", "      - id: stock")],
        &["`axes[1].values[1].id`", "stock", "twice"],
    ),
    (
        "a value whose name holds the mark that joins an id",
        &[("- id: tight", "- id: tight.er")],
        &["`axes[0].values[0].id`", "tight.er"],
    ),
    (
        "a shipped value the axis does not have",
        &[(
            "  - name: ink\n    shipped: stock",
            "  - name: ink\n    shipped: pale",
        )],
        &["`axes[1].shipped`", "pale"],
    ),
    (
        "no shipped value",
        &[(
            "  - name: spacing\n    shipped: stock\n",
            "  - name: spacing\n",
        )],
        &["`axes[0].shipped`"],
    ),
    (
        "an unknown key of a value",
        &[(
            "      - id: tight\n        rules:",
            "      - id: tight\n        rule:",
        )],
        &["`axes[0].values[0].rule`", "unknown key"],
    ),
    (
        "an unknown key of a rule",
        &[(
            "{selector: ':root', set: {--space: 0.5rem, --gap: 0.25rem}}",
            "{selector: ':root', sets: {--space: 0.5rem, --gap: 0.25rem}}",
        )],
        &["`axes[0].values[0].rules[0].sets`", "unknown key"],
    ),
    (
        "a rule without a selector",
        &[(
            "{selector: ':root', set: {--space: 0.5rem, --gap: 0.25rem}}",
            "{set: {--space: 0.5rem, --gap: 0.25rem}}",
        )],
        &["`axes[0].values[0].rules[0].selector`"],
    ),
    (
        "a rule that sets nothing",
        &[("set: {--space: 0.5rem, --gap: 0.25rem}", "set: {}")],
        &["`axes[0].values[0].rules[0].set`"],
    ),
    (
        "a property that is no custom property",
        &[("--space: 0.5rem", "margin: 0.5rem")],
        &[
            "`axes[0].values[0].rules[0].set`",
            "margin",
            "custom property",
        ],
    ),
    (
        "a value that closes its declaration",
        &[("--space: 0.5rem,", "--space: '0.5rem; color: red',")],
        &["`axes[0].values[0].rules[0].set`", "semicolon"],
    ),
    (
        "a rule for a mode that is not declared",
        &[("mode: light, set: {--ink:", "mode: sepia, set: {--ink:")],
        &["`axes[1].values[1].rules[0].mode`", "sepia"],
    ),
    (
        "one property set twice by one value",
        &[(
            "{selector: .badge, set: {--badge-ink: 'var(--ink)'}}",
            "{selector: ':root', set: {--ink: black}}",
        )],
        &[
            "`axes[1].values[1].rules[2].set`",
            "--ink",
            "`axes[1].values[1].rules[0].set`",
        ],
    ),
    (
        "one property set by two axes",
        &[("--gap: 0.25rem", "--ink: 0.25rem")],
        &[
            "`axes[1].values[1].rules[0].set`",
            "--ink",
            "`axes[0].values[0].rules[0].set`",
        ],
    ),
    (
        "no primary action",
        &[("primary: 'button.primary'\n", "")],
        &["`primary`"],
    ),
    (
        "regions without a selector",
        &[("regions: 'header, main > section, footer'", "regions: ''")],
        &["`regions`"],
    ),
    (
        "an unknown key of an always-visible item",
        &[(
            "{name: navigation, selector: nav}",
            "{name: navigation, css: nav}",
        )],
        &["`always_visible[0].css`", "unknown key"],
    ),
    (
        "an always-visible item without a selector",
        &[("{name: navigation, selector: nav}", "{name: navigation}")],
        &["`always_visible[0].selector`"],
    ),
    (
        "two always-visible items with one name",
        &[("{name: save,", "{name: navigation,")],
        &["`always_visible[1].name`", "navigation", "twice"],
    ),
    (
        "a small target of no size",
        &[("small_target_px: 24", "small_target_px: 0")],
        &["`small_target_px`"],
    ),
    (
        "a hard rule the catalogue does not have",
        &[("no_h_scroll, named]", "no_h_scroll, named, pretty]")],
        &["`hard[3]`", "pretty", "primary_in_view"],
    ),
    (
        "one hard rule twice",
        &[(
            "hard: [primary_in_view, no_h_scroll, named]",
            "hard: [named, named]",
        )],
        &["`hard[1]`", "named", "twice"],
    ),
    (
        "a term the catalogue does not have",
        &[("terms: [always_visible_misses,", "terms: [beauty,")],
        &["`terms[0]`", "beauty", "low_contrast"],
    ),
    (
        "the hard rules named as a term",
        &[("terms: [always_visible_misses,", "terms: [hard,")],
        &["`terms[0]`", "hard", "first"],
    ),
    (
        "one term twice",
        &[("overflow, primary_px", "overflow, overflow")],
        &["`terms[4]`", "overflow", "twice"],
    ),
    (
        "no term",
        &[(
            "terms: [always_visible_misses, low_contrast, text_under_14, overflow, primary_px, regions, small_targets]",
            "terms: []",
        )],
        &["`terms`"],
    ),
    (
        "the regions term without regions",
        &[("regions: 'header, main > section, footer'\n", "")],
        &["`terms[5]`", "regions", "`regions`"],
    ),
    (
        "the always-visible term without an item",
        &[(
            "always_visible:\n  - {name: navigation, selector: nav}\n  - {name: save, selector: 'button.primary'}\n",
            "",
        )],
        &["`terms[0]`", "always_visible_misses", "`always_visible`"],
    ),
];

#[test]
fn a_broken_declaration_is_refused_with_the_key_it_breaks_at() {
    for (name, changes, says) in BROKEN {
        let mut text = small_text();
        for (from, to) in *changes {
            assert!(text.contains(from), "{name}: the fixture has no `{from}`");
            text = text.replacen(from, to, 1);
        }
        let refused = match Declaration::parse(&text) {
            Ok(_) => panic!("{name}: the declaration is read"),
            Err(refused) => refused,
        };
        for part in *says {
            assert!(refused.contains(part), "{name}: `{refused}` lacks `{part}`");
        }
    }
}

/// The small declaration in `ux-search-campaign/2`, with what that format adds: `rows`, the
/// `container` of an always-visible item, and the terms `rows_in_view` and `primary_bottom_px`.
fn small_2_text() -> String {
    let mut text = small_text();
    for (from, to) in [
        (
            "format: ux-search-campaign/1",
            "format: ux-search-campaign/2",
        ),
        (
            "small_target_px: 24\n",
            "small_target_px: 24\nrows: 'tbody tr'\n",
        ),
        (
            "{name: navigation, selector: nav}",
            "{name: navigation, selector: nav, container: aside}",
        ),
        (
            "terms: [always_visible_misses,",
            "terms: [rows_in_view, primary_bottom_px, always_visible_misses,",
        ),
    ] {
        assert!(text.contains(from), "the fixture has no `{from}`");
        text = text.replacen(from, to, 1);
    }
    text
}

#[test]
fn a_declaration_of_the_second_format_names_rows_a_container_and_the_list_terms() {
    let declaration = Declaration::parse(&small_2_text()).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(declaration.rows.as_deref(), Some("tbody tr"));
    assert_eq!(
        declaration.always_visible[0].container.as_deref(),
        Some("aside")
    );
    assert_eq!(declaration.always_visible[1].container, None);
    assert_eq!(
        declaration.terms[..2],
        ["rows_in_view", "primary_bottom_px"]
    );
    // The first format is still read, and has no rows and no container.
    let first = small();
    assert_eq!(first.rows, None);
    assert!(
        first
            .always_visible
            .iter()
            .all(|item| item.container.is_none())
    );
    // `rows_in_view` counts the rows the key `rows` names.
    let refused = Declaration::parse(&small_2_text().replace("rows: 'tbody tr'\n", ""))
        .expect_err("rows_in_view without rows");
    assert!(
        refused.contains("`terms[0]`") && refused.contains("`rows`"),
        "{refused}"
    );
}

/// `rows_in_view` is the one term where more is better: the worst environment is the one with
/// the fewest rows, and a variant with more rows in view ranks before one with fewer. The
/// report prints the count.
#[test]
fn rows_in_view_is_a_term_where_more_is_better() {
    let declaration = Declaration::parse(&small_2_text()).unwrap_or_else(|error| panic!("{error}"));
    let mut more = vec![render(200, 0); 4];
    for render in &mut more {
        render.rows_in_view = 16;
    }
    let mut fewer = more.clone();
    fewer[2].rows_in_view = 13;
    let (more, fewer) = (
        report::terms(&declaration, &more),
        report::terms(&declaration, &fewer),
    );
    assert!(more < fewer, "{more:?} {fewer:?}");
    assert!(
        report::terms_text(&declaration, &more).contains(" rows_in_view=16 "),
        "{}",
        report::terms_text(&declaration, &more)
    );
    assert!(report::terms_text(&declaration, &fewer).contains(" rows_in_view=13 "));
}

#[test]
fn a_text_that_is_no_declaration_is_refused() {
    for (text, says) in [
        ("- a\n- b\n", "not a map"),
        ("", "not a map"),
        ("format: [", "line 1"),
        ("page: {}\n", "`format`"),
    ] {
        let refused = Declaration::parse(text).expect_err(text);
        assert!(
            refused.contains(says),
            "`{text}`: `{refused}` lacks `{says}`"
        );
    }
}

/// A campaign is exhaustive: a space of more variants than a browser measures in one run is
/// refused when it is declared, not after hours.
#[test]
fn a_space_of_too_many_variants_is_refused() {
    let (head, tail) = small_text()
        .split_once("axes:\n")
        .map(|(head, rest)| {
            let tail = &rest[rest.find("primary:").expect("the primary action")..];
            (head.to_owned(), tail.to_owned())
        })
        .expect("the axes");
    let mut text = format!("{head}axes:\n");
    for axis in 0..13 {
        text.push_str(&format!(
            "  - name: a{axis}\n    shipped: off\n    values:\n      - id: off\n      - id: on\n        rules:\n          - {{selector: ':root', set: {{--a{axis}: 1}}}}\n"
        ));
    }
    text.push_str(&tail);
    let refused = Declaration::parse(&text).expect_err("8192 variants");
    for part in ["`axes`", "8192", "4096"] {
        assert!(refused.contains(part), "`{refused}` lacks `{part}`");
    }
}

/// A refusal of a declaration file names the file before the key.
#[test]
fn a_refused_file_is_named_with_its_key() {
    let directory = crate::state_dir("campaign-refused-file");
    std::fs::create_dir_all(&directory).expect("a scratch directory");
    let path = directory.join("broken.campaign.yaml");
    std::fs::write(&path, small_text().replace("- id: deep", "- id: stock")).expect("a file");
    let refused = Declaration::read(&path).expect_err("two values with one name");
    assert!(
        refused.starts_with(&format!("{}: `axes[1].values[1].id`", path.display())),
        "{refused}"
    );
    let missing = directory.join("no-such.campaign.yaml");
    let refused = Declaration::read(&missing).expect_err("no file");
    assert!(
        refused.starts_with(&missing.display().to_string()),
        "{refused}"
    );
}

// The space.

#[test]
fn the_variants_are_every_choice_once_the_last_axis_fastest() {
    let declaration = small();
    let variants = declaration.variants();
    let ids: Vec<&str> = variants.iter().map(|variant| variant.id.as_str()).collect();
    assert_eq!(
        ids,
        ["tight.stock", "tight.deep", "stock.stock", "stock.deep"]
    );
    let choices: Vec<&[usize]> = variants
        .iter()
        .map(|variant| variant.choice.as_slice())
        .collect();
    assert_eq!(choices, [[0, 0], [0, 1], [1, 0], [1, 1]]);
    let shipped = declaration.shipped();
    assert_eq!(shipped.id, "stock.stock");
    assert_eq!(variants[2], shipped);
}

// The injected style sheet.

#[test]
fn a_variants_sheet_sets_its_custom_properties_by_selector_for_the_mode() {
    let declaration = small();
    let far = variant(&declaration, "tight.deep");
    assert_eq!(
        declaration.sheet(&far, "light"),
        ":root {\n  --space: 0.5rem;\n  --gap: 0.25rem;\n  --ink: #123456;\n}\n\n.badge {\n  --badge-ink: var(--ink);\n}\n"
    );
    assert_eq!(
        declaration.sheet(&far, "dark"),
        ":root {\n  --space: 0.5rem;\n  --gap: 0.25rem;\n}\n\n:root[data-theme=\"dark\"] {\n  --ink: #abcdef;\n}\n\n.badge {\n  --badge-ink: var(--ink);\n}\n"
    );
    assert_eq!(
        declaration.sheet(&variant(&declaration, "tight.stock"), "dark"),
        ":root {\n  --space: 0.5rem;\n  --gap: 0.25rem;\n}\n"
    );
}

#[test]
fn the_shipped_variant_injects_an_empty_sheet() {
    let declaration = small();
    for mode in ["light", "dark"] {
        assert_eq!(declaration.sheet(&declaration.shipped(), mode), "");
        assert!(declaration.checks(&declaration.shipped(), mode).is_empty());
    }
}

/// What a run checks on the page after it injected a sheet: each property the sheet sets, with
/// the key of the declaration that set it, so a selector that matches nothing and a value that
/// a rule of the page overrides are refused with that key.
#[test]
fn every_property_a_sheet_sets_is_checked_under_the_key_that_set_it() {
    let declaration = small();
    let checks = declaration.checks(&variant(&declaration, "tight.deep"), "dark");
    let seen: Vec<(&str, &str, &str, &str)> = checks
        .iter()
        .map(|check| {
            (
                check.key.as_str(),
                check.selector.as_str(),
                check.property.as_str(),
                check.value.as_str(),
            )
        })
        .collect();
    assert_eq!(
        seen,
        [
            ("axes[0].values[0].rules[0]", ":root", "--space", "0.5rem"),
            ("axes[0].values[0].rules[0]", ":root", "--gap", "0.25rem"),
            (
                "axes[1].values[1].rules[1]",
                ":root[data-theme=\"dark\"]",
                "--ink",
                "#abcdef"
            ),
            (
                "axes[1].values[1].rules[2]",
                ".badge",
                "--badge-ink",
                "var(--ink)"
            ),
        ]
    );
}

#[test]
fn a_patch_prints_its_custom_properties_per_selector_and_mode() {
    let declaration = small();
    assert_eq!(
        declaration.patch_lines(&variant(&declaration, "tight.deep")),
        [
            ":root { --space: 0.5rem; --gap: 0.25rem }",
            "light: :root { --ink: #123456 }",
            "dark: :root[data-theme=\"dark\"] { --ink: #abcdef }",
            ".badge { --badge-ink: var(--ink) }",
        ]
    );
    assert_eq!(
        declaration.patch_lines(&declaration.shipped()),
        ["no property set: the page as shipped"]
    );
}

/// The page of an environment on a server: the declared path under the origin, with the query
/// of the mode.
#[test]
fn the_page_of_a_mode_is_its_path_on_the_server_with_the_modes_query() {
    let declaration = small();
    let site = ux_browser::target::parse_url("http://127.0.0.1:4173").expect("an origin");
    assert_eq!(
        declaration.url(&site, "light"),
        Ok("http://127.0.0.1:4173/orders/list".to_owned())
    );
    assert_eq!(
        declaration.url(&site, "dark"),
        Ok("http://127.0.0.1:4173/orders/list?theme=dark".to_owned())
    );
    let mounted = ux_browser::target::parse_url("http://127.0.0.1:4173/console").expect("a URL");
    assert_eq!(
        declaration.url(&mounted, "dark"),
        Ok("http://127.0.0.1:4173/console/orders/list?theme=dark".to_owned())
    );
}

// The record.

/// One render of the made-up record.
fn render(primary_px: u64, low_contrast: u64) -> Render {
    Render {
        primary_in_view: true,
        h_scroll: false,
        unnamed: 0,
        primary_px,
        small_targets: 1,
        regions: 2,
        text_runs: 40,
        text_under_14: 10,
        low_contrast,
        overflow: 0,
        always_visible_misses: 0,
        rows_in_view: 0,
        primary_bottom_px: 0,
    }
}

fn note(environment: usize, term: &str, elements: &[&str]) -> Note {
    Note {
        environment,
        term: term.to_owned(),
        elements: elements.iter().map(|&element| element.to_owned()).collect(),
    }
}

const PALE_HINT: &str =
    "main > p.hint \"Updated a minute ago\": 3.10 under 4.5, #8a8a8a on #ffffff";
const PALE_CELL: &str = "td.muted \"n/a\": 2.32 under 4.5, #aaaaaa on #ffffff";
const SMALL_ICON: &str = "nav > a.icon: 16 x 16 px";

/// A record made up for the small declaration: the tight spacing brings the primary action
/// 40 px up, the deep ink takes both low-contrast runs away, and the second environment is
/// 10 px worse than the others. Each axis has one value that changes the page; the page as
/// shipped and the frontier's one variant name the elements behind their counts.
fn small_record() -> Recorded {
    let declaration = small();
    let variants = declaration
        .variants()
        .into_iter()
        .map(|variant| {
            let px = if variant.choice[0] == 0 { 200 } else { 240 };
            let low = if variant.choice[1] == 1 { 0 } else { 2 };
            let renders = (0..4)
                .map(|at| render(if at == 1 { px + 10 } else { px }, low))
                .collect();
            (variant.id, renders)
        })
        .collect();
    Recorded {
        format: record::FORMAT,
        adapter: "ux-browser 0.10.0".to_owned(),
        browser: "HeadlessChrome/1.2.3.4".to_owned(),
        declaration,
        variants,
        changing: vec![1, 1],
        notes: vec![
            Vec::new(),
            vec![note(0, "small_targets", &[SMALL_ICON])],
            vec![
                note(0, "low_contrast", &[PALE_HINT, PALE_CELL]),
                note(0, "small_targets", &[SMALL_ICON]),
                note(3, "low_contrast", &[PALE_CELL]),
            ],
            Vec::new(),
        ],
    }
}

#[test]
fn a_record_reads_back_what_was_written_and_writes_the_same_bytes() {
    let recorded = small_record();
    recorded.check().unwrap_or_else(|error| panic!("{error}"));
    let text = recorded.text();
    assert!(
        text.starts_with("{\n  \"format\": \"ux-search-campaign-measurements/3\",\n"),
        "{text}"
    );
    assert!(text.contains(&format!(
        "\"declaration\": {{\"digest\": \"{}\", ",
        recorded.declaration.digest
    )));
    assert!(
        text.contains("      [1,0,0,200,1,2,40,10,2,0,0,0,0],\n"),
        "{text}"
    );
    // Where the pointer rested, per environment; what each axis changes; a variant's notes.
    assert!(
        text.contains(
            "  \"environments\": [\"1366x768 light\", \"1366x768 dark\", \"390x844 light\", \"390x844 dark\"],\n  \"pointer\": [[380,760],[380,760],[380,760],[380,760]],\n  \"columns\": ["
        ),
        "{text}"
    );
    assert!(
        text.contains(
            "  \"axes\": [{\"name\": \"spacing\", \"changing\": 1}, {\"name\": \"ink\", \"changing\": 1}],\n  \"variants\": [\n"
        ),
        "{text}"
    );
    assert!(
        text.contains(
            "      [1,0,0,200,1,2,40,10,0,0,0,0,0]\n    ], \"notes\": [\n      {\"environment\": 0, \"term\": \"small_targets\", \"elements\": [\"nav > a.icon: 16 x 16 px\"]}\n    ]},\n"
        ),
        "{text}"
    );
    assert!(
        text.contains(
            "      {\"environment\": 0, \"term\": \"low_contrast\", \"elements\": [\"main > p.hint \\\"Updated a minute ago\\\": 3.10 under 4.5, #8a8a8a on #ffffff\", \"td.muted \\\"n/a\\\": 2.32 under 4.5, #aaaaaa on #ffffff\"]},\n"
        ),
        "{text}"
    );
    // A variant without notes is written as before them.
    assert!(
        text.contains("      [1,0,0,200,1,2,40,10,2,0,0,0,0]\n    ]},\n"),
        "{text}"
    );
    assert!(
        text.ends_with(&format!("  \"digest\": \"{}\"\n}}\n", recorded.digest())),
        "{text}"
    );
    let read = Recorded::parse(&text).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(read, recorded);
    assert_eq!(read.text(), text);
    assert_eq!(record::COLUMNS.len(), 13);
}

/// A record whose numbers, declaration or shape were changed after it was written is refused:
/// the file states the digest of what it holds, and holds the declaration it was measured for.
#[test]
fn a_tampered_record_is_refused() {
    let text = small_record().text();
    let changes: &[(&str, &str, &str, &str)] = &[
        (
            "a number",
            "[1,0,0,200,1,2,40,10,2,0,0,0,0]",
            "[1,0,0,199,1,2,40,10,2,0,0,0,0]",
            "`digest`",
        ),
        (
            "the browser",
            "HeadlessChrome/1.2.3.4",
            "HeadlessChrome/1.2.3.5",
            "`digest`",
        ),
        (
            "the declaration",
            "settle_ms: 100",
            "settle_ms: 101",
            "`declaration.digest`",
        ),
        (
            "a variant's name",
            "\"id\": \"tight.deep\"",
            "\"id\": \"tight.dark\"",
            "tight.dark",
        ),
        (
            "the order of the variants",
            "\"id\": \"tight.stock\"",
            "\"id\": \"stock.deep\"",
            "stock.deep",
        ),
        (
            "a render",
            "      [1,0,0,200,1,2,40,10,2,0,0,0,0],\n",
            "",
            "renders",
        ),
        ("a flag", "[1,0,0,200,", "[2,0,0,200,", "`primary_in_view`"),
        (
            "a number that is none",
            "[1,0,0,200,",
            "[1,0,0,-200,",
            "-200",
        ),
        ("a column", "\"unnamed\", ", "", "`columns`"),
        (
            "an environment",
            "\"1366x768 dark\", ",
            "",
            "`environments`",
        ),
        (
            "the format",
            "ux-search-campaign-measurements/3",
            "ux-search-landing/1",
            "`format`",
        ),
        (
            "the format before the pointer was parked",
            "ux-search-campaign-measurements/3",
            "ux-search-campaign-measurements/1",
            "the pointer wherever the browser left it",
        ),
        (
            "where the pointer rested",
            "\"pointer\": [[380,760],",
            "\"pointer\": [[0,0],",
            "`pointer`",
        ),
        (
            "an environment's pointer",
            "[380,760],[380,760],[380,760],[380,760]",
            "[380,760],[380,760],[380,760]",
            "`pointer`",
        ),
        (
            "what an axis changes",
            "{\"name\": \"ink\", \"changing\": 1}",
            "{\"name\": \"ink\", \"changing\": 0}",
            "`digest`",
        ),
        (
            "more changing values than the axis has",
            "{\"name\": \"ink\", \"changing\": 1}",
            "{\"name\": \"ink\", \"changing\": 2}",
            "axis `ink`",
        ),
        (
            "an axis's name",
            "{\"name\": \"ink\", \"changing\": 1}",
            "{\"name\": \"paper\", \"changing\": 1}",
            "`axes`",
        ),
        (
            "an axis",
            ", {\"name\": \"ink\", \"changing\": 1}",
            "",
            "`axes`",
        ),
        (
            "a note's element",
            "nav > a.icon: 16 x 16 px",
            "nav > a.icon: 20 x 20 px",
            "`digest`",
        ),
        (
            "a note of a count the catalogue does not name",
            "{\"environment\": 3, \"term\": \"low_contrast\",",
            "{\"environment\": 3, \"term\": \"regions\",",
            "`regions`",
        ),
        (
            "a note of an environment the record does not have",
            "{\"environment\": 3, \"term\": \"low_contrast\",",
            "{\"environment\": 4, \"term\": \"low_contrast\",",
            "environment 4",
        ),
        (
            "a note of a count that is 0",
            "{\"environment\": 3, \"term\": \"low_contrast\",",
            "{\"environment\": 3, \"term\": \"overflow\",",
            "`overflow`, which counts 0",
        ),
    ];
    for &(name, from, to, says) in changes {
        assert!(text.contains(from), "{name}: the record has no `{from}`");
        let refused = match Recorded::parse(&text.replacen(from, to, 1)) {
            Ok(_) => panic!("{name}: the record is read"),
            Err(refused) => refused,
        };
        assert!(refused.contains(says), "{name}: `{refused}` lacks `{says}`");
    }
    let digest = small_record().digest();
    let other = format!("sha256:{}", "0".repeat(64));
    assert!(text.contains(&digest));
    let refused = Recorded::parse(&text.replacen(&digest, &other, 1)).expect_err("another digest");
    assert!(refused.contains("`digest`"), "{refused}");
    assert!(Recorded::parse("[]").is_err());
    assert!(Recorded::parse("{").is_err());
}

// Terms, ranking and the frontier.

#[test]
fn hard_is_summed_over_the_environments_and_every_other_term_is_the_worst_of_them() {
    let declaration = small();
    let mut renders = vec![render(200, 0); 4];
    renders[1].primary_px = 260;
    renders[2].primary_in_view = false;
    renders[2].h_scroll = true;
    renders[3].unnamed = 3;
    renders[3].overflow = 5;
    renders[0].always_visible_misses = 1;
    renders[0].low_contrast = 4;
    renders[1].small_targets = 7;
    renders[2].regions = 3;
    renders[3].text_runs = 30;
    renders[3].text_under_14 = 17;
    assert_eq!(report::term_names(&declaration)[0], "hard");
    assert_eq!(
        report::terms(&declaration, &renders),
        [3, 1, 4, 566, 5, 260, 3, 7]
    );
    assert_eq!(
        report::terms_text(&declaration, &report::terms(&declaration, &renders)),
        "hard=3 always_visible_misses=1 low_contrast=4 text_under_14=0.566 overflow=5 primary_px=260 regions=3 small_targets=7"
    );
}

/// Only the hard rules a declaration names are counted; `contrast` is one when it is named.
#[test]
fn only_the_declared_hard_rules_count() {
    let mut renders = vec![render(200, 2); 4];
    renders[0].primary_in_view = false;
    renders[0].h_scroll = true;
    renders[0].unnamed = 1;
    assert_eq!(report::terms(&small(), &renders)[0], 3);
    let contrast = Declaration::parse(&small_text().replace(
        "hard: [primary_in_view, no_h_scroll, named]",
        "hard: [contrast]",
    ))
    .expect("a declaration");
    assert_eq!(report::terms(&contrast, &renders)[0], 4);
    let none = Declaration::parse(
        &small_text().replace("hard: [primary_in_view, no_h_scroll, named]", "hard: []"),
    )
    .expect("a declaration");
    assert_eq!(report::terms(&none, &renders)[0], 0);
}

/// The share of text under 14 px is written with three places, the same on every run; a page
/// without text has none.
#[test]
fn the_share_of_small_text_is_a_fixed_point_number() {
    let declaration = small();
    let mut renders = vec![render(200, 0); 4];
    for (runs, under, text) in [
        (39, 22, "0.564"),
        (40, 40, "1.000"),
        (40, 0, "0.000"),
        (0, 0, "0.000"),
        (3, 1, "0.333"),
    ] {
        for render in &mut renders {
            render.text_runs = runs;
            render.text_under_14 = under;
        }
        let terms = report::terms(&declaration, &renders);
        assert!(
            report::terms_text(&declaration, &terms).contains(&format!(" text_under_14={text} ")),
            "{under} of {runs}: {}",
            report::terms_text(&declaration, &terms)
        );
    }
}

#[test]
fn the_ranking_is_lexicographic_and_the_frontier_holds_no_dominated_variant() {
    let recorded = small_record();
    let ranked = report::rank(&recorded);
    let order: Vec<&str> = ranked
        .ranking
        .iter()
        .map(|&index| ranked.scored[index].id.as_str())
        .collect();
    assert_eq!(
        order,
        ["tight.deep", "stock.deep", "tight.stock", "stock.stock"]
    );
    let frontier: Vec<&str> = ranked
        .frontier
        .iter()
        .map(|&index| ranked.scored[index].id.as_str())
        .collect();
    assert_eq!(frontier, ["tight.deep"]);
}

#[test]
fn a_variant_that_breaks_a_hard_rule_stands_on_no_frontier() {
    let mut recorded = small_record();
    // The best variant scrolls sideways in one environment.
    recorded.variants[1].1[3].h_scroll = true;
    let ranked = report::rank(&recorded);
    let frontier: Vec<&str> = ranked
        .frontier
        .iter()
        .map(|&index| ranked.scored[index].id.as_str())
        .collect();
    assert_eq!(frontier, ["stock.deep", "tight.stock"]);
    assert_eq!(
        ranked.scored[*ranked.ranking.last().expect("a last")].id,
        "tight.deep"
    );
}

const SMALL_REPORT: &str = "# ux-search campaign: the page /orders/list on the browser adapter
# adapter ux-browser 0.10.0, browser HeadlessChrome/1.2.3.4
# declaration DIGEST
# axes: spacing 2 (tight, stock) x ink 2 (stock, deep) = 4 variants
# environments 4: 1366x768 light, 1366x768 dark, 390x844 light, 390x844 dark; 16 renders
# hard rules: primary_in_view, no_h_scroll, named
# terms, lexicographic, lower is better: hard always_visible_misses low_contrast text_under_14 overflow primary_px regions small_targets; hard summed over the environments, each other term the worst of them; ties by variant id
# frontier: of the variants with hard=0, those no other of them beats on one term without losing on another; equal terms stand on it together
shipped stock.stock: rank 4 of 4, 3 variants before it on their terms, 0 equal to it, off the frontier
frontier 1, sets of terms 1
   1 tight.deep hard=0 always_visible_misses=0 low_contrast=0 text_under_14=0.250 overflow=0 primary_px=210 regions=2 small_targets=1
     :root { --space: 0.5rem; --gap: 0.25rem }
     light: :root { --ink: #123456 }
     dark: :root[data-theme=\"dark\"] { --ink: #abcdef }
     .badge { --badge-ink: var(--ink) }
ranking 4, sets of terms 4
   1 tight.deep hard=0 always_visible_misses=0 low_contrast=0 text_under_14=0.250 overflow=0 primary_px=210 regions=2 small_targets=1
   2 stock.deep hard=0 always_visible_misses=0 low_contrast=0 text_under_14=0.250 overflow=0 primary_px=250 regions=2 small_targets=1
   3 tight.stock hard=0 always_visible_misses=0 low_contrast=2 text_under_14=0.250 overflow=0 primary_px=210 regions=2 small_targets=1
   4 stock.stock hard=0 always_visible_misses=0 low_contrast=2 text_under_14=0.250 overflow=0 primary_px=250 regions=2 small_targets=1
";

#[test]
fn the_report_prints_the_frontier_with_its_patches_then_the_ranking() {
    let recorded = small_record();
    assert_eq!(
        report::report(&recorded),
        Ok(SMALL_REPORT.replace("DIGEST", &recorded.declaration.digest))
    );
}

const SMALL_MARKDOWN: &str = "| | |
|---|---|
| page | `/orders/list` |
| variants | 4: spacing 2 x ink 2 |
| environments | 4: 1366x768 light, 1366x768 dark, 390x844 light, 390x844 dark |
| renders measured | 16 |
| variants that break a hard rule | 0 |
| frontier | variants 1, sets of terms 1 |
| the page as shipped | `stock.stock`: rank 4 of 4, 3 variants before it on their terms, 0 equal to it, off the frontier |
| browser | `HeadlessChrome/1.2.3.4` |
| adapter | `ux-browser 0.10.0` |
| declaration | `DIGEST` |

| rank | variant | terms | 1366x768 light | 1366x768 dark | 390x844 dark |
|---|---|---|---|---|---|
| 1 | `tight.deep` | `hard=0 always_visible_misses=0 low_contrast=0 text_under_14=0.250 overflow=0 primary_px=210 regions=2 small_targets=1` | [png](shots/tight.deep.1366x768-light.png) | [png](shots/tight.deep.1366x768-dark.png) | [png](shots/tight.deep.390x844-dark.png) |
";

#[test]
fn the_report_as_markdown_is_the_run_and_the_frontier_with_its_screenshots() {
    let recorded = small_record();
    assert_eq!(
        report::markdown(&recorded, "shots/"),
        Ok(SMALL_MARKDOWN.replace("DIGEST", &recorded.declaration.digest))
    );
}

/// A record whose variants are not the declaration's is not ranked.
#[test]
fn a_record_of_other_variants_is_not_reported() {
    let mut recorded = small_record();
    recorded.variants.pop();
    let refused = report::report(&recorded).expect_err("three of four variants");
    assert!(refused.contains("3 variants"), "{refused}");
    let mut recorded = small_record();
    recorded.variants[0].1.pop();
    let refused = report::report(&recorded).expect_err("three of four renders");
    assert!(refused.contains("tight.stock"), "{refused}");
}

const SMALL_NOTES: &str = "notes: the first elements behind each count, at most 3 per term and environment, of the page as shipped and of the frontier
shipped stock.stock
     1366x768 light low_contrast: main > p.hint \"Updated a minute ago\": 3.10 under 4.5, #8a8a8a on #ffffff
     1366x768 light, 390x844 dark low_contrast: td.muted \"n/a\": 2.32 under 4.5, #aaaaaa on #ffffff
     1366x768 light small_targets: nav > a.icon: 16 x 16 px
   1 tight.deep
     1366x768 light small_targets: nav > a.icon: 16 x 16 px
";

/// What `campaign report --notes` adds: the elements the record names for the page as shipped
/// and for each frontier variant, one line per element, in the record's order.
#[test]
fn the_notes_name_the_elements_of_the_page_as_shipped_and_of_the_frontier() {
    let recorded = small_record();
    assert_eq!(report::notes(&recorded), Ok(SMALL_NOTES.to_owned()));

    // A variant the record names nothing for says so, and why.
    let mut unnamed = small_record();
    unnamed.notes[1].clear();
    let notes = report::notes(&unnamed).expect("the notes");
    assert!(
        notes.ends_with("   1 tight.deep\n     the record names no element for it\n"),
        "{notes}"
    );
    let mut clean = small_record();
    clean.notes[1].clear();
    for render in &mut clean.variants[1].1 {
        render.small_targets = 0;
    }
    let notes = report::notes(&clean).expect("the notes");
    assert!(
        notes.ends_with(
            "   1 tight.deep\n     nothing to name: low_contrast, overflow, small_targets and always_visible_misses are 0 in every environment\n"
        ),
        "{notes}"
    );

    // The page as shipped on the frontier is named once.
    let mut alone = small_record();
    for (_, renders) in &mut alone.variants {
        *renders = vec![render(200, 2); 4];
    }
    alone.notes = vec![Vec::new(); 4];
    alone.notes[2] = vec![note(0, "low_contrast", &[PALE_CELL])];
    let notes = report::notes(&alone).expect("the notes");
    assert!(
        notes.contains("shipped stock.stock\n     1366x768 light low_contrast: td.muted"),
        "{notes}"
    );
    assert!(
        notes.contains(" stock.stock\n     the page as shipped, above\n"),
        "{notes}"
    );
}

/// An element the same in every environment of every variant the notes are printed for is
/// printed once, before the variants, with how many variants and environments it stands for;
/// one the same in every environment of one variant is printed once for it.
#[test]
fn the_notes_name_an_element_once_when_it_is_the_same_in_every_environment_and_variant() {
    let mut recorded = small_record();
    recorded.notes[1] = (0..4)
        .map(|at| note(at, "small_targets", &[SMALL_ICON]))
        .collect();
    recorded.notes[2] = vec![
        note(0, "low_contrast", &[PALE_HINT, PALE_CELL]),
        note(0, "small_targets", &[SMALL_ICON]),
        note(1, "low_contrast", &[PALE_CELL]),
        note(1, "small_targets", &[SMALL_ICON]),
        note(2, "low_contrast", &[PALE_CELL]),
        note(2, "small_targets", &[SMALL_ICON]),
        note(3, "low_contrast", &[PALE_CELL]),
        note(3, "small_targets", &[SMALL_ICON]),
    ];
    assert_eq!(
        report::notes(&recorded),
        Ok(format!(
            "notes: the first elements behind each count, at most 3 per term and environment, of the page as shipped and of the frontier
the same in every environment of every variant below, 2 variants in 4 environments, 8 renders:
     small_targets: {SMALL_ICON}
shipped stock.stock
     1366x768 light low_contrast: {PALE_HINT}
     all 4 environments low_contrast: {PALE_CELL}
   1 tight.deep
     only the elements of every variant, above
"
        ))
    );
}

/// A mode, or a viewport, whose renders measure the same numbers as another one's for every
/// variant is named, one line each; one that differs in one render is not.
#[test]
fn a_mode_or_a_viewport_that_measures_what_another_one_measures_is_named() {
    let mut recorded = small_record();
    assert_eq!(report::idle_environments(&recorded), Vec::<String>::new());
    // The second environment, 1366x768 dark, measured 10 px more than the others.
    for (_, renders) in &mut recorded.variants {
        renders[1].primary_px -= 10;
    }
    assert_eq!(
        report::idle_environments(&recorded),
        [
            "mode dark: every variant measures the same numbers as in the mode light, at every viewport",
            "viewport 390x844: every variant measures the same numbers as at 1366x768, in every mode",
        ]
    );
    // One render of one variant differs: the viewport changes a number, the mode none.
    recorded.variants[3].1[3].overflow = 1;
    recorded.variants[3].1[2].overflow = 1;
    assert_eq!(
        report::idle_environments(&recorded),
        [
            "mode dark: every variant measures the same numbers as in the mode light, at every viewport"
        ]
    );
}

/// The renders of a restricted run are no campaign: the report refuses their file.
#[test]
fn the_renders_of_a_restricted_run_are_not_reported() {
    let text = small_record().text().replace(
        "\"format\": \"ux-search-campaign-measurements/3\"",
        "\"format\": \"ux-search-campaign-renders/1\"",
    );
    let refused = Recorded::parse(&text).expect_err("the renders of a restricted run");
    assert!(refused.contains("a restricted run"), "{refused}");
}

/// A restriction to a variant or an environment the declaration does not have is refused with
/// the ones it has, before a browser starts.
#[test]
fn campaign_run_refuses_a_restriction_the_declaration_does_not_have() {
    let directory = crate::state_dir("campaign-run-restricted");
    std::fs::create_dir_all(&directory).expect("a scratch directory");
    let declaration = directory.join("small.campaign.yaml");
    std::fs::write(&declaration, small_text()).expect("a file");
    let out = directory.join("measurements.json");
    let _ = std::fs::remove_file(&out);
    for (flag, value, known) in [
        (
            "--variant",
            "loose.deep",
            "tight.stock, tight.deep, stock.stock, stock.deep",
        ),
        (
            "--environment",
            "1366x768 dim",
            "1366x768 light, 1366x768 dark, 390x844 light, 390x844 dark",
        ),
    ] {
        let output = ux_search(&[
            "campaign",
            "run",
            "--declaration",
            path(&declaration),
            "--url",
            "http://127.0.0.1:1",
            "--out",
            path(&out),
            flag,
            value,
        ]);
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        let said = String::from_utf8_lossy(&output.stderr);
        assert!(said.contains(&format!("{flag} `{value}`")), "{said}");
        assert!(said.contains(known), "{said}");
        assert!(!said.contains("chromium"), "{said}");
        assert!(!out.exists());
    }
}

/// A frontier variant's screenshots: every mode at the first viewport, and the last
/// environment. So a mode the page does not take shows in a screenshot, and so does the last
/// viewport.
#[test]
fn a_frontier_variant_is_shot_in_every_mode_and_in_the_last_environment() {
    let declaration = small();
    let environments = declaration.environments();
    assert_eq!(run::screenshots_of(&declaration), [0, 1, 3]);
    assert_eq!(run::screenshot_environments(2, 2), [0, 1, 3]);
    assert_eq!(run::screenshot_environments(1, 1), [0]);
    assert_eq!(run::screenshot_environments(1, 2), [0, 1]);
    assert_eq!(run::screenshot_environments(3, 1), [0, 2]);
    assert_eq!(run::screenshot_environments(3, 3), [0, 1, 2, 8]);
    assert_eq!(
        run::screenshot_name("tight.deep", &environments[3]),
        "tight.deep.390x844-dark.png"
    );
}

/// The probe returns every number the record holds, under the column's name.
#[test]
fn the_probe_measures_every_column_of_the_record() {
    for column in record::COLUMNS {
        assert!(
            run::PROBE.contains(&format!("\n      {column}: ")),
            "probes/campaign.js returns no `{column}`"
        );
    }
}

// The campaign on the studio.

fn studio(name: &str) -> PathBuf {
    fixture("studio").join(name)
}

fn studio_declaration() -> Declaration {
    Declaration::read(&studio("frontier.campaign.yaml")).unwrap_or_else(|error| panic!("{error}"))
}

fn studio_record() -> Recorded {
    let path = studio("measurements.json");
    Recorded::parse(&read(&path))
        .unwrap_or_else(|error| panic!("{}: {error}; record again: {RECORD}", path.display()))
}

/// The proof on the studio: one page of the generated app, at least three axes and 24 variants,
/// the two gating viewports of the loop round in both modes, and no property the studio's theme
/// does not declare.
#[test]
fn the_studio_declaration_is_over_properties_the_studio_theme_declares() {
    let declaration = studio_declaration();
    assert!(
        declaration.axes.len() >= 3,
        "{} axes",
        declaration.axes.len()
    );
    assert!(declaration.variants().len() >= 24);
    let environments: Vec<String> = declaration
        .environments()
        .iter()
        .map(declaration::Environment::name)
        .collect();
    assert_eq!(
        environments,
        [
            "1366x768 light",
            "1366x768 dark",
            "1280x720 light",
            "1280x720 dark"
        ]
    );
    let theme = read(&repository("themes/studio.css"));
    let mut properties = BTreeSet::new();
    for axis in &declaration.axes {
        assert!(
            axis.values[axis.shipped].rules.is_empty(),
            "the shipped value of `{}` sets a property: a change of the theme would leave it behind",
            axis.name
        );
        for value in &axis.values {
            for rule in &value.rules {
                for (property, _) in &rule.css.values {
                    assert!(
                        patch::custom_property(&theme, ":root", property).is_some(),
                        "{}: themes/studio.css declares no `{property}` in `:root`",
                        rule.key
                    );
                    properties.insert(property.clone());
                }
            }
        }
    }
    assert!(properties.len() >= 3, "{properties:?}");
}

#[test]
fn the_studio_record_is_of_the_committed_declaration() {
    let (declaration, recorded) = (studio_declaration(), studio_record());
    assert_eq!(
        recorded.declaration.digest, declaration.digest,
        "the measurements were recorded for another declaration; record again: {RECORD}"
    );
    assert_eq!(recorded.variants.len(), declaration.variants().len());
    assert!(
        crate::adapter_version(&recorded.adapter).is_some(),
        "the adapter line `{}` is not `ux-browser <version>`",
        recorded.adapter
    );
    assert!(recorded.browser.contains("Chrome/"), "{}", recorded.browser);
    assert_eq!(recorded.text(), read(&studio("measurements.json")));
}

/// The studio campaign's files name versions in three places, and each is the version of the
/// recording, not of this build, so a release changes none of them: the adapter line of the
/// measurements (above), the report and the document's recorded block, which are printed from the
/// measurements (`the_studio_report_is_the_committed_one_and_its_frontier_is_sound`,
/// `the_campaign_document_holds_the_numbers_of_the_record`), and the engine line of the
/// document's `versions` block, which the script writes from the first line of
/// `ux-search --version` and which is held to that form here.
#[test]
fn the_campaign_document_names_the_engine_of_the_recording() {
    let document = read(&repository(&format!("{SHOTS}.md")));
    let made_with = document
        .split_once("<!-- versions: begin -->\n")
        .and_then(|(_, rest)| rest.split_once("<!-- versions: end -->"))
        .map(|(block, _)| block)
        .unwrap_or_else(|| panic!("{SHOTS}.md has no versions block"));
    let engines: Vec<&str> = made_with
        .lines()
        .filter_map(|line| line.strip_prefix("| engine | `"))
        .filter_map(|rest| rest.strip_suffix("` |"))
        .collect();
    assert_eq!(engines.len(), 1, "one engine line: {made_with}");
    let version = engines[0].strip_prefix("ux-search ");
    assert!(
        version.is_some_and(crate::is_semver),
        "the engine line `{}` is not `ux-search <version>`; record again: {RECORD}",
        engines[0]
    );
    let adapter = format!("| adapter | `{}` |", studio_record().adapter);
    assert!(
        document.lines().any(|line| line == adapter),
        "{SHOTS}.md has no line `{adapter}`"
    );
}

#[test]
fn the_studio_report_is_the_committed_one_and_its_frontier_is_sound() {
    let recorded = studio_record();
    let ranked = report::rank(&recorded);
    let text = report::report(&recorded).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        text,
        read(&studio("report.txt")),
        "the report over the committed measurements is not the committed report; record again: {RECORD}"
    );
    assert_eq!(ranked.ranking.len(), recorded.variants.len());
    assert!(
        !ranked.frontier.is_empty(),
        "the campaign ends without a frontier"
    );
    for &index in &ranked.frontier {
        let terms = &ranked.scored[index].terms;
        assert_eq!(
            terms[0], 0,
            "{} breaks a hard rule",
            ranked.scored[index].id
        );
        assert!(
            !ranked
                .scored
                .iter()
                .any(|other| other.terms[0] == 0 && ux_search::rank::dominates(&other.terms, terms)),
            "{} is dominated",
            ranked.scored[index].id
        );
    }
    let shipped = recorded.declaration.shipped();
    let place = ranked
        .ranking
        .iter()
        .position(|&index| ranked.scored[index].id == shipped.id)
        .expect("the page as shipped is ranked")
        + 1;
    assert!(
        text.contains(&format!(
            "\nshipped {}: rank {place} of {}, ",
            shipped.id,
            recorded.variants.len()
        )),
        "{text}"
    );
}

/// The screenshots kept beside the campaign's document are those of the recorded frontier, one
/// per variant and screenshot environment, and nothing else lies there.
#[test]
fn the_kept_screenshots_are_the_frontier_of_the_studio_record() {
    let recorded = studio_record();
    let ranked = report::rank(&recorded);
    let environments = recorded.declaration.environments();
    let mut expected = BTreeSet::new();
    for &index in &ranked.frontier {
        for at in run::screenshots_of(&recorded.declaration) {
            expected.insert(run::screenshot_name(
                &ranked.scored[index].id,
                &environments[at],
            ));
        }
    }
    let directory = repository(SHOTS);
    let kept: BTreeSet<String> = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("{}: {error}", directory.display()))
        .map(|entry| {
            entry
                .expect("a directory entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(kept, expected, "record again: {RECORD}");
}

/// The campaign's document holds the run and the frontier as the record gives them: the script
/// writes that part again with every recording.
#[test]
fn the_campaign_document_holds_the_numbers_of_the_record() {
    let document = read(&repository(&format!("{SHOTS}.md")));
    let name = Path::new(SHOTS)
        .file_name()
        .expect("a name")
        .to_string_lossy();
    let block = report::markdown(&studio_record(), &format!("{name}/"))
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(
        document.contains(&format!("<!-- recorded: begin -->\n{block}")),
        "{SHOTS}.md does not hold the run and the frontier of the committed measurements; record again: {RECORD}\n{block}"
    );
}

/// The script records into the places these tests read.
#[test]
fn the_campaign_script_records_into_the_places_the_tests_read() {
    let script = read(&repository("tools/studio-campaign.sh"));
    for part in [
        "fixtures=engine/search/tests/fixtures/campaign/studio\n",
        &format!("shots={SHOTS}\n"),
        &format!("document={SHOTS}.md\n"),
        "campaign run --declaration \"$fixtures/frontier.campaign.yaml\"",
        "campaign report --measurements",
        "cmp ",
    ] {
        assert!(
            script.contains(part),
            "tools/studio-campaign.sh has no `{part}`"
        );
    }
}

// The command.

fn ux_search(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ux-search"))
        .args(args)
        .output()
        .expect("ux-search runs")
}

fn path(path: &Path) -> &str {
    path.to_str().expect("a UTF-8 path")
}

#[test]
fn campaign_report_prints_the_report_of_a_measurements_file() {
    let measurements = studio("measurements.json");
    let output = ux_search(&["campaign", "report", "--measurements", path(&measurements)]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        read(&studio("report.txt"))
    );
    let output = ux_search(&[
        "campaign",
        "report",
        "--measurements",
        path(&measurements),
        "--format",
        "markdown",
        "--links",
        "shots/",
    ]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        report::markdown(&studio_record(), "shots/").expect("the report")
    );
}

#[test]
fn campaign_report_refuses_a_tampered_file_and_names_it() {
    let directory = crate::state_dir("campaign-report-tampered");
    std::fs::create_dir_all(&directory).expect("a scratch directory");
    let tampered = directory.join("measurements.json");
    std::fs::write(
        &tampered,
        small_record()
            .text()
            .replacen("[1,0,0,200,", "[1,0,0,100,", 1),
    )
    .expect("a file");
    let output = ux_search(&["campaign", "report", "--measurements", path(&tampered)]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty());
    let said = String::from_utf8_lossy(&output.stderr);
    assert!(said.contains(path(&tampered)), "{said}");
    assert!(said.contains("`digest`"), "{said}");
    assert!(!said.contains("usage:"), "{said}");
}

#[test]
fn campaign_refuses_a_file_it_cannot_read_and_a_wrong_invocation() {
    let missing = fixture("no-such.json");
    let output = ux_search(&["campaign", "report", "--measurements", path(&missing)]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty());
    let small = fixture("small.campaign.yaml");
    for args in [
        &["campaign"][..],
        &["campaign", "climb"],
        &["campaign", "report"],
        &[
            "campaign",
            "report",
            "--measurements",
            "m.json",
            "--format",
            "html",
        ],
        &[
            "campaign",
            "report",
            "--measurements",
            "m.json",
            "--links",
            "shots/",
        ],
        &["campaign", "run", "--declaration", path(&small)],
        &[
            "campaign",
            "run",
            "--declaration",
            path(&small),
            "--url",
            "http://127.0.0.1:1",
        ],
        &[
            "campaign",
            "run",
            "--declaration",
            path(&small),
            "--url",
            "http://127.0.0.1:1",
            "--out",
            "m.json",
            "--jobs",
            "0",
        ],
        &[
            "campaign",
            "run",
            "--declaration",
            path(&small),
            "--url",
            "http://127.0.0.1:1/?theme=dark",
            "--out",
            "m.json",
        ],
        &[
            "campaign",
            "run",
            "--declaration",
            path(&small),
            "--url",
            "127.0.0.1:1",
            "--out",
            "m.json",
        ],
    ] {
        let output = ux_search(args);
        assert_eq!(output.status.code(), Some(2), "{args:?}: {output:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("usage:"),
            "{args:?}"
        );
    }
}

/// A declaration that is refused starts no browser and writes nothing; the refusal names the
/// file and the key.
#[test]
fn campaign_run_refuses_a_declaration_before_it_starts_a_browser() {
    let directory = crate::state_dir("campaign-run-refused");
    std::fs::create_dir_all(&directory).expect("a scratch directory");
    let declaration = directory.join("broken.campaign.yaml");
    std::fs::write(
        &declaration,
        small_text().replace("terms: [always_visible_misses,", "terms: [beauty,"),
    )
    .expect("a file");
    let out = directory.join("measurements.json");
    let shots = directory.join("shots");
    let _ = std::fs::remove_file(&out);
    let output = ux_search(&[
        "campaign",
        "run",
        "--declaration",
        path(&declaration),
        "--url",
        "http://127.0.0.1:1",
        "--out",
        path(&out),
        "--shots",
        path(&shots),
    ]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty());
    let said = String::from_utf8_lossy(&output.stderr);
    assert!(
        said.contains(&format!("{}: `terms[0]`", declaration.display())),
        "{said}"
    );
    assert!(said.contains("beauty"), "{said}");
    assert!(!out.exists());
    assert!(!shots.exists());
}

#[test]
fn the_usage_names_the_campaign_commands() {
    let help = ux_search(&["--help"]);
    assert_eq!(help.status.code(), Some(0));
    let usage = String::from_utf8_lossy(&help.stdout);
    for line in [
        "ux-search campaign run --declaration <campaign.yaml> --url <origin> --out <measurements.json> [--shots <dir>] [--jobs <n>] [--profile <dir>]",
        "ux-search campaign report --measurements <measurements.json> [--format text|markdown] [--links <prefix>] [--notes]",
        "--profile a directory of the run's own under the system's temp directory (TMPDIR), removed when the run ends",
    ] {
        assert!(usage.contains(line), "{usage}");
    }
    assert!(!usage.contains("profile-<pid>"), "{usage}");
}

/// `--help` and `-h` after a command are a question, not a flag without a value: the usage on
/// standard output and exit 0, whatever stands beside them. Every command answers so.
#[test]
fn every_command_answers_help_with_the_usage() {
    let usage = ux_search(&["--help"]).stdout;
    assert!(String::from_utf8_lossy(&usage).starts_with("usage: ux-search climb"));
    let small = fixture("small.campaign.yaml");
    for args in [
        &["campaign", "--help"][..],
        &["campaign", "-h"],
        &["campaign", "run", "--help"],
        &["campaign", "run", "-h"],
        &["campaign", "report", "--help"],
        &["campaign", "report", "-h"],
        &["campaign", "run", "--declaration", path(&small), "--help"],
        &["campaign", "report", "--help", "--measurements"],
        &["landing", "--help"],
        &["landing", "run", "--help"],
        &["landing", "report", "-h"],
        &["landing", "promote", "--help"],
        &["climb", "--help"],
        &["cost", "-h"],
    ] {
        let output = ux_search(args);
        assert_eq!(output.status.code(), Some(0), "{args:?}: {output:?}");
        assert_eq!(output.stdout, usage, "{args:?}");
        assert!(output.stderr.is_empty(), "{args:?}: {output:?}");
    }
    // The value of a flag is not a question.
    let output = ux_search(&["campaign", "report", "--measurements", "--help"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty());
}

/// Without `--profile` the browser's profile is a directory of the run's own under the system's
/// temp directory, there when the browser starts and gone when the run ends: the working
/// directory, which may be another repository, gets nothing but the files the command names.
/// The browser here is a stand-in that says where its profile is and ends.
#[test]
fn without_a_profile_flag_the_profile_is_made_under_the_temp_directory_and_removed() {
    let directory = crate::state_dir("campaign-default-profile");
    let _ = std::fs::remove_dir_all(&directory);
    let (cwd, tmp) = (directory.join("cwd"), directory.join("tmp"));
    for made in [&cwd, &tmp] {
        std::fs::create_dir_all(made).expect("a scratch directory");
    }
    let (chrome, seen) = (directory.join("chrome"), directory.join("seen"));
    crate::executable(
        &chrome,
        &format!(
            "#!/bin/sh\nfor arg in \"$@\"; do\n\tcase $arg in\n\t--user-data-dir=*)\n\t\tprofile=${{arg#--user-data-dir=}}\n\t\tif [ -d \"$profile\" ]; then echo \"there $profile\"; else echo \"absent $profile\"; fi >'{}'\n\t\t;;\n\tesac\ndone\nexit 9\n",
            seen.display()
        ),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_ux-search"))
        .current_dir(&cwd)
        .env("TMPDIR", &tmp)
        .env("UXUI_CHROME", &chrome)
        .args(["campaign", "run", "--declaration"])
        .arg(studio("frontier.campaign.yaml"))
        .args(["--url", "http://127.0.0.1:1"])
        .args(["--out", "measurements.json", "--shots", "shots"])
        .output()
        .expect("ux-search runs");
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let said = String::from_utf8_lossy(&output.stderr);
    assert!(
        said.contains("Chromium exited before it listened"),
        "{said}"
    );
    let seen = read(&seen);
    let profile = seen
        .trim_end()
        .strip_prefix("there ")
        .unwrap_or_else(|| panic!("the browser started without its profile directory: {seen}"));
    assert_eq!(
        Path::new(profile).parent(),
        Some(tmp.as_path()),
        "the profile is not a directory of the temp directory"
    );
    assert!(!Path::new(profile).exists(), "{profile} is left behind");
    for (name, directory) in [("working", &cwd), ("temp", &tmp)] {
        let left: Vec<_> = std::fs::read_dir(directory)
            .expect("the directory")
            .map(|entry| entry.expect("an entry").file_name())
            .collect();
        assert!(left.is_empty(), "the {name} directory holds {left:?}");
    }
}

// Chromium switches of the caller (`story:campaign-chrome-args`).

/// The switches Chromium started with in 0.13.1, in their order: a campaign without
/// `--chrome-arg` starts it with these, the profile and a blank page, and nothing else.
const SWITCHES_OF_0_13_1: &[&str] = &[
    "--headless=new",
    "--disable-gpu",
    "--in-process-gpu",
    "--remote-debugging-port=0",
    "--no-first-run",
    "--no-default-browser-check",
    "--disable-extensions",
    "--disable-background-networking",
    "--disable-component-update",
    "--disable-sync",
    "--disable-default-apps",
    "--disable-breakpad",
    "--disable-dev-shm-usage",
    "--metrics-recording-only",
    "--password-store=basic",
    "--use-mock-keychain",
    "--hide-scrollbars",
    "--mute-audio",
    "--force-color-profile=srgb",
    "--blink-settings=primaryHoverType=2,availableHoverTypes=2,primaryPointerType=4,availablePointerTypes=4",
];

/// What one `ux-search campaign run` with a stand-in browser did: the run's output, and the
/// browser's command line, one argument per line, or `None` when no browser was started.
struct Started {
    output: std::process::Output,
    line: Option<Vec<String>>,
    profile: PathBuf,
}

/// Runs the studio campaign with `args` after its outputs and a stand-in browser that writes
/// its arguments, one per line, and ends without listening.
fn started_with(test: &str, args: &[&str]) -> Started {
    let directory = crate::state_dir(test);
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("a scratch directory");
    let (chrome, seen) = (directory.join("chrome"), directory.join("seen"));
    crate::executable(
        &chrome,
        &format!(
            "#!/bin/sh\nfor arg in \"$@\"; do printf '%s\\n' \"$arg\"; done >'{}'\nexit 9\n",
            seen.display()
        ),
    );
    let profile = directory.join("profile");
    let output = Command::new(env!("CARGO_BIN_EXE_ux-search"))
        .current_dir(&directory)
        .env("UXUI_CHROME", &chrome)
        .args(["campaign", "run", "--declaration"])
        .arg(studio("frontier.campaign.yaml"))
        .args(["--url", "http://127.0.0.1:1", "--out", "measurements.json"])
        .arg("--profile")
        .arg(&profile)
        .args(args)
        .output()
        .expect("ux-search runs");
    let line = std::fs::read_to_string(&seen)
        .ok()
        .map(|text| text.lines().map(str::to_owned).collect());
    Started {
        output,
        line,
        profile,
    }
}

impl Started {
    /// The command line of a browser that was started, after the run ended as it does when the
    /// browser does not listen.
    fn line(&self) -> &[String] {
        assert_eq!(self.output.status.code(), Some(1), "{:?}", self.output);
        assert!(
            String::from_utf8_lossy(&self.output.stderr)
                .contains("Chromium exited before it listened"),
            "{:?}",
            self.output
        );
        self.line.as_deref().expect("the browser was started")
    }

    /// The command line of 0.13.1 with `added` after the engine's switches.
    fn expected(&self, added: &[&str]) -> Vec<String> {
        SWITCHES_OF_0_13_1
            .iter()
            .chain(added)
            .map(|arg| arg.to_string())
            .chain([
                format!("--user-data-dir={}", self.profile.display()),
                "about:blank".to_owned(),
            ])
            .collect()
    }

    /// A refused invocation: exit 2, a message that names `--chrome-arg`, and no browser.
    fn refused_naming_the_flag(&self, part: &str) {
        let said = String::from_utf8_lossy(&self.output.stderr);
        assert_eq!(self.output.status.code(), Some(2), "{said}");
        assert!(said.contains("--chrome-arg"), "{said}");
        assert!(said.contains(part), "`{said}` lacks `{part}`");
        assert!(self.line.is_none(), "a browser started: {:?}", self.line);
    }
}

#[test]
fn without_a_chrome_arg_chromium_starts_with_the_command_line_of_0_13_1() {
    let started = started_with("chrome-arg-none", &[]);
    assert_eq!(started.line(), started.expected(&[]));
}

/// Each `--chrome-arg` reaches Chromium's command line in the order given, after the engine's
/// own switches; a switch given twice is passed twice. The engine adds none of its own: no
/// `--no-sandbox` unless the caller names it.
#[test]
fn each_chrome_arg_reaches_chromium_in_the_order_given_after_the_engines_switches() {
    let started = started_with(
        "chrome-arg-order",
        &[
            "--chrome-arg",
            "--no-sandbox",
            "--chrome-arg",
            "--disable-features=Translate",
            "--chrome-arg",
            "--lang=de-DE",
            "--chrome-arg",
            "--disable-features=Translate",
        ],
    );
    assert_eq!(
        started.line(),
        started.expected(&[
            "--no-sandbox",
            "--disable-features=Translate",
            "--lang=de-DE",
            "--disable-features=Translate",
        ])
    );
    let alone = started_with("chrome-arg-alone", &["--chrome-arg", "--lang=de-DE"]);
    assert_eq!(alone.line(), alone.expected(&["--lang=de-DE"]));
    assert!(!alone.line().iter().any(|arg| arg == "--no-sandbox"));
}

/// The value of `--chrome-arg` is the browser adapter's: one switch, or a JSON list of them.
#[test]
fn a_chrome_arg_may_be_a_json_list_of_switches_as_in_the_browser_adapter() {
    let started = started_with(
        "chrome-arg-list",
        &[
            "--chrome-arg",
            r#"["--no-sandbox", "--lang=de-DE"]"#,
            "--chrome-arg",
            "--mute-audio",
        ],
    );
    assert_eq!(
        started.line(),
        started.expected(&["--no-sandbox", "--lang=de-DE", "--mute-audio"])
    );
}

/// An empty value, a word that is no switch, a switch the engine sets with another value and
/// the profile's switch are refused as a wrong invocation that names the flag, before a browser
/// starts.
#[test]
fn a_chrome_arg_that_is_empty_or_no_allowed_switch_is_refused_naming_the_flag() {
    for (test, value, part) in [
        ("chrome-arg-empty", "", "empty"),
        ("chrome-arg-blank", "  ", "empty"),
        ("chrome-arg-word", "no-sandbox", "not a Chromium switch"),
        ("chrome-arg-headless", "--headless=old", "--headless=new"),
        ("chrome-arg-profile", "--user-data-dir=x", "--profile"),
    ] {
        started_with(test, &["--chrome-arg", value]).refused_naming_the_flag(part);
    }
    let output = ux_search(&[
        "campaign",
        "run",
        "--declaration",
        path(&studio("frontier.campaign.yaml")),
        "--chrome-arg",
    ]);
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("`--chrome-arg` has no value"),
        "{output:?}"
    );
}

/// `--chrome-arg` belongs to the command that starts a browser: `campaign report` starts none
/// and takes no such flag.
#[test]
fn campaign_report_takes_no_chrome_arg() {
    let output = ux_search(&[
        "campaign",
        "report",
        "--measurements",
        path(&studio("measurements.json")),
        "--chrome-arg",
        "--no-sandbox",
    ]);
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("unknown argument `--chrome-arg`"),
        "{output:?}"
    );
}

/// The usage names the flag on the line of `campaign run`, says it repeats, and says that the
/// engine adds no switch of its own.
#[test]
fn the_usage_of_campaign_run_names_chrome_arg() {
    let output = ux_search(&["campaign", "run", "--help"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let usage = String::from_utf8_lossy(&output.stdout);
    let run_line = usage
        .lines()
        .find(|line| line.contains("ux-search campaign run "))
        .unwrap_or_else(|| panic!("{usage}"));
    assert!(
        run_line.contains("[--chrome-arg <Chromium switch, or a JSON list of them>]..."),
        "{run_line}"
    );
    assert!(
        usage.contains(
            "--chrome-arg adds a switch to Chromium's command line after the engine's own"
        ),
        "{usage}"
    );
}

/// `--notes` adds the elements behind the counts after the report; the report before them is
/// the same bytes, and a record without a note says so.
#[test]
fn campaign_report_with_notes_prints_the_report_and_then_the_notes() {
    let measurements = studio("measurements.json");
    let output = ux_search(&[
        "campaign",
        "report",
        "--measurements",
        path(&measurements),
        "--notes",
    ]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let printed = String::from_utf8_lossy(&output.stdout).into_owned();
    let report = read(&studio("report.txt"));
    assert!(printed.starts_with(&report), "{printed}");
    assert!(
        printed[report.len()..].starts_with(
            "notes: the first elements behind each count, at most 3 per term and environment, of the page as shipped and of the frontier\nshipped "
        ),
        "{}",
        &printed[report.len()..]
    );
    let output = ux_search(&[
        "campaign",
        "report",
        "--measurements",
        path(&measurements),
        "--format",
        "markdown",
        "--notes",
    ]);
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("--notes goes with --format text"),
        "{output:?}"
    );
}
