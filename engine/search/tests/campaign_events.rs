//! The live view of a declared campaign (`story:campaign-live-view`): with `--events <dir>`,
//! `ux-search campaign run` appends one line to the candidates file per finished variant and one
//! line to the phase file per phase, so a live page that polls two JSON-lines files shows the
//! climb while it runs.
//!
//! No test here starts a browser. The recorded studio campaign (`fixtures/campaign/studio/`) is
//! fed through the run's own path, `events::Progress`, which the run feeds from its pages: each
//! environment's page as shipped, then each render as it is measured. The browser lane
//! (`campaign_live`) runs the fixture page with `--events` and holds the measurements to a run
//! without it.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use ux_search::campaign::events::{self, Events, Progress};
use ux_search::campaign::record::Recorded;
use ux_search::campaign::report;
use ux_search::landing::patch::{self, Value};

const AT: &str = "2026-11-27T10:48:05+01:00";

fn clock() -> events::Clock {
    Box::new(|| AT.to_owned())
}

fn studio() -> Recorded {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/campaign/studio/measurements.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    Recorded::parse(&text).unwrap_or_else(|error| panic!("{error}"))
}

fn scratch(name: &str) -> PathBuf {
    let directory = crate::state_dir(&format!("campaign-events-{name}"));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("a scratch directory");
    directory
}

/// The lines of a JSON-lines file, each parsed; every line ends with a newline.
fn lines(path: &Path) -> Vec<Value> {
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    assert!(text.is_empty() || text.ends_with('\n'), "{text}");
    text.lines()
        .map(|line| patch::parse(line).unwrap_or_else(|error| panic!("{line}: {error}")))
        .collect()
}

fn candidates(directory: &Path) -> Vec<Value> {
    lines(&directory.join(events::CANDIDATES_FILE))
}

fn phases(directory: &Path) -> Vec<Value> {
    lines(&directory.join(events::PHASE_FILE))
}

/// The studio campaign replayed into a directory of the test's own.
fn replayed(name: &str) -> (Recorded, PathBuf) {
    let recorded = studio();
    let directory = scratch(name);
    events::replay(&recorded, &directory, clock()).unwrap_or_else(|error| panic!("{error}"));
    (recorded, directory)
}

fn int(value: Option<&Value>) -> i64 {
    match value {
        Some(Value::Int(n)) => *n,
        other => panic!("{other:?} is no whole number"),
    }
}

fn number(value: Option<&Value>) -> f64 {
    match value {
        Some(Value::Int(n)) => *n as f64,
        Some(Value::Float(f)) => *f,
        other => panic!("{other:?} is no number"),
    }
}

fn flag(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Bool(flag)) => *flag,
        other => panic!("{other:?} is not true or false"),
    }
}

fn texts(value: Option<&Value>) -> Vec<String> {
    value
        .map(Value::list)
        .unwrap_or_default()
        .iter()
        .map(|text| text.str().expect("a text").to_owned())
        .collect()
}

fn keys(value: &Value) -> Vec<String> {
    match value {
        Value::Map(entries) => entries.iter().map(|(key, _)| key.clone()).collect(),
        other => panic!("{other:?} is no object"),
    }
}

/// The report's ranking: each variant's id and its terms as `name=value`, best first.
fn ranking(report: &str) -> Vec<(String, Vec<(String, String)>)> {
    report
        .split_once("\nranking ")
        .map(|(_, rest)| rest)
        .unwrap_or_else(|| panic!("no ranking: {report}"))
        .lines()
        .skip(1)
        .map(|line| {
            let mut words = line.split_whitespace();
            let (_, id) = (words.next().expect("a place"), words.next().expect("an id"));
            let terms = words
                .map(|word| {
                    let (name, value) = word.split_once('=').expect("name=value");
                    (name.to_owned(), value.to_owned())
                })
                .collect();
            (id.to_owned(), terms)
        })
        .collect()
}

