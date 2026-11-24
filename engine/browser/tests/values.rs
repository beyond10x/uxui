//! The values file of a checks document (`ux-browser values`, `checks --values`): what a consumer's
//! ratchet compares and what its hard gates fail on. The reference is the file the first consumer's
//! conversion script (`ux/page-checks.sh` of its 0.2.0 patch, POSIX sh and jq) wrote from its
//! fixture document: `fixtures/values/page-checks.json` and `fixtures/values/variants/`, written by
//! that script once and kept as bytes.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use ux_browser::checks::{self, KeyboardResult, KeyboardUnit, PageResult, ShiftUnit, UnitResult};
use ux_browser::evidence::{self, Header};
use ux_browser::render::FileRef;
use ux_browser::values;
use uxui_doc::yaml::Value;

fn tree() -> PathBuf {
    crate::fixture("values")
}

/// A fresh directory for one test under cargo's temporary directory for integration tests.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("values")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).unwrap();
        }
    }
}

/// A copy of the fixture tree in `scratch(name)/root`.
fn tree_copy(name: &str) -> PathBuf {
    let root = scratch(name).join("root");
    copy_dir(&tree(), &root);
    root
}

/// The fixture document with only the results of `item`, as the variants were made from it.
fn evidence_of(root: &Path, item: &str) -> PathBuf {
    let mut doc = crate::fixture_json("values/page-checks.evidence.json");
    let Value::Map(top) = &mut doc else { panic!() };
    let (_, e) = top.iter_mut().find(|(k, _)| k == "evidence").unwrap();
    let Value::Map(fields) = e else { panic!() };
    let (_, results) = fields.iter_mut().find(|(k, _)| k == "results").unwrap();
    let Value::List(list) = results else { panic!() };
    list.retain(|r| r.get_str("item") == Some(item));
    let file = root.join(format!("{}.evidence.json", item.replace('/', "_")));
    std::fs::write(&file, uxui_doc::json::write(&doc)).unwrap();
    file
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ux-browser"))
        .args(args)
        .output()
        .unwrap()
}

/// `ux-browser values` on `evidence` with the fixture configuration under `root`; returns the exit
/// status, the file written (if any) and standard error.
fn values_run(root: &Path, evidence: &Path, out: &Path) -> (Option<i32>, Option<String>, String) {
    let config = root.join("checks.yaml");
    let o = run(&[
        "values",
        "--config",
        config.to_str().unwrap(),
        "--evidence",
        evidence.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
        "--root",
        root.to_str().unwrap(),
    ]);
    (
        o.status.code(),
        std::fs::read_to_string(out).ok(),
        String::from_utf8_lossy(&o.stderr).to_string(),
    )
}

fn golden(name: &str) -> String {
    std::fs::read_to_string(tree().join(name)).unwrap()
}

#[test]
fn the_values_of_the_fixture_document_are_the_bytes_the_conversion_script_wrote() {
    let out = scratch("all").join("page-checks.json");
    let (code, written, stderr) =
        values_run(&tree(), &tree().join("page-checks.evidence.json"), &out);
    assert_eq!(code, Some(1), "{stderr}");
    assert_eq!(
        written.as_deref(),
        Some(golden("page-checks.json").as_str())
    );
    assert!(
        stderr.contains("incomplete: /llm/docs/status layout.page_scroll@1: incompatible_target")
    );
    assert!(stderr.contains(
        "hard gate failure not waived by an open finding: gate1|/llm/docs/status/roadmap|390x844|light|page-scroll"
    ));
    assert!(
        stderr
            .contains("waived: gate5|/llm/search|390x844|dark|keyboard:skip-link-first by UXF-005")
    );
}

#[test]
fn a_run_whose_only_failures_are_waived_exits_0() {
    let root = tree_copy("search");
    let out = root.join("out.json");
    let (code, written, stderr) = values_run(&root, &evidence_of(&root, "/llm/search"), &out);
    assert_eq!(code, Some(0), "{stderr}");
    assert_eq!(
        written.as_deref(),
        Some(golden("variants/search.page-checks.json").as_str())
    );
}

