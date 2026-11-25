//! The keyboard walk without a skip link, to declared targets, with a focus mark and, per target,
//! Enter or Space that acts (`docs/browser-adapter.md`, The keyboard walk): the configuration, the
//! decisions on recorded walks, and what the document says of them.

use crate::checks::LLM_LIKE;
use crate::fixture_json;
use ux_browser::checks::{self, Activation, FocusStyle, KeyboardConfig, KeyboardRaw, Name};
use ux_browser::evidence::{self, Header, Target};
use ux_browser::json;
use ux_browser::render::FileRef;
use uxui_doc::yaml::Value;

const SKIP_LINE: &str = "  skip_link_contains: [skip to content, skip to main content]\n";

/// LLM_LIKE without its skip link, with `extra` lines added to its keyboard section.
fn without_skip_link(extra: &str) -> String {
    let text = LLM_LIKE.replace(SKIP_LINE, "");
    assert_ne!(text, LLM_LIKE, "LLM_LIKE declares a skip link");
    format!("{text}{extra}")
}

fn keyboard_config(text: &str) -> KeyboardConfig {
    checks::config(text).unwrap().keyboard.unwrap()
}

fn raw(text: &str) -> KeyboardRaw {
    KeyboardRaw::from_value(&json::parse(text).unwrap()).unwrap()
}

/// A walk recorded without a skip link: no first Tab stop, both targets reached.
const NO_SKIP_WALK: &str =
    r#"{"located":{"search":true,"navbar":true},"tabs":[[],["navbar"],["search"]]}"#;

#[test]
fn the_skip_link_is_optional() {
    let with = keyboard_config(LLM_LIKE);
    assert_eq!(
        with.skip_link,
        Some(Name::Contains(vec![
            "skip to content".into(),
            "skip to main content".into()
        ]))
    );
    assert!(!with.focus_mark, "focus_mark defaults to false");
    assert!(with.activates.is_empty(), "activates defaults to none");
    let without = keyboard_config(&without_skip_link(""));
    assert_eq!(without.skip_link, None);
    assert_eq!(without.must_reach, ["search", "navbar"]);
}

#[test]
fn reads_focus_mark_and_activates() {
    let k = keyboard_config(&without_skip_link(
        "  focus_mark: true\n  activates: [navbar]\n",
    ));
    assert!(k.focus_mark);
    assert_eq!(k.activates, ["navbar"]);
    let off = keyboard_config(&without_skip_link("  focus_mark: false\n"));
    assert!(!off.focus_mark);
}

#[test]
fn refuses_a_keyboard_section_that_does_not_hold_together() {
    for extra in [
        "  focus_mark: 1\n",
        "  focus_mark: yes please\n",
        "  activates: [footer]\n",
        "  activates: [navbar, navbar]\n",
        "  activates: navbar\n",
        "  skip_link_is: Skip\n  skip_link_contains: [skip]\n",
        "  skip_link_contains: []\n",
        "  focus_ring: true\n",
    ] {
        assert!(
            checks::config(&without_skip_link(extra)).is_err(),
            "{extra}"
        );
    }
}

#[test]
fn without_a_skip_link_a_walk_has_no_skip_link_misses() {
    let k = keyboard_config(&without_skip_link(""));
    let r = checks::keyboard_of(&raw(NO_SKIP_WALK), &k).unwrap();
    assert!(!r.skip_link, "no skip-link checks without a skip link");
    assert_eq!(r.misses(), 0);
    // The same walk where the page's first Tab stop is no skip link: still nothing to miss.
    let first = raw(
        r#"{"first_label":"Outbox","after_enter":false,"after_tab":null,
            "located":{"search":true,"navbar":true},"tabs":[["navbar"],["search"]]}"#,
    );
    assert_eq!(checks::keyboard_of(&first, &k).unwrap().misses(), 0);
}

