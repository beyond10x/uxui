//! Where a checks run spends its time (`docs/browser-adapter.md`, Timings): the milliseconds of each
//! phase per page and viewport in the run section, outside the evidence, and the page loads a walk
//! needs. `fixtures/checks/timing/` holds a page of this repository's documentation build recorded
//! with `uxui-docs.checks.yaml` (`ux-browser checks --capture`, a real headless run).

use ux_browser::checks::{self, Config};
use ux_browser::evidence::{self, Header, Target};
use ux_browser::measure::{self, Start, WalkPlan};
use ux_browser::render::FileRef;
use uxui_doc::yaml::Value;

fn config_of(name: &str) -> Config {
    checks::config(&std::fs::read_to_string(crate::fixture(name)).unwrap()).unwrap()
}

fn docs_config() -> Config {
    config_of("checks/uxui-docs.checks.yaml")
}

fn llm_config() -> Config {
    config_of("../../examples/llm.checks.yaml")
}

fn recorded() -> Value {
    crate::fixture_json("checks/timing/uxui-docs-home.capture.json")
}

/// The document of the recorded page and the same document with its times.
fn documents() -> (Value, Value) {
    let config = docs_config();
    let recorded = recorded();
    let page = checks::page_result(&config, &recorded).unwrap();
    let config_file = FileRef {
        path: Some("engine/browser/tests/fixtures/checks/uxui-docs.checks.yaml".to_string()),
        digest: "sha256:aa".to_string(),
    };
    let build = FileRef {
        path: None,
        digest: "sha256:bb".to_string(),
    };
    let header = Header {
        project: "uxui-docs",
        assessment: "page-checks",
        class: "local",
        config: &config_file,
        target: Target::Build(&build),
        pages: None,
        local_inputs: vec![("jobs", "4".to_string())],
        browser: "Chrome/149.0.7827.55",
        started_at: "2026-11-25T10:00:00Z".to_string(),
        ended_at: "2026-11-25T10:01:00Z".to_string(),
    };
    let document = evidence::document(&header, &config, &[page]);
    let timed = evidence::with_timing(document.clone(), &[recorded]);
    (document, timed)
}

fn ms(v: &Value, key: &str) -> i64 {
    match v.get(key) {
        Some(Value::Int(n)) if *n >= 0 => *n,
        other => panic!("`{key}` is not a whole number of milliseconds: {other:?}"),
    }
}

#[test]
fn the_run_section_holds_the_milliseconds_of_each_phase_per_page_and_viewport() {
    let (_, timed) = documents();
    let timing = timed.get("run").unwrap().get("timing").expect("run.timing");
    assert_eq!(timing.get_str("unit"), Some("ms"));
    let pages = timing.get("pages").unwrap().list();
    let seen: Vec<(&str, &str)> = pages
        .iter()
        .map(|p| (p.get_str("path").unwrap(), p.get_str("viewport").unwrap()))
        .collect();
    assert_eq!(seen, [("/uxui/", "1440x900"), ("/uxui/", "390x844")]);
    let config = docs_config();
    for p in pages {
        assert_eq!(
            ms(p, "loads"),
            measure::planned_loads(&config, p.get_str("viewport").unwrap()) as i64,
            "the loads the plan names"
        );
        for key in ["open", "load", "close", "total"] {
            ms(p, key);
        }
        assert!(ms(p, "settle") >= 250, "the declared settle time");
        let modes = p.get("modes").unwrap().list();
        let names: Vec<&str> = modes.iter().map(|m| m.get_str("mode").unwrap()).collect();
        assert_eq!(names, ["light", "dark"]);
        for m in modes {
            for key in ["switch", "overflow", "top", "bottom"] {
                ms(m, key);
            }
            assert!(ms(m, "scroll_settle") >= 200, "the declared scroll settle");
        }
        assert!(ms(&modes[1], "switch") >= 100, "the declared mode settle");
        assert_eq!(modes[0].get("keyboard"), None, "no walk in light");
        let walk = modes[1].get("keyboard").expect("the walk in dark");
        ms(walk, "ms");
        ms(walk, "reset");
        let loads = walk.get("loads").unwrap().list();
        assert_eq!(loads.len(), 1, "one load: the Tab walk after the skip link");
        assert!(loads.iter().all(|l| matches!(l, Value::Int(n) if *n >= 0)));
        assert!(ms(walk, "tabs") >= 2, "the skip link's Tab and the walk's");
        assert_eq!(ms(p, "loads"), 1 + loads.len() as i64);
    }
}

