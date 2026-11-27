//! The browser lane of the declared campaign: `ux-search campaign run` in Chromium on a page of
//! this repository's own, `fixtures/campaign/live/site/list.html` and, for the terms of a list
//! page, `site/rows.html`, served by the browser adapter's static server.
//!
//! Every test here is ignored in a plain `cargo test`, and so in the gate, which has no browser
//! (`tools/gate.sh`). The lane is one command, from the repository root, with a Chromium:
//!
//! ```console
//! UXUI_CHROME=<chrome> cargo test --locked --manifest-path engine/search/Cargo.toml -- --ignored campaign_live
//! ```
//!
//! Without a browser the lane fails; it does not skip. The page sets every size a test reads in
//! px, so no number depends on a font of the host.
//!
//! Budget: twelve campaigns, so twelve browser starts, for all tests of the lane: the three campaigns
//! that several tests read run once per test process (`cargo test` runs the tests of this binary
//! in one process), and nine tests run one of their own. The largest is 4 variants in 4
//! environments; the lane ended in 7 s on a loaded host, and no test of it comes near the 60 s
//! a test has.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use ux_search::landing::patch::{self, Value};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/campaign/live")
        .join(name)
}

fn list_text() -> String {
    let path = fixture("list.campaign.yaml");
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// The fixture site on a loopback port, started once.
fn origin() -> &'static str {
    static SERVER: OnceLock<String> = OnceLock::new();
    SERVER.get_or_init(|| {
        ux_browser::serve::start(&fixture("site"), "/")
            .unwrap_or_else(|error| panic!("{error}"))
            .origin
    })
}

/// One `ux-search campaign run`, and what it left.
struct Ran {
    status: Option<i32>,
    stdout: String,
    stderr: String,
    /// The declaration file the run read.
    declaration: PathBuf,
    /// The working directory of the run: empty before it.
    cwd: PathBuf,
    /// The run's `TMPDIR`: empty before it.
    tmp: PathBuf,
}

impl Ran {
    fn measurements(&self) -> Value {
        let path = self.cwd.join("measurements.json");
        let text = std::fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!("{}: {error}\n{}", path.display(), self.stderr);
        });
        patch::parse(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
    }

    /// One number of one render: the variant, the environment and the column by name.
    fn number(&self, variant: &str, environment: &str, column: &str) -> i64 {
        let measurements = self.measurements();
        let at = |key: &str, name: &str| {
            measurements
                .get(key)
                .map(Value::list)
                .unwrap_or_default()
                .iter()
                .position(|item| item.str() == Some(name))
                .unwrap_or_else(|| panic!("the measurements have no `{name}` in `{key}`"))
        };
        let (environment, column) = (at("environments", environment), at("columns", column));
        let renders = measurements
            .get("variants")
            .map(Value::list)
            .unwrap_or_default()
            .iter()
            .find(|item| item.get_str("id") == Some(variant))
            .and_then(|item| item.get("renders"))
            .unwrap_or_else(|| panic!("the measurements have no variant `{variant}`"));
        match renders.list()[environment].list()[column] {
            Value::Int(number) => number,
            ref other => panic!("{} is no whole number", patch::compact(other)),
        }
    }

    fn succeeded(&self) -> &Ran {
        assert_eq!(self.status, Some(0), "{}", self.stderr);
        self
    }
}

