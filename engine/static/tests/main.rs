//! The one integration-test binary of ux-static: the studio, and planted broken copies of it.

use std::path::{Path, PathBuf};
use ux_static::{Inputs, assess};
use uxui_doc::yaml::{self, Value};

const STRUCTURE: &str = "structure.required_present@1";
const ACTIONS: &str = "effort.actions_per_journey@1";
const NAMES: &str = "a11y.accessible_names@1";

const J_STOP: &str = "stopping a running campaign asks first and sends the command";
const J_PROGRESS: &str = "a progress event lands in the live climb";
const J_SWITCH: &str = "switching one axis of a variant sends the switch";
const J_OWNER: &str = "the owner sees what gets worse and is offered only the accept that applies";
const J_START: &str = "starting a campaign from the catalog opens the form on that target";

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn studio() -> (PathBuf, PathBuf, PathBuf) {
    let ui = repo().join("ess/ui");
    (
        ui.join("studio.ui.yaml"),
        ui.join("tests/studio.yaml"),
        ui.join("studio.uxui.yaml"),
    )
}

fn run(files: &(PathBuf, PathBuf, PathBuf)) -> Result<Vec<Value>, String> {
    assess(&Inputs {
        document: &files.0,
        journeys: &files.1,
        declarations: &files.2,
        class: "local",
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

/// A copy of the studio's three inputs under the test's own directory, each edited by `edit`.
fn copy(name: &str, edit: impl Fn(&str, String) -> String) -> (PathBuf, PathBuf, PathBuf) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    let (doc, tests, decl) = studio();
    let out = (
        dir.join("studio.ui.yaml"),
        dir.join("tests/studio.yaml"),
        dir.join("studio.uxui.yaml"),
    );
    for (from, to, kind) in [
        (&doc, &out.0, "document"),
        (&tests, &out.1, "journeys"),
        (&decl, &out.2, "declarations"),
    ] {
        std::fs::write(to, edit(kind, std::fs::read_to_string(from).unwrap())).unwrap();
    }
    out
}

fn find<'a>(records: &'a [Value], metric: &str, scope: &str) -> &'a Value {
    records
        .iter()
        .find(|r| {
            r.get("measurement").and_then(|m| m.get_str("output")) == Some(metric)
                && r.get("scope").and_then(|s| s.get_str("id")) == Some(scope)
        })
        .unwrap_or_else(|| panic!("no record for {metric} at {scope}"))
}

fn value(record: &Value) -> Option<i64> {
    match record.get("value") {
        Some(Value::Int(n)) => Some(*n),
        _ => None,
    }
}

