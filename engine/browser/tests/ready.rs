//! A ready condition the page sets (`ready`, `ready_timeout_ms`): the configuration and its refusals,
//! the probe that watches the page for it, what a page that does not set it in time records, and the
//! condition the document names. Without the key every value and every reading is 0.3.0's.

use crate::checks::LLM_LIKE;
use ux_browser::checks::{self, Ready};
use ux_browser::evidence::{self, Header};
use ux_browser::json;
use ux_browser::measure::{self, Failure};
use ux_browser::render::FileRef;
use uxui_doc::yaml::Value;

fn with(extra: &str) -> String {
    format!("{LLM_LIKE}{extra}")
}

#[test]
fn without_ready_the_page_is_ready_at_load_as_before() {
    let c = checks::config(LLM_LIKE).unwrap();
    assert!(c.ready.is_none());
    assert_eq!(c.ready_timeout_ms, checks::DEFAULT_READY_TIMEOUT_MS);
    assert_eq!(checks::DEFAULT_READY_TIMEOUT_MS, 10_000);
    assert_eq!(checks::ready_condition(&c), "load");
    assert_eq!(
        measure::ready_init(&c),
        None,
        "no script beside the shift observer"
    );
}

#[test]
fn reads_a_selector_or_a_global_and_a_timeout() {
    let c = checks::config(&with(
        "ready: {selector: 'main[data-ready]'}\nready_timeout_ms: 4000\n",
    ))
    .unwrap();
    assert_eq!(
        c.ready,
        Some(Ready::Selector("main[data-ready]".to_string()))
    );
    assert_eq!(c.ready_timeout_ms, 4000);
    assert_eq!(checks::ready_condition(&c), "selector main[data-ready]");
    let g = checks::config(&with("ready: {global: appReady}\n")).unwrap();
    assert_eq!(g.ready, Some(Ready::Global("appReady".to_string())));
    assert_eq!(g.ready_timeout_ms, 10_000);
    assert_eq!(checks::ready_condition(&g), "global appReady");
}

#[test]
fn refuses_a_ready_condition_that_does_not_read() {
    for extra in [
        "ready: '[data-ready]'\n",
        "ready: {}\n",
        "ready: {selector: '[data-ready]', global: appReady}\n",
        "ready: {selector: ''}\n",
        "ready: {selector: '   '}\n",
        "ready: {global: ''}\n",
        "ready: {global: 'app.ready'}\n",
        "ready: {global: '1st'}\n",
        "ready: {global: 'a b'}\n",
        "ready: {attribute: data-ready}\n",
        "ready: {selector: '[data-ready]', timeout_ms: 5}\n",
        "ready: {selector: '[data-ready]'}\nready_timeout_ms: 0\n",
        "ready: {selector: '[data-ready]'}\nready_timeout_ms: -1\n",
        "ready: {selector: '[data-ready]'}\nready_timeout_ms: soon\n",
        "ready_timeout_ms: 5000\n",
    ] {
        let refused = checks::config(&with(extra));
        assert!(refused.is_err(), "{extra:?} is refused");
        let message = refused.err().unwrap();
        assert!(message.contains("ready"), "{extra:?}: {message}");
        assert!(
            !message.contains("unknown key `ready"),
            "{extra:?} is refused for its value, not as an unknown key: {message}"
        );
    }
}

#[test]
fn the_probe_watches_the_page_and_then_waits_for_fonts_and_two_frames() {
    let probe = measure::READY_PROBE;
    assert!(
        probe
            .trim_start()
            .starts_with("(function (kind, value, ms)")
    );
    assert!(
        probe.contains("MutationObserver"),
        "a selector is watched, not polled"
    );
    assert!(probe.contains("addEventListener('uxb:ready'"));
    assert!(probe.contains("setTimeout"), "the declared time bounds it");
    assert!(!probe.contains("setInterval"), "no polling in the page");
    let fonts = probe.find("document.fonts.ready").expect("fonts");
    let frames = probe
        .find("requestAnimationFrame(() => requestAnimationFrame(")
        .expect("two frames");
    let watched = probe.find("MutationObserver").unwrap();
    assert!(
        watched < fonts && fonts < frames,
        "the condition, then fonts, then two frames"
    );
    assert!(probe.contains("performance.now()"), "the ready instant");
}

#[test]
fn a_global_is_trapped_before_the_page_s_own_scripts() {
    let c = checks::config(&with("ready: {global: appReady}\n")).unwrap();
    let init = measure::ready_init(&c).expect("an init script for a global");
    assert!(init.contains("Object.defineProperty(window, name"));
    assert!(init.contains("uxb:ready"));
    assert!(
        init.trim_end().ends_with("(\"appReady\")"),
        "the name is passed as a JSON string: {init}"
    );
    let s = checks::config(&with("ready: {selector: '[data-ready]'}\n")).unwrap();
    assert_eq!(measure::ready_init(&s), None, "a selector needs none");
}

