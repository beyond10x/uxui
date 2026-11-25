//! The page-scroll slack and the overflow rule as configuration (`layout`): on geometry recorded
//! from `fixtures/checks/layout-site/` (a page that scrolls 1 px sideways, a canvas 2 px past its
//! box, a `pre` 50 px past a `.wide-demo` box, an inline-block `span` 30 px past its box) and on
//! the recorded pages of the first consumer's site.
//!
//! `layout-defaults.capture.json` was recorded by ux-browser 0.3.0 (`checks --capture` with
//! `layout.checks.yaml`); its document gave page scroll 1 and overflow 2 (`div.box` at 202 past
//! 200, `div.wide-demo "x"` at 150 past 100). `layout-inputs.capture.json` was recorded with the
//! probe inputs of `INPUTS` (the default elements and `span`, `.wide-demo` excluded).

use crate::checks::LLM_LIKE;
use crate::fixture_json;
use ux_browser::checks::{self, Overflow, OverflowRaw, PageResult};
use ux_browser::evidence::{self, Header, Target};
use ux_browser::json;
use ux_browser::render::FileRef;
use uxui_doc::yaml::Value;

const DEFAULT_ELEMENTS: [&str; 10] = [
    "img", "svg", "video", "canvas", "iframe", "input", "select", "textarea", "table", "pre",
];

/// The probe inputs `layout-inputs.capture.json` was recorded with.
const INPUTS: &str = "layout:
  overflow:
    elements: [img, svg, video, canvas, iframe, input, select, textarea, table, pre, span]
    exclude: ['.wide-demo']
";

fn layout_config(extra: &str) -> checks::Config {
    let base = std::fs::read_to_string(crate::fixture("checks/layout.checks.yaml")).unwrap();
    checks::config(&format!("{base}{extra}")).unwrap()
}

fn result_of(capture: &str, extra: &str) -> PageResult {
    let page = fixture_json(&format!("checks/{capture}.capture.json"));
    checks::page_result(&layout_config(extra), &page).unwrap()
}

fn overflowed(r: &PageResult) -> Vec<(String, i64, i64, &'static str)> {
    r.units
        .iter()
        .flat_map(|u| &u.overflow)
        .map(|o| (o.desc.clone(), o.right, o.limit, o.against))
        .collect()
}

#[test]
fn without_layout_keys_the_rule_is_0_3_0_s() {
    let c = checks::config(LLM_LIKE).unwrap();
    assert_eq!(c.layout.page_slack_px, 0.0);
    assert_eq!(c.layout.overflow.tolerance_px, 1.0);
    assert_eq!(c.layout.overflow.elements, DEFAULT_ELEMENTS);
    assert!(c.layout.overflow.exclude.is_empty());
}

#[test]
fn reads_the_layout_keys() {
    let c = layout_config(
        "layout:
  page_slack_px: 1
  overflow:
    tolerance_px: 2.5
    elements: [canvas, span]
    exclude: ['.wide-demo', 'nav pre']
",
    );
    assert_eq!(c.layout.page_slack_px, 1.0);
    assert_eq!(c.layout.overflow.tolerance_px, 2.5);
    assert_eq!(c.layout.overflow.elements, ["canvas", "span"]);
    assert_eq!(c.layout.overflow.exclude, [".wide-demo", "nav pre"]);
    let partial = layout_config("layout: {page_slack_px: 0.5}\n");
    assert_eq!(partial.layout.page_slack_px, 0.5);
    assert_eq!(
        partial.layout.overflow.tolerance_px, 1.0,
        "a key left out keeps its default"
    );
    assert_eq!(partial.layout.overflow.elements, DEFAULT_ELEMENTS);
}

#[test]
fn refuses_layout_keys_that_do_not_read() {
    for wrong in [
        "layout: 1\n",
        "layout: {slack: 1}\n",
        "layout: {page_slack_px: -1}\n",
        "layout: {page_slack_px: one}\n",
        "layout:\n  overflow: 1\n",
        "layout:\n  overflow: {tolerance: 1}\n",
        "layout:\n  overflow: {tolerance_px: -0.5}\n",
        "layout:\n  overflow: {tolerance_px: true}\n",
        "layout:\n  overflow: {elements: []}\n",
        "layout:\n  overflow: {elements: img}\n",
        "layout:\n  overflow: {elements: [img, '']}\n",
        "layout:\n  overflow: {elements: [img, img]}\n",
        "layout:\n  overflow: {exclude: '.ad'}\n",
        "layout:\n  overflow: {exclude: ['']}\n",
        "layout:\n  overflow: {exclude: ['.ad', '.ad']}\n",
    ] {
        let base = std::fs::read_to_string(crate::fixture("checks/layout.checks.yaml")).unwrap();
        let error = match checks::config(&format!("{base}{wrong}")) {
            Ok(_) => panic!("read: {wrong}"),
            Err(e) => e,
        };
        assert!(error.contains("layout"), "{wrong}: {error}");
    }
}

