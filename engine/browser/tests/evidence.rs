//! The `ratchet-evidence/1` document of a checks run: target, inputs, conditions, coverage, one
//! number per check and page, and the failed checks of each page as an identity set.

use crate::checks::LLM_LIKE;
use ux_browser::checks::{
    self, KeyboardResult, KeyboardUnit, Overflow, PageResult, ShiftUnit, Status, UnitResult,
    Visible,
};
use ux_browser::evidence::{self, Header};
use ux_browser::render::FileRef;
use uxui_doc::yaml::Value;

/// The tree digest of `fixtures/site`, computed outside the crate:
/// `find . -type f | sort | xargs sha256sum` lines (`<hex>  <path>`), then `sha256sum` of them.
const SITE_TREE_DIGEST: &str =
    "sha256:9fd557d3566c4f8cb267de5e6f70b77742657647666fe1484373bfa51df24147";

#[test]
fn the_tree_digest_is_the_digest_of_the_sorted_file_digest_lines() {
    assert_eq!(
        evidence::tree_digest(&crate::fixture("site")).unwrap(),
        SITE_TREE_DIGEST
    );
}

#[test]
fn times_are_utc_seconds() {
    assert_eq!(evidence::utc(0), "1970-01-01T00:00:00Z");
    assert_eq!(evidence::utc(1_795_920_556), "2026-11-29T02:49:16Z");
    assert_eq!(evidence::utc(951_782_400), "2000-02-29T00:00:00Z");
    assert_eq!(evidence::utc(4_107_542_399), "2100-02-28T23:59:59Z");
}

fn unit(
    viewport: &str,
    mode: &str,
    scroll: f64,
    overflow: usize,
    seen: &[(&str, Status)],
) -> UnitResult {
    UnitResult {
        viewport: viewport.to_string(),
        mode: mode.to_string(),
        mode_applied: true,
        page_scroll_px: scroll,
        overflow: (0..overflow)
            .map(|i| Overflow {
                desc: format!("p.wide{i}"),
                right: 420,
                limit: 390,
                against: "viewport",
            })
            .collect(),
        visible: seen
            .iter()
            .map(|(item, status)| Visible {
                item: item.to_string(),
                position: "top",
                status: *status,
            })
            .collect(),
    }
}

fn walk(viewport: &str, skip_first: bool) -> KeyboardUnit {
    KeyboardUnit {
        viewport: viewport.to_string(),
        mode: "dark".to_string(),
        result: KeyboardResult {
            skip_link_first: skip_first,
            skip_moves_focus: skip_first,
            reached: vec![("search".to_string(), true), ("navbar".to_string(), true)],
        },
    }
}

fn shift(viewport: &str, after_ready: f64) -> ShiftUnit {
    ShiftUnit {
        viewport: viewport.to_string(),
        after_ready: Some(after_ready),
        load: Some(0.01),
    }
}

fn pages() -> Vec<PageResult> {
    let home = PageResult {
        path: "/llm/".to_string(),
        error: None,
        units: vec![
            unit("1440x900", "light", 0.0, 0, &[("navbar", Status::Ok)]),
            unit("390x844", "light", 0.0, 0, &[("navbar", Status::Ok)]),
        ],
        keyboard: vec![walk("1440x900", true), walk("390x844", true)],
        shifts: vec![shift("1440x900", 0.0), shift("390x844", 0.0004)],
    };
    let wide = PageResult {
        path: "/llm/docs/wide".to_string(),
        error: None,
        units: vec![
            unit(
                "1440x900",
                "light",
                0.0,
                0,
                &[("navbar", Status::Ok), ("search", Status::NotApplicable)],
            ),
            unit("390x844", "light", 30.0, 2, &[("navbar", Status::Covered)]),
        ],
        keyboard: vec![walk("1440x900", false), walk("390x844", true)],
        shifts: vec![shift("1440x900", 0.0)],
    };
    let gone = PageResult {
        path: "/llm/gone".to_string(),
        error: Some("HTTP status 404".to_string()),
        units: Vec::new(),
        keyboard: Vec::new(),
        shifts: Vec::new(),
    };
    vec![home, wide, gone]
}

