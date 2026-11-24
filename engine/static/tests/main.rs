//! The one integration-test binary of ux-static: the studio, planted broken copies of it, and
//! ratchet's examples of `ratchet-evidence/1`.
//!
//! `fixtures/ratchet-example/` holds a fixed copy of the studio's three inputs (the ones ratchet's
//! examples were written from) and the three JSON blocks of ratchet `docs/evidence-format.md` § 6,
//! copied verbatim from branch `evidence-format` at c3cb020: `example.evidence.json`,
//! `example-incomplete.results.json` and `example-not-executed.json`. `golden.evidence.json` is
//! ux-static's own output over those inputs; `UX_STATIC_BLESS=1` rewrites it.

use std::path::{Path, PathBuf};
use ux_static::{Inputs, assess};
use uxui_doc::yaml::{self, Value};

const STRUCTURE: &str = "structure.required_present@1";
const ACTIONS: &str = "effort.actions_per_journey@1";
const NAMES: &str = "a11y.accessible_names@1";
const FINDINGS: &str = "findings";

const J_STOP: &str = "stopping a running campaign asks first and sends the command";
const J_PROGRESS: &str = "a progress event lands in the live climb";
const J_SWITCH: &str = "switching one axis of a variant sends the switch";
const J_OWNER: &str = "the owner sees what gets worse and is offered only the accept that applies";
const J_START: &str = "starting a campaign from the catalog opens the form on that target";
const JOURNEYS: [&str; 5] = [J_STOP, J_PROGRESS, J_SWITCH, J_OWNER, J_START];

/// Three input files and the repository root they lie under.
struct Files {
    document: PathBuf,
    journeys: PathBuf,
    declarations: PathBuf,
    root: PathBuf,
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ratchet-example")
}

fn under(root: PathBuf) -> Files {
    let ui = root.join("ess/ui");
    Files {
        document: ui.join("studio.ui.yaml"),
        journeys: ui.join("tests/studio.yaml"),
        declarations: ui.join("studio.uxui.yaml"),
        root,
    }
}

fn studio() -> Files {
    under(repo())
}

fn run(files: &Files) -> Result<Value, String> {
    run_as(files, "local")
}

fn run_as(files: &Files, class: &str) -> Result<Value, String> {
    assess(&Inputs {
        document: &files.document,
        journeys: &files.journeys,
        declarations: &files.declarations,
        root: &files.root,
        project: "uxui",
        assessment: "studio-static",
        class,
    })
}

/// `text` with `from` replaced by `to`; fails when `from` is not there, so a plant is never a no-op.
fn plant(text: &str, from: &str, to: &str) -> String {
    assert!(
        text.contains(from),
        "the plant's anchor is not in the file: {from}"
    );
    text.replacen(from, to, 1)
}

/// A copy of the studio's three inputs under the test's own directory, each edited by `edit`; the
/// copy's directory is its repository root.
fn copy(name: &str, edit: impl Fn(&str, String) -> String) -> Files {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let out = under(dir);
    std::fs::create_dir_all(out.journeys.parent().unwrap()).unwrap();
    let from = studio();
    for (from, to, kind) in [
        (&from.document, &out.document, "document"),
        (&from.journeys, &out.journeys, "journeys"),
        (&from.declarations, &out.declarations, "declarations"),
    ] {
        std::fs::write(to, edit(kind, std::fs::read_to_string(from).unwrap())).unwrap();
    }
    out
}

fn evidence(doc: &Value) -> &Value {
    doc.get("evidence").expect("an `evidence` member")
}

fn results(doc: &Value) -> &[Value] {
    evidence(doc).get("results").map(Value::list).unwrap_or(&[])
}

fn find<'a>(doc: &'a Value, output: &str, item: &str) -> &'a Value {
    results(doc)
        .iter()
        .find(|r| r.get_str("output") == Some(output) && r.get_str("item") == Some(item))
        .unwrap_or_else(|| panic!("no result for {output} at {item}"))
}

