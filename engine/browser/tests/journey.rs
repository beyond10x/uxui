//! Journeys (`docs/browser-adapter.md`, Journeys): steps named by CSS selector that bring a page into
//! a state, and the checks of that state. `fixtures/checks/journey-site/` measured with
//! `journey.checks.yaml` (400 x 300, one mode): `journey/` holds the captures of its three pages and
//! of its five journeys (`ux-browser checks --capture`, a real headless run).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use ux_browser::checks::{self, PageResult};
use ux_browser::evidence::{self, Header, Target};
use ux_browser::journey::{self, JourneyResult, Step};
use ux_browser::render::FileRef;
use ux_browser::values;
use uxui_doc::yaml::Value;

fn config_text() -> String {
    std::fs::read_to_string(crate::fixture("checks/journey.checks.yaml")).unwrap()
}

fn config() -> checks::Config {
    checks::config(&config_text()).unwrap()
}

fn page(name: &str) -> PageResult {
    let capture = crate::fixture_json(&format!("checks/journey/{name}.capture.json"));
    checks::page_result(&config(), &capture).unwrap()
}

fn journey_of(name: &str) -> JourneyResult {
    let capture = crate::fixture_json(&format!("checks/journey/journey-{name}.capture.json"));
    journey::journey_result(&config(), &capture).unwrap()
}

/// The document of the recorded run: its three pages and its five journeys.
pub fn document() -> Value {
    let config = config();
    let pages: Vec<PageResult> = ["details", "menu", "search"].map(page).into();
    let journeys: Vec<JourneyResult> = [
        "open-details",
        "open-menu",
        "search",
        "missing-button",
        "slow-wait",
    ]
    .map(journey_of)
    .into();
    let config_file = FileRef {
        path: Some("engine/browser/tests/fixtures/checks/journey.checks.yaml".to_string()),
        digest: "sha256:aa".to_string(),
    };
    let build = FileRef {
        path: None,
        digest: "sha256:bb".to_string(),
    };
    let header = Header {
        project: "journeys",
        assessment: "page-checks",
        class: "local",
        config: &config_file,
        target: Target::Build(&build),
        pages: None,
        local_inputs: Vec::new(),
        browser: "Chrome/149.0.7827.55",
        started_at: "2026-11-25T10:00:00Z".to_string(),
        ended_at: "2026-11-25T10:01:00Z".to_string(),
    };
    evidence::document_with_journeys(&header, &config, &pages, &journeys)
}

fn results(doc: &Value) -> &[Value] {
    doc.get("evidence").unwrap().get("results").unwrap().list()
}

/// The result of `output` at `scope` for `item`.
fn result<'a>(doc: &'a Value, output: &str, scope: &str, item: &str) -> &'a Value {
    results(doc)
        .iter()
        .find(|r| {
            r.get_str("output") == Some(output)
                && r.get_str("scope") == Some(scope)
                && r.get_str("item") == Some(item)
        })
        .unwrap_or_else(|| panic!("no {output} at {scope} {item}"))
}

fn value<'a>(doc: &'a Value, output: &str, scope: &str, item: &str) -> Option<&'a str> {
    result(doc, output, scope, item).get_str("value")
}

/// The findings of a scope item: `(rule, node_path, reason)`.
fn findings(doc: &Value, scope: &str, item: &str) -> Vec<(String, String, String)> {
    result(doc, evidence::FINDINGS, scope, item)
        .get("items")
        .unwrap()
        .list()
        .iter()
        .map(|i| {
            let id = i.get("identity").unwrap();
            (
                id.get_str("rule").unwrap().to_string(),
                id.get_str("node_path").unwrap().to_string(),
                i.get("fields")
                    .unwrap()
                    .get_str("reason")
                    .unwrap()
                    .to_string(),
            )
        })
        .collect()
}

// ---------------------------------------------------------------- configuration