// The candidates file.

/// One line per variant, in the order the run finishes them (the order of the declaration's
/// variants), each naming the variant and its axis values; the page as shipped is the baseline.
#[test]
fn a_replayed_campaign_writes_one_candidates_line_per_variant_in_run_order() {
    let (recorded, directory) = replayed("order");
    let declaration = &recorded.declaration;
    let lines = candidates(&directory);
    let variants = declaration.variants();
    assert_eq!(lines.len(), variants.len());
    let shipped = declaration.shipped();
    for (at, (line, variant)) in lines.iter().zip(&variants).enumerate() {
        assert_eq!(int(line.get("iteration")), at as i64 + 1);
        assert_eq!(line.get_str("at"), Some(AT));
        let change = line.get_str("change").expect("a change");
        let values: Vec<String> = declaration
            .axes
            .iter()
            .zip(&variant.choice)
            .map(|(axis, &value)| format!("{}={}", axis.name, axis.values[value].id))
            .collect();
        assert_eq!(change, format!("{}: {}", variant.id, values.join(" ")));
        let kind = if *variant == shipped {
            "baseline"
        } else {
            "candidate"
        };
        assert_eq!(line.get_str("kind"), Some(kind), "{change}");
    }
}

/// `score_keys` are the report's terms in order, and `metrics` the variant's values of them as
/// the report prints them: `text_under_14` a share, `rows_in_view` a count.
#[test]
fn each_candidates_line_has_the_variants_terms_as_metrics() {
    let (recorded, directory) = replayed("metrics");
    let text = report::report(&recorded).unwrap_or_else(|error| panic!("{error}"));
    let ranked = ranking(&text);
    let names: Vec<String> = report::term_names(&recorded.declaration)
        .into_iter()
        .map(str::to_owned)
        .collect();
    for line in candidates(&directory) {
        assert_eq!(texts(line.get("score_keys")), names);
        let metrics = line.get("metrics").expect("metrics");
        assert_eq!(keys(metrics), names);
        let change = line.get_str("change").expect("a change");
        let id = change.split_once(": ").expect("id: values").0;
        let (_, terms) = ranked
            .iter()
            .find(|(ranked, _)| ranked == id)
            .unwrap_or_else(|| panic!("{id} is not ranked"));
        for (name, value) in terms {
            let printed: f64 = value.parse().expect("a number");
            assert!(
                (number(metrics.get(name)) - printed).abs() < 1e-9,
                "{id} {name}: {} against {value}",
                patch::compact(metrics)
            );
        }
    }
}

fn dominates(a: &[u64], b: &[u64]) -> bool {
    a.iter().zip(b).all(|(a, b)| a <= b) && a.iter().zip(b).any(|(a, b)| a < b)
}

/// `score` is the variant's place in the ranking of the variants finished so far (1 the best,
/// lower is better), and `accepted` whether it stands on the frontier of those variants: no
/// hard rule broken, and no variant finished so far beats it on one term without losing on
/// another.
#[test]
fn the_score_is_the_place_so_far_and_accepted_is_the_frontier_so_far() {
    let (recorded, directory) = replayed("score");
    let declaration = &recorded.declaration;
    let scored: Vec<(Vec<u64>, &str)> = recorded
        .variants
        .iter()
        .map(|(id, renders)| (report::terms(declaration, renders), id.as_str()))
        .collect();
    let lines = candidates(&directory);
    let mut accepted = 0;
    for (at, line) in lines.iter().enumerate() {
        let own = &scored[at];
        let before = scored[..at].iter().filter(|other| *other < own).count();
        assert_eq!(int(line.get("score")), before as i64 + 1, "{}", own.1);
        let on = own.0[0] == 0
            && !scored[..at]
                .iter()
                .any(|other| other.0[0] == 0 && dominates(&other.0, &own.0));
        assert_eq!(flag(line.get("accepted")), on, "{}", own.1);
        accepted += usize::from(on);
    }
    assert!(accepted > 0 && accepted < lines.len(), "{accepted}");
    assert_eq!(int(lines[0].get("score")), 1);
}

