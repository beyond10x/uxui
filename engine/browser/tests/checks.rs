//! The five shared page checks, computed from what the in-page probes return: synthetic geometry
//! for each rule, then geometry recorded from a real run on the first consumer's documentation site
//! (`fixtures/checks/`).

use crate::fixture_json;
use ux_browser::checks::{self, Candidate, KeyboardRaw, Name, OverflowRaw, Shifts, Status};
use ux_browser::json;

fn overflow_raw(text: &str) -> OverflowRaw {
    OverflowRaw::from_value(&json::parse(text).unwrap()).unwrap()
}

#[test]
fn page_scroll_is_the_scroll_width_past_the_client_width() {
    let fits = overflow_raw(r#"{"vw":390,"scroll_width":390,"blocks":[],"items":[]}"#);
    assert_eq!(checks::page_scroll(&fits), 0.0);
    let wide = overflow_raw(r#"{"vw":390,"scroll_width":412,"blocks":[],"items":[]}"#);
    assert_eq!(checks::page_scroll(&wide), 22.0);
}

#[test]
fn overflow_counts_one_failure_per_block_past_the_viewport_or_its_block() {
    // Blocks: 0 ends at 380, 1 ends at 200. Items: [right, width, height, context, block].
    let raw = overflow_raw(
        r#"{"vw":390,"scroll_width":390,
            "blocks":[["p \"long word\"",380],["div.card",200]],
            "items":[[395,50,20,0,0],[399,60,20,0,0],[205,40,20,0,1],[201,40,20,0,1]]}"#,
    );
    let found = checks::overflow(&raw);
    let seen: Vec<(&str, i64, i64, &str)> = found
        .iter()
        .map(|o| (o.desc.as_str(), o.right, o.limit, o.against))
        .collect();
    assert_eq!(
        seen,
        [
            ("p \"long word\"", 395, 390, "viewport"),
            ("div.card", 205, 200, "parent")
        ]
    );
}

#[test]
fn overflow_leaves_out_fixed_scrolling_clipped_and_empty_items_and_the_one_pixel_slack() {
    let raw = overflow_raw(
        r#"{"vw":390,"scroll_width":390,
            "blocks":[["nav.menu",390],["pre",390],["div.clip",390],["p",300],["p.full",390]],
            "items":[[600,50,20,1,0],[900,50,20,2,1],[700,50,20,3,2],[500,0,20,0,3],[500,20,0.5,0,3],[391,20,20,0,4],[301,20,20,0,3]]}"#,
    );
    assert!(checks::overflow(&raw).is_empty());
}

#[test]
fn a_name_is_matched_exactly_or_by_a_case_blind_part() {
    assert!(Name::Is("Main".into()).matches("Main"));
    assert!(Name::Is("Main".into()).matches("  Main "));
    assert!(!Name::Is("Main".into()).matches("main"));
    assert!(!Name::Is("Main".into()).matches("Main menu"));
    let toggle = Name::Contains(vec!["dark and light mode".into()]);
    assert!(toggle.matches("Switch between Dark and Light mode (currently system mode)"));
    assert!(!toggle.matches("Dark mode"));
    let skip = Name::Contains(vec![
        "skip to content".into(),
        "skip to main content".into(),
    ]);
    assert!(skip.matches("Skip to main content"));
    assert!(Name::Any.matches(""));
}

#[test]
fn ax_candidates_are_the_unignored_nodes_of_the_role_and_name_in_tree_order() {
    let tree = json::parse(
        r#"{"nodes":[
          {"nodeId":"1","ignored":false,"role":{"value":"RootWebArea"},"name":{"value":"Docs"},"backendDOMNodeId":1},
          {"nodeId":"2","ignored":false,"role":{"value":"navigation"},"name":{"value":"Main"},"backendDOMNodeId":10},
          {"nodeId":"3","ignored":true,"role":{"value":"navigation"},"name":{"value":"Main"},"backendDOMNodeId":11},
          {"nodeId":"4","ignored":false,"role":{"value":"navigation"},"name":{"value":"Docs sidebar"},"backendDOMNodeId":12},
          {"nodeId":"5","ignored":false,"role":{"value":"navigation"},"name":{"value":"Main"},"backendDOMNodeId":13},
          {"nodeId":"6","ignored":false,"role":{"value":"button"},"name":{"value":"Main"}}
        ]}"#,
    )
    .unwrap();
    assert_eq!(
        checks::ax_candidates(&tree, "navigation", &Name::Is("Main".into())),
        [10, 13]
    );
    assert_eq!(
        checks::ax_candidates(&tree, "navigation", &Name::Any),
        [10, 12, 13]
    );
    assert!(
        checks::ax_candidates(&tree, "button", &Name::Any).is_empty(),
        "a node without a DOM node is no candidate"
    );
}