#[test]
fn a_journey_declares_its_page_its_steps_and_its_states() {
    let c = config();
    let names: Vec<&str> = c.journeys.iter().map(|j| j.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "open details",
            "open menu",
            "search",
            "missing button",
            "slow wait"
        ]
    );
    let search = &c.journeys[2];
    assert_eq!(search.page, "/search/");
    assert_eq!(search.timeout_ms, 5000, "the default wait");
    assert_eq!(
        search.steps,
        [
            Step::Type {
                css: "#q".to_string(),
                text: "averyveryverylongwordthatnoboxof200pixelscanhold".to_string()
            },
            Step::Press("Enter".to_string()),
            Step::Wait("#results p".to_string()),
            Step::Scroll(journey::Edge::Bottom),
            Step::Press("Escape".to_string()),
            Step::State("results".to_string()),
        ]
    );
    assert_eq!(
        c.journeys[0].steps,
        [
            Step::Click("details.more > summary".to_string()),
            Step::State("opened".to_string())
        ]
    );
    assert_eq!(c.journeys[4].timeout_ms, 300);
    let none = checks::config(&config_text()[..config_text().find("journeys:").unwrap()]).unwrap();
    assert!(none.journeys.is_empty(), "a configuration may declare none");
}

/// Runs `ux-browser` as on a CI runner without a browser (`tests/values.rs`), so that a refusal of
/// the configuration is seen to come before the browser lookup (exit 3).
fn run(args: &[&str]) -> Output {
    let no_browsers = Path::new(env!("CARGO_TARGET_TMPDIR")).join("journey-no-browsers");
    std::fs::create_dir_all(&no_browsers).unwrap();
    Command::new(env!("CARGO_BIN_EXE_ux-browser"))
        .env_remove("UXUI_CHROME")
        .env("PLAYWRIGHT_BROWSERS_PATH", &no_browsers)
        .args(args)
        .output()
        .unwrap()
}

/// A fresh directory of this file's own under cargo's temporary directory for integration tests.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("journey")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// `ux-browser checks` with the fixture configuration whose journeys are `journeys`.
fn checks_with(dir: &Path, journeys: &str) -> Output {
    let text = config_text();
    let text = format!("{}{journeys}", &text[..text.find("journeys:").unwrap()]);
    let file = dir.join("checks.yaml");
    std::fs::write(&file, text).unwrap();
    run(&[
        "checks",
        "--config",
        file.to_str().unwrap(),
        "--build",
        crate::fixture("checks/journey-site").to_str().unwrap(),
        "--project",
        "p",
        "--assessment",
        "a",
        "--out",
        dir.join("out.json").to_str().unwrap(),
    ])
}