#[test]
fn a_one_pixel_page_scroll_is_1_without_slack_and_0_with_slack_1() {
    let plain = result_of("layout-defaults", "");
    assert_eq!(plain.units[0].page_scroll_px, 1.0, "the page scrolls 1 px");
    assert_eq!(plain.page_scroll(), 1);
    assert_eq!(
        result_of("layout-defaults", "layout: {page_slack_px: 1}\n").page_scroll(),
        0
    );
    assert_eq!(
        result_of("layout-defaults", "layout: {page_slack_px: 0.5}\n").page_scroll(),
        1
    );
}

#[test]
fn the_overflow_tolerance_is_how_far_an_edge_may_pass_its_limit() {
    let canvas = ("div.box".to_string(), 202, 200, "parent");
    let pre = ("div.wide-demo \"x\"".to_string(), 150, 100, "parent");
    let plain = result_of("layout-defaults", "");
    assert_eq!(overflowed(&plain), [canvas.clone(), pre.clone()]);
    let two = result_of(
        "layout-defaults",
        "layout:\n  overflow: {tolerance_px: 2}\n",
    );
    assert_eq!(overflowed(&two), [pre]);
    assert_eq!(two.overflow(), 1);
    let fifty = result_of(
        "layout-defaults",
        "layout:\n  overflow: {tolerance_px: 50}\n",
    );
    assert_eq!(fifty.overflow(), 0, "150 is not past 100 + 50");
}

#[test]
fn an_excluded_selector_and_more_elements_change_the_overflow_as_recorded() {
    let r = result_of("layout-inputs", INPUTS);
    assert_eq!(
        overflowed(&r),
        [
            ("div.box".to_string(), 202, 200, "parent"),
            ("div.chips".to_string(), 130, 100, "parent"),
        ],
        "the pre under .wide-demo is excluded, the span is measured"
    );
    assert_eq!(
        r.page_scroll(),
        1,
        "the page scroll is no element and not excluded"
    );
    let tolerant = result_of(
        "layout-inputs",
        &INPUTS.replace("  overflow:\n", "  overflow:\n    tolerance_px: 2\n"),
    );
    assert_eq!(tolerant.overflow(), 1);
}

#[test]
fn a_capture_recorded_with_other_probe_inputs_does_not_read() {
    let defaults = fixture_json("checks/layout-defaults.capture.json");
    let inputs = fixture_json("checks/layout-inputs.capture.json");
    assert!(checks::page_result(&layout_config(INPUTS), &defaults).is_err());
    assert!(checks::page_result(&layout_config(""), &inputs).is_err());
    let rust_only = "layout:\n  page_slack_px: 1\n  overflow: {tolerance_px: 3}\n";
    assert!(
        checks::page_result(&layout_config(rust_only), &defaults).is_ok(),
        "the slack and the tolerance are applied in Rust, after the probe"
    );
}

#[test]
fn the_overflow_rule_skips_an_excluded_item_on_synthetic_geometry() {
    // items: [right, width, height, context, block, excluded]; excluded indexes `exclude`, -1 none.
    let raw = OverflowRaw::from_value(
        &json::parse(
            r#"{"vw":390,"scroll_width":390,"elements":["pre"],"exclude":[".ad"],
                "blocks":[["div.ad",200],["div.card",200]],
                "items":[[260,60,20,0,0,0],[203,40,20,0,1,-1]]}"#,
        )
        .unwrap(),
    )
    .unwrap();
    let rule = checks::OverflowRule {
        tolerance_px: 1.0,
        elements: vec!["pre".to_string()],
        exclude: vec![".ad".to_string()],
    };
    let found = checks::overflow_with(&raw, &rule);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].desc, "div.card");
    let tolerant = checks::OverflowRule {
        tolerance_px: 3.0,
        ..rule
    };
    assert!(checks::overflow_with(&raw, &tolerant).is_empty());
}

fn llm_config(extra: &str) -> checks::Config {
    let base = std::fs::read_to_string(crate::fixture("checks/llm.checks.yaml")).unwrap();
    checks::config(&format!("{base}{extra}")).unwrap()
}

/// A unit's viewport, mode, page scroll, whether it counts, and its overflow.
type Shape = (String, String, f64, bool, Vec<Overflow>);