#[test]
fn an_unwaived_failure_exits_1_and_a_closed_finding_waives_nothing() {
    let root = tree_copy("closed");
    let record = root.join("findings/UXF-005.md");
    let text = std::fs::read_to_string(&record).unwrap();
    std::fs::write(&record, text.replace("status: open\n", "status: closed\n")).unwrap();
    let out = root.join("out.json");
    let (code, written, stderr) = values_run(&root, &evidence_of(&root, "/llm/search"), &out);
    assert_eq!(code, Some(1), "{stderr}");
    let written = written.expect("the values are written before the exit");
    assert_eq!(written, golden("variants/closed.page-checks.json"));
    assert!(!written.contains("UXF-005"));
    assert!(stderr.contains(
        "hard gate failure not waived by an open finding: gate5|/llm/search|1440x900|dark|keyboard:skip-link-first"
    ));
}

#[test]
fn a_page_without_a_scenario_exits_1() {
    let root = tree_copy("unmapped");
    std::fs::remove_dir_all(root.join("scenarios/page-search")).unwrap();
    let out = root.join("out.json");
    let (code, written, stderr) = values_run(&root, &evidence_of(&root, "/llm/search"), &out);
    assert_eq!(code, Some(1), "{stderr}");
    assert_eq!(
        written.as_deref(),
        Some(golden("variants/unmapped.page-checks.json").as_str())
    );
    assert!(stderr.contains("no page scenario for /llm/search"));
}

#[test]
fn an_incomplete_result_exits_1() {
    let root = tree_copy("incomplete");
    let out = root.join("out.json");
    let (code, written, stderr) = values_run(&root, &evidence_of(&root, "/llm/docs/status"), &out);
    assert_eq!(code, Some(1), "{stderr}");
    assert_eq!(
        written.as_deref(),
        Some(golden("variants/incomplete.page-checks.json").as_str())
    );
    assert!(stderr.contains("incomplete: /llm/docs/status findings: incompatible_target"));
}

#[test]
fn refused_inputs_exit_1_and_leave_no_file() {
    let root = tree_copy("refused");
    let out = root.join("out.json");
    std::fs::write(&out, "stale").unwrap();
    let bad = root.join("bad.evidence.json");
    std::fs::write(&bad, "{not json").unwrap();
    let (code, written, _) = values_run(&root, &bad, &out);
    assert_eq!(code, Some(1));
    assert_eq!(
        written, None,
        "a stale file is removed before anything is read"
    );
    // A configuration without a `values` section.
    let config = crate::fixture("checks/llm.checks.yaml");
    let o = run(&[
        "values",
        "--config",
        config.to_str().unwrap(),
        "--evidence",
        tree().join("page-checks.evidence.json").to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&o.stderr).contains("`values`"));
}

#[test]
fn a_wrong_values_invocation_exits_2() {
    assert_eq!(
        run(&["values", "--config", "x.yaml"]).status.code(),
        Some(2)
    );
    assert_eq!(
        run(&[
            "values",
            "--config",
            "x",
            "--evidence",
            "y",
            "--out",
            "z",
            "--jobs",
            "2"
        ])
        .status
        .code(),
        Some(2)
    );
}

#[test]
fn checks_with_values_refuses_a_configuration_without_a_values_section_before_the_browser() {
    let out = scratch("checks-values").join("v.json");
    let o = run(&[
        "checks",
        "--config",
        crate::fixture("checks/llm.checks.yaml").to_str().unwrap(),
        "--build",
        crate::fixture("site").to_str().unwrap(),
        "--project",
        "p",
        "--assessment",
        "a",
        "--values",
        out.to_str().unwrap(),
    ]);
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&o.stderr).contains("`values`"));
}