fn cand(text: &str) -> Candidate {
    Candidate::from_value(&json::parse(text).unwrap()).unwrap()
}

#[test]
fn the_first_visible_candidate_in_scope_is_checked_whole_inside_the_viewport_and_on_top() {
    let hidden = cand(
        r#"{"in_scope":true,"visible":false,"box":{"x":0,"y":0,"width":0,"height":0},"hit":false}"#,
    );
    let outside = cand(
        r#"{"in_scope":false,"visible":true,"box":{"x":10,"y":10,"width":50,"height":20},"hit":true}"#,
    );
    let ok = cand(
        r#"{"in_scope":true,"visible":true,"box":{"x":10,"y":10,"width":50,"height":20},"hit":true}"#,
    );
    assert_eq!(
        checks::first_visible(&[hidden.clone(), outside.clone(), ok.clone()]),
        Some(2)
    );
    assert_eq!(checks::first_visible(&[hidden, outside]), None);
    assert_eq!(checks::visible_status(None, 390.0, 844.0), Status::Missing);
    assert_eq!(checks::visible_status(Some(&ok), 390.0, 844.0), Status::Ok);
    let covered = cand(
        r#"{"in_scope":true,"visible":true,"box":{"x":10,"y":10,"width":50,"height":20},"hit":false}"#,
    );
    assert_eq!(
        checks::visible_status(Some(&covered), 390.0, 844.0),
        Status::Covered
    );
    let past_right = cand(
        r#"{"in_scope":true,"visible":true,"box":{"x":350,"y":10,"width":41,"height":20},"hit":true}"#,
    );
    assert_eq!(
        checks::visible_status(Some(&past_right), 390.0, 844.0),
        Status::Clipped
    );
    let half_pixel = cand(
        r#"{"in_scope":true,"visible":true,"box":{"x":-0.4,"y":0,"width":390.8,"height":20},"hit":true}"#,
    );
    assert_eq!(
        checks::visible_status(Some(&half_pixel), 390.0, 844.0),
        Status::Ok
    );
    let above = cand(
        r#"{"in_scope":true,"visible":true,"box":{"x":0,"y":-60,"width":390,"height":60},"hit":false}"#,
    );
    assert_eq!(
        checks::visible_status(Some(&above), 390.0, 844.0),
        Status::Clipped
    );
}

#[test]
fn an_item_applies_from_its_minimum_width_and_under_its_path_prefix() {
    let config = checks::config(LLM_LIKE).unwrap();
    let sidebar = config
        .always_visible
        .iter()
        .find(|a| a.name == "sidebar")
        .unwrap();
    assert!(checks::applies(sidebar, "/llm/docs/intro", 1440));
    assert!(!checks::applies(sidebar, "/llm/docs/intro", 390));
    assert!(!checks::applies(sidebar, "/llm/", 1440));
}

fn keyboard_raw(text: &str) -> KeyboardRaw {
    KeyboardRaw::from_value(&json::parse(text).unwrap()).unwrap()
}

fn skip_name() -> Name {
    Name::Contains(vec![
        "skip to content".into(),
        "skip to main content".into(),
    ])
}