fn names(directory: &Path) -> BTreeSet<String> {
    std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("{}: {error}", directory.display()))
        .map(|entry| {
            entry
                .expect("a directory entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

/// Runs the campaign of a declaration in an empty working directory, with a `TMPDIR` of its
/// own, naming its outputs by paths from that directory, two pages at once.
fn run(name: &str, declaration: &str) -> Ran {
    run_with(name, declaration, &["--jobs", "2"])
}

/// [`run`] with other arguments after the outputs: `--jobs` and a restriction.
fn run_with(name: &str, declaration: &str, args: &[&str]) -> Ran {
    let directory = crate::state_dir(&format!("campaign-live-{name}"));
    let _ = std::fs::remove_dir_all(&directory);
    let (cwd, tmp) = (directory.join("cwd"), directory.join("tmp"));
    for made in [&cwd, &tmp] {
        std::fs::create_dir_all(made).expect("a scratch directory");
    }
    let file = directory.join("list.campaign.yaml");
    std::fs::write(&file, declaration).expect("a declaration file");
    let output = Command::new(env!("CARGO_BIN_EXE_ux-search"))
        .current_dir(&cwd)
        .env("TMPDIR", &tmp)
        .args(["campaign", "run", "--declaration"])
        .arg(&file)
        .args(["--url", origin()])
        .args(["--out", "measurements.json", "--shots", "shots"])
        .args(args)
        .output()
        .expect("ux-search runs");
    Ran {
        status: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        declaration: file,
        cwd,
        tmp,
    }
}

/// The campaign as the fixture declares it: no pointer named, the dark mode by a query that
/// expects nothing.
fn at_rest() -> &'static Ran {
    static RAN: OnceLock<Ran> = OnceLock::new();
    RAN.get_or_init(|| run("at-rest", &list_text()))
}

const DARK: &str = "    - {name: dark, query: theme=dark}\n";

fn changed(changes: &[(&str, &str)]) -> String {
    let mut text = list_text();
    for (from, to) in changes {
        assert!(text.contains(from), "the fixture has no `{from}`");
        text = text.replacen(from, to, 1);
    }
    text
}

/// The same page with the pointer declared in the rail's corner, one viewport, and a dark mode
/// that says what the page shows when it took the query.
fn hovered() -> &'static Ran {
    static RAN: OnceLock<Ran> = OnceLock::new();
    RAN.get_or_init(|| {
        run(
            "hovered",
            &changed(&[
                (
                    "  ready: 'main[data-ready]'\n",
                    "  ready: 'main[data-ready]'\n  pointer: [1, 1]\n",
                ),
                ("viewports: [800x600, 640x480]", "viewports: [800x600]"),
                (
                    DARK,
                    "    - {name: dark, query: theme=dark, expect: {attribute: {name: data-theme, value: dark}, selector: 'body.is-dark'}}\n",
                ),
            ]),
        )
    })
}

// The pointer.

/// The rail of the page opens when the pointer enters it and lies in the corner a headless
/// pointer starts in. A campaign measures the page at rest: the rail closed, 40 px high, so the
/// primary action 16 px under it, and the rail's hint not shown.
#[test]
#[ignore = "starts a browser: the lane is `cargo test -- --ignored campaign_live`"]
fn the_pointer_rests_in_the_far_corner_and_the_page_is_measured_at_rest() {
    let ran = at_rest().succeeded();
    for environment in [
        "800x600 light",
        "800x600 dark",
        "640x480 light",
        "640x480 dark",
    ] {
        assert_eq!(
            ran.number("stock.unset", environment, "primary_px"),
            56,
            "{environment}: the rail is open (216) when the pointer rests on it"
        );
        assert_eq!(
            ran.number("tight.unset", environment, "primary_px"),
            48,
            "{environment}"
        );
        assert_eq!(
            ran.number("stock.unset", environment, "text_runs"),
            9,
            "{environment}: the rail's hint is a tenth run, shown while the rail is hovered"
        );
    }
    assert_eq!(
        patch::compact(ran.measurements().get("pointer").unwrap_or(&Value::Null)),
        "[[799,599],[799,599],[639,479],[639,479]]",
        "the measurements say where the pointer was, per environment"
    );
}

/// A declared pointer is where the pointer is: in the rail's corner the page is measured with
/// the rail open.
#[test]
#[ignore = "starts a browser: the lane is `cargo test -- --ignored campaign_live`"]
fn a_declared_pointer_is_where_the_page_is_measured_from() {
    let ran = hovered().succeeded();
    for environment in ["800x600 light", "800x600 dark"] {
        assert_eq!(
            ran.number("stock.unset", environment, "primary_px"),
            216,
            "{environment}"
        );
        assert_eq!(
            ran.number("stock.unset", environment, "text_runs"),
            10,
            "{environment}"
        );
    }
    assert_eq!(
        patch::compact(ran.measurements().get("pointer").unwrap_or(&Value::Null)),
        "[[1,1],[1,1]]"
    );
}

// A mode's expectation.

/// A query the page does not know leaves it light. With an expectation the run reads the page
/// back and refuses, naming the mode, the environment and what it read.
#[test]
#[ignore = "starts a browser: the lane is `cargo test -- --ignored campaign_live`"]
fn a_mode_whose_expected_attribute_is_not_on_the_page_is_refused() {
    let ran = run(
        "mode-attribute",
        &changed(&[(
            DARK,
            "    - {name: dark, query: theem=dark, expect: {attribute: {name: data-theme, value: dark}}}\n",
        )]),
    );
    assert_eq!(ran.status, Some(1), "{}", ran.stderr);
    assert!(ran.stdout.is_empty(), "{}", ran.stdout);
    for part in [
        format!(
            "{}: `environments.modes[1].expect.attribute`",
            ran.declaration.display()
        ),
        "the mode `dark`".to_owned(),
        "`data-theme`".to_owned(),
        "`dark`".to_owned(),
        "unset".to_owned(),
        "theem=dark".to_owned(),
        "(800x600 dark)".to_owned(),
    ] {
        assert!(
            ran.stderr.contains(&part),
            "`{}` lacks `{part}`",
            ran.stderr
        );
    }
    assert_eq!(
        names(&ran.cwd),
        BTreeSet::new(),
        "a refused run writes nothing"
    );
}

#[test]
#[ignore = "starts a browser: the lane is `cargo test -- --ignored campaign_live`"]
fn a_mode_whose_expected_selector_matches_nothing_is_refused() {
    let ran = run(
        "mode-selector",
        &changed(&[(
            DARK,
            "    - {name: dark, query: theme=dark, expect: {attribute: {name: data-theme, value: dark}, selector: 'body.dark'}}\n",
        )]),
    );
    assert_eq!(ran.status, Some(1), "{}", ran.stderr);
    for part in [
        format!(
            "{}: `environments.modes[1].expect.selector`",
            ran.declaration.display()
        ),
        "the mode `dark`".to_owned(),
        "`body.dark` matches nothing".to_owned(),
        "(800x600 dark)".to_owned(),
    ] {
        assert!(
            ran.stderr.contains(&part),
            "`{}` lacks `{part}`",
            ran.stderr
        );
    }
}

/// A mode by query that expects nothing is measured, and the run says that nothing read it
/// back; a mode that expects something is not named.
#[test]
#[ignore = "starts a browser: the lane is `cargo test -- --ignored campaign_live`"]
fn a_mode_by_query_without_an_expectation_is_named_as_not_read_back() {
    let line = "ux-search: note: mode `dark`: set by the query `theme=dark` and not read back";
    let ran = at_rest().succeeded();
    assert_eq!(ran.stderr.matches(line).count(), 1, "{}", ran.stderr);
    let read_back = hovered().succeeded();
    assert!(
        !read_back.stderr.contains("not read back"),
        "{}",
        read_back.stderr
    );
    assert!(
        read_back
            .stderr
            .contains("the mode `dark` shows what it expects in 4 renders"),
        "{}",
        read_back.stderr
    );
}

/// A frontier variant is shot in every mode at the first viewport, and in the last environment.
#[test]
#[ignore = "starts a browser: the lane is `cargo test -- --ignored campaign_live`"]
fn a_frontier_variant_is_shot_in_every_mode() {
    let ran = at_rest().succeeded();
    let expected: BTreeSet<String> = ["tight.set", "tight.unset"]
        .iter()
        .flat_map(|id| {
            ["800x600-light", "800x600-dark", "640x480-dark"]
                .iter()
                .map(move |environment| format!("{id}.{environment}.png"))
        })
        .collect();
    assert_eq!(names(&ran.cwd.join("shots")), expected);
}

// What a run writes.

/// A run writes the files it names and nothing else: no directory for the browser's profile
/// in the working directory, and nothing left in the temp directory the profile was made in.
#[test]
#[ignore = "starts a browser: the lane is `cargo test -- --ignored campaign_live`"]
fn a_run_leaves_nothing_but_the_files_it_names() {
    let ran = at_rest().succeeded();
    assert_eq!(
        names(&ran.cwd),
        ["measurements.json", "shots"]
            .iter()
            .map(|&name| name.to_owned())
            .collect::<BTreeSet<_>>()
    );
    assert_eq!(names(&ran.tmp), BTreeSet::new());
}

// The terms.

/// The icon of the page is a character of a colour with alpha 0. It is a text run, and no
/// contrast is judged for it: the one text under the threshold is the pale note.
#[test]
#[ignore = "starts a browser: the lane is `cargo test -- --ignored campaign_live`"]
fn a_text_of_a_transparent_colour_is_not_judged_for_contrast() {
    let ran = at_rest().succeeded();
    for environment in ["800x600 light", "800x600 dark"] {
        assert_eq!(
            ran.number("stock.unset", environment, "low_contrast"),
            1,
            "{environment}"
        );
    }
}

/// The counts the report ranks by come with the first elements they count, and `campaign report
/// --notes` prints them for the page as shipped and for the frontier.
#[test]
#[ignore = "starts a browser: the lane is `cargo test -- --ignored campaign_live`"]
fn the_report_names_the_elements_behind_the_counts() {
    let ran = at_rest().succeeded();
    for (column, count) in [
        ("low_contrast", 1),
        ("small_targets", 1),
        ("overflow", 2),
        ("always_visible_misses", 1),
    ] {
        assert_eq!(
            ran.number("stock.unset", "800x600 light", column),
            count,
            "{column}"
        );
    }
    let report = |notes: bool| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ux-search"));
        command.current_dir(&ran.cwd).args([
            "campaign",
            "report",
            "--measurements",
            "measurements.json",
        ]);
        if notes {
            command.arg("--notes");
        }
        let output = command.output().expect("ux-search runs");
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        String::from_utf8_lossy(&output.stdout).into_owned()
    };
    let (plain, noted) = (report(false), report(true));
    assert_eq!(plain, ran.stdout, "the report of the run");
    assert!(noted.starts_with(&plain), "{noted}");
    let notes = &noted[plain.len()..];
    for line in [
        "notes: the first elements behind each count, at most 3 per term and environment, of the page as shipped and of the frontier\n",
        "shipped stock.unset\n",
        "     low_contrast: main > p.pale \"Pale note\": 2.32 under 4.5, #aaaaaa on #ffffff\n",
        "     small_targets: p > a.tiny \"i\": 10 x 10 px\n",
        "     all 4 environments overflow: div.frame > button.wide \"Export all orders\": ends at 166 px, the box it shows in at 116\n",
        "     always_visible_misses: the end (`p.end`): not in view at the top of the page\n",
        "   1 tight.set\n",
        "   2 tight.unset\n",
        "     all 4 environments overflow: div.frame > button.wide \"Export all orders\": ends at 158 px, the box it shows in at 108\n",
    ] {
        assert!(notes.contains(line), "the notes lack `{line}`:\n{notes}");
    }
    assert!(!notes.contains("span.icon"), "{notes}");
}