#[test]
fn checks_with_values_refuses_scenario_files_that_do_not_read_before_the_browser() {
    let root = tree_copy("checks-bad-scenario");
    std::fs::write(
        root.join("scenarios/page-search/page.json"),
        "{\"kind\": \"page\"}",
    )
    .unwrap();
    let o = run(&[
        "checks",
        "--config",
        root.join("checks.yaml").to_str().unwrap(),
        "--build",
        crate::fixture("site").to_str().unwrap(),
        "--project",
        "p",
        "--assessment",
        "a",
        "--root",
        root.to_str().unwrap(),
        "--values",
        root.join("v.json").to_str().unwrap(),
    ]);
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&o.stderr).contains("page-search"));
}

#[test]
fn a_waiver_pattern_matches_a_whole_key_with_stars_for_any_run() {
    let key = "gate5|/llm/search|390x844|dark|keyboard:skip-link-first";
    assert!(values::waiver_matches(
        "gate5|/llm/search|*|*|keyboard:skip-link-first",
        key
    ));
    assert!(values::waiver_matches("*", key));
    assert!(values::waiver_matches("gate5|*", key));
    assert!(!values::waiver_matches(
        "gate5|/llm/search|*",
        "gate1|/llm/search|x"
    ));
    assert!(
        !values::waiver_matches("gate5|/llm/search", key),
        "the whole key"
    );
    assert!(
        !values::waiver_matches(
            "gate5|/llm/search|390x844|dark|keyboard:skip.link-first",
            key
        ),
        "a dot is a dot"
    );
}

