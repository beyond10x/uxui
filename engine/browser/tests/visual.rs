//! The visual ratchet (`visual` section, `visual.diff_share@1`, `visual.diff_px@1`). No test starts
//! a browser: the shots are PNG files Chromium wrote in real runs of the two fixture sites
//! (`fixtures/visual/site-base/`, `fixtures/visual/site-changed/`) with `fixtures/visual/visual.checks.yaml`:
//! `base/` is the run of the base site, taken as the baseline, `changed/` the run of the changed
//! copy (one moved block, one colour change, the live clock at a later time), with its captures.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use ux_browser::checks::{self, PageResult};
use ux_browser::evidence::{self, Header, Target};
use ux_browser::png::{self, Image};
use ux_browser::render::FileRef;
use ux_browser::values;
use ux_browser::visual::{self, Compared, Outcome, Tolerance, Unit, UnitResult};
use uxui_doc::yaml::Value;

/// A fresh directory for one test under cargo's temporary directory for integration tests.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("visual")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn config_text() -> String {
    std::fs::read_to_string(crate::fixture("visual/visual.checks.yaml")).unwrap()
}

/// The fixture configuration with its `dir` at `dir` and each `(from, to)` replaced.
fn config_at(dir: &Path, edits: &[(&str, &str)]) -> checks::Config {
    let mut text = config_text().replace("dir: visual", &format!("dir: '{}'", dir.display()));
    for (from, to) in edits {
        assert!(text.contains(from), "{from}");
        text = text.replace(from, to);
    }
    checks::config(&text).unwrap()
}

fn copy_pngs(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|x| x == "png") {
            std::fs::copy(&path, to.join(path.file_name().unwrap())).unwrap();
        }
    }
}

/// A ratchet directory for `name`: the base run as the baseline, the changed run as the shots.
fn ratchet(name: &str, edits: &[(&str, &str)]) -> checks::Config {
    let dir = scratch(name);
    copy_pngs(&crate::fixture("visual/base"), &dir.join("baseline"));
    copy_pngs(&crate::fixture("visual/changed"), &dir.join("shots"));
    config_at(&dir, edits)
}

fn units(config: &checks::Config, page: &str) -> Vec<Unit> {
    visual::units(config, config.visual.as_ref().unwrap(), page)
}

fn judged(config: &checks::Config, page: &str) -> Vec<UnitResult> {
    visual::judge(config.visual.as_ref().unwrap(), &units(config, page)).unwrap()
}

fn compared(r: &UnitResult) -> &Compared {
    match &r.outcome {
        Outcome::Compared(c) => c,
        Outcome::NotRun(why) => panic!("{}: not run: {why}", r.unit.file),
    }
}

/// The page of the changed run's capture, read against `config`.
fn page(config: &checks::Config, name: &str) -> PageResult {
    let capture = crate::fixture_json(&format!("visual/changed/{name}.capture.json"));
    checks::page_result(config, &capture).unwrap()
}

fn header_document(config: &checks::Config, pages: &[PageResult]) -> Value {
    let file = FileRef {
        path: None,
        digest: "sha256:aa".to_string(),
    };
    let header = Header {
        project: "fixture",
        assessment: "visual",
        class: "local",
        config: &file,
        target: Target::Build(&file),
        pages: None,
        local_inputs: Vec::new(),
        browser: "Chrome/149.0.0.0",
        started_at: "2026-11-27T10:00:00Z".to_string(),
        ended_at: "2026-11-27T10:01:00Z".to_string(),
    };
    evidence::document(&header, config, pages)
}

fn results(doc: &Value) -> &[Value] {
    doc.get("evidence").unwrap().get("results").unwrap().list()
}

fn result<'a>(doc: &'a Value, output: &str, item: &str) -> &'a Value {
    results(doc)
        .iter()
        .find(|r| r.get_str("output") == Some(output) && r.get_str("item") == Some(item))
        .unwrap_or_else(|| panic!("no {output} of {item}"))
}

fn coverage(doc: &Value, key: &str) -> i64 {
    match doc
        .get("evidence")
        .unwrap()
        .get("coverage")
        .unwrap()
        .get(key)
    {
        Some(Value::Int(n)) => *n,
        other => panic!("{key}: {other:?}"),
    }
}

/// The document of the changed run of `/moved/` with its visual results: one output of each.
pub fn document() -> Value {
    let config = ratchet("document", &[]);
    let pages = vec![page(&config, "moved")];
    let found = judged(&config, "/moved/");
    visual::with_visual(header_document(&config, &pages), &pages, &found)
}

// ---------------------------------------------------------------- configuration