fn document() -> Value {
    let config = checks::config(LLM_LIKE).unwrap();
    let config_file = FileRef {
        path: Some("engine/browser/examples/llm.checks.yaml".to_string()),
        digest: "sha256:aa".to_string(),
    };
    let build = FileRef {
        path: None,
        digest: SITE_TREE_DIGEST.to_string(),
    };
    let header = Header {
        project: "llm-docs",
        assessment: "page-checks",
        class: "local",
        config: &config_file,
        build: &build,
        browser: "Chrome/150.0.0.0",
        started_at: "2026-11-24T10:00:00Z".to_string(),
        ended_at: "2026-11-24T10:01:00Z".to_string(),
    };
    evidence::document(&header, &config, &pages())
}

fn result<'a>(doc: &'a Value, output: &str, item: &str) -> &'a Value {
    doc.get("evidence")
        .unwrap()
        .get("results")
        .unwrap()
        .list()
        .iter()
        .find(|r| r.get_str("output") == Some(output) && r.get_str("item") == Some(item))
        .unwrap_or_else(|| panic!("no {output} for {item}"))
}

#[test]
fn the_document_names_the_assessor_the_build_and_the_conditions() {
    let doc = document();
    assert_eq!(doc.get_str("format"), Some("ratchet-evidence/1"));
    let e = doc.get("evidence").unwrap();
    assert_eq!(e.get_str("assessor"), Some("ux-browser@1"));
    assert_eq!(e.get_str("candidate"), Some(SITE_TREE_DIGEST));
    let target = e.get("target").unwrap();
    assert_eq!(target.get_str("kind"), Some("document"));
    assert_eq!(target.get_str("selection"), Some("candidate"));
    let ids = target.get("identifiers").unwrap();
    assert_eq!(ids.get_str("digest"), Some(SITE_TREE_DIGEST));
    assert_eq!(ids.get_str("digest_form"), Some("tree"));
    assert_eq!(
        ids.get("path"),
        None,
        "a build outside the root has no path"
    );
    let config = e.get("inputs").unwrap().get("config").unwrap();
    assert_eq!(
        config.get_str("value"),
        Some("engine/browser/examples/llm.checks.yaml")
    );
    let conditions = e.get("conditions").unwrap();
    let tools = conditions.get("tools").unwrap();
    assert_eq!(tools.get_str("ux-browser"), Some(env!("CARGO_PKG_VERSION")));
    assert_eq!(tools.get_str("chromium"), Some("Chrome/150.0.0.0"));
    let other = conditions.get("other").unwrap();
    assert_eq!(other.get_str("viewports"), Some("1440x900 390x844"));
    assert_eq!(other.get_str("modes"), Some("light dark"));
    assert_eq!(other.get_str("settle_ms"), Some("250"));
    let run = doc.get("run").unwrap();
    assert_eq!(run.get_str("started_at"), Some("2026-11-24T10:00:00Z"));
    assert_eq!(run.get_str("ended_at"), Some("2026-11-24T10:01:00Z"));
}

#[test]
fn each_page_has_one_number_per_check() {
    let doc = document();
    let value = |output: &str, item: &str| {
        result(&doc, output, item)
            .get_str("value")
            .map(str::to_string)
    };
    assert_eq!(value(checks::PAGE_SCROLL, "/llm/").as_deref(), Some("0"));
    assert_eq!(
        value(checks::PAGE_SCROLL, "/llm/docs/wide").as_deref(),
        Some("1")
    );
    assert_eq!(
        value(checks::OVERFLOW, "/llm/docs/wide").as_deref(),
        Some("2")
    );
    assert_eq!(
        value(checks::ALWAYS_VISIBLE, "/llm/docs/wide").as_deref(),
        Some("1")
    );
    assert_eq!(
        value(checks::KEYBOARD, "/llm/docs/wide").as_deref(),
        Some("2")
    );
    assert_eq!(value(checks::KEYBOARD, "/llm/").as_deref(), Some("0"));
    let shift = result(&doc, checks::LAYOUT_SHIFT, "/llm/");
    assert_eq!(
        shift.get_str("value"),
        Some("0.0004"),
        "the largest over the viewports"
    );
    assert_eq!(shift.get_str("kind"), Some("sampled"));
    assert_eq!(shift.get("samples"), Some(&Value::Int(2)));
    let overflow = result(&doc, checks::OVERFLOW, "/llm/");
    assert_eq!(overflow.get_str("kind"), Some("exact"));
    assert_eq!(overflow.get_str("scope"), Some("page"));
}