#[test]
fn a_path_pattern_matches_inside_one_segment_in_byte_order() {
    let found = values::glob(&tree(), "scenarios/page-*/page.json").unwrap();
    let rel: Vec<String> = found
        .iter()
        .map(|p| {
            p.strip_prefix(tree())
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .collect();
    assert_eq!(
        rel,
        [
            "scenarios/page-docs-status-roadmap/page.json",
            "scenarios/page-docs-status/page.json",
            "scenarios/page-search/page.json",
        ]
    );
    assert!(
        values::glob(&tree(), "scenarios/*.json")
            .unwrap()
            .is_empty()
    );
    assert_eq!(values::glob(&tree(), "findings/UXF-*.md").unwrap().len(), 5);
}

const VALUES_SECTION: &str = "values:
  scenarios: {files: scenarios/page-*/page.json}
  waivers: {files: findings/UXF-*.md}
  gates:
    layout.page_scroll@1: 1
    layout.overflow@1: 1
    always_visible.misses@1: 2
    keyboard.misses@1: 5
";

#[test]
fn reads_the_values_section_with_its_defaults() {
    let c = checks::config(&format!("{}{VALUES_SECTION}", crate::checks::LLM_LIKE)).unwrap();
    let v = c.values.expect("a values section");
    assert_eq!(v.scenarios.files, "scenarios/page-*/page.json");
    assert_eq!(v.scenarios.path_key, "path");
    assert_eq!(v.waivers.closed, ["closed"]);
    assert_eq!(
        v.gates,
        [
            ("layout.page_scroll@1".to_string(), 1),
            ("layout.overflow@1".to_string(), 1),
            ("always_visible.misses@1".to_string(), 2),
            ("keyboard.misses@1".to_string(), 5),
        ]
    );
    assert_eq!(v.gating, None);
    assert!(
        checks::config(crate::checks::LLM_LIKE)
            .unwrap()
            .values
            .is_none()
    );
}

#[test]
fn refuses_a_values_section_that_does_not_hold_together() {
    let base = format!("{}{VALUES_SECTION}", crate::checks::LLM_LIKE);
    for (from, to) in [
        (
            "    keyboard.misses@1: 5\n",
            "    layout_shift.after_ready@1: 5\n",
        ),
        (
            "    keyboard.misses@1: 5\n",
            "    keyboard.misses@1: five\n",
        ),
        (
            "    keyboard.misses@1: 5\n",
            "    keyboard.misses@1: 5\n  gating: [800x600]\n",
        ),
        ("  waivers: {files: findings/UXF-*.md}\n", ""),
        ("{files: scenarios/page-*/page.json}", "{path_key: path}"),
        (
            "    keyboard.misses@1: 5\n",
            "    keyboard.misses@1: 5\n  axe: true\n",
        ),
    ] {
        let text = base.replace(from, to);
        assert_ne!(text, base, "{from}");
        assert!(checks::config(&text).is_err(), "{to}");
    }
}

#[test]
fn layout_shift_load_is_an_opt_in_check() {
    let on = checks::config(&format!(
        "{}opt_in: [layout_shift.load]\n",
        crate::checks::LLM_LIKE
    ))
    .unwrap();
    assert!(on.layout_shift_load);
    assert!(
        !checks::config(crate::checks::LLM_LIKE)
            .unwrap()
            .layout_shift_load
    );
    assert!(
        checks::config(&format!("{}opt_in: [axe]\n", crate::checks::LLM_LIKE)).is_err(),
        "only a check ux-browser has"
    );
}

#[test]
fn the_keyboard_walk_may_be_limited_to_some_viewports() {
    let limited = crate::checks::LLM_LIKE.replace(
        "  max_tabs: 40\n",
        "  max_tabs: 40\n  viewports: [1440x900]\n",
    );
    let k = checks::config(&limited).unwrap().keyboard.unwrap();
    assert_eq!(k.viewports.as_deref(), Some(&["1440x900".to_string()][..]));
    let wrong = crate::checks::LLM_LIKE.replace(
        "  max_tabs: 40\n",
        "  max_tabs: 40\n  viewports: [800x600]\n",
    );
    assert!(checks::config(&wrong).is_err());
    let all = checks::config(crate::checks::LLM_LIKE).unwrap();
    assert_eq!(all.keyboard.unwrap().viewports, None);
}

/// The load-time shift of the recorded protocols page: every shift since the navigation, read with
/// the shift after ready.
#[test]
fn a_recorded_page_has_its_load_time_layout_shift() {
    let config = checks::config(&format!(
        "{}opt_in: [layout_shift.load]\n",
        std::fs::read_to_string(crate::fixture("checks/llm.checks.yaml")).unwrap()
    ))
    .unwrap();
    let page = crate::fixture_json("checks/llm-protocols.capture.json");
    let result = checks::page_result(&config, &page).unwrap();
    let load = result.shift_load().expect("a value at every viewport");
    let ready = result.shift_after_ready().unwrap();
    assert!(
        load >= ready,
        "load {load} holds the shifts after ready {ready}"
    );
}

// ---------------------------------------------------------------- documents with details

fn unit(viewport: &str, mode: &str, scroll: f64) -> UnitResult {
    UnitResult {
        viewport: viewport.to_string(),
        mode: mode.to_string(),
        mode_applied: true,
        page_scroll_px: scroll,
        overflow: Vec::new(),
        visible: Vec::new(),
    }
}

fn walk(viewport: &str, skip_first: bool) -> KeyboardUnit {
    KeyboardUnit {
        viewport: viewport.to_string(),
        mode: "dark".to_string(),
        result: KeyboardResult {
            skip_link_first: skip_first,
            skip_moves_focus: skip_first,
            reached: Vec::new(),
        },
    }
}

fn shift(viewport: &str, after_ready: f64, load: f64) -> ShiftUnit {
    ShiftUnit {
        viewport: viewport.to_string(),
        after_ready: Some(after_ready),
        load: Some(load),
    }
}

const ROUND_LIKE: &str = "format: ux-browser-checks/1
base_path: /llm/
pages: {sitemap: sitemap.xml}
viewports:
  - {name: 1440x900, width: 1440, height: 900}
  - {name: 1024x768, width: 1024, height: 768}
  - {name: 390x844, width: 390, height: 844}
modes: [light, dark]
opt_in: [layout_shift.load]
keyboard:
  skip_link_contains: [skip to content]
  max_tabs: 40
  viewports: [1440x900, 390x844]
values:
  scenarios: {files: scenarios/page-*/page.json}
  waivers: {files: findings/UXF-*.md}
  gates:
    layout.page_scroll@1: 1
    keyboard.misses@1: 5
  gating: [1440x900, 390x844]
";

/// `/llm/search` scrolls sideways at the measured 1024 x 768 in both modes and at 390 x 844 in one;
/// its skip link is not first at 390 x 844 (waived by UXF-005 in the fixture tree).
pub fn round_document() -> Value {
    let config = checks::config(ROUND_LIKE).unwrap();
    let page = PageResult {
        path: "/llm/search".to_string(),
        error: None,
        units: vec![
            unit("1440x900", "light", 0.0),
            unit("1440x900", "dark", 0.0),
            unit("1024x768", "light", 12.0),
            unit("1024x768", "dark", 12.0),
            unit("390x844", "light", 3.0),
            unit("390x844", "dark", 0.0),
        ],
        keyboard: vec![walk("1440x900", true), walk("390x844", false)],
        shifts: vec![
            shift("1440x900", 0.0004, 0.0123),
            shift("1024x768", 0.05, 0.2),
            shift("390x844", 0.0, 0.031),
        ],
    };
    let file = FileRef {
        path: None,
        digest: "sha256:aa".to_string(),
    };
    let header = Header {
        project: "llm-docs",
        assessment: "round",
        class: "local",
        config: &file,
        build: &file,
        pages: None,
        local_inputs: Vec::new(),
        browser: "Chrome/150.0.0.0",
        started_at: "2026-11-24T10:00:00Z".to_string(),
        ended_at: "2026-11-24T10:01:00Z".to_string(),
    };
    evidence::document(&header, &config, &[page])
}

fn result<'a>(doc: &'a Value, output: &str) -> &'a Value {
    doc.get("evidence")
        .unwrap()
        .get("results")
        .unwrap()
        .list()
        .iter()
        .find(|r| r.get_str("output") == Some(output))
        .unwrap_or_else(|| panic!("no {output}"))
}