fn int(record: &Value, path: &[&str]) -> i64 {
    let mut at = record;
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

fn findings(record: &Value) -> Vec<(String, String)> {
    record
        .get("findings")
        .map(Value::list)
        .unwrap_or(&[])
        .iter()
        .map(|f| {
            (
                f.get_str("node_path").unwrap_or("").to_string(),
                f.get_str("outcome").unwrap_or("").to_string(),
            )
        })
        .collect()
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
fn the_studio_gives_one_record_per_metric_and_scope_each_measured() {
    let records = run(&studio()).unwrap();
    let count = |metric: &str| {
        records
            .iter()
            .filter(|r| r.get("measurement").and_then(|m| m.get_str("output")) == Some(metric))
            .count()
    };
    assert_eq!(
        count(STRUCTURE),
        6,
        "five journeys and the variant's always-visible nodes"
    );
    assert_eq!(count(ACTIONS), 5, "one per journey");
    assert_eq!(count(NAMES), 10, "one per page");
    assert_eq!(records.len(), 21);
    for record in &records {
        assert_eq!(record.get_str("result"), Some("measured"), "{record:?}");
        assert!(value(record).is_some(), "{record:?}");
        assert_eq!(
            record.get("measurement").unwrap().get_str("kind"),
            Some("exact")
        );
        assert_eq!(record.get_str("assessor"), Some("ux-static@1"));
    }
    for journey in [J_STOP, J_PROGRESS, J_SWITCH, J_OWNER, J_START] {
        assert_eq!(
            value(find(&records, STRUCTURE, journey)),
            Some(0),
            "{journey}"
        );
    }
    assert_eq!(value(find(&records, STRUCTURE, "document")), Some(0));
    let actions: Vec<_> = [J_STOP, J_PROGRESS, J_SWITCH, J_OWNER, J_START]
        .iter()
        .map(|j| value(find(&records, ACTIONS, j)))
        .collect();
    assert_eq!(actions, [Some(2), Some(0), Some(1), Some(1), Some(1)]);
    let list = find(&records, NAMES, "pages/campaigns.list");
    assert_eq!(value(list), Some(0));
    assert_eq!(
        int(list, &["outcomes", "passed"]),
        6,
        "header start, climb, frontier, stop, its confirm, the empty state's start"
    );
    assert_eq!(int(list, &["outcomes", "cantTell"]), 0);
    let start = find(&records, NAMES, "pages/campaigns.start");
    assert_eq!(
        int(start, &["outcomes", "passed"]),
        10,
        "nine form inputs and the submit button"
    );
}

#[test]
fn a_record_names_its_resolved_target_by_digest_journey_and_tool_version() {
    let files = studio();
    let records = run(&files).unwrap();
    let record = find(&records, ACTIONS, J_SWITCH);
    let target = record.get("resolved_target").unwrap();
    let digest = |p: &Path| uxui_doc::sha256::hex(&std::fs::read(p).unwrap());
    assert_eq!(
        target.get("document").unwrap().get_str("digest"),
        Some(digest(&files.0).as_str())
    );
    assert_eq!(
        target.get("journeys").unwrap().get_str("digest"),
        Some(digest(&files.1).as_str())
    );
    assert_eq!(
        target.get("declarations").unwrap().get_str("digest"),
        Some(digest(&files.2).as_str())
    );
    assert_eq!(target.get_str("journey"), Some(J_SWITCH));
    assert_eq!(target.get_str("tool"), Some("ux-static"));
    assert_eq!(
        target.get_str("tool_version"),
        Some(env!("CARGO_PKG_VERSION"))
    );
    assert_eq!(
        record.get("target").unwrap().get_str("class"),
        Some("local")
    );
    assert!(
        find(&records, NAMES, "pages/campaigns.list")
            .get("resolved_target")
            .unwrap()
            .get("journey")
            .is_none()
    );
}

#[test]
fn a_required_node_removed_is_counted_missing_at_its_path() {
    let files = copy("required-removed", |kind, text| match kind {
        "document" => plant(
            &text,
            "          - name: stop\n            does: uxsearch.search.StopCampaign\n            bind: {campaign_id: row.campaign_id}\n            visible: row.state == Running\n            confirm: {title: Stop this campaign}\n            label: Stop\n",
            "",
        ),
        _ => text,
    });
    let records = run(&files).unwrap();
    let structure = find(&records, STRUCTURE, J_STOP);
    assert_eq!(structure.get_str("result"), Some("measured"));
    assert_eq!(value(structure), Some(2));
    let missing = findings(structure);
    assert!(
        missing.contains(&(
            "pages/campaigns.list/sections/list/row_actions/stop".into(),
            "failed".into()
        )),
        "{missing:?}"
    );
    assert!(missing.contains(&(
        "pages/campaigns.list/sections/list/row_actions/stop/confirm/overlay".into(),
        "failed".into()
    )));
    assert_eq!(
        value(find(&records, STRUCTURE, J_OWNER)),
        Some(0),
        "other journeys are untouched"
    );
    let actions = find(&records, ACTIONS, J_STOP);
    assert_eq!(
        actions.get_str("result"),
        Some("insufficient"),
        "a journey that acts on a node the document lacks cannot be counted"
    );
    assert_eq!(actions.get("value"), Some(&Value::Null));
}

#[test]
fn a_button_without_a_name_fails_its_node() {
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
    let records = run(&files).unwrap();
    let climb = find(&records, NAMES, "pages/campaigns.climb");
    assert_eq!(climb.get_str("result"), Some("measured"));
    assert_eq!(value(climb), Some(1));
    assert_eq!(
        findings(climb),
        [(
            "pages/campaigns.climb/sections/hill_kpis/row_actions/show".to_string(),
            "failed".to_string()
        )]
    );
    let compare = find(&records, NAMES, "pages/variants.compare");
    assert_eq!(value(compare), Some(1));
    assert_eq!(
        findings(compare),
        [(
            "pages/variants.compare/header/actions/judge".to_string(),
            "failed".to_string()
        )]
    );
    assert_eq!(
        value(find(&records, NAMES, "pages/campaigns.list")),
        Some(0),
        "a page-scope finding stays on its page"
    );
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
    let records = run(&files).unwrap();
    assert_eq!(value(find(&records, ACTIONS, J_SWITCH)), Some(2));
    assert_eq!(
        value(find(&records, ACTIONS, J_STOP)),
        Some(2),
        "other journeys are untouched"
    );
}

#[test]
fn what_cannot_be_computed_is_insufficient_never_a_pass() {
    let files = copy("insufficient", |kind, text| match kind {
        "document" => plant(
            &text,
            "        component: chart\n        chart: line\n        title: Best value per step\n",
            "        component: sparkline\n        title: Best value per step\n",
        ),
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
    });
    let records = run(&files).unwrap();
    for (metric, scope) in [
        (ACTIONS, J_PROGRESS),
        (STRUCTURE, J_PROGRESS),
        (NAMES, "pages/campaigns.climb"),
    ] {
        let record = find(&records, metric, scope);
        assert_eq!(
            record.get_str("result"),
            Some("insufficient"),
            "{metric} {scope}"
        );
        assert_eq!(record.get("value"), Some(&Value::Null), "{metric} {scope}");
        assert!(
            record
                .get("insufficient")
                .and_then(|i| i.get_str("cause"))
                .is_some(),
            "{metric} {scope} names its cause"
        );
    }
    let climb = find(&records, NAMES, "pages/campaigns.climb");
    assert!(
        findings(climb).contains(&(
            "pages/campaigns.climb/sections/hill".into(),
            "cantTell".into()
        )),
        "{:?}",
        findings(climb)
    );
    assert_eq!(
        find(&records, ACTIONS, J_SWITCH).get_str("result"),
        Some("measured"),
        "the rest is still measured"
    );
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
}

#[test]
fn the_binary_writes_one_json_line_per_record_and_the_same_bytes_every_time() {
    let (doc, tests, decl) = studio();
    let exe = env!("CARGO_BIN_EXE_ux-static");
    let once = || {
        let out = std::process::Command::new(exe)
            .args([
                "--document".as_ref(),
                doc.as_os_str(),
                "--journeys".as_ref(),
                tests.as_os_str(),
                "--declarations".as_ref(),
                decl.as_os_str(),
            ])
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
    let lines: Vec<_> = text.lines().collect();
    assert_eq!(lines.len(), 21);
    for line in lines {
        let record = yaml::parse(line).unwrap_or_else(|e| panic!("{e}: {line}"));
        assert_eq!(record.get_str("record"), Some("assessment-run"));
    }
    let usage = std::process::Command::new(exe).output().unwrap();
    assert_eq!(usage.status.code(), Some(2));
}