/// The value of a number result, read from its text.
fn value(result: &Value) -> Option<i64> {
    result
        .get("value")
        .map(|v| v.str().expect("a value is text").parse().expect("a count"))
}

fn cause(result: &Value) -> Option<&str> {
    result.get("incomplete").and_then(|i| i.get_str("cause"))
}

fn int(at: &Value, path: &[&str]) -> i64 {
    let mut at = at;
    for key in path {
        at = at
            .get(key)
            .unwrap_or_else(|| panic!("no `{key}` in {path:?}"));
    }
    match at {
        Value::Int(n) => *n,
        other => panic!("{path:?} is {other:?}"),
    }
}

fn coverage(doc: &Value, key: &str) -> i64 {
    int(evidence(doc), &["coverage", key])
}

/// The node paths of the findings of one scope item.
fn findings(doc: &Value, item: &str) -> Vec<String> {
    find(doc, FINDINGS, item)
        .get("items")
        .expect("an identity set has `items`, also when empty")
        .list()
        .iter()
        .map(|i| {
            i.get("identity")
                .and_then(|id| id.get_str("node_path"))
                .unwrap()
                .to_string()
        })
        .collect()
}

fn not_executed(doc: &Value) -> Vec<(String, String)> {
    evidence(doc)
        .get("coverage")
        .and_then(|c| c.get("not_executed"))
        .map(Value::list)
        .unwrap_or(&[])
        .iter()
        .map(|e| {
            (
                e.get_str("id").unwrap().to_string(),
                e.get_str("state").unwrap().to_string(),
            )
        })
        .collect()
}

fn get_mut<'a>(value: &'a mut Value, key: &str) -> &'a mut Value {
    match value {
        Value::Map(entries) => entries
            .iter_mut()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
            .unwrap_or_else(|| panic!("no `{key}`")),
        other => panic!("not a map: {other:?}"),
    }
}

fn remove(value: &mut Value, key: &str) {
    if let Value::Map(entries) = value {
        entries.retain(|(k, _)| k != key);
    }
}

/// Ratchet's example leaves out the list of every node with its name under the accessible names'
/// `details` (§ 6); so does this comparison.
fn without_name_items(mut result: Value) -> Value {
    if result.get_str("output") == Some(NAMES) {
        remove(get_mut(&mut result, "details"), "items");
    }
    result
}