/// A property that is in force and that no rule reads changes nothing. The run says so per
/// axis, and the measurements hold the number of values that change the page.
#[test]
#[ignore = "starts a browser: the lane is `cargo test -- --ignored campaign_live`"]
fn an_axis_whose_values_change_nothing_on_the_page_is_named() {
    let ran = at_rest().succeeded();
    for line in [
        "ux-search: axis spacing: 1 of the 1 values beside the shipped one change a box or a computed style on this page\n",
        "ux-search: note: axis unread: no value changes a box or a computed style on this page\n",
    ] {
        assert!(ran.stderr.contains(line), "`{}` lacks `{line}`", ran.stderr);
    }
    assert_eq!(
        patch::compact(ran.measurements().get("axes").unwrap_or(&Value::Null)),
        "[{\"name\":\"spacing\",\"changing\":1},{\"name\":\"unread\",\"changing\":0}]"
    );
}

/// A value that only colours a generated box (`h1::before`, the dot before the heading) moves no
/// element's box and no element's colour, and no measured number. It changes the page all the
/// same, and the axis is not named as one that changes nothing.
#[test]
#[ignore = "starts a browser: the lane is `cargo test -- --ignored campaign_live`"]
fn a_value_that_only_colours_a_generated_box_changes_the_page() {
    let ran = run(
        "generated-box",
        &changed(&[
            ("viewports: [800x600, 640x480]", "viewports: [800x600]"),
            (DARK, ""),
            ("  - name: unread\n", "  - name: dot\n"),
            ("{--nothing-reads-this: 1}", "{--dot: '#00aa00'}"),
        ]),
    );
    ran.succeeded();
    assert!(
        ran.stderr.contains(
            "ux-search: axis dot: 1 of the 1 values beside the shipped one change a box or a computed style on this page\n"
        ),
        "{}",
        ran.stderr
    );
    assert!(!ran.stderr.contains("ux-search: note:"), "{}", ran.stderr);
    for column in ["primary_px", "text_runs", "low_contrast", "overflow"] {
        assert_eq!(
            ran.number("stock.set", "800x600 light", column),
            ran.number("stock.unset", "800x600 light", column),
            "{column}"
        );
    }
}