#[test]
fn reads_the_visual_section_with_its_defaults() {
    let config = checks::config(&config_text()).unwrap();
    let v = config.visual.as_ref().unwrap();
    assert_eq!(v.dir, Path::new("visual"));
    assert!(v.full_page);
    assert_eq!(v.masks, [".clock"]);
    assert_eq!(v.shots(), ["viewport", "page"]);
    let strict = Tolerance {
        share: 0.001,
        threshold: 0,
    };
    assert_eq!(v.tolerance_for("/moved/", "small"), strict);
    assert_eq!(
        v.tolerance_for("/colour/", "small"),
        Tolerance {
            share: 0.001,
            threshold: 6
        },
        "the rule's threshold, the section's share"
    );
    let minimal = checks::config(&format!(
        "{}visual: {{dir: ux/visual}}\n",
        config_text().split("visual:\n").next().unwrap()
    ))
    .unwrap();
    let m = minimal.visual.unwrap();
    assert!(!m.full_page);
    assert!(m.masks.is_empty());
    assert_eq!(m.shots(), ["viewport"]);
    assert_eq!(
        m.tolerance_for("/moved/", "small"),
        Tolerance {
            share: 0.0,
            threshold: 0
        },
        "no tolerance by default: one differing pixel fails"
    );
    let units = visual::units(&config, v, "/moved/");
    let files: Vec<&str> = units.iter().map(|u| u.file.as_str()).collect();
    assert_eq!(
        files,
        [
            "moved.small.light.viewport.png",
            "moved.small.light.page.png",
            "moved.small.dark.viewport.png",
            "moved.small.dark.page.png"
        ]
    );
}

#[test]
fn a_configuration_without_visual_has_no_visual_section() {
    for name in [
        "checks/layout.checks.yaml",
        "checks/hidden.checks.yaml",
        "checks/journey.checks.yaml",
        "checks/llm.checks.yaml",
    ] {
        let text = std::fs::read_to_string(crate::fixture(name)).unwrap();
        assert_eq!(checks::config(&text).unwrap().visual, None, "{name}");
    }
}

const HEAD: &str = "format: ux-browser-checks/1
base_path: /
pages: {paths: [/]}
viewports:
  - {name: small, width: 240, height: 160}
modes: [light]
";

#[test]
fn refuses_a_visual_section_that_does_not_hold_together() {
    for (section, expected) in [
        ("{dir: v, colour: red}", "visual: unknown key `colour`"),
        ("{full_page: true}", "visual: no `dir`"),
        ("{dir: v, full_page: 1}", "`full_page` is true or false"),
        ("{dir: v, masks: '.a'}", "`masks` is a list"),
        ("{dir: v, masks: ['.a', '.a']}", "mask `.a` is named twice"),
        (
            "{dir: v, tolerance: {share: 1.5}}",
            "`share` is a number from 0 to 1",
        ),
        (
            "{dir: v, tolerance: {share: -0.1}}",
            "`share` is a number from 0 to 1",
        ),
        (
            "{dir: v, tolerance: {threshold: 256}}",
            "`threshold` is a whole number from 0 to 255",
        ),
        ("{dir: v, tolerance: {ratio: 1}}", "unknown key `ratio`"),
        (
            "{dir: v, tolerances: [{viewport: huge, share: 0.1}]}",
            "viewport `huge` is not declared",
        ),
        (
            "{dir: v, tolerances: [{share: 0.1}]}",
            "names no `page` and no `viewport`",
        ),
        (
            "{dir: v, tolerances: [{page: moved, share: 0.1}]}",
            "does not start with `/`",
        ),
        ("{dir: v, tolerances: {page: /}}", "`tolerances` is a list"),
    ] {
        let text = format!("{HEAD}visual: {section}\n");
        match checks::config(&text) {
            Ok(_) => panic!("{section} is read"),
            Err(e) => assert!(e.contains(expected), "{section}: {e}"),
        }
    }
}

/// Runs `ux-browser` as on a CI runner without a browser (no `UXUI_CHROME`, an empty Playwright
/// cache), so a refusal that must come before the browser is not masked by a host that has one.
fn run(args: &[&str]) -> Output {
    let no_browsers = Path::new(env!("CARGO_TARGET_TMPDIR")).join("visual-no-browsers");
    std::fs::create_dir_all(&no_browsers).unwrap();
    Command::new(env!("CARGO_BIN_EXE_ux-browser"))
        .env_remove("UXUI_CHROME")
        .env("PLAYWRIGHT_BROWSERS_PATH", &no_browsers)
        .args(args)
        .output()
        .unwrap()
}