#[test]
fn a_page_that_did_not_load_has_no_values_and_its_checks_are_not_run() {
    let doc = document();
    for output in [
        checks::PAGE_SCROLL,
        checks::OVERFLOW,
        checks::ALWAYS_VISIBLE,
        checks::KEYBOARD,
        checks::LAYOUT_SHIFT,
        evidence::FINDINGS,
    ] {
        let r = result(&doc, output, "/llm/gone");
        assert_eq!(r.get("value"), None, "{output}");
        let why = r.get("incomplete").unwrap();
        assert_eq!(why.get_str("cause"), Some("incompatible_target"));
    }
    let coverage = doc.get("evidence").unwrap().get("coverage").unwrap();
    let not_executed = coverage.get("not_executed").unwrap().list();
    assert!(not_executed.iter().any(|n| {
        n.get_str("id") == Some("/llm/gone") && n.get_str("state") == Some("not_run")
    }));
}

#[test]
fn coverage_counts_each_check_once_and_the_counts_add_up() {
    let doc = document();
    let c = doc.get("evidence").unwrap().get("coverage").unwrap();
    let n = |k: &str| match c.get(k) {
        Some(Value::Int(n)) => *n,
        other => panic!("{k}: {other:?}"),
    };
    assert_eq!(c.get_str("unit"), Some("check"));
    // Home: 2 units x (scroll, overflow, navbar) = 6, 2 walks x 4 = 8, 2 shifts counted.
    // Wide: 2 units x (scroll, overflow) + 2 applicable visible = 6, 2 walks x 4 = 8, 1 shift.
    assert_eq!(n("executed"), 6 + 8 + 2 + 6 + 8 + 1);
    assert_eq!(
        n("failed"),
        3 + 2,
        "scroll, overflow and the covered navbar of one unit; two skip-link checks"
    );
    assert_eq!(
        n("passed"),
        n("executed") - n("failed") - 3,
        "shifts are counted, not judged"
    );
    assert_eq!(n("not_run"), 1);
}

#[test]
fn the_failed_checks_of_a_page_are_its_findings() {
    let doc = document();
    let set = result(&doc, evidence::FINDINGS, "/llm/docs/wide");
    assert_eq!(set.get_str("value_type"), Some("identity_set"));
    let ids: Vec<(String, String)> = set
        .get("items")
        .unwrap()
        .list()
        .iter()
        .map(|i| {
            let id = i.get("identity").unwrap();
            (
                id.get_str("rule").unwrap().to_string(),
                id.get_str("node_path").unwrap().to_string(),
            )
        })
        .collect();
    let want = [
        (checks::PAGE_SCROLL, "390x844/light/page"),
        (checks::OVERFLOW, "390x844/light/p.wide0"),
        (checks::OVERFLOW, "390x844/light/p.wide1"),
        (checks::ALWAYS_VISIBLE, "390x844/light/navbar@top"),
        (checks::KEYBOARD, "1440x900/dark/skip-link-first"),
        (checks::KEYBOARD, "1440x900/dark/skip-link-moves-focus"),
    ];
    let want: Vec<(String, String)> = want
        .iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect();
    assert_eq!(ids, want);
    let home = result(&doc, evidence::FINDINGS, "/llm/");
    assert!(home.get("items").unwrap().list().is_empty());
}

#[test]
fn the_document_text_is_compact_with_one_result_per_line() {
    let text = evidence::text(&document());
    assert!(text.ends_with("}\n"));
    assert_eq!(ux_browser::json::parse(&text).unwrap(), document());
    assert!(text.lines().count() > 10);
}