// What the second run on a page outside this repository left open
// (`story:campaign-run-follow-ups`).

/// `ux-search campaign report` over the measurements of a run, with more arguments.
fn report_of(ran: &Ran, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ux-search"))
        .current_dir(&ran.cwd)
        .args(["campaign", "report", "--measurements", "measurements.json"])
        .args(args)
        .output()
        .expect("ux-search runs")
}

/// A mode the page does not take is refused before the first variant is rendered. One page at
/// a time, the light mode first: a value whose rule matches nothing in the light mode would be
/// refused in the light renders, before the dark mode's first render, if the modes were not
/// read back first.
#[test]
#[ignore = "starts a browser: the lane is `cargo test -- --ignored campaign_live`"]
fn a_mode_the_page_does_not_show_is_refused_before_the_first_variant_is_rendered() {
    let ran = run_with(
        "mode-early",
        &changed(&[
            (
                DARK,
                "    - {name: dark, query: theem=dark, expect: {attribute: {name: data-theme, value: dark}}}\n",
            ),
            (
                "{selector: ':root', set: {--space: 8px}}",
                "{selector: ':root', set: {--space: 8px}}\n          - {selector: 'p.missing', mode: light, set: {--space: 8px}}",
            ),
        ]),
        &["--jobs", "1"],
    );
    assert_eq!(ran.status, Some(1), "{}", ran.stderr);
    for part in [
        "`environments.modes[1].expect.attribute`",
        "(800x600 dark)",
        "no variant was rendered",
    ] {
        assert!(ran.stderr.contains(part), "`{}` lacks `{part}`", ran.stderr);
    }
    assert!(!ran.stderr.contains("p.missing"), "{}", ran.stderr);
    assert_eq!(names(&ran.cwd), BTreeSet::new());
}