fn checks_run(config: &Path, extra: &[&str]) -> (Option<i32>, String) {
    let site = crate::fixture("visual/site-base");
    let mut args = vec![
        "checks",
        "--config",
        config.to_str().unwrap(),
        "--build",
        site.to_str().unwrap(),
        "--project",
        "p",
        "--assessment",
        "a",
    ];
    args.extend(extra);
    let o = run(&args);
    (
        o.status.code(),
        String::from_utf8_lossy(&o.stderr).to_string(),
    )
}

#[test]
fn checks_refuses_a_broken_visual_section_and_a_visual_flag_without_one_before_the_browser() {
    let dir = scratch("refusals");
    let broken = dir.join("broken.yaml");
    std::fs::write(
        &broken,
        format!("{HEAD}visual: {{dir: v, tolerance: {{threshold: 300}}}}\n"),
    )
    .unwrap();
    let (code, stderr) = checks_run(&broken, &[]);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(stderr.contains("threshold"), "{stderr}");
    let plain = dir.join("plain.yaml");
    std::fs::write(&plain, HEAD).unwrap();
    let (code, stderr) = checks_run(&plain, &["--visual", "update"]);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(stderr.contains("declares no `visual` section"), "{stderr}");
    let (code, stderr) = checks_run(&plain, &["--visual", "sometimes"]);
    assert_eq!(code, Some(2), "{stderr}");
    assert!(
        stderr.contains("compare") && stderr.contains("update"),
        "{stderr}"
    );
    assert!(
        !dir.join("v").exists(),
        "nothing is written before the browser"
    );
}

// ---------------------------------------------------------------- PNG

#[test]
fn reads_the_screenshots_chromium_wrote_and_writes_pngs_it_reads_back() {
    let mut read = 0;
    for run in ["base", "changed"] {
        for entry in std::fs::read_dir(crate::fixture(&format!("visual/{run}"))).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|x| x != "png") {
                continue;
            }
            let bytes = std::fs::read(&path).unwrap();
            let image = png::decode(&bytes).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            assert_eq!(
                Some((image.width, image.height)),
                ux_browser::render::png_size(&bytes)
            );
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            let size = if name.ends_with(".viewport.png") {
                (240, 160)
            } else {
                (240, 320)
            };
            assert_eq!((image.width, image.height), size, "{name}");
            assert_eq!(png::decode(&png::encode(&image)).unwrap(), image, "{name}");
            assert_eq!(
                png::encode(&image),
                png::encode(&image),
                "{name}: the same bytes"
            );
            read += 1;
        }
    }
    assert_eq!(read, 32, "4 pages x 2 modes x 2 shots, two runs");
}

#[test]
fn inflates_stored_fixed_and_dynamic_blocks() {
    // A stored block "abc", final.
    assert_eq!(
        png::inflate(&[0x01, 0x03, 0x00, 0xfc, 0xff, b'a', b'b', b'c']).unwrap(),
        b"abc"
    );
    // Fixed codes: what the writer makes, runs and repeats in it.
    let mut data = Vec::new();
    for i in 0..5000u32 {
        data.push((i % 7) as u8);
        data.extend_from_slice(&[9; 20]);
        data.push((i * 31 % 251) as u8);
    }
    assert_eq!(png::inflate(&png::deflate(&data)).unwrap(), data);
    assert_eq!(png::inflate(&png::deflate(&[])).unwrap(), b"");
    assert!(png::decode(b"GIF89a").is_err());
    // Dynamic codes are what Chromium writes: every fixture shot above reads.
}

#[test]
fn a_masked_box_is_filled_in_the_shot_and_a_box_outside_leaves_the_bytes() {
    let image = Image::filled(10, 10, [1, 2, 3, 255]);
    let bytes = png::encode(&image);
    assert_eq!(visual::mask(&bytes, &[]).unwrap(), bytes);
    assert_eq!(
        visual::mask(&bytes, &[[20.0, 0.0, 30.0, 5.0]]).unwrap(),
        bytes
    );
    let masked = png::decode(&visual::mask(&bytes, &[[2.5, 1.0, 4.2, 3.0]]).unwrap()).unwrap();
    assert_eq!(masked.pixel(2, 1), visual::MASK, "outward to whole pixels");
    assert_eq!(masked.pixel(4, 2), visual::MASK);
    assert_eq!(masked.pixel(5, 2), [1, 2, 3, 255]);
    assert_eq!(masked.pixel(2, 3), [1, 2, 3, 255]);
}

// ---------------------------------------------------------------- comparison