#[test]
fn a_journey_that_does_not_read_is_refused_before_the_browser_naming_the_journey_and_the_step() {
    let dir = scratch("refused");
    let one = |steps: &str| {
        format!("journeys:\n  - name: open details\n    page: /details/\n    steps:\n{steps}")
    };
    let cases: Vec<(String, &[&str])> = vec![
        (
            one("      - {state: a}\n      - {hover: summary}\n"),
            &["journey `open details`", "step 2", "`hover`"],
        ),
        (
            one("      - {click: summary, state: a}\n"),
            &["journey `open details`", "step 1", "one of"],
        ),
        (
            one("      - {click: ''}\n      - {state: a}\n"),
            &["journey `open details`", "step 1", "`click`", "selector"],
        ),
        (
            one("      - {state: a}\n      - {type: '#q'}\n"),
            &["journey `open details`", "step 2", "`text`"],
        ),
        (
            one("      - {press: F5}\n      - {state: a}\n"),
            &["journey `open details`", "step 1", "`F5`"],
        ),
        (
            one("      - {scroll: middle}\n      - {state: a}\n"),
            &["journey `open details`", "step 1", "`scroll`"],
        ),
        (
            one("      - {wait: '#x'}\n      - {state: a}\n      - {state: a}\n"),
            &["journey `open details`", "step 3", "state `a`", "twice"],
        ),
        (
            one("      - {click: summary}\n"),
            &["journey `open details`", "no `state`"],
        ),
        (
            "journeys:\n  - name: open details\n    page: /details/\n    steps: []\n".to_string(),
            &["journey `open details`", "no steps"],
        ),
        (
            "journeys:\n  - name: open details\n    page: /details/\n    hover: true\n    steps:\n      - {state: a}\n".to_string(),
            &["journey `open details`", "`hover`"],
        ),
        (
            "journeys:\n  - name: open details\n    page: /details/\n    timeout_ms: 0\n    steps:\n      - {state: a}\n".to_string(),
            &["journey `open details`", "`timeout_ms`"],
        ),
        (
            "journeys:\n  - name: open details\n    steps:\n      - {state: a}\n".to_string(),
            &["journey `open details`", "`page`"],
        ),
        (
            "journeys:\n  - name: twice\n    page: /details/\n    steps:\n      - {state: a}\n  - name: twice\n    page: /menu/\n    steps:\n      - {state: a}\n".to_string(),
            &["journey `twice`", "twice"],
        ),
        (
            "journeys:\n  - page: /details/\n    steps:\n      - {state: a}\n".to_string(),
            &["journey", "`name`"],
        ),
        (
            "journeys:\n  - name: a/b\n    page: /details/\n    steps:\n      - {state: a}\n".to_string(),
            &["journey `a/b`", "`/`"],
        ),
        (
            one("      - {state: a/b}\n"),
            &["journey `open details`", "step 1", "`/`"],
        ),
        (
            "journeys:\n  - name: elsewhere\n    page: other\n    steps:\n      - {state: a}\n".to_string(),
            &["journey `elsewhere`", "`other`"],
        ),
    ];
    for (journeys, wanted) in &cases {
        let o = checks_with(&dir, journeys);
        let stderr = String::from_utf8_lossy(&o.stderr);
        assert_eq!(o.status.code(), Some(1), "{journeys}\n{stderr}");
        for w in *wanted {
            assert!(stderr.contains(w), "{journeys}: wants {w:?} in: {stderr}");
        }
        assert!(
            !dir.join("out.json").exists(),
            "{journeys}: nothing written"
        );
    }
}

// ---------------------------------------------------------------- the checks of a state

#[test]
fn a_long_line_in_a_closed_details_overflows_in_the_state_after_its_summary_is_clicked() {
    let doc = document();
    assert_eq!(
        value(&doc, checks::OVERFLOW, "page", "/details/"),
        Some("0")
    );
    assert_eq!(
        value(&doc, checks::OVERFLOW, "state", "open details/opened"),
        Some("1")
    );
    assert_eq!(
        findings(&doc, "state", "open details/opened"),
        [(
            checks::OVERFLOW.to_string(),
            "journey/open details/opened/400x300/light/p.full \"a line that is much wider than its card\""
                .to_string(),
            "right edge 263 past its parent at 200".to_string()
        )]
    );
    assert!(findings(&doc, "page", "/details/").is_empty());
}

#[test]
fn an_opened_menu_covers_an_always_visible_item_in_its_state_and_not_after_load() {
    let doc = document();
    assert_eq!(
        value(&doc, checks::ALWAYS_VISIBLE, "page", "/menu/"),
        Some("0"),
        "two items, top and bottom, after load"
    );
    assert_eq!(
        value(&doc, checks::ALWAYS_VISIBLE, "state", "open menu/closed"),
        Some("0")
    );
    assert_eq!(
        value(&doc, checks::ALWAYS_VISIBLE, "state", "open menu/opened"),
        Some("1")
    );
    assert_eq!(
        findings(&doc, "state", "open menu/opened"),
        [(
            checks::ALWAYS_VISIBLE.to_string(),
            "journey/open menu/opened/400x300/light/brand@state".to_string(),
            "covered".to_string()
        )]
    );
}