/// An element that is the same in every environment and frontier variant is named once, with
/// the number of variants and environments it stands for; one that is the same in every
/// environment of one variant is named once for that variant.
#[test]
#[ignore = "starts a browser: the lane is `cargo test -- --ignored campaign_live`"]
fn the_notes_name_an_element_once_when_it_is_the_same_everywhere() {
    let ran = at_rest().succeeded();
    let output = report_of(ran, &["--notes"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let noted = String::from_utf8_lossy(&output.stdout).into_owned();
    let notes = &noted[ran.stdout.len()..];
    let tiny = "     small_targets: p > a.tiny \"i\": 10 x 10 px\n";
    assert_eq!(notes.matches("a.tiny").count(), 1, "{notes}");
    let (common, rest) = notes
        .split_once("shipped stock.unset\n")
        .unwrap_or_else(|| panic!("{notes}"));
    assert!(
        common.contains(
            "\nthe same in every environment of every variant below, 3 variants in 4 environments, 12 renders:\n"
        ),
        "{notes}"
    );
    assert!(common.contains(tiny), "{notes}");
    assert!(!rest.contains("small_targets"), "{notes}");
    assert!(
        notes.contains(
            "shipped stock.unset\n     all 4 environments overflow: div.frame > button.wide \"Export all orders\": ends at 166 px, the box it shows in at 116\n"
        ),
        "{notes}"
    );
    assert!(notes.lines().count() < 20, "{notes}");
}

/// A mode and a viewport that measure the same numbers as another one for every variant are
/// named, one line each: on the fixture page neither the dark mode nor the smaller viewport
/// moves a number.
#[test]
#[ignore = "starts a browser: the lane is `cargo test -- --ignored campaign_live`"]
fn an_environment_that_changes_no_number_is_named() {
    let ran = at_rest().succeeded();
    for line in [
        "ux-search: note: mode dark: every variant measures the same numbers as in the mode light, at every viewport\n",
        "ux-search: note: viewport 640x480: every variant measures the same numbers as at 800x600, in every mode\n",
    ] {
        assert_eq!(ran.stderr.matches(line).count(), 1, "{}", ran.stderr);
    }
}

/// A run restricted to one variant and one environment measures that render alone, shoots it
/// with `--shots`, and writes a file the report refuses to rank as a campaign.
#[test]
#[ignore = "starts a browser: the lane is `cargo test -- --ignored campaign_live`"]
fn a_run_restricted_to_one_variant_and_one_environment_is_no_campaign() {
    let ran = run_with(
        "restricted",
        &list_text(),
        &[
            "--jobs",
            "2",
            "--variant",
            "tight.unset",
            "--environment",
            "640x480 dark",
        ],
    );
    ran.succeeded();
    let lines: Vec<&str> = ran
        .stdout
        .lines()
        .filter(|line| line.starts_with("tight.unset "))
        .collect();
    assert_eq!(lines.len(), 1, "{}", ran.stdout);
    assert!(
        lines[0].starts_with(
            "tight.unset 640x480 dark: primary_in_view=1 h_scroll=0 unnamed=0 primary_px=48 "
        ),
        "{}",
        lines[0]
    );
    assert_eq!(
        names(&ran.cwd.join("shots")),
        BTreeSet::from(["tight.unset.640x480-dark.png".to_owned()])
    );
    let measurements = ran.measurements();
    assert_eq!(
        measurements.get_str("format"),
        Some("ux-search-campaign-renders/1")
    );
    let output = report_of(&ran, &[]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("a restricted run"),
        "{output:?}"
    );
}

/// Two runs of one declaration write screenshots of the same bytes. The page focuses a field
/// when it loads, and its caret blinks: a run shoots the page with the caret hidden.
#[test]
#[ignore = "starts a browser: the lane is `cargo test -- --ignored campaign_live`"]
fn two_runs_write_screenshots_of_the_same_bytes() {
    let first = at_rest().succeeded();
    let again = run("at-rest-again", &list_text());
    again.succeeded();
    let shots = names(&first.cwd.join("shots"));
    assert_eq!(shots, names(&again.cwd.join("shots")));
    assert!(!shots.is_empty());
    for shot in &shots {
        let bytes = |ran: &Ran| std::fs::read(ran.cwd.join("shots").join(shot)).expect("a shot");
        assert!(bytes(first) == bytes(&again), "{shot} differs");
    }
}

/// A shot does not depend on when it is taken: the caret of the focused field is on for the
/// first half second and off for the next, and a run that waits 700 ms after the page is ready
/// shoots it in the other phase than one that does not wait. The shots of both are the same
/// bytes.
#[test]
#[ignore = "starts a browser: the lane is `cargo test -- --ignored campaign_live`"]
fn a_shot_taken_later_is_the_same_bytes() {
    let first = at_rest().succeeded();
    let later = run(
        "at-rest-later",
        &changed(&[(
            "  ready: 'main[data-ready]'\n",
            "  ready: 'main[data-ready]'\n  settle_ms: 700\n",
        )]),
    );
    later.succeeded();
    let shots = names(&first.cwd.join("shots"));
    assert_eq!(shots, names(&later.cwd.join("shots")));
    for shot in &shots {
        let bytes = |ran: &Ran| std::fs::read(ran.cwd.join("shots").join(shot)).expect("a shot");
        assert!(bytes(first) == bytes(&later), "{shot} differs");
    }
}

/// A page that changes the mode's attribute after it was ready is refused, and the refusal says
/// so: the run set the attribute before the page's scripts ran and again when it was ready.
#[test]
#[ignore = "starts a browser: the lane is `cargo test -- --ignored campaign_live`"]
fn a_page_that_changes_the_modes_attribute_after_it_was_ready_is_refused() {
    let ran = run(
        "attribute-flip",
        &changed(&[(
            DARK,
            "    - {name: dark, attribute: {name: data-theme, value: dark}, query: flip}\n",
        )]),
    );
    assert_eq!(ran.status, Some(1), "{}", ran.stderr);
    for part in [
        "`environments.modes[1].attribute`",
        "the page changed the attribute after it was ready",
        "`light`",
    ] {
        assert!(ran.stderr.contains(part), "`{}` lacks `{part}`", ran.stderr);
    }
    assert!(
        !ran.stderr.contains("sets the attribute itself"),
        "{}",
        ran.stderr
    );
}

// The terms of a list page (`story:campaign-terms-for-list-pages`), on `site/rows.html`.

/// The campaign of `rows.campaign.yaml`, run once.
fn rows() -> &'static Ran {
    static RAN: OnceLock<Ran> = OnceLock::new();
    RAN.get_or_init(|| {
        let path = fixture("rows.campaign.yaml");
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        run("rows", &text)
    })
}

/// The ranking of a report: each variant's id and its terms, best first.
fn ranking(report: &str) -> Vec<(String, String)> {
    report
        .split_once("\nranking ")
        .map(|(_, rest)| rest)
        .unwrap_or_else(|| panic!("no ranking: {report}"))
        .lines()
        .skip(1)
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            let (_, id) = (words.next()?, words.next()?);
            Some((id.to_owned(), words.collect::<Vec<_>>().join(" ")))
        })
        .collect()
}