#[test]
fn keyboard_reach_counts_the_skip_link_its_target_and_each_unreached_target() {
    let all_good = keyboard_raw(
        r#"{"first_label":"Skip to main content","after_enter":true,"after_tab":null,
            "located":{"search":true,"navbar":true},"tabs":[[],["navbar"],["navbar"],["navbar","search"]]}"#,
    );
    let r = checks::keyboard(&all_good, &skip_name(), 40);
    assert!(r.skip_link_first && r.skip_moves_focus);
    assert_eq!(r.misses(), 0);

    let second_tab = keyboard_raw(
        r#"{"first_label":"Skip to main content","after_enter":false,"after_tab":true,
            "located":{"search":true,"navbar":true},"tabs":[["navbar"],["search"]]}"#,
    );
    assert_eq!(checks::keyboard(&second_tab, &skip_name(), 40).misses(), 0);

    let no_skip_link = keyboard_raw(
        r#"{"first_label":"Docs","after_enter":false,"after_tab":null,
            "located":{"search":false,"navbar":true},"tabs":[["navbar"]]}"#,
    );
    let r = checks::keyboard(&no_skip_link, &skip_name(), 40);
    assert!(!r.skip_link_first && !r.skip_moves_focus);
    assert_eq!(
        r.reached,
        [("search".to_string(), false), ("navbar".to_string(), true)]
    );
    assert_eq!(r.misses(), 3);
}

#[test]
fn keyboard_reach_counts_only_the_declared_number_of_tabs() {
    let late = keyboard_raw(
        r#"{"first_label":"Skip to content","after_enter":true,"after_tab":null,
            "located":{"navbar":true},"tabs":[[],[],["navbar"]]}"#,
    );
    assert_eq!(checks::keyboard(&late, &skip_name(), 3).misses(), 0);
    assert_eq!(checks::keyboard(&late, &skip_name(), 2).misses(), 1);
}