#[test]
fn times_never_enter_the_evidence() {
    let (document, timed) = documents();
    assert_eq!(timed.get("evidence"), document.get("evidence"));
    assert_eq!(
        evidence::text(&document)
            .lines()
            .take_while(|l| !l.contains("\"run\":"))
            .collect::<Vec<_>>(),
        evidence::text(&timed)
            .lines()
            .take_while(|l| !l.contains("\"run\":"))
            .collect::<Vec<_>>(),
        "the evidence's bytes"
    );
    let run = timed.get("run").unwrap();
    assert_eq!(run.get_str("started_at"), Some("2026-11-25T10:00:00Z"));
    assert_eq!(run.get("local_inputs").unwrap().get_str("jobs"), Some("4"));
    let text = evidence::text(&timed);
    assert_eq!(ux_browser::json::parse(&text).unwrap(), timed);
    assert!(
        text.lines()
            .any(|l| l.starts_with("{\"path\":\"/uxui/\",\"viewport\":\"390x844\"")),
        "one line per page and viewport"
    );
}

#[test]
fn a_recorded_page_without_times_leaves_the_run_section_as_it_was() {
    let capture = crate::fixture_json("checks/llm-search.capture.json");
    let document = crate::evidence::document();
    assert_eq!(evidence::with_timing(document.clone(), &[capture]), document);
}

#[test]
fn the_first_walk_of_a_page_starts_on_the_measured_page_and_a_skip_link_reloads_once() {
    let llm = llm_config();
    let k = llm.keyboard.as_ref().unwrap();
    assert_eq!(
        measure::walk_plan(k, true),
        WalkPlan {
            skip_link: Some(Start::Reset),
            tabs: Start::Load
        }
    );
    assert_eq!(
        measure::walk_plan(k, false),
        WalkPlan {
            skip_link: Some(Start::Load),
            tabs: Start::Load
        },
        "a walk after another starts on a load"
    );
    let mut no_skip = docs_config();
    let k = no_skip.keyboard.as_mut().unwrap();
    k.skip_link = None;
    assert_eq!(
        measure::walk_plan(k, true),
        WalkPlan {
            skip_link: None,
            tabs: Start::Reset
        }
    );
}

/// llm's configuration (a skip link, the walk in dark): up to 0.9.1 and on main before this change
/// each page and viewport loaded 3 times (the measured load, then a fresh load for the skip link
/// and one for the Tab walk); now 2.
#[test]
fn an_llm_like_run_loads_each_page_and_viewport_twice_where_it_loaded_three_times() {
    const BEFORE: usize = 3;
    for config in [llm_config(), docs_config()] {
        for vp in &config.viewports {
            let loads = measure::planned_loads(&config, &vp.name);
            assert_eq!(loads, 2, "{}", vp.name);
            assert!(loads < BEFORE);
        }
    }
}

#[test]
fn the_loads_count_each_walk_and_a_viewport_without_one() {
    let base = std::fs::read_to_string(crate::fixture("checks/uxui-docs.checks.yaml")).unwrap();
    let both = checks::config(&base.replace("modes: [dark]", "modes: [light, dark]")).unwrap();
    assert_eq!(
        measure::planned_loads(&both, "1440x900"),
        4,
        "the measured load, the first walk's Tab walk, the second walk's two"
    );
    let narrow = checks::config(&base.replace(
        "  modes: [dark]",
        "  modes: [dark]\n  viewports: [390x844]",
    ))
    .unwrap();
    assert_eq!(measure::planned_loads(&narrow, "1440x900"), 1);
    assert_eq!(measure::planned_loads(&narrow, "390x844"), 2);
    let none = checks::config(&base[..base.find("keyboard:").unwrap()]).unwrap();
    assert_eq!(measure::planned_loads(&none, "390x844"), 1);
    let no_skip = checks::config(&base.replace(
        "  skip_link_contains: [skip to content, skip to main content]\n",
        "",
    ))
    .unwrap();
    assert_eq!(
        measure::planned_loads(&no_skip, "390x844"),
        1,
        "the walk on the measured page"
    );
}