#[test]
fn an_unreached_target_is_one_miss() {
    let k = keyboard_config(&without_skip_link(""));
    let late = raw(r#"{"located":{"search":true,"navbar":true},"tabs":[["navbar"]]}"#);
    let r = checks::keyboard_of(&late, &k).unwrap();
    assert_eq!(
        r.reached,
        [("search".to_string(), false), ("navbar".to_string(), true)]
    );
    assert_eq!(r.misses(), 1);
    let absent = raw(r#"{"located":{"search":false,"navbar":true},"tabs":[["navbar"]]}"#);
    assert_eq!(checks::keyboard_of(&absent, &k).unwrap().misses(), 1);
}

fn style(text: &str) -> FocusStyle {
    FocusStyle::from_value(&json::parse(text).unwrap()).unwrap()
}

#[test]
fn a_focus_mark_is_a_drawn_outline_or_a_box_shadow_on_a_focus_visible_element() {
    let marked = [
        r#"{"element":"a","focus_visible":true,"outline_style":"solid","outline_width":2,"outline_color":"rgb(80, 140, 255)","box_shadow":"none"}"#,
        r#"{"element":"a","focus_visible":true,"outline_style":"auto","outline_width":1,"outline_color":"rgb(16, 16, 16)","box_shadow":"none"}"#,
        r#"{"element":"button","focus_visible":true,"outline_style":"none","outline_width":0,"outline_color":"rgb(0, 0, 0)","box_shadow":"rgb(80, 140, 255) 0px 0px 0px 3px"}"#,
    ];
    for m in marked {
        assert!(checks::focus_marked(&style(m)), "{m}");
    }
    let unmarked = [
        r#"{"element":"a","focus_visible":true,"outline_style":"none","outline_width":0,"outline_color":"rgb(0, 0, 0)","box_shadow":"none"}"#,
        r#"{"element":"a","focus_visible":true,"outline_style":"solid","outline_width":0,"outline_color":"rgb(0, 0, 0)","box_shadow":"none"}"#,
        r#"{"element":"a","focus_visible":true,"outline_style":"solid","outline_width":2,"outline_color":"rgba(0, 0, 0, 0)","box_shadow":"none"}"#,
        r#"{"element":"a","focus_visible":true,"outline_style":"solid","outline_width":2,"outline_color":"transparent","box_shadow":"none"}"#,
        r#"{"element":"a","focus_visible":false,"outline_style":"solid","outline_width":2,"outline_color":"rgb(0, 0, 0)","box_shadow":"none"}"#,
    ];
    for m in unmarked {
        assert!(!checks::focus_marked(&style(m)), "{m}");
    }
}

/// A walk without a skip link that reached both targets, with the focused element's style.
fn marked_walk(search_outline: &str) -> KeyboardRaw {
    raw(&format!(
        r#"{{"located":{{"search":true,"navbar":true}},"tabs":[["navbar"],["search"]],
            "marks":{{
              "navbar":{{"element":"a.navbar__brand","focus_visible":true,"outline_style":"auto","outline_width":1,"outline_color":"rgb(16, 16, 16)","box_shadow":"none"}},
              "search":{{"element":"input.navbar__search-input","focus_visible":true,"outline_style":"{search_outline}","outline_width":{w},"outline_color":"rgb(16, 16, 16)","box_shadow":"none"}}}}}}"#,
        w = if search_outline == "none" { 0 } else { 2 }
    ))
}

#[test]
fn a_reached_target_without_a_focus_mark_is_one_miss() {
    let k = keyboard_config(&without_skip_link("  focus_mark: true\n"));
    let present = checks::keyboard_of(&marked_walk("solid"), &k).unwrap();
    assert_eq!(present.misses(), 0);
    assert_eq!(present.focus_marks.len(), 2, "one judged mark per target");
    let absent = checks::keyboard_of(&marked_walk("none"), &k).unwrap();
    assert_eq!(absent.misses(), 1);
    let miss = absent.focus_marks.iter().find(|j| !j.ok).unwrap();
    assert_eq!(miss.target, "search");
    assert!(
        miss.detail.contains("input.navbar__search-input"),
        "{}",
        miss.detail
    );
    // Without `focus_mark` the same walk misses nothing: the marks are not judged.
    let off = keyboard_config(&without_skip_link(""));
    let r = checks::keyboard_of(&marked_walk("none"), &off).unwrap();
    assert_eq!(r.misses(), 0);
    assert!(r.focus_marks.is_empty());
}

#[test]
fn an_unreached_target_is_not_judged_for_its_focus_mark() {
    let k = keyboard_config(&without_skip_link("  focus_mark: true\n"));
    let walk = raw(
        r#"{"located":{"search":false,"navbar":true},"tabs":[["navbar"]],
            "marks":{"navbar":{"element":"a","focus_visible":true,"outline_style":"auto","outline_width":1,"outline_color":"rgb(16, 16, 16)","box_shadow":"none"}}}"#,
    );
    let r = checks::keyboard_of(&walk, &k).unwrap();
    assert_eq!(
        r.misses(),
        1,
        "the target is not reached; no mark is judged"
    );
    assert_eq!(r.focus_marks.len(), 1);
}

#[test]
fn a_reached_target_with_no_recorded_mark_is_refused() {
    let k = keyboard_config(&without_skip_link("  focus_mark: true\n"));
    assert!(checks::keyboard_of(&raw(NO_SKIP_WALK), &k).is_err());
}

fn activation(text: &str) -> Activation {
    Activation::from_value(&json::parse(text).unwrap()).unwrap()
}

const NOTHING: &str =
    r#"{"navigated":false,"focus_moved":false,"state_changed":false,"dom_changed":false}"#;