fn read_json(path: &Path) -> Value {
    let text = std::fs::read_to_string(path).unwrap();
    yaml::parse(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn strings<'a>(value: &'a Value, out: &mut Vec<&'a str>) {
    match value {
        Value::Str(s) => out.push(s),
        Value::List(items) => items.iter().for_each(|i| strings(i, out)),
        Value::Map(entries) => entries.iter().for_each(|(k, v)| {
            out.push(k);
            strings(v, out)
        }),
        _ => {}
    }
}

#[test]
fn sha256_matches_the_published_vectors() {
    use uxui_doc::sha256::hex;
    assert_eq!(
        hex(b""),
        "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        hex(b"abc"),
        "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    let long = "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
    assert_eq!(
        hex(long.as_bytes()),
        "sha256:248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
}

#[test]
fn the_studio_gives_one_document_with_a_result_per_output_and_scope_item_each_complete() {
    let doc = run(&studio()).unwrap();
    assert_eq!(doc.get_str("format"), Some("ratchet-evidence/1"));
    let keys: Vec<_> = doc.entries().iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(keys, ["format", "evidence"], "an exact run writes no `run`");
    let count = |output: &str| {
        results(&doc)
            .iter()
            .filter(|r| r.get_str("output") == Some(output))
            .count()
    };
    assert_eq!(
        count(STRUCTURE),
        6,
        "five journeys and the variant's always-visible nodes"
    );
    assert_eq!(count(ACTIONS), 5, "one per journey");
    assert_eq!(count(NAMES), 10, "one per page");
    assert_eq!(count(FINDINGS), 16, "one per scope item of a rule");
    assert_eq!(results(&doc).len(), 37);
    let e = evidence(&doc);
    assert_eq!(e.get_str("assessor"), Some("ux-static@1"));
    assert_eq!(e.get_str("completion"), Some("completed"));
    for result in results(&doc) {
        assert_eq!(result.get_str("kind"), Some("exact"));
        assert!(result.get("incomplete").is_none(), "{result:?}");
        match result.get_str("value_type") {
            Some("number") => assert!(value(result).is_some(), "{result:?}"),
            Some("identity_set") => assert_eq!(result.get("items"), Some(&Value::List(vec![]))),
            other => panic!("value type {other:?}"),
        }
    }
    for journey in JOURNEYS {
        assert_eq!(value(find(&doc, STRUCTURE, journey)), Some(0), "{journey}");
    }
    assert_eq!(value(find(&doc, STRUCTURE, "document")), Some(0));
    let actions: Vec<_> = JOURNEYS
        .iter()
        .map(|j| value(find(&doc, ACTIONS, j)))
        .collect();
    assert_eq!(actions, [Some(2), Some(0), Some(1), Some(1), Some(1)]);
    let list = find(&doc, NAMES, "pages/campaigns.list");
    assert_eq!(value(list), Some(0));
    assert_eq!(
        int(list, &["details", "outcomes", "passed"]),
        6,
        "header start, climb, frontier, stop, its confirm, the empty state's start"
    );
    assert_eq!(int(list, &["details", "outcomes", "cantTell"]), 0);
    let start = find(&doc, NAMES, "pages/campaigns.start");
    assert_eq!(
        int(start, &["details", "outcomes", "passed"]),
        10,
        "nine form inputs and the submit button"
    );
    assert_eq!(coverage(&doc, "unsupported"), 0);
    assert_eq!(coverage(&doc, "not_run"), 0);
    assert!(not_executed(&doc).is_empty());
}

#[test]
fn a_run_names_its_target_inputs_and_tool_by_digest_and_repository_relative_path() {
    let files = studio();
    let doc = run(&files).unwrap();
    let e = evidence(&doc);
    let digest = |p: &Path| uxui_doc::sha256::hex(&std::fs::read(p).unwrap());
    let document = digest(&files.document);
    assert_eq!(e.get_str("candidate"), Some(document.as_str()));
    let target = e.get("target").unwrap();
    assert_eq!(target.get_str("kind"), Some("document"));
    assert_eq!(target.get_str("class"), Some("local"));
    assert_eq!(target.get_str("selection"), Some("candidate"));
    let ids = target.get("identifiers").unwrap();
    assert_eq!(ids.get_str("path"), Some("ess/ui/studio.ui.yaml"));
    assert_eq!(ids.get_str("digest"), Some(document.as_str()));
    assert_eq!(ids.get_str("digest_form"), Some("bytes"));
    let inputs = e.get("inputs").unwrap();
    for (name, path, file) in [
        ("journeys", "ess/ui/tests/studio.yaml", &files.journeys),
        (
            "declarations",
            "ess/ui/studio.uxui.yaml",
            &files.declarations,
        ),
    ] {
        let input = inputs.get(name).unwrap();
        assert_eq!(input.get_str("value"), Some(path));
        assert_eq!(input.get_str("digest"), Some(digest(file).as_str()));
        assert_eq!(input.get_str("digest_form"), Some("bytes"));
    }
    assert_eq!(
        e.get("conditions")
            .and_then(|c| c.get("tools"))
            .and_then(|t| t.get_str("ux-static")),
        Some(env!("CARGO_PKG_VERSION"))
    );
    let ci = run_as(&files, "ci").unwrap();
    assert_eq!(
        evidence(&ci).get("target").unwrap().get_str("class"),
        Some("ci")
    );
}

#[test]
fn a_required_node_removed_is_a_finding_and_its_journey_an_incompatible_target() {
    let files = copy("required-removed", |kind, text| match kind {
        "document" => plant(
            &text,
            "          - name: stop\n            does: uxsearch.search.StopCampaign\n            bind: {campaign_id: row.campaign_id}\n            visible: row.state == Running\n            confirm: {title: Stop this campaign}\n            label: Stop\n",
            "",
        ),
        _ => text,
    });
    let doc = run(&files).unwrap();
    let structure = find(&doc, STRUCTURE, J_STOP);
    assert!(structure.get("incomplete").is_none());
    assert_eq!(value(structure), Some(2));
    assert_eq!(
        findings(&doc, J_STOP),
        [
            "pages/campaigns.list/sections/list/row_actions/stop",
            "pages/campaigns.list/sections/list/row_actions/stop/confirm/overlay",
        ]
    );
    let item = &find(&doc, FINDINGS, J_STOP).get("items").unwrap().list()[0];
    assert_eq!(
        item.get("identity").unwrap().get_str("rule"),
        Some(STRUCTURE)
    );
    assert_eq!(
        item.get("fields").unwrap().get_str("reason"),
        Some("no node at this path in the document")
    );
    assert_eq!(
        value(find(&doc, STRUCTURE, J_OWNER)),
        Some(0),
        "other journeys are untouched"
    );
    assert!(findings(&doc, J_OWNER).is_empty());
    let actions = find(&doc, ACTIONS, J_STOP);
    assert_eq!(
        cause(actions),
        Some("incompatible_target"),
        "a journey that acts on a node the document lacks cannot be followed"
    );
    assert!(actions.get("value").is_none(), "absent, never null");
    assert!(actions.get("at_least").is_none());
    assert_eq!(coverage(&doc, "not_run"), 2, "the two steps on `stop`");
    assert_eq!(coverage(&doc, "failed"), 2, "the two declared nodes");
    assert_eq!(
        not_executed(&doc),
        [
            (format!("{ACTIONS} {J_STOP} step 3"), "not_run".to_string()),
            (format!("{ACTIONS} {J_STOP} step 4"), "not_run".to_string()),
        ]
    );
}

#[test]
fn a_button_without_a_name_is_a_finding_of_its_page_only() {
    let files = copy("nameless-button", |kind, text| match kind {
        "document" => {
            let text = plant(
                &text,
                "{name: show, sets: {state.kpi: row.kpi_id}, label: Show its climb}",
                "{name: show, sets: {state.kpi: row.kpi_id}, label: \"\"}",
            );
            plant(
                &text,
                "{name: judge, opens: judge, label: Record judgement}",
                "{name: judge, opens: judge, as: icon}",
            )
        }
        _ => text,
    });
    let doc = run(&files).unwrap();
    let climb = find(&doc, NAMES, "pages/campaigns.climb");
    assert!(climb.get("incomplete").is_none());
    assert_eq!(value(climb), Some(1));
    assert_eq!(
        findings(&doc, "pages/campaigns.climb"),
        ["pages/campaigns.climb/sections/hill_kpis/row_actions/show"]
    );
    assert_eq!(value(find(&doc, NAMES, "pages/variants.compare")), Some(1));
    assert_eq!(
        findings(&doc, "pages/variants.compare"),
        ["pages/variants.compare/header/actions/judge"]
    );
    assert_eq!(
        value(find(&doc, NAMES, "pages/campaigns.list")),
        Some(0),
        "a page-scope finding stays on its page"
    );
    assert_eq!(coverage(&doc, "failed"), 2);
}

#[test]
fn a_journey_step_added_raises_its_actions() {
    let files = copy("step-added", |kind, text| match kind {
        "journeys" => plant(
            &text,
            "      - act: pages/variants.detail/sections/axes/rows/av-04/row_actions/switch\n",
            "      - select: pages/variants.detail/sections/axes/rows/av-04\n      - act: pages/variants.detail/sections/axes/rows/av-04/row_actions/switch\n",
        ),
        _ => text,
    });
    let doc = run(&files).unwrap();
    assert_eq!(value(find(&doc, ACTIONS, J_SWITCH)), Some(2));
    assert_eq!(
        value(find(&doc, ACTIONS, J_STOP)),
        Some(2),
        "other journeys are untouched"
    );
}

fn insufficient_copy(name: &str, empty_label: bool) -> Files {
    copy(name, move |kind, text| match kind {
        "document" => {
            let text = plant(
                &text,
                "        component: chart\n        chart: line\n        title: Best value per step\n",
                "        component: sparkline\n        title: Best value per step\n",
            );
            if empty_label {
                plant(
                    &text,
                    "{name: show, sets: {state.kpi: row.kpi_id}, label: Show its climb}",
                    "{name: show, sets: {state.kpi: row.kpi_id}, label: \"\"}",
                )
            } else {
                text
            }
        }
        "journeys" => plant(
            &text,
            "      - advance: 1s\n",
            "      - hover: pages/campaigns.climb/sections/points\n      - advance: 1s\n",
        ),
        _ => plant(
            &text,
            "  a progress event lands in the live climb:\n    required:\n      - pages/campaigns.climb/sections/progress\n      - pages/campaigns.climb/sections/points\n    signal: pages/campaigns.climb/sections/points\n",
            "",
        ),
    })
}

#[test]
fn what_cannot_be_computed_is_incomplete_with_its_cause_never_a_pass() {
    let doc = run(&insufficient_copy("insufficient", false)).unwrap();
    for (output, item) in [
        (ACTIONS, J_PROGRESS),
        (STRUCTURE, J_PROGRESS),
        (NAMES, "pages/campaigns.climb"),
        (FINDINGS, J_PROGRESS),
        (FINDINGS, "pages/campaigns.climb"),
    ] {
        let result = find(&doc, output, item);
        assert_eq!(
            cause(result),
            Some("coverage_below_plan"),
            "{output} {item}: an unknown step, nothing declared, a cantTell node"
        );
        assert!(result.get("value").is_none(), "{output} {item}");
        assert!(
            result.get("at_least").is_none(),
            "a bound of 0 says nothing"
        );
    }
    assert_eq!(
        find(&doc, FINDINGS, J_PROGRESS).get("items"),
        Some(&Value::List(vec![])),
        "nothing declared: an empty set, incomplete"
    );
    assert!(
        find(&doc, STRUCTURE, J_PROGRESS).get("details").is_none(),
        "no check ran for it"
    );
    assert!(
        find(&doc, ACTIONS, J_SWITCH).get("incomplete").is_none(),
        "the rest is still complete"
    );
}

#[test]
fn a_cant_tell_check_is_counted_only_as_unsupported_and_listed_in_not_executed() {
    let clean = run(&studio()).unwrap();
    let doc = run(&insufficient_copy("cant-tell", false)).unwrap();
    let hill = "pages/campaigns.climb/sections/hill";
    assert!(
        !findings(&doc, "pages/campaigns.climb")
            .iter()
            .any(|p| p == hill),
        "a node that could not be judged is no finding"
    );
    assert_eq!(
        not_executed(&doc),
        [
            (
                format!("{ACTIONS} {J_PROGRESS} step 4"),
                "unsupported".to_string()
            ),
            (format!("{NAMES} {hill}"), "unsupported".to_string()),
        ]
    );
    assert_eq!(coverage(&doc, "unsupported"), 2);
    assert_eq!(coverage(&doc, "not_run"), 0);
    assert_eq!(coverage(&doc, "skipped"), 0);
    let executed = coverage(&doc, "executed");
    assert!(coverage(&doc, "passed") + coverage(&doc, "failed") <= executed);
    let names = find(&doc, NAMES, "pages/campaigns.climb");
    assert_eq!(int(names, &["details", "outcomes", "cantTell"]), 1);
    assert_eq!(
        executed,
        coverage(&clean, "executed") - 2,
        "the journey's two declared nodes are gone; the unknown step and the opaque chart are not executed"
    );
}

#[test]
fn a_lower_bound_is_the_value_with_at_least() {
    let doc = run(&insufficient_copy("lower-bound", true)).unwrap();
    let climb = find(&doc, NAMES, "pages/campaigns.climb");
    assert_eq!(climb.get("value"), Some(&Value::Str("1".into())));
    assert_eq!(climb.get("at_least"), Some(&Value::Bool(true)));
    assert_eq!(cause(climb), Some("coverage_below_plan"));
    assert_eq!(
        findings(&doc, "pages/campaigns.climb"),
        ["pages/campaigns.climb/sections/hill_kpis/row_actions/show"],
        "an incomplete set holds the items that were found"
    );
    assert_eq!(
        cause(find(&doc, FINDINGS, "pages/campaigns.climb")),
        Some("coverage_below_plan")
    );
}

#[test]
fn a_node_declared_twice_for_one_scope_item_is_one_check() {
    let doc = run(&studio()).unwrap();
    for (journey, passed) in [(J_PROGRESS, 2), (J_START, 3), (J_STOP, 4)] {
        assert_eq!(
            int(
                find(&doc, STRUCTURE, journey),
                &["details", "outcomes", "passed"]
            ),
            passed,
            "{journey}: the signal is one of the required nodes, or not"
        );
    }
}

#[test]
fn the_output_matches_ratchets_example_document() {
    let fixtures = fixtures();
    let mut doc = run(&under(fixtures.clone())).unwrap();
    let results = get_mut(get_mut(&mut doc, "evidence"), "results");
    if let Value::List(items) = results {
        let taken = std::mem::take(items);
        *items = taken.into_iter().map(without_name_items).collect();
    }
    let mut example = read_json(&fixtures.join("example.evidence.json"));
    // Ratchet's example counts a node declared twice for one journey as two checks; ux-static
    // counts it once (`a_node_declared_twice_for_one_scope_item_is_one_check`): two journeys
    // declare their signal among their required nodes.
    let counts = get_mut(get_mut(&mut example, "evidence"), "coverage");
    *get_mut(counts, "executed") = Value::Int(95);
    *get_mut(counts, "passed") = Value::Int(73);
    let results = get_mut(get_mut(&mut example, "evidence"), "results");
    if let Value::List(items) = results {
        for (journey, passed) in [(J_PROGRESS, 2), (J_START, 3)] {
            let result = items
                .iter_mut()
                .find(|r| {
                    r.get_str("output") == Some(STRUCTURE) && r.get_str("item") == Some(journey)
                })
                .unwrap();
            *get_mut(get_mut(get_mut(result, "details"), "outcomes"), "passed") =
                Value::Int(passed);
        }
    }
    assert_eq!(doc, example);
}

#[test]
fn incomplete_results_and_not_executed_match_ratchets_examples() {
    let fixtures = fixtures();
    let doc = run(&insufficient_copy("ratchet-incomplete", true)).unwrap();
    let expected = read_json(&fixtures.join("example-incomplete.results.json"));
    assert_eq!(expected.list().len(), 4);
    for want in expected.list() {
        let got = find(
            &doc,
            want.get_str("output").unwrap(),
            want.get_str("item").unwrap(),
        );
        assert_eq!(&without_name_items(got.clone()), want);
    }
    let want = read_json(&fixtures.join("example-not-executed.json"));
    assert_eq!(
        evidence(&doc).get("coverage").unwrap().get("not_executed"),
        Some(&want)
    );
}

#[test]
fn the_output_over_fixed_inputs_is_the_golden_file_byte_for_byte() {
    let fixtures = fixtures();
    let text = ux_static::json(&run(&under(fixtures.clone())).unwrap());
    let golden = fixtures.join("golden.evidence.json");
    if std::env::var_os("UX_STATIC_BLESS").is_some() {
        std::fs::write(&golden, &text).unwrap();
    }
    assert_eq!(text, std::fs::read_to_string(&golden).unwrap());
}

#[test]
fn no_absolute_path_is_in_the_output_and_an_input_outside_the_root_is_refused() {
    let files = copy("relative-paths", |_, text| text);
    let doc = run(&files).unwrap();
    let root = files.root.canonicalize().unwrap();
    let mut all = Vec::new();
    strings(&doc, &mut all);
    for s in &all {
        assert!(!s.starts_with('/'), "absolute: {s}");
        assert!(!s.contains('\\'), "not `/`: {s}");
        assert!(!s.contains(root.to_str().unwrap()), "holds the root: {s}");
    }
    let ids = evidence(&doc)
        .get("target")
        .unwrap()
        .get("identifiers")
        .unwrap();
    assert_eq!(ids.get_str("path"), Some("ess/ui/studio.ui.yaml"));
    let roundabout = Files {
        document: files.root.join("ess/ui/tests/../studio.ui.yaml"),
        journeys: files.journeys.clone(),
        declarations: files.declarations.clone(),
        root: files.root.join("ess/.."),
    };
    assert_eq!(
        run(&roundabout).unwrap(),
        doc,
        "the same path, however spelled"
    );
    let narrow = Files {
        document: files.document.clone(),
        journeys: files.journeys.clone(),
        declarations: files.declarations.clone(),
        root: files.root.join("ess/ui/tests"),
    };
    let error = run(&narrow).unwrap_err();
    assert!(error.contains("not under the repository root"), "{error}");
}

#[test]
fn inputs_that_do_not_belong_together_are_refused() {
    let other = copy("other-document", |kind, text| match kind {
        "declarations" => plant(&text, "document: studio.ui.yaml", "document: other.ui.yaml"),
        _ => text,
    });
    let error = run(&other).unwrap_err();
    assert!(error.contains("other.ui.yaml"), "{error}");
    let typo = copy("unknown-key", |kind, text| match kind {
        "declarations" => plant(&text, "always:", "alway:"),
        _ => text,
    });
    let error = run(&typo).unwrap_err();
    assert!(error.contains("alway"), "{error}");
    let stray = copy("stray-journey", |kind, text| match kind {
        "declarations" => {
            text + "  a journey nobody wrote:\n    required: [pages/campaigns.list]\n"
        }
        _ => text,
    });
    let error = run(&stray).unwrap_err();
    assert!(error.contains("a journey nobody wrote"), "{error}");
    let error = run_as(&studio(), "laptop").unwrap_err();
    assert!(error.contains("laptop"), "{error}");
}

#[test]
fn the_binary_writes_one_document_and_the_same_bytes_every_time() {
    let fixtures = fixtures();
    let files = under(fixtures.clone());
    let exe = env!("CARGO_BIN_EXE_ux-static");
    let once = || {
        let out = std::process::Command::new(exe)
            .current_dir(&fixtures)
            .args(["--document", "ess/ui/studio.ui.yaml"])
            .args(["--journeys".as_ref(), files.journeys.as_os_str()])
            .args(["--declarations", "ess/ui/studio.uxui.yaml"])
            .args(["--project", "uxui", "--assessment", "studio-static"])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    };
    let text = once();
    assert_eq!(text, once(), "deterministic");
    assert_eq!(
        text,
        std::fs::read_to_string(fixtures.join("golden.evidence.json")).unwrap(),
        "the root defaults to the working directory"
    );
    let doc = yaml::parse(&text).unwrap();
    assert_eq!(doc.get_str("format"), Some("ratchet-evidence/1"));
    let usage = std::process::Command::new(exe).output().unwrap();
    assert_eq!(usage.status.code(), Some(2));
    let no_plan = std::process::Command::new(exe)
        .current_dir(&fixtures)
        .args(["--document", "ess/ui/studio.ui.yaml"])
        .args(["--journeys", "ess/ui/tests/studio.yaml"])
        .args(["--declarations", "ess/ui/studio.uxui.yaml"])
        .output()
        .unwrap();
    assert_eq!(
        no_plan.status.code(),
        Some(2),
        "project and assessment are required"
    );
}
