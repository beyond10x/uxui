//! The browser lane of the declared campaign: `ux-search campaign run` in Chromium on a page of
//! this repository's own, `fixtures/campaign/live/site/list.html`, served by the browser
//! adapter's static server.
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
//! Budget: five campaigns, so five browser starts, for all tests of the lane: the two campaigns
//! that several tests read run once per test process (`cargo test` runs the tests of this binary
//! in one process), and three tests run one of their own. The largest is 4 variants in 4
//! environments; the lane ends within seconds, and no test of it comes near the 60 s a test has.

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
/// own, naming its outputs by paths from that directory.
fn run(name: &str, declaration: &str) -> Ran {
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
        .args(["--jobs", "2"])
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
        "     800x600 light low_contrast: main > p.pale \"Pale note\": 2.32 under 4.5, #aaaaaa on #ffffff\n",
        "     800x600 light small_targets: p > a.tiny \"i\": 10 x 10 px\n",
        "     800x600 light overflow: div.frame > button.wide \"Export all orders\": ends at 166 px, the box it shows in at 116\n",
        "     800x600 light always_visible_misses: the end (`p.end`): not in view at the top of the page\n",
        "     640x480 dark always_visible_misses: the end (`p.end`): not in view at the top of the page\n",
        "   1 tight.set\n",
        "   2 tight.unset\n",
        "     800x600 light overflow: div.frame > button.wide \"Export all orders\": ends at 158 px, the box it shows in at 108\n",
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