/// `deltas` are each metric less the page as shipped's, `worse` the terms on which the variant
/// is worse than the page as shipped: a higher value, or for `rows_in_view` a lower one.
#[test]
fn the_deltas_and_the_worse_terms_are_against_the_page_as_shipped() {
    let (_, directory) = replayed("deltas");
    let lines = candidates(&directory);
    let baseline = lines
        .iter()
        .find(|line| line.get_str("kind") == Some("baseline"))
        .expect("a baseline line");
    let base = baseline.get("metrics").expect("metrics");
    let mut some_worse = false;
    for line in &lines {
        let metrics = line.get("metrics").expect("metrics");
        let deltas = line.get("deltas").expect("deltas");
        assert_eq!(keys(deltas), keys(metrics));
        let mut worse = Vec::new();
        for name in keys(metrics) {
            let (own, shipped) = (number(metrics.get(&name)), number(base.get(&name)));
            let delta = number(deltas.get(&name));
            assert!((delta - (own - shipped)).abs() < 1e-9, "{name}: {delta}");
            let is_worse = if name == "rows_in_view" {
                own < shipped
            } else {
                own > shipped
            };
            if is_worse {
                worse.push(name);
            }
        }
        some_worse |= !worse.is_empty();
        assert_eq!(texts(line.get("worse")), worse);
    }
    assert!(some_worse, "no variant of the studio is worse on a term");
    assert!(texts(baseline.get("worse")).is_empty());
}