#[test]
fn enter_or_space_acts_when_the_page_shows_a_change() {
    for fact in ["navigated", "focus_moved", "state_changed", "dom_changed"] {
        let enter = NOTHING.replace(&format!("\"{fact}\":false"), &format!("\"{fact}\":true"));
        let a = activation(&format!(
            r#"{{"refocused":true,"enter":{enter},"space":null}}"#
        ));
        assert!(checks::acted(&a), "enter {fact}");
    }
    let space = NOTHING.replace("\"dom_changed\":false", "\"dom_changed\":true");
    assert!(checks::acted(&activation(&format!(
        r#"{{"refocused":true,"enter":{NOTHING},"space":{space}}}"#
    ))));
    assert!(!checks::acted(&activation(&format!(
        r#"{{"refocused":true,"enter":{NOTHING},"space":{NOTHING}}}"#
    ))));
    assert!(
        !checks::acted(&activation(
            r#"{"refocused":false,"enter":null,"space":null}"#
        )),
        "a target not focused again is not activated"
    );
}

/// A walk without a skip link that reached both targets, with the activation of `navbar`.
fn activated_walk(navbar: &str) -> KeyboardRaw {
    raw(&format!(
        r#"{{"located":{{"search":true,"navbar":true}},"tabs":[["navbar"],["search"]],
            "activations":{{"navbar":{navbar}}}}}"#
    ))
}

#[test]
fn a_target_that_does_not_act_is_one_miss() {
    let k = keyboard_config(&without_skip_link("  activates: [navbar]\n"));
    let nav = NOTHING.replace("\"navigated\":false", "\"navigated\":true");
    let acts = activated_walk(&format!(
        r#"{{"refocused":true,"enter":{nav},"space":null}}"#
    ));
    let r = checks::keyboard_of(&acts, &k).unwrap();
    assert_eq!(r.misses(), 0);
    assert_eq!(r.activated.len(), 1);
    let still = activated_walk(&format!(
        r#"{{"refocused":true,"enter":{NOTHING},"space":{NOTHING}}}"#
    ));
    let r = checks::keyboard_of(&still, &k).unwrap();
    assert_eq!(r.misses(), 1);
    assert_eq!(r.activated[0].target, "navbar");
    assert!(!r.activated[0].ok);
    // A target that was not reached is not activated: its one miss is the reach.
    let unreached = raw(r#"{"located":{"search":true,"navbar":true},"tabs":[["search"]]}"#);
    let r = checks::keyboard_of(&unreached, &k).unwrap();
    assert_eq!(r.misses(), 1);
    assert!(r.activated.is_empty());
}

fn llm_config() -> checks::Config {
    checks::config(&std::fs::read_to_string(crate::fixture("checks/llm.checks.yaml")).unwrap())
        .unwrap()
}

/// A configuration of 0.4.0 (a skip link, no focus mark, no activation) counts every recorded
/// walk as 0.4.0 did.
#[test]
fn the_defaults_count_as_0_4_0_did() {
    let config = llm_config();
    let k = config.keyboard.as_ref().unwrap();
    let skip = k.skip_link.clone().unwrap();
    let mut walks = 0;
    for (file, misses) in [
        ("checks/llm-run-a-local-turn.capture.json", 0),
        ("checks/llm-protocols.capture.json", 0),
        ("checks/llm-search.capture.json", 4),
    ] {
        let page = fixture_json(file);
        for vp in page.get("viewports").unwrap().list() {
            for walk in vp.get("keyboard").map(Value::list).unwrap_or_default() {
                let raw = KeyboardRaw::from_value(walk.get("walk").unwrap()).unwrap();
                let old = checks::keyboard(&raw, &skip, k.max_tabs);
                let new = checks::keyboard_of(&raw, k).unwrap();
                assert_eq!(new, old, "{file}");
                assert!(new.skip_link);
                walks += 1;
            }
        }
        let result = checks::page_result(&config, &page).unwrap();
        assert_eq!(result.keyboard_misses(), misses, "{file}");
    }
    assert_eq!(walks, 6, "two walks per recorded page");
}

fn control_plane() -> checks::Config {
    checks::config(
        &std::fs::read_to_string(crate::fixture("checks/control-plane.checks.yaml")).unwrap(),
    )
    .unwrap()
}

/// A page of the second consumer's snapshot (a control plane web application, no skip link),
/// recorded by `ux-browser checks --capture` and cut to its walks: both walks reach the screens
/// navigation, its first link shows its outline, Enter on it navigates. 0 misses, as the
/// consumer's own keyboard rule has on the same snapshot (0.3.0 counted 4: two skip-link checks per
/// walk).
#[test]
fn the_recorded_control_plane_page_misses_nothing_without_a_skip_link() {
    let page = fixture_json("checks/control-plane-today.capture.json");
    let result = checks::page_result(&control_plane(), &page).unwrap();
    assert_eq!(result.keyboard.len(), 2);
    for walk in &result.keyboard {
        assert!(!walk.result.skip_link);
        assert_eq!(walk.result.reached, [("screens".to_string(), true)]);
        assert_eq!(walk.result.focus_marks.len(), 1);
        assert_eq!(walk.result.activated.len(), 1);
    }
    assert_eq!(result.keyboard_misses(), 0);
}

/// The same page with `outline: none` on focus (a scratch copy of the page, the control of the
/// focus-mark check): one miss per walk.
#[test]
fn the_recorded_control_plane_page_without_its_outline_misses_its_focus_mark() {
    let page = fixture_json("checks/control-plane-no-outline.capture.json");
    let result = checks::page_result(&control_plane(), &page).unwrap();
    assert_eq!(result.keyboard_misses(), 2);
    for walk in &result.keyboard {
        let mark = &walk.result.focus_marks[0];
        assert!(!mark.ok);
        assert!(mark.detail.contains("outline none"), "{}", mark.detail);
    }
}

/// A recorded page at one viewport with one walk, no other unit.
fn page_with_walk(walk: &str) -> Value {
    json::parse(&format!(
        r#"{{"path":"/llm/docs/x","viewports":[{{"viewport":"1440x900","shifts":{{"ready":1,"entries":[]}},
            "units":[],"keyboard":[{{"mode":"dark","walk":{walk}}}]}}]}}"#
    ))
    .unwrap()
}