fn details(doc: &Value, output: &str) -> Vec<(String, String)> {
    result(doc, output)
        .get("details")
        .and_then(|d| d.get("viewports"))
        .map(Value::entries)
        .unwrap_or_default()
        .iter()
        .map(|(k, v)| (k.clone(), v.str().unwrap().to_string()))
        .collect()
}

fn pairs(items: &[(&str, &str)]) -> Vec<(String, String)> {
    items
        .iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
}

#[test]
fn every_number_has_its_per_viewport_values_and_the_load_shift_is_written_when_opted_in() {
    let doc = round_document();
    assert_eq!(
        details(&doc, checks::PAGE_SCROLL),
        pairs(&[("1440x900", "0"), ("1024x768", "2"), ("390x844", "1")])
    );
    assert_eq!(
        details(&doc, checks::KEYBOARD),
        pairs(&[("1440x900", "0"), ("390x844", "2")]),
        "no walk at 1024 x 768"
    );
    assert_eq!(
        result(&doc, checks::PAGE_SCROLL).get_str("value"),
        Some("3")
    );
    let load = result(&doc, checks::LAYOUT_SHIFT_LOAD);
    assert_eq!(
        load.get_str("value"),
        Some("0.2"),
        "the largest over the viewports"
    );
    assert_eq!(load.get_str("kind"), Some("sampled"));
    assert_eq!(load.get_str("unit"), Some("score"));
    assert_eq!(
        details(&doc, checks::LAYOUT_SHIFT_LOAD),
        pairs(&[
            ("1440x900", "0.0123"),
            ("1024x768", "0.2"),
            ("390x844", "0.031")
        ])
    );
    let without = crate::evidence::document();
    let outputs: Vec<&str> = without
        .get("evidence")
        .unwrap()
        .get("results")
        .unwrap()
        .list()
        .iter()
        .filter_map(|r| r.get_str("output"))
        .collect();
    assert!(!outputs.contains(&checks::LAYOUT_SHIFT_LOAD), "opt-in only");
}