/// A variant measured in some environments is not finished: its line is written when the last
/// environment measured it, and lines come in the order of the variants, whatever the order the
/// environments finish them in.
#[test]
fn a_variant_is_finished_when_every_environment_measured_it() {
    let recorded = studio();
    let directory = scratch("finished");
    let declaration = &recorded.declaration;
    let shipped = declaration.shipped();
    let at = declaration
        .variants()
        .iter()
        .position(|variant| *variant == shipped)
        .expect("a shipped variant");
    let environments = declaration.environments().len();
    let events =
        Events::create(&directory, declaration, clock()).unwrap_or_else(|error| panic!("{error}"));
    let mut progress = Progress::new(events, recorded.variants.len(), environments);
    progress.events.start().expect("start");
    let renders = |index: usize| recorded.variants[index].1.clone();
    for environment in 0..environments {
        progress
            .built(environment, renders(at)[environment])
            .expect("built");
    }
    // The first environment runs ahead through the first three variants.
    for index in 0..3 {
        progress.starting(index).expect("starting");
        progress
            .measured(0, index, renders(index)[0], None)
            .expect("measured");
    }
    assert!(candidates(&directory).is_empty());
    // The last environment measures the second variant before the first.
    for environment in 1..environments {
        progress
            .measured(environment, 1, renders(1)[environment], None)
            .expect("measured");
    }
    assert!(candidates(&directory).is_empty());
    for environment in 1..environments {
        progress
            .measured(environment, 0, renders(0)[environment], None)
            .expect("measured");
    }
    let written = candidates(&directory);
    assert_eq!(
        written
            .iter()
            .map(|line| int(line.get("iteration")))
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    let measuring: Vec<i64> = phases(&directory)
        .iter()
        .filter(|line| line.get_str("phase") == Some("measuring"))
        .map(|line| int(line.get("iteration")))
        .collect();
    assert_eq!(
        measuring,
        vec![1, 2, 3],
        "one line per variant, when it starts"
    );
}

/// With screenshots, a line names each by its environment, as a path under the events
/// directory.
#[test]
fn a_line_names_its_screenshots_by_environment() {
    let recorded = studio();
    let directory = scratch("screenshots");
    let declaration = &recorded.declaration;
    let mut events =
        Events::create(&directory, declaration, clock()).unwrap_or_else(|error| panic!("{error}"));
    events.start().expect("start");
    let shipped = declaration.shipped();
    let at = declaration
        .variants()
        .iter()
        .position(|variant| *variant == shipped)
        .expect("a shipped variant");
    events.baseline(recorded.variants[at].1.clone());
    events.measuring(0).expect("measuring");
    let shot = (
        "1366x768 light".to_owned(),
        "shots/space-tight.ink-stock.fill-stock.head-stock.1366x768-light.png".to_owned(),
    );
    events
        .finished(
            0,
            recorded.variants[0].1.clone(),
            std::slice::from_ref(&shot),
        )
        .expect("finished");
    let lines = candidates(&directory);
    let screenshots = lines[0].get("screenshots").expect("screenshots");
    assert_eq!(keys(screenshots), vec![shot.0.clone()]);
    assert_eq!(screenshots.get_str(&shot.0), Some(shot.1.as_str()));
}

// The phase file.

/// The phase file says when the run starts (the size of the space and the environments), when
/// each variant starts to be measured, the ranking after each variant, and the end.
#[test]
fn the_phase_file_starts_measures_ranks_and_ends() {
    let (recorded, directory) = replayed("phases");
    let lines = phases(&directory);
    let variants = recorded.variants.len() as i64;
    let first = lines.first().expect("a line");
    assert_eq!(first.get_str("phase"), Some("start"));
    assert_eq!(int(first.get("iteration")), 0);
    let start = first.get_str("text").expect("a text");
    for part in [
        "48 variants",
        "4 environments",
        "1366x768 light, 1366x768 dark, 1280x720 light, 1280x720 dark",
    ] {
        assert!(start.contains(part), "`{start}` lacks `{part}`");
    }
    let last = lines.last().expect("a line");
    assert_eq!(last.get_str("phase"), Some("end"));
    assert_eq!(int(last.get("iteration")), variants);
    let of = |phase: &str| -> Vec<i64> {
        lines
            .iter()
            .filter(|line| line.get_str("phase") == Some(phase))
            .map(|line| int(line.get("iteration")))
            .collect()
    };
    let each: Vec<i64> = (1..=variants).collect();
    assert_eq!(of("measuring"), each);
    assert_eq!(of("ranking"), each);
    assert_eq!(lines.len() as i64, 2 + 2 * variants);
    for line in &lines {
        assert_eq!(line.get_str("at"), Some(AT));
        assert!(!line.get_str("text").unwrap_or_default().is_empty());
    }
    // A variant starts to be measured before it is ranked.
    let place = |phase: &str, iteration: i64| {
        lines
            .iter()
            .position(|line| {
                line.get_str("phase") == Some(phase) && int(line.get("iteration")) == iteration
            })
            .expect("a line")
    };
    for iteration in 1..=variants {
        assert!(place("measuring", iteration) < place("ranking", iteration));
    }
}

/// After the run the last ranking line names the same top three as `campaign report` on the
/// run's measurements; the first names the one variant finished then.
#[test]
fn the_last_ranking_line_names_the_reports_top_three() {
    let (recorded, directory) = replayed("top-three");
    let text = report::report(&recorded).unwrap_or_else(|error| panic!("{error}"));
    let top: Vec<String> = ranking(&text)
        .into_iter()
        .take(3)
        .map(|(id, _)| id)
        .collect();
    let rankings: Vec<String> = phases(&directory)
        .iter()
        .filter(|line| line.get_str("phase") == Some("ranking"))
        .map(|line| line.get_str("text").expect("a text").to_owned())
        .collect();
    assert_eq!(
        rankings.last().expect("a ranking line"),
        &format!("top 3: {}", top.join(", "))
    );
    assert_eq!(
        rankings[0],
        format!("top 3: {}", recorded.variants[0].0),
        "after the first variant"
    );
}

/// A run that fails says so in the phase file, with the reason.
#[test]
fn a_failed_run_says_so_in_the_phase_file() {
    let recorded = studio();
    let directory = scratch("failed");
    let mut events = Events::create(&directory, &recorded.declaration, clock())
        .unwrap_or_else(|error| panic!("{error}"));
    events.start().expect("start");
    events.failed("the page answered 404").expect("failed");
    let lines = phases(&directory);
    let last = lines.last().expect("a line");
    assert_eq!(last.get_str("phase"), Some("failed"));
    assert_eq!(last.get_str("text"), Some("the page answered 404"));
}

// Writing.

/// Each line is on disk as soon as it is written: a reader that polls the files while the run
/// goes sees it, not when the run ends.
#[test]
fn each_line_is_on_disk_when_it_is_written() {
    let recorded = studio();
    let directory = scratch("flushed");
    let declaration = &recorded.declaration;
    let mut events =
        Events::create(&directory, declaration, clock()).unwrap_or_else(|error| panic!("{error}"));
    assert!(phases(&directory).is_empty() && candidates(&directory).is_empty());
    events.start().expect("start");
    assert_eq!(phases(&directory).len(), 1);
    let shipped = declaration.shipped();
    let at = declaration
        .variants()
        .iter()
        .position(|variant| *variant == shipped)
        .expect("a shipped variant");
    events.baseline(recorded.variants[at].1.clone());
    events.measuring(0).expect("measuring");
    assert_eq!(phases(&directory).len(), 2);
    events
        .finished(0, recorded.variants[0].1.clone(), &[])
        .expect("finished");
    assert_eq!(candidates(&directory).len(), 1);
    assert_eq!(phases(&directory).len(), 3);
    drop(events);
}

/// A run starts both files empty: lines of an earlier run in the directory are not kept.
#[test]
fn a_run_starts_both_files_empty() {
    let recorded = studio();
    let directory = scratch("empty");
    for name in [events::CANDIDATES_FILE, events::PHASE_FILE] {
        std::fs::write(directory.join(name), "{\"old\":1}\n").expect("an old file");
    }
    let mut events = Events::create(&directory, &recorded.declaration, clock())
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(candidates(&directory).is_empty());
    events.start().expect("start");
    assert_eq!(phases(&directory).len(), 1);
}

/// A time stamp is the local time with its offset, RFC 3339 to the second.
#[test]
fn a_time_stamp_is_local_time_with_its_offset() {
    assert_eq!(
        events::rfc3339(1_795_772_885, 3600),
        "2026-11-27T10:48:05+01:00"
    );
    assert_eq!(events::rfc3339(951_868_799, 0), "2000-02-29T23:59:59+00:00");
    assert_eq!(
        events::rfc3339(951_868_799, -(5 * 3600 + 1800)),
        "2000-02-29T18:29:59-05:30"
    );
    let now = (events::local_clock())();
    let bytes = now.as_bytes();
    assert_eq!(bytes.len(), 25, "{now}");
    for (at, byte) in [
        (4, b'-'),
        (7, b'-'),
        (10, b'T'),
        (13, b':'),
        (16, b':'),
        (22, b':'),
    ] {
        assert_eq!(bytes[at], byte, "{now}");
    }
    assert!(matches!(bytes[19], b'+' | b'-'), "{now}");
}

// The document.

/// The fields of one table: each name, and whether its row says the field is optional.
type Fields = Vec<(String, bool)>;

/// The fields of the two tables of the live view in `docs/targets.md`, in order: the
/// candidates file's, then the phase file's.
fn documented() -> (Fields, Fields) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/targets.md");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let section = text
        .split_once("### The live view")
        .map(|(_, rest)| rest)
        .unwrap_or_else(|| panic!("docs/targets.md has no section `The live view`"));
    let section = section.split("\n#").next().unwrap_or_default();
    let mut tables: Vec<Fields> = Vec::new();
    let mut open = false;
    for line in section.lines() {
        let row = line
            .strip_prefix("| `")
            .and_then(|rest| rest.split_once('`'));
        match row {
            Some((name, rest)) => {
                if !open {
                    tables.push(Vec::new());
                    open = true;
                }
                let optional = rest.contains("optional");
                tables
                    .last_mut()
                    .expect("a table")
                    .push((name.to_owned(), optional));
            }
            None => open = false,
        }
    }
    assert_eq!(tables.len(), 2, "two tables in the live view: {tables:?}");
    let phase = tables.pop().expect("a table");
    let candidate = tables.pop().expect("a table");
    (candidate, phase)
}