#[test]
fn layout_shift_after_ready_sums_the_shifts_after_the_ready_time_to_four_places() {
    let shifts = Shifts::from_value(
        &json::parse(
            r#"{"ready":120.5,"entries":[[50,0.2],[120.5,0.1],[300,0.00041],[310,0.00012]]}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(checks::shift_after_ready(&shifts), Some(0.0005));
    assert_eq!(checks::shift_load(&shifts), Some(0.3005));
    let unsupported =
        Shifts::from_value(&json::parse(r#"{"ready":1,"entries":null}"#).unwrap()).unwrap();
    assert_eq!(checks::shift_after_ready(&unsupported), None);
}

#[test]
fn reads_the_paths_of_a_sitemap_sorted_and_once() {
    let xml = "<urlset><url><loc>https://example.org/llm/docs/b</loc></url><url><loc>https://example.org/llm/</loc></url><url><loc>https://example.org/llm/docs/b</loc></url></urlset>";
    assert_eq!(checks::sitemap_paths(xml), ["/llm/", "/llm/docs/b"]);
}

pub const LLM_LIKE: &str = "format: ux-browser-checks/1
base_path: /llm/
pages: {sitemap: sitemap.xml}
viewports:
  - {name: 1440x900, width: 1440, height: 900}
  - {name: 390x844, width: 390, height: 844}
modes: [light, dark]
mode_attribute: data-theme
reduced_motion: true
settle_ms: 250
always_visible:
  - name: navbar
    role: navigation
    name_is: Main
  - name: search
    role: searchbox
    name_contains: [search]
    css: '.navbar__search-input, .navbar__search'
    scope: .navbar
  - name: sidebar
    role: navigation
    name_is: Docs sidebar
    min_width: 997
    path_prefix: /llm/docs
keyboard:
  skip_link_contains: [skip to content, skip to main content]
  must_reach: [search, navbar]
  max_tabs: 40
";

#[test]
fn reads_a_checks_configuration_with_its_defaults() {
    let c = checks::config(LLM_LIKE).unwrap();
    assert_eq!(c.base_path, "/llm/");
    assert_eq!(c.viewports.len(), 2);
    assert_eq!(c.viewports[1].name, "390x844");
    assert_eq!(c.modes, ["light", "dark"]);
    assert_eq!(c.settle_ms, 250);
    assert_eq!(c.scroll_settle_ms, 200);
    assert_eq!(c.mode_attribute.as_deref(), Some("data-theme"));
    let k = c.keyboard.as_ref().unwrap();
    assert_eq!(k.max_tabs, 40);
    assert_eq!(
        k.modes,
        ["dark"],
        "the keyboard walk defaults to the last mode"
    );
    assert_eq!(k.must_reach, ["search", "navbar"]);
}

#[test]
fn refuses_a_configuration_that_does_not_hold_together() {
    for (from, to) in [
        ("must_reach: [search, navbar]", "must_reach: [footer]"),
        ("modes: [light, dark]", "modes: [sepia]"),
        ("base_path: /llm/", "base_path: llm"),
        (
            "    name_is: Main\n",
            "    name_is: Main\n    name_contains: [x]\n",
        ),
        ("settle_ms: 250", "settle_ms: 250\naxe: true"),
        ("format: ux-browser-checks/1", "format: other/1"),
    ] {
        assert!(checks::config(&LLM_LIKE.replace(from, to)).is_err(), "{to}");
    }
}

#[test]
fn a_page_may_send_reads_to_its_server_unless_the_configuration_allows_more() {
    assert_eq!(checks::READ_METHODS, ["GET", "HEAD"]);
    let c = checks::config(LLM_LIKE).unwrap();
    assert_eq!(c.request_methods, ["GET", "HEAD"]);
    let more = LLM_LIKE.replace(
        "settle_ms: 250\n",
        "settle_ms: 250\nrequest_methods: [GET, HEAD, POST]\n",
    );
    assert_eq!(
        checks::config(&more).unwrap().request_methods,
        ["GET", "HEAD", "POST"]
    );
    for wrong in [
        "[HEAD]",
        "[GET, post]",
        "[GET, GET]",
        "[]",
        "GET",
        "[GET, 'PO ST']",
    ] {
        let text = LLM_LIKE.replace(
            "settle_ms: 250\n",
            &format!("settle_ms: 250\nrequest_methods: {wrong}\n"),
        );
        assert!(checks::config(&text).is_err(), "request_methods: {wrong}");
    }
}

fn llm_config() -> checks::Config {
    checks::config(&std::fs::read_to_string(crate::fixture("checks/llm.checks.yaml")).unwrap())
        .unwrap()
}

/// A guide of the first consumer's site at both gating viewports in both modes, recorded by
/// `ux-browser checks --capture`: every check 0, as llm's own harness has it on the same build.
#[test]
fn the_recorded_guide_page_computes_llm_s_values() {
    let page = fixture_json("checks/llm-run-a-local-turn.capture.json");
    let result = checks::page_result(&llm_config(), &page).unwrap();
    assert_eq!(result.path, "/llm/docs/guides/run-a-local-turn");
    assert_eq!(result.error, None);
    assert_eq!(result.units.len(), 4, "two viewports by two modes");
    assert_eq!(result.page_scroll(), 0);
    assert_eq!(result.overflow(), 0);
    assert_eq!(result.always_visible_misses(), 0);
    assert_eq!(result.keyboard_misses(), 0);
    assert_eq!(result.shift_after_ready(), Some(0.0));
    assert_eq!(
        result.keyboard.len(),
        2,
        "one walk per viewport, in the last mode"
    );
    // The sidebar is checked from 997 px wide only: 4 items at 1440, 3 at 390, top and bottom.
    let checked = |vp: &str| {
        result
            .units
            .iter()
            .filter(|u| u.viewport == vp)
            .flat_map(|u| &u.visible)
            .filter(|v| v.status != Status::NotApplicable)
            .count()
    };
    assert_eq!((checked("1440x900"), checked("390x844")), (16, 12));
}

/// The one page with a layout shift after ready on this build: 0.0004 at 1440 x 900, none at
/// 390 x 844; llm's harness reads the same.
#[test]
fn the_recorded_protocols_page_computes_llm_s_layout_shift() {
    let page = fixture_json("checks/llm-protocols.capture.json");
    let result = checks::page_result(&llm_config(), &page).unwrap();
    let by_viewport: Vec<(&str, Option<f64>)> = result
        .shifts
        .iter()
        .map(|s| (s.viewport.as_str(), s.after_ready))
        .collect();
    assert_eq!(
        by_viewport,
        [("1440x900", Some(0.0004)), ("390x844", Some(0.0))]
    );
    assert_eq!(result.shift_after_ready(), Some(0.0004));
}

/// The search page, where llm's baseline holds 4 keyboard misses (its finding UXF-005: the query
/// box takes focus on load, so the skip link is not the first Tab stop, at both viewports).
#[test]
fn the_recorded_search_page_computes_llm_s_four_keyboard_misses() {
    let page = fixture_json("checks/llm-search.capture.json");
    let result = checks::page_result(&llm_config(), &page).unwrap();
    assert_eq!(result.keyboard_misses(), 4);
    assert!(result.keyboard.iter().all(|k| !k.result.skip_link_first));
}