#[test]
fn gating_viewports_give_the_values_and_the_failures_and_every_viewport_is_reported() {
    let root = tree_copy("gating");
    let doc = round_document();
    let config = checks::config(ROUND_LIKE).unwrap();
    let vc = config.values.as_ref().unwrap();
    let inputs = values::inputs(vc, &root).unwrap();
    let v = values::values(vc, &inputs, &doc).unwrap();
    let file = ux_browser::json::parse(&v.text).unwrap();
    let search = file.get("values").unwrap().get("page-search").unwrap();
    let n = |k: &str| search.get(k).cloned();
    assert_eq!(
        n("layout.page_scroll"),
        Some(Value::Int(1)),
        "390 x 844 light only"
    );
    assert_eq!(n("keyboard.misses"), Some(Value::Int(2)));
    assert_eq!(n("layout_shift.after_ready"), Some(Value::Float(0.0004)));
    assert_eq!(n("layout_shift.load"), Some(Value::Float(0.031)));
    let keys: Vec<String> = file
        .get("units")
        .unwrap()
        .list()
        .iter()
        .flat_map(|u| u.get("failures").unwrap().list().to_vec())
        .map(|f| f.get_str("key").unwrap().to_string())
        .collect();
    assert_eq!(
        keys,
        [
            "gate5|/llm/search|390x844|dark|keyboard:skip-link-first",
            "gate1|/llm/search|390x844|light|page-scroll",
        ],
        "no failure at the measured 1024 x 768"
    );
    let measured = file.get("viewports").unwrap().get("1024x768").unwrap();
    assert_eq!(measured.get("gating"), Some(&Value::Bool(false)));
    assert_eq!(measured.get("pages"), Some(&Value::Int(1)));
    assert_eq!(measured.get("layout.page_scroll"), Some(&Value::Int(2)));
    assert_eq!(measured.get("layout_shift.load"), Some(&Value::Float(0.2)));
    assert_eq!(measured.get("keyboard.misses"), None, "not walked there");
    let gating = file.get("viewports").unwrap().get("390x844").unwrap();
    assert_eq!(gating.get("gating"), Some(&Value::Bool(true)));
    assert_eq!(
        v.problems,
        [
            "hard gate failure not waived by an open finding: gate1|/llm/search|390x844|light|page-scroll"
        ]
    );
    assert_eq!(
        v.waived,
        ["gate5|/llm/search|390x844|dark|keyboard:skip-link-first by UXF-005"]
    );
}

fn example(name: &str) -> checks::Config {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name);
    checks::config(&std::fs::read_to_string(&path).unwrap())
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The first consumer's two configurations: every merge, and its round.
#[test]
fn the_example_configurations_declare_the_every_merge_check_and_the_round() {
    let merge = example("llm.checks.yaml");
    let fixture =
        checks::config(&std::fs::read_to_string(tree().join("checks.yaml")).unwrap()).unwrap();
    assert_eq!(
        merge.values.as_ref().unwrap().gates,
        fixture.values.as_ref().unwrap().gates
    );
    assert!(!merge.layout_shift_load);
    let round = example("llm-round.checks.yaml");
    let names: Vec<&str> = round.viewports.iter().map(|v| v.name.as_str()).collect();
    assert_eq!(
        names,
        ["1440x900", "1280x800", "1024x768", "768x1024", "390x844"]
    );
    assert_eq!(round.settle_ms, 750);
    assert!(round.layout_shift_load);
    let gating = ["1440x900".to_string(), "390x844".to_string()];
    assert_eq!(round.values.unwrap().gating.as_deref(), Some(&gating[..]));
    assert_eq!(
        round.keyboard.unwrap().viewports.as_deref(),
        Some(&gating[..])
    );
}