/// The block moved 40 px to the right: 40 x 40 px uncovered and 40 x 40 px covered.
const MOVED_PX: u64 = 2 * 40 * 40;

#[test]
fn a_moved_block_fails_its_tolerance_with_a_finding_and_a_diff_image() {
    let config = ratchet("moved", &[]);
    let found = judged(&config, "/moved/");
    assert_eq!(found.len(), 4);
    for r in &found {
        let c = compared(r);
        assert_eq!(c.px, MOVED_PX, "{}", r.unit.file);
        assert_eq!(r.passed(), Some(false), "{}", r.unit.file);
        let diff_file = config
            .visual
            .as_ref()
            .unwrap()
            .diff_dir()
            .join(c.diff.as_ref().unwrap());
        let diff = png::decode(&std::fs::read(&diff_file).unwrap()).unwrap();
        assert_eq!((diff.width, diff.height), (c.width, c.height));
        let marked = diff
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| **p == visual::MARK)
            .count() as u64;
        assert_eq!(marked, MOVED_PX, "every changed pixel is marked, no other");
        assert_eq!(diff.pixel(30, 30), visual::MARK, "uncovered");
        assert_eq!(diff.pixel(120, 30), visual::MARK, "covered");
        assert_ne!(diff.pixel(80, 30), visual::MARK, "covered before and after");
    }
    let viewport = &found[0];
    assert_eq!(
        viewport.share_micro(),
        Some(83_334),
        "3200 / 38400, rounded up"
    );
    assert_eq!(found[1].share_micro(), Some(41_667), "3200 / 76800");

    let pages = vec![page(&config, "moved")];
    let before = header_document(&config, &pages);
    let doc = visual::with_visual(before.clone(), &pages, &found);
    let share = result(&doc, visual::DIFF_SHARE, "/moved/");
    assert_eq!(share.get_str("value"), Some("0.083334"), "the largest unit");
    assert_eq!(share.get_str("unit"), Some("ratio"));
    assert_eq!(share.get("incomplete"), None);
    let px = result(&doc, visual::DIFF_PX, "/moved/");
    assert_eq!(
        px.get_str("value"),
        Some("12800"),
        "summed over the four units"
    );
    assert_eq!(
        px.get("details")
            .unwrap()
            .get("viewports")
            .unwrap()
            .get_str("small"),
        Some("12800")
    );
    let units = share.get("details").unwrap().get("units").unwrap().list();
    assert_eq!(units.len(), 4);
    assert_eq!(units[0].get_str("result"), Some("failed"));
    assert_eq!(
        units[0].get_str("diff"),
        Some("diff/moved.small.light.viewport.png")
    );
    let set = result(&doc, evidence::FINDINGS, "/moved/");
    let nodes: Vec<&str> = set
        .get("items")
        .unwrap()
        .list()
        .iter()
        .filter(|f| f.get("identity").unwrap().get_str("rule") == Some(visual::DIFF_SHARE))
        .map(|f| f.get("identity").unwrap().get_str("node_path").unwrap())
        .collect();
    assert_eq!(
        nodes,
        [
            "small/light/viewport",
            "small/light/page",
            "small/dark/viewport",
            "small/dark/page"
        ]
    );
    let reason = set.get("items").unwrap().list()[0]
        .get("fields")
        .unwrap()
        .get_str("reason")
        .unwrap()
        .to_string();
    assert_eq!(
        reason,
        "3200 of 38400 pixels differ (0.083334), above the tolerance 0.001 at threshold 0"
    );
    assert_eq!(
        coverage(&doc, "executed"),
        coverage(&before, "executed") + 4
    );
    assert_eq!(coverage(&doc, "failed"), coverage(&before, "failed") + 4);
    assert_eq!(coverage(&doc, "passed"), coverage(&before, "passed"));
    let first_set = results(&doc)
        .iter()
        .position(|r| r.get_str("value_type") == Some("identity_set"))
        .unwrap();
    assert_eq!(
        results(&doc)[first_set - 1].get_str("output"),
        Some(visual::DIFF_PX),
        "the visual numbers stand after the other numbers, before the sets"
    );
}