/// Rows in view: the tighter rows put more of them in the viewport, and the variant with them
/// has the higher count and ranks before the looser one, the term where more is better.
#[test]
#[ignore = "starts a browser: the lane is `cargo test -- --ignored campaign_live`"]
fn the_tighter_rows_have_more_rows_in_view_and_rank_before_the_looser_ones() {
    let ran = rows().succeeded();
    for control in ["tall", "short"] {
        let (tight, stock) = (format!("tight.{control}"), format!("stock.{control}"));
        assert_eq!(ran.number(&tight, "800x600 light", "rows_in_view"), 16);
        assert_eq!(ran.number(&stock, "800x600 light", "rows_in_view"), 13);
        let ranking = ranking(&ran.stdout);
        let place = |id: &str| {
            ranking
                .iter()
                .position(|(ranked, _)| ranked == id)
                .unwrap_or_else(|| panic!("{id} is not ranked: {}", ran.stdout))
        };
        assert!(place(&tight) < place(&stock), "{}", ran.stdout);
        assert!(
            ranking[place(&tight)].1.contains("rows_in_view=16 "),
            "{}",
            ran.stdout
        );
        assert!(
            ranking[place(&stock)].1.contains("rows_in_view=13 "),
            "{}",
            ran.stdout
        );
    }
}