#[test]
fn typing_pressing_waiting_and_scrolling_reach_a_state() {
    let doc = document();
    assert_eq!(value(&doc, checks::OVERFLOW, "page", "/search/"), Some("0"));
    assert_eq!(
        value(&doc, checks::OVERFLOW, "state", "search/results"),
        Some("1"),
        "the typed word, sent with Enter, in a box of 200 px"
    );
    assert_eq!(
        value(&doc, checks::PAGE_SCROLL, "state", "search/results"),
        Some("0")
    );
}

#[test]
fn a_state_has_the_three_checks_that_need_no_fresh_load_and_names_its_journey_and_page() {
    let doc = document();
    let of_state: Vec<&str> = results(&doc)
        .iter()
        .filter(|r| r.get_str("item") == Some("open details/opened"))
        .map(|r| r.get_str("output").unwrap())
        .collect();
    assert_eq!(
        of_state,
        [
            checks::PAGE_SCROLL,
            checks::OVERFLOW,
            checks::ALWAYS_VISIBLE,
            evidence::FINDINGS
        ]
    );
    for r in results(&doc)
        .iter()
        .filter(|r| r.get_str("scope") == Some("state"))
    {
        let d = r.get("details").unwrap();
        let item = r.get_str("item").unwrap();
        let (j, s) = item.split_once('/').unwrap();
        assert_eq!(d.get_str("journey"), Some(j), "{item}");
        assert_eq!(d.get_str("state"), Some(s), "{item}");
        let page = d.get_str("page").unwrap();
        assert!(
            ["/details/", "/menu/", "/search/"].contains(&page),
            "{item}"
        );
    }
    let r = result(&doc, checks::OVERFLOW, "state", "open details/opened");
    assert_eq!(
        r.get("details").unwrap().get("viewports"),
        Some(&uxui_doc::yaml::parse("{400x300: '1'}").unwrap())
    );
}

#[test]
fn a_step_that_fails_ends_the_journey_and_its_later_states_are_not_run() {
    let doc = document();
    assert_eq!(
        value(
            &doc,
            checks::ALWAYS_VISIBLE,
            "state",
            "missing button/before"
        ),
        Some("0")
    );
    for (item, why) in [
        ("missing button/after", "step 2, click `#nothing`"),
        ("slow wait/never", "step 1, wait `#never`"),
    ] {
        for output in [
            checks::PAGE_SCROLL,
            checks::OVERFLOW,
            checks::ALWAYS_VISIBLE,
            evidence::FINDINGS,
        ] {
            let r = result(&doc, output, "state", item);
            assert_eq!(r.get("value"), None, "{item} {output}");
            let incomplete = r.get("incomplete").unwrap();
            assert_eq!(incomplete.get_str("cause"), Some("incompatible_target"));
            assert!(
                incomplete.get_str("detail").unwrap().contains(why),
                "{item} {output}: {incomplete:?}"
            );
        }
    }
    let coverage = doc.get("evidence").unwrap().get("coverage").unwrap();
    let skipped: Vec<(&str, &str, &str)> = coverage
        .get("not_executed")
        .unwrap()
        .list()
        .iter()
        .map(|n| {
            (
                n.get_str("id").unwrap(),
                n.get_str("state").unwrap(),
                n.get_str("reason").unwrap(),
            )
        })
        .collect();
    assert_eq!(skipped.len(), 2, "{skipped:?}");
    assert_eq!(
        (skipped[0].0, skipped[0].1),
        ("journey/missing button/after/400x300/light", "not_run")
    );
    assert!(
        skipped[0].2.contains("`#nothing`") && skipped[0].2.contains("no element"),
        "{skipped:?}"
    );
    assert_eq!(
        (skipped[1].0, skipped[1].1),
        ("journey/slow wait/never/400x300/light", "not_run")
    );
    assert!(skipped[1].2.contains("within 300 ms"), "{skipped:?}");
}