/// The block's red went from 0x33 to 0x3a: 7 of 255 on one channel, on its 80 x 40 px.
#[test]
fn a_colour_change_counts_below_its_threshold_and_not_above() {
    let config = ratchet("colour", &[]);
    for r in judged(&config, "/colour/") {
        assert_eq!(r.tolerance.threshold, 6, "the page's rule");
        assert_eq!(compared(&r).px, 80 * 40, "{}", r.unit.file);
        assert_eq!(r.passed(), Some(false));
    }
    let config = ratchet("colour-above", &[("threshold: 6", "threshold: 7")]);
    for r in judged(&config, "/colour/") {
        let c = compared(&r);
        assert_eq!(c.px, 0, "{}", r.unit.file);
        assert_eq!(c.diff, None, "no diff image without a changed pixel");
        assert_eq!(r.passed(), Some(true));
    }
    let base = png::decode(
        &std::fs::read(crate::fixture(
            "visual/base/colour.small.light.viewport.png",
        ))
        .unwrap(),
    )
    .unwrap();
    let changed = png::decode(
        &std::fs::read(crate::fixture(
            "visual/changed/colour.small.light.viewport.png",
        ))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(base.pixel(30, 30), [0x33, 0x66, 0xcc, 255]);
    assert_eq!(changed.pixel(30, 30), [0x3a, 0x66, 0xcc, 255]);
}

#[test]
fn a_masked_live_clock_is_equal_and_the_same_clock_unmasked_differs() {
    let config = ratchet("clock", &[]);
    for r in judged(&config, "/clock/") {
        assert_eq!(compared(&r).px, 0, "{}", r.unit.file);
        assert_eq!(r.passed(), Some(true));
    }
    let shot = png::decode(
        &std::fs::read(crate::fixture(
            "visual/changed/clock.small.light.viewport.png",
        ))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        shot.pixel(30, 90),
        visual::MASK,
        "the clock's box is blanked"
    );
    for r in judged(&config, "/clock-bare/") {
        assert!(compared(&r).px > 0, "{}: the time moved on", r.unit.file);
        assert_eq!(r.passed(), Some(false));
    }
}

#[test]
fn a_unit_without_a_baseline_is_not_run_and_never_a_pass() {
    let config = ratchet("missing", &[]);
    let v = config.visual.as_ref().unwrap();
    std::fs::remove_file(v.baseline_dir().join("clock.small.dark.page.png")).unwrap();
    let found = judged(&config, "/clock/");
    assert_eq!(
        found[3].outcome,
        Outcome::NotRun("no baseline baseline/clock.small.dark.page.png".to_string())
    );
    assert_eq!(found[3].passed(), None);
    let pages = vec![page(&config, "clock")];
    let before = header_document(&config, &pages);
    let doc = visual::with_visual(before.clone(), &pages, &found);
    for output in [visual::DIFF_SHARE, visual::DIFF_PX] {
        let r = result(&doc, output, "/clock/");
        assert_eq!(r.get("value"), None, "{output}: no complete 0");
        assert_eq!(
            r.get("incomplete").unwrap().get_str("cause"),
            Some("coverage_below_plan"),
            "{output}"
        );
    }
    assert_eq!(coverage(&doc, "not_run"), coverage(&before, "not_run") + 1);
    assert_eq!(coverage(&doc, "passed"), coverage(&before, "passed") + 3);
    assert_eq!(
        coverage(&doc, "executed"),
        coverage(&before, "executed") + 3
    );
    let entries = doc
        .get("evidence")
        .unwrap()
        .get("coverage")
        .unwrap()
        .get("not_executed")
        .unwrap()
        .list();
    let last = entries.last().unwrap();
    assert_eq!(
        last.get_str("id"),
        Some("/clock//small/dark/visual/page"),
        "`<item>/<viewport>/<mode>/visual/<shot>`, as every other coverage id"
    );
    assert_eq!(last.get_str("state"), Some("not_run"));
    // A page with a differing unit and a missing one keeps its count as a lower bound.
    std::fs::remove_file(v.baseline_dir().join("moved.small.dark.page.png")).unwrap();
    let pages = vec![page(&config, "moved")];
    let doc = visual::with_visual(
        header_document(&config, &pages),
        &pages,
        &judged(&config, "/moved/"),
    );
    let px = result(&doc, visual::DIFF_PX, "/moved/");
    assert_eq!(px.get_str("value"), Some("9600"));
    assert_eq!(px.get("at_least"), Some(&Value::Bool(true)));
}

#[test]
fn a_shot_of_another_size_than_its_baseline_fails_naming_both() {
    let config = ratchet("size", &[]);
    let v = config.visual.as_ref().unwrap();
    std::fs::copy(
        crate::fixture("visual/base/moved.small.light.page.png"),
        v.baseline_dir().join("moved.small.light.viewport.png"),
    )
    .unwrap();
    let found = judged(&config, "/moved/");
    let c = compared(&found[0]);
    assert_eq!(found[0].passed(), Some(false));
    assert_eq!(
        found[0].reason().as_deref(),
        Some("size 240x160 against the baseline's 240x320")
    );
    assert_eq!(c.diff, None);
    assert_eq!(found[0].share_micro(), Some(1_000_000));
}

#[test]
fn the_baseline_is_written_only_when_asked_for() {
    let dir = scratch("update");
    copy_pngs(&crate::fixture("visual/changed"), &dir.join("shots"));
    let config = config_at(&dir, &[]);
    let v = config.visual.as_ref().unwrap();
    let all = units(&config, "/moved/");
    let found = visual::judge(v, &all).unwrap();
    assert!(
        found
            .iter()
            .all(|r| matches!(r.outcome, Outcome::NotRun(_)))
    );
    assert!(!v.baseline_dir().exists(), "comparing writes no baseline");
    assert_eq!(visual::update_baseline(v, &all).unwrap(), 4);
    for u in &all {
        assert_eq!(
            std::fs::read(v.baseline_dir().join(&u.file)).unwrap(),
            std::fs::read(v.shots_dir().join(&u.file)).unwrap()
        );
    }
    let again = visual::judge(v, &all).unwrap();
    assert!(again.iter().all(|r| r.passed() == Some(true)));
}

#[test]
fn prepare_removes_the_shots_and_diffs_of_a_run_before_and_keeps_the_baseline() {
    let config = ratchet("prepare", &[]);
    let v = config.visual.as_ref().unwrap();
    judged(&config, "/moved/");
    assert!(v.diff_dir().join("moved.small.light.viewport.png").exists());
    visual::prepare(v).unwrap();
    assert_eq!(std::fs::read_dir(v.shots_dir()).unwrap().count(), 0);
    assert_eq!(std::fs::read_dir(v.diff_dir()).unwrap().count(), 0);
    assert_eq!(std::fs::read_dir(v.baseline_dir()).unwrap().count(), 16);
}

// ---------------------------------------------------------------- values

fn unit_result(viewport: &str, px: u64, total: u64) -> UnitResult {
    UnitResult {
        unit: Unit {
            path: "/moved/".to_string(),
            viewport: viewport.to_string(),
            mode: "light".to_string(),
            shot: "viewport",
            file: format!("moved.{viewport}.light.viewport.png"),
        },
        tolerance: Tolerance {
            share: 0.001,
            threshold: 0,
        },
        outcome: Outcome::Compared(Compared {
            width: 240,
            height: (total / 240) as u32,
            px,
            total,
            size: None,
            baseline: "sha256:bb".to_string(),
            shot: "sha256:cc".to_string(),
            diff: None,
        }),
    }
}

#[test]
fn the_values_file_carries_both_outputs_and_folds_the_share_to_the_largest() {
    let text = format!(
        "{}values:\n  scenarios: {{files: 'none/*.json'}}\n  waivers: {{files: 'none/*.md'}}\n  gates: {{}}\n  gating: [small, wide]\n",
        config_text().replace(
            "  - {name: small, width: 240, height: 160}\n",
            "  - {name: small, width: 240, height: 160}\n  - {name: wide, width: 240, height: 320}\n"
        )
    );
    let config = checks::config(&text).unwrap();
    let pages = vec![page(&config, "moved")];
    let found = vec![
        unit_result("small", 3200, 38_400),
        unit_result("wide", 100, 76_800),
    ];
    let doc = visual::with_visual(header_document(&config, &pages), &pages, &found);
    let section = config.values.as_ref().unwrap();
    let inputs = values::Inputs {
        scenarios: vec![("/moved/".to_string(), "page-moved".to_string())],
        waivers: Vec::new(),
    };
    let v = values::values(section, &inputs, &doc).unwrap();
    let file = ux_browser::json::parse(&v.text).unwrap();
    let moved = file.get("values").unwrap().get("page-moved").unwrap();
    assert_eq!(
        moved.get("visual.diff_share"),
        Some(&Value::Float(0.083334)),
        "the largest of 0.083334 and 0.001303"
    );
    assert_eq!(moved.get("visual.diff_px"), Some(&Value::Int(3300)));
    let wide = file.get("viewports").unwrap().get("wide").unwrap();
    assert_eq!(wide.get("visual.diff_share"), Some(&Value::Float(0.001303)));
}

// ---------------------------------------------------------------- the visual gate

/// A ratchet directory for `name` (the base run as the baseline, the changed run as the shots)
/// with `checks.yaml`, the fixture configuration with a values section whose `gates` are the lines
/// `gates`, and a page scenario for `/moved/` and `/clock/`.
fn gated(name: &str, gates: &str) -> (PathBuf, String) {
    let dir = scratch(name);
    copy_pngs(&crate::fixture("visual/base"), &dir.join("baseline"));
    copy_pngs(&crate::fixture("visual/changed"), &dir.join("shots"));
    let text = format!(
        "{}values:\n  scenarios: {{files: 'scenarios/*/page.json'}}\n  waivers: {{files: 'findings/*.md'}}\n  gates:\n{gates}",
        config_text().replace("dir: visual", &format!("dir: '{}'", dir.display()))
    );
    for (scenario, path) in [("page-moved", "/moved/"), ("page-clock", "/clock/")] {
        let at = dir.join("scenarios").join(scenario);
        std::fs::create_dir_all(&at).unwrap();
        std::fs::write(at.join("page.json"), format!("{{\"path\": \"{path}\"}}\n")).unwrap();
    }
    std::fs::write(dir.join("checks.yaml"), &text).unwrap();
    (dir, text)
}

/// `ux-browser values` in `dir` on the document of the changed run of the capture `name`: the exit
/// status, the values file and standard error.
fn gate_run(dir: &Path, name: &str) -> (Option<i32>, Value, String) {
    let config =
        checks::config(&std::fs::read_to_string(dir.join("checks.yaml")).unwrap()).unwrap();
    let pages = vec![page(&config, name)];
    let found = judged(&config, &format!("/{name}/"));
    let doc = visual::with_visual(header_document(&config, &pages), &pages, &found);
    let evidence = dir.join("visual.evidence.json");
    std::fs::write(&evidence, evidence::text(&doc)).unwrap();
    let out = dir.join("values.json");
    let o = run(&[
        "values",
        "--config",
        dir.join("checks.yaml").to_str().unwrap(),
        "--evidence",
        evidence.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
        "--root",
        dir.to_str().unwrap(),
    ]);
    let stderr = String::from_utf8_lossy(&o.stderr).to_string();
    let file = std::fs::read_to_string(&out).unwrap_or_else(|_| panic!("no values file: {stderr}"));
    (
        o.status.code(),
        ux_browser::json::parse(&file).unwrap(),
        stderr,
    )
}

/// The failure keys of a values file, in its order.
fn failure_keys(file: &Value) -> Vec<String> {
    file.get("units")
        .unwrap()
        .list()
        .iter()
        .flat_map(|u| u.get("failures").unwrap().list())
        .map(|f| f.get_str("key").unwrap().to_string())
        .collect()
}

const SHARE_GATE: &str = "    visual.diff_share@1: {gate: 6, max: 0.001}\n";
const PX_GATE: &str = "    visual.diff_px@1: {gate: 7, max: 100}\n";

#[test]
fn values_gates_take_a_visual_output_with_a_bound() {
    let (_, text) = gated(
        "gate-config",
        &format!("{SHARE_GATE}{PX_GATE}    layout.page_scroll@1: 1\n"),
    );
    let config = checks::config(&text).unwrap();
    let v = config.values.as_ref().unwrap();
    assert_eq!(
        v.gates,
        [
            ("visual.diff_share@1".to_string(), 6),
            ("visual.diff_px@1".to_string(), 7),
            ("layout.page_scroll@1".to_string(), 1),
        ]
    );
    assert_eq!(
        v.bounds,
        [
            ("visual.diff_share@1".to_string(), 0.001),
            ("visual.diff_px@1".to_string(), 100.0),
        ]
    );
    for (from, to, names) in [
        (
            SHARE_GATE,
            "    visual.diff_share@1: 6\n",
            "visual.diff_share@1",
        ),
        (
            SHARE_GATE,
            "    visual.diff_share@1: {gate: 6}\n",
            "visual.diff_share@1",
        ),
        (
            SHARE_GATE,
            "    visual.diff_share@1: {max: 0.001}\n",
            "visual.diff_share@1",
        ),
        (
            SHARE_GATE,
            "    visual.diff_share@1: {gate: 0, max: 0.001}\n",
            "visual.diff_share@1",
        ),
        (
            SHARE_GATE,
            "    visual.diff_share@1: {gate: 6, max: 1.5}\n",
            "visual.diff_share@1",
        ),
        (
            SHARE_GATE,
            "    visual.diff_share@1: {gate: 6, max: -0.1}\n",
            "visual.diff_share@1",
        ),
        (
            SHARE_GATE,
            "    visual.diff_share@1: {gate: 6, max: 0.001, min: 0}\n",
            "visual.diff_share@1",
        ),
        (
            PX_GATE,
            "    visual.diff_px@1: {gate: 7, max: 0.5}\n",
            "visual.diff_px@1",
        ),
        (
            PX_GATE,
            "    visual.diff_px@1: {gate: 7, max: -1}\n",
            "visual.diff_px@1",
        ),
        (
            "    layout.page_scroll@1: 1\n",
            "    layout.page_scroll@1: {gate: 1, max: 0}\n",
            "layout.page_scroll@1",
        ),
    ] {
        let broken = text.replace(from, to);
        assert_ne!(broken, text, "{to}");
        let err = checks::config(&broken)
            .err()
            .unwrap_or_else(|| panic!("{to}"));
        assert!(err.contains(names), "{to}: {err}");
    }
}

#[test]
fn a_visual_gate_fails_each_unit_over_its_bound_naming_page_viewport_and_mode() {
    let (dir, _) = gated("gate-over", SHARE_GATE);
    let (code, file, stderr) = gate_run(&dir, "moved");
    assert_eq!(code, Some(1), "{stderr}");
    assert_eq!(
        failure_keys(&file),
        [
            "gate6|/moved/|small|dark|visual.diff_share:page",
            "gate6|/moved/|small|dark|visual.diff_share:viewport",
            "gate6|/moved/|small|light|visual.diff_share:page",
            "gate6|/moved/|small|light|visual.diff_share:viewport",
        ]
    );
    let unit = &file.get("units").unwrap().list()[0];
    assert_eq!(unit.get_str("path"), Some("/moved/"));
    assert_eq!(unit.get_str("viewport"), Some("small"));
    assert_eq!(unit.get_str("theme"), Some("dark"));
    let failure = &unit.get("failures").unwrap().list()[1];
    assert_eq!(failure.get("gate"), Some(&Value::Int(6)));
    assert_eq!(
        failure.get_str("detail"),
        Some("visual.diff_share:viewport")
    );
    assert_eq!(failure.get("waivedBy"), Some(&Value::Null));
    assert!(
        stderr.contains(
            "ux-browser: hard gate failure not waived by an open finding: \
             gate6|/moved/|small|light|visual.diff_share:viewport \
             (share 0.083334 above the bound 0.001)\n"
        ),
        "{stderr}"
    );
    // The differing pixels: 3200 in each unit; a bound of 3199 fails each.
    let (dir, _) = gated(
        "gate-over-px",
        "    visual.diff_px@1: {gate: 7, max: 3199}\n",
    );
    let (code, file, stderr) = gate_run(&dir, "moved");
    assert_eq!(code, Some(1), "{stderr}");
    assert_eq!(failure_keys(&file).len(), 4);
    assert!(
        stderr.contains(
            "gate7|/moved/|small|light|visual.diff_px:page (3200 px above the bound 3199)\n"
        ),
        "{stderr}"
    );
}

#[test]
fn a_run_within_the_visual_bound_passes_the_gate() {
    let (dir, _) = gated(
        "gate-within",
        "    visual.diff_share@1: {gate: 6, max: 0.1}\n    visual.diff_px@1: {gate: 7, max: 3200}\n",
    );
    let (code, file, stderr) = gate_run(&dir, "moved");
    assert_eq!(code, Some(0), "{stderr}");
    assert!(
        failure_keys(&file).is_empty(),
        "units over their tolerance but within the bound are values, no gate failure"
    );
    assert!(!stderr.contains("hard gate failure"), "{stderr}");
}

#[test]
fn a_unit_without_a_baseline_is_no_pass_of_the_visual_gate() {
    let (dir, _) = gated(
        "gate-missing",
        "    visual.diff_share@1: {gate: 6, max: 1}\n",
    );
    let (code, file, stderr) = gate_run(&dir, "clock");
    assert_eq!(
        code,
        Some(0),
        "every unit compared and within the bound: {stderr}"
    );
    assert!(failure_keys(&file).is_empty());

    std::fs::remove_file(dir.join("baseline/clock.small.dark.page.png")).unwrap();
    let (code, file, stderr) = gate_run(&dir, "clock");
    assert_eq!(code, Some(1), "{stderr}");
    assert_eq!(
        failure_keys(&file),
        ["gate6|/clock/|small|dark|visual.diff_share:page:not-run"]
    );
    assert!(
        stderr.contains(
            "ux-browser: hard gate failure not waived by an open finding: \
             gate6|/clock/|small|dark|visual.diff_share:page:not-run \
             (no baseline baseline/clock.small.dark.page.png)\n"
        ),
        "{stderr}"
    );
    assert!(
        stderr
            .contains("ux-browser: incomplete: /clock/ visual.diff_share@1: coverage_below_plan\n"),
        "{stderr}"
    );
}