/// The create action is centred in the header: a smaller action moves its top edge down, so
/// `primary_px` ranks it worse, and its bottom edge up. `primary_bottom_px` does not rank the
/// variant that only shrinks the action worse than the page as shipped.
#[test]
#[ignore = "starts a browser: the lane is `cargo test -- --ignored campaign_live`"]
fn a_smaller_primary_action_does_not_rank_worse_on_its_bottom_edge() {
    let ran = rows().succeeded();
    let environment = "800x600 light";
    assert_eq!(ran.number("stock.tall", environment, "primary_px"), 12);
    assert_eq!(ran.number("stock.short", environment, "primary_px"), 16);
    assert_eq!(
        ran.number("stock.tall", environment, "primary_bottom_px"),
        52
    );
    assert_eq!(
        ran.number("stock.short", environment, "primary_bottom_px"),
        48
    );
    let ranking = ranking(&ran.stdout);
    let terms = |id: &str| {
        ranking
            .iter()
            .find(|(ranked, _)| ranked == id)
            .map(|(_, terms)| terms.clone())
            .unwrap_or_else(|| panic!("{id} is not ranked"))
    };
    assert!(
        terms("stock.tall").contains(" primary_bottom_px=52 "),
        "{}",
        ran.stdout
    );
    assert!(
        terms("stock.short").contains(" primary_bottom_px=48 "),
        "{}",
        ran.stdout
    );
}

/// The navigation is taller than the viewport and lies in a scroll container that clips it on
/// purpose. Named with its container it is no miss; the same element without one is.
#[test]
#[ignore = "starts a browser: the lane is `cargo test -- --ignored campaign_live`"]
fn an_element_clipped_only_by_its_named_scroll_container_is_no_miss() {
    let ran = rows().succeeded();
    for variant in ["tight.tall", "tight.short", "stock.tall", "stock.short"] {
        assert_eq!(
            ran.number(variant, "800x600 light", "always_visible_misses"),
            1,
            "{variant}"
        );
    }
    let output = report_of(ran, &["--notes"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let notes = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        notes.contains(
            "always_visible_misses: navigation unclipped (`nav.side-nav`): not in view at the top of the page and at its end\n"
        ),
        "{notes}"
    );
    assert!(!notes.contains(": navigation (`"), "{notes}");
}

// The live view (`story:campaign-live-view`).

/// The lines of one JSON-lines file the run wrote under its events directory.
fn jsonl(ran: &Ran, name: &str) -> Vec<Value> {
    let path = ran.cwd.join("events").join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}\n{}", path.display(), ran.stderr));
    assert!(text.ends_with('\n'), "{text}");
    text.lines()
        .map(|line| patch::parse(line).unwrap_or_else(|error| panic!("{line}: {error}")))
        .collect()
}