#[test]
fn coverage_counts_the_checks_of_each_state() {
    let doc = document();
    let c = doc.get("evidence").unwrap().get("coverage").unwrap();
    let n = |k: &str| match c.get(k) {
        Some(Value::Int(n)) => *n,
        other => panic!("{k}: {other:?}"),
    };
    // Pages: 3 units x (scroll, overflow) + 2 items x top and bottom on the menu page, and 3
    // layout-shift readings. States: 5 measured x (scroll, overflow), and 2 items on the menu
    // page's 3 measured states.
    assert_eq!(n("executed"), 3 * 2 + 4 + 3 + 5 * 2 + 3 * 2);
    assert_eq!(
        n("failed"),
        3,
        "the details' and the search's overflow, the covered brand"
    );
    assert_eq!(n("not_run"), 2);
}

#[test]
fn a_document_without_journeys_is_the_one_written_before() {
    let config = checks::config(crate::checks::LLM_LIKE).unwrap();
    assert!(config.journeys.is_empty());
    let doc = crate::evidence::document();
    assert!(
        results(&doc)
            .iter()
            .all(|r| r.get_str("scope") == Some("page")),
        "no result of a state"
    );
    let before = evidence::text(&doc);
    assert!(!before.contains("journey"));
    assert!(!before.contains("\"scope\":\"state\""));
}

// ---------------------------------------------------------------- values

fn values_section(gates: &str) -> values::ValuesConfig {
    let v = uxui_doc::yaml::parse(&format!(
        "scenarios: {{files: 'scenarios/*/page.json'}}\nwaivers: {{files: 'findings/*.md'}}\ngates: {gates}\n"
    ))
    .unwrap();
    values::config(&v, &["400x300".to_string()]).unwrap()
}

fn inputs() -> values::Inputs {
    values::Inputs {
        scenarios: vec![
            ("/details/".to_string(), "page-details".to_string()),
            ("/menu/".to_string(), "page-menu".to_string()),
            ("/search/".to_string(), "page-search".to_string()),
        ],
        waivers: Vec::new(),
    }
}

#[test]
fn the_values_file_has_the_values_and_failures_of_each_state_under_its_page_scenario() {
    let section = values_section("{layout.overflow@1: 2, always_visible.misses@1: 3}");
    let v = values::values(&section, &inputs(), &document()).unwrap();
    let file = ux_browser::json::parse(&v.text).unwrap();
    let details = file.get("values").unwrap().get("page-details").unwrap();
    assert_eq!(details.get("layout.overflow"), Some(&Value::Int(0)));
    assert_eq!(
        details.get("layout.overflow#open details/opened"),
        Some(&Value::Int(1))
    );
    let menu = file.get("values").unwrap().get("page-menu").unwrap();
    assert_eq!(menu.get("always_visible.misses"), Some(&Value::Int(0)));
    assert_eq!(
        menu.get("always_visible.misses#open menu/opened"),
        Some(&Value::Int(1))
    );
    assert_eq!(
        menu.get("always_visible.misses#missing button/after"),
        None,
        "a state not run has no value"
    );
    let keys: Vec<&str> = file
        .get("units")
        .unwrap()
        .list()
        .iter()
        .flat_map(|u| u.get("failures").unwrap().list())
        .map(|f| f.get_str("key").unwrap())
        .collect();
    assert_eq!(
        keys,
        [
            "gate2|/details/#open details/opened|400x300|light|overflow:p.full \"a line that is much wider than its card\"",
            "gate3|/menu/#open menu/opened|400x300|light|brand:covered@state",
            "gate2|/search/#search/results|400x300|light|overflow:p \"averyveryverylongwordthatnoboxof200pixel\"",
        ]
    );
    assert!(
        v.problems
            .iter()
            .any(|p| p.starts_with("incomplete: /menu/#missing button/after layout.overflow@1")),
        "{:?}",
        v.problems
    );
    assert!(
        !v.problems.iter().any(|p| p.contains("no page scenario")),
        "a state's page has its scenario: {:?}",
        v.problems
    );
}