/// What a page's results are, as the evidence reads them.
fn shape(r: &PageResult) -> Vec<Shape> {
    r.units
        .iter()
        .map(|u| {
            (
                u.viewport.clone(),
                u.mode.clone(),
                u.page_scroll_px,
                u.scrolls(),
                u.overflow.clone(),
            )
        })
        .collect()
}

#[test]
fn the_defaults_written_out_give_0_3_0_s_values_on_every_recorded_page() {
    let spelled = "layout:
  page_slack_px: 0
  overflow:
    tolerance_px: 1
    elements: [img, svg, video, canvas, iframe, input, select, textarea, table, pre]
    exclude: []
";
    for name in ["llm-run-a-local-turn", "llm-protocols", "llm-search"] {
        let page = fixture_json(&format!("checks/{name}.capture.json"));
        let plain = checks::page_result(&llm_config(""), &page).unwrap();
        let written = checks::page_result(&llm_config(spelled), &page).unwrap();
        assert_eq!((plain.page_scroll(), plain.overflow()), (0, 0), "{name}");
        assert_eq!(shape(&plain), shape(&written), "{name}");
    }
    let plain = result_of("layout-defaults", "");
    let written = result_of("layout-defaults", spelled);
    assert_eq!(
        (plain.page_scroll(), plain.overflow()),
        (1, 2),
        "0.3.0's document"
    );
    assert_eq!(shape(&plain), shape(&written));
}

fn document(config: &checks::Config, pages: &[PageResult]) -> Value {
    let file = FileRef {
        path: None,
        digest: "sha256:aa".to_string(),
    };
    let header = Header {
        project: "fixture",
        assessment: "layout",
        class: "local",
        config: &file,
        target: Target::Build(&file),
        pages: None,
        local_inputs: Vec::new(),
        browser: "Chrome/150.0.0.0",
        started_at: "2026-11-25T10:00:00Z".to_string(),
        ended_at: "2026-11-25T10:01:00Z".to_string(),
    };
    evidence::document(&header, config, pages)
}

fn other(doc: &Value) -> Vec<(String, String)> {
    doc.get("evidence")
        .and_then(|e| e.get("conditions"))
        .and_then(|c| c.get("other"))
        .unwrap()
        .entries()
        .iter()
        .filter(|(k, _)| k.starts_with("page_slack") || k.starts_with("overflow"))
        .map(|(k, v)| (k.clone(), v.str().unwrap().to_string()))
        .collect()
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[test]
fn the_evidence_conditions_name_the_rule_values_used() {
    let plain = layout_config("");
    assert_eq!(
        other(&document(&plain, &[])),
        pairs(&[
            ("page_slack_px", "0"),
            ("overflow_tolerance_px", "1"),
            (
                "overflow_elements",
                "img, svg, video, canvas, iframe, input, select, textarea, table, pre"
            ),
            ("overflow_exclude", ""),
        ])
    );
    let set = layout_config(&INPUTS.replace("layout:\n", "layout:\n  page_slack_px: 1\n"));
    assert_eq!(
        other(&document(&set, &[])),
        pairs(&[
            ("page_slack_px", "1"),
            ("overflow_tolerance_px", "1"),
            (
                "overflow_elements",
                "img, svg, video, canvas, iframe, input, select, textarea, table, pre, span"
            ),
            ("overflow_exclude", ".wide-demo"),
        ])
    );
}

#[test]
fn a_page_scroll_within_the_slack_is_no_value_and_no_finding() {
    let page = fixture_json("checks/layout-defaults.capture.json");
    let value = |extra: &str| {
        let config = layout_config(extra);
        let r = checks::page_result(&config, &page).unwrap();
        let doc = document(&config, &[r]);
        let results = doc.get("evidence").unwrap().get("results").unwrap().list();
        let scroll = results
            .iter()
            .find(|r| r.get_str("output") == Some(checks::PAGE_SCROLL))
            .and_then(|r| r.get_str("value"))
            .unwrap()
            .to_string();
        let findings = results
            .iter()
            .find(|r| r.get_str("output") == Some("findings"))
            .unwrap()
            .get("items")
            .unwrap()
            .list()
            .iter()
            .filter(|i| {
                i.get("identity").and_then(|i| i.get_str("rule")) == Some(checks::PAGE_SCROLL)
            })
            .count();
        (scroll, findings)
    };
    assert_eq!(value(""), ("1".to_string(), 1));
    assert_eq!(value("layout: {page_slack_px: 1}\n"), ("0".to_string(), 0));
}