/// With `--events` the run writes one candidates line per variant and one phase line per
/// phase, with a screenshot of each variant as it is measured; the measurements are the bytes
/// of a run without it, and the last ranking line names the report's top three.
#[test]
#[ignore = "starts a browser: the lane is `cargo test -- --ignored campaign_live`"]
fn a_run_with_events_writes_the_live_view_and_the_same_measurements() {
    let without = at_rest().succeeded();
    let with = run_with(
        "events",
        &list_text(),
        &["--jobs", "2", "--events", "events"],
    );
    with.succeeded();
    let bytes = |ran: &Ran| std::fs::read(ran.cwd.join("measurements.json")).expect("a file");
    assert!(
        bytes(without) == bytes(&with),
        "the measurements differ with --events"
    );
    assert_eq!(with.stdout, without.stdout, "the report of the run");

    let candidates = jsonl(&with, "hill-climb.jsonl");
    let ids = ["tight.unset", "tight.set", "stock.unset", "stock.set"];
    assert_eq!(candidates.len(), ids.len());
    for (at, (line, id)) in candidates.iter().zip(ids).enumerate() {
        assert_eq!(line.get("iteration"), Some(&Value::Int(at as i64 + 1)));
        let change = line.get_str("change").unwrap_or_default();
        assert!(change.starts_with(&format!("{id}: spacing=")), "{change}");
        let kind = if id == "stock.unset" {
            "baseline"
        } else {
            "candidate"
        };
        assert_eq!(line.get_str("kind"), Some(kind), "{change}");
        let screenshots = line.get("screenshots").expect("screenshots");
        for environment in ["800x600 light", "800x600 dark", "640x480 dark"] {
            let path = screenshots
                .get_str(environment)
                .unwrap_or_else(|| panic!("{id} has no screenshot in {environment}"));
            let file = with.cwd.join("events").join(path);
            assert!(file.is_file(), "{}", file.display());
        }
    }
    let primary = |line: &Value| match line.get("metrics").and_then(|m| m.get("primary_px")) {
        Some(Value::Int(n)) => *n,
        other => panic!("{other:?}"),
    };
    // A term is the worst of the environments.
    for (line, id) in candidates.iter().zip(ids) {
        let worst = [
            "800x600 light",
            "800x600 dark",
            "640x480 light",
            "640x480 dark",
        ]
        .iter()
        .map(|environment| with.number(id, environment, "primary_px"))
        .max();
        assert_eq!(Some(primary(line)), worst, "{id}");
    }
    assert!(
        primary(&candidates[0]) < primary(&candidates[2]),
        "the tighter padding moves the action up"
    );

    // The measuring and ranking lines interleave as the pages go; the order of each is fixed.
    let phases = jsonl(&with, "hill-climb-events.jsonl");
    let named: Vec<&str> = phases
        .iter()
        .map(|line| line.get_str("phase").unwrap_or_default())
        .collect();
    assert_eq!(named.len(), 10, "{named:?}");
    assert_eq!(named.first(), Some(&"start"));
    assert_eq!(named.last(), Some(&"end"));
    for phase in ["measuring", "ranking"] {
        assert_eq!(
            named.iter().filter(|&&name| name == phase).count(),
            4,
            "{named:?}"
        );
    }
    let top: Vec<String> = ranking(&with.stdout)
        .into_iter()
        .take(3)
        .map(|(id, _)| id)
        .collect();
    let last = phases
        .iter()
        .rev()
        .find(|line| line.get_str("phase") == Some("ranking"))
        .and_then(|line| line.get_str("text"))
        .expect("a ranking line");
    assert_eq!(last, format!("top 3: {}", top.join(", ")));
}