#[test]
fn the_probe_s_answer_is_the_ready_time_or_the_page_s_failure() {
    let c = checks::config(&with(
        "ready: {selector: '[data-ready]'}\nready_timeout_ms: 4000\n",
    ))
    .unwrap();
    let answer = |text: &str| checks::ready_time(&c, &json::parse(text).unwrap());
    assert_eq!(answer(r#"{"met":true,"ready":1234.5}"#), Ok(1234.5));
    assert_eq!(
        answer(r#"{"met":false}"#),
        Err("ready: the page did not set `selector [data-ready]` within 4000 ms".to_string())
    );
    assert_eq!(
        answer(r#"{"invalid":true}"#),
        Err("ready: `[data-ready]` is not a valid selector".to_string())
    );
    assert!(
        answer(r#"{"met":true}"#).is_err(),
        "a met condition has a time"
    );
    assert!(answer("null").is_err());
}

#[test]
fn a_page_that_does_not_set_its_condition_in_time_is_incomplete_naming_it() {
    let config = checks::config(&with(
        "ready: {selector: '[data-ready]'}\nready_timeout_ms: 4000\n",
    ))
    .unwrap();
    let missed =
        checks::ready_time(&config, &json::parse(r#"{"met":false}"#).unwrap()).unwrap_err();
    let recorded = measure::page_record(
        "/llm/slow",
        vec![("1440x900".to_string(), Err(Failure::Page(missed)))],
    )
    .unwrap();
    let page = checks::page_result(&config, &recorded).unwrap();
    assert_eq!(
        page.error.as_deref(),
        Some("ready: the page did not set `selector [data-ready]` within 4000 ms")
    );
    let doc = document(&config, &[page]);
    let e = doc.get("evidence").unwrap();
    let results = e.get("results").unwrap().list();
    assert_eq!(results.len(), 6, "five numbers and the findings");
    for r in results {
        assert_eq!(r.get("value"), None);
        let why = r.get("incomplete").expect("incomplete");
        assert_eq!(why.get_str("cause"), Some("incompatible_target"));
        assert!(
            why.get_str("detail")
                .unwrap()
                .contains("`selector [data-ready]`"),
            "the detail names the condition"
        );
    }
    let coverage = e.get("coverage").unwrap();
    assert_eq!(coverage.get("not_run"), Some(&Value::Int(1)));
}

fn document(config: &checks::Config, pages: &[checks::PageResult]) -> Value {
    let file = FileRef {
        path: Some("checks.yaml".to_string()),
        digest: "sha256:aa".to_string(),
    };
    let build = FileRef {
        path: None,
        digest: "sha256:bb".to_string(),
    };
    let header = Header {
        project: "p",
        assessment: "a",
        class: "local",
        config: &file,
        build: &build,
        pages: None,
        local_inputs: Vec::new(),
        browser: "Chrome/150.0.0.0",
        started_at: "2026-11-25T10:00:00Z".to_string(),
        ended_at: "2026-11-25T10:01:00Z".to_string(),
    };
    evidence::document(&header, config, pages)
}

fn ready_of(doc: &Value) -> Option<String> {
    doc.get("evidence")?
        .get("conditions")?
        .get("other")?
        .get_str("ready")
        .map(str::to_string)
}

#[test]
fn the_document_names_the_ready_condition() {
    let none = checks::config(LLM_LIKE).unwrap();
    assert_eq!(ready_of(&document(&none, &[])).as_deref(), Some("load"));
    let selector = checks::config(&with("ready: {selector: '[data-ready]'}\n")).unwrap();
    assert_eq!(
        ready_of(&document(&selector, &[])).as_deref(),
        Some("selector [data-ready]")
    );
    let global = checks::config(&with("ready: {global: appReady}\n")).unwrap();
    assert_eq!(
        ready_of(&document(&global, &[])).as_deref(),
        Some("global appReady")
    );
}

#[test]
fn a_recorded_page_reads_as_before_with_or_without_a_condition() {
    // The capture of 0.3.0 holds `shifts.ready` as the instant fonts and two frames were done; with a
    // condition it is the instant the condition, fonts and two frames were done. The shifts after it
    // count, the ones before it do not.
    let page = crate::fixture_json("checks/llm-protocols.capture.json");
    let plain =
        checks::config(&std::fs::read_to_string(crate::fixture("checks/llm.checks.yaml")).unwrap())
            .unwrap();
    let mut text = std::fs::read_to_string(crate::fixture("checks/llm.checks.yaml")).unwrap();
    text.push_str("ready: {selector: '[data-ready]'}\n");
    let signalled = checks::config(&text).unwrap();
    let a = checks::page_result(&plain, &page).unwrap();
    let b = checks::page_result(&signalled, &page).unwrap();
    assert_eq!(a.shift_after_ready(), Some(0.0004));
    assert_eq!(b.shift_after_ready(), a.shift_after_ready());
    let shifts = checks::Shifts {
        ready: 100.0,
        entries: Some(vec![(40.0, 0.5), (100.0, 0.25), (130.0, 0.0125)]),
    };
    assert_eq!(checks::shift_after_ready(&shifts), Some(0.0125));
    assert_eq!(checks::shift_load(&shifts), Some(0.7625));
}