fn document(config: &checks::Config, pages: &[checks::PageResult]) -> Value {
    let config_file = FileRef {
        path: Some("checks.yaml".to_string()),
        digest: "sha256:aa".to_string(),
    };
    let build = FileRef {
        path: None,
        digest: "sha256:bb".to_string(),
    };
    let header = Header {
        project: "p",
        assessment: "page-checks",
        class: "local",
        config: &config_file,
        target: Target::Build(&build),
        pages: None,
        local_inputs: Vec::new(),
        browser: "Chrome/150.0.0.0",
        started_at: "2026-11-25T10:00:00Z".to_string(),
        ended_at: "2026-11-25T10:01:00Z".to_string(),
    };
    evidence::document(&header, config, pages)
}

fn result<'a>(doc: &'a Value, output: &str) -> &'a Value {
    doc.get("evidence")
        .unwrap()
        .get("results")
        .unwrap()
        .list()
        .iter()
        .find(|r| r.get_str("output") == Some(output))
        .unwrap()
}

#[test]
fn the_document_names_each_keyboard_miss_by_its_check_and_target() {
    let config = checks::config(&without_skip_link("  focus_mark: true\n")).unwrap();
    let walk = r#"{"located":{"search":true,"navbar":true},"tabs":[["navbar"]],
        "marks":{"navbar":{"element":"a.navbar__brand","focus_visible":true,"outline_style":"none","outline_width":0,"outline_color":"rgb(0, 0, 0)","box_shadow":"none"}}}"#;
    let page = checks::page_result(&config, &page_with_walk(walk)).unwrap();
    assert_eq!(page.keyboard_misses(), 2);
    let doc = document(&config, &[page]);
    let number = result(&doc, checks::KEYBOARD);
    assert_eq!(number.get_str("value"), Some("2"));
    let walks = number.get("details").unwrap().get("walks").unwrap().list();
    assert_eq!(walks.len(), 1);
    let keys: Vec<&str> = walks[0].entries().iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(keys, ["viewport", "mode", "reached", "focus_marks"]);
    let findings: Vec<(String, String)> = result(&doc, evidence::FINDINGS)
        .get("items")
        .unwrap()
        .list()
        .iter()
        .map(|i| {
            (
                i.get("identity")
                    .unwrap()
                    .get_str("node_path")
                    .unwrap()
                    .to_string(),
                i.get("fields")
                    .unwrap()
                    .get_str("reason")
                    .unwrap()
                    .to_string(),
            )
        })
        .collect();
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert_eq!(findings[0].0, "1440x900/dark/tab-reaches-search");
    assert_eq!(findings[0].1, "missed");
    assert_eq!(findings[1].0, "1440x900/dark/focus-mark-navbar");
    assert!(
        findings[1].1.contains("a.navbar__brand"),
        "{}",
        findings[1].1
    );
    let coverage = doc.get("evidence").unwrap().get("coverage").unwrap();
    // Two targets and one judged mark (the unreached target has none); no skip-link check.
    assert_eq!(coverage.get("failed"), Some(&Value::Int(2)));
    assert_eq!(coverage.get("passed"), Some(&Value::Int(1)));
}