/// Every field the document lists is what the writer writes, in that order, and the writer
/// writes no other: the fields the document calls optional may be left out of a line.
fn held(lines: &[Value], fields: &[(String, bool)]) {
    let names: Vec<&str> = fields.iter().map(|(name, _)| name.as_str()).collect();
    let mut seen = BTreeSet::new();
    for line in lines {
        let own = keys(line);
        let mut listed = names
            .iter()
            .filter(|name| own.iter().any(|key| key == *name));
        assert!(
            own.iter().all(|key| listed.next() == Some(&key.as_str())),
            "{own:?} is not in the order of {names:?}"
        );
        for (name, optional) in fields {
            assert!(
                *optional || own.contains(name),
                "{} lacks `{name}`",
                patch::compact(line)
            );
        }
        seen.extend(own);
    }
    let documented: BTreeSet<String> = names.iter().map(|name| (*name).to_owned()).collect();
    assert_eq!(seen, documented);
}

#[test]
fn every_field_the_document_lists_is_what_the_run_writes() {
    let (candidate, phase) = documented();
    let (_, directory) = replayed("document");
    let shot = ("1366x768 light".to_owned(), "shots/a.png".to_owned());
    let recorded = studio();
    let shots = scratch("document-shots");
    let mut events = Events::create(&shots, &recorded.declaration, clock())
        .unwrap_or_else(|error| panic!("{error}"));
    events.start().expect("start");
    events.baseline(recorded.variants[0].1.clone());
    events.measuring(0).expect("measuring");
    events
        .finished(0, recorded.variants[0].1.clone(), &[shot])
        .expect("finished");
    events.failed("why").expect("failed");
    let mut all = candidates(&directory);
    all.extend(candidates(&shots));
    held(&all, &candidate);
    let mut all = phases(&directory);
    all.extend(phases(&shots));
    held(&all, &phase);
    assert!(
        candidate
            .iter()
            .any(|(name, optional)| name == "screenshots" && *optional)
    );
}

// The command.

fn ux_search(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ux-search"))
        .args(args)
        .output()
        .expect("ux-search runs")
}

/// `--events` goes with a whole campaign: a run restricted to one variant or one environment
/// ranks nothing, and is refused with it before a browser starts.
#[test]
fn events_go_with_a_whole_campaign_only() {
    let declaration =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/campaign/small.campaign.yaml");
    let declaration = declaration.to_str().expect("a UTF-8 path");
    let directory = scratch("restricted");
    let events = directory.join("events");
    let events = events.to_str().expect("a UTF-8 path");
    for restriction in [["--variant", "a"], ["--environment", "800x600 light"]] {
        let output = ux_search(&[
            "campaign",
            "run",
            "--declaration",
            declaration,
            "--url",
            "http://127.0.0.1:9",
            "--out",
            "measurements.json",
            "--events",
            events,
            restriction[0],
            restriction[1],
        ]);
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        let said = String::from_utf8_lossy(&output.stderr);
        assert!(said.contains("--events"), "{said}");
        assert!(!directory.join("events").exists());
    }
    let usage = String::from_utf8_lossy(&ux_search(&["--help"]).stdout).into_owned();
    assert!(usage.contains("[--events <dir>]"), "{usage}");
}
