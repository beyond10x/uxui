//! `ux-search landing promote` (`story:docs-site-first-climb`, phase B): one variant of the
//! frontier of a recorded campaign is written to the two sources, with one `ratchet-evidence/1`
//! document that names the campaign. No test starts a browser, runs a campaign or touches
//! `website/`: each promotes on copies of `fixtures/docs-landing/` under cargo's scratch
//! directory for integration tests.
//!
//! `fixtures/docs-landing/promote.evidence.json` is the evidence document of promoting
//! [`CHOSEN`] on the fixture's sources and measurements, with `{version}` where the version of
//! this crate stands. `UX_SEARCH_BLESS=1` rewrites it: after a new campaign was recorded, or
//! when the document's fields change.
//!
//! The repository has no reader of `ratchet-evidence/1` that checks a document's shape, so
//! [`check_shape`] holds the document to the members and the rules 1, 2, 3, 5, 6 and 9 of
//! ratchet's `docs/evidence-format.md`, revision 2, and to the members of ratchet's own example
//! (`engine/static/tests/fixtures/ratchet-example/example.evidence.json`).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use ux_search::landing::patch::{self, Value};
use ux_search::landing::promote::{self, Options};
use ux_search::landing::record::{self, Recorded};
use ux_search::landing::report;
use ux_search::landing::space;

/// A variant on the frontier of the recorded campaign that is not the page as shipped: it
/// changes both sources.
const CHOSEN: &str = "docs-system.centred.1+0.features-first.d85.h85";
/// A variant no campaign has on its frontier: three links, the loosest density, the large
/// heading.
const OFF_THE_FRONTIER: &str = "docs-system.left.1+3.features-first.d115.h100";
const CHOSEN_BY: &str = "the owner, 2026-11-26";

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/docs-landing")
        .join(name)
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn json(text: &str) -> Value {
    patch::parse(text).unwrap_or_else(|error| panic!("{error}"))
}

fn recorded() -> Recorded {
    Recorded::parse(&read(&fixture("measurements.json"))).unwrap_or_else(|error| panic!("{error}"))
}

/// Copies of the fixture's sources and measurements in a directory of the test's own, and the
/// two files a promotion adds or rewrites there.
struct Copies {
    measurements: PathBuf,
    product: PathBuf,
    css: PathBuf,
    evidence: PathBuf,
    shipped: PathBuf,
}

impl Copies {
    fn options(&self, id: &str) -> Options {
        Options {
            measurements: self.measurements.clone(),
            variant: id.to_owned(),
            product: self.product.clone(),
            css: self.css.clone(),
            evidence: self.evidence.clone(),
            chosen_by: Some(CHOSEN_BY.to_owned()),
            shipped_file: Some(self.shipped.clone()),
        }
    }

    /// What a refused promotion must leave: the sources and the shipped file as they were, and
    /// no evidence document.
    fn assert_untouched(&self, case: &str, product: &str, css: &str) {
        assert_eq!(read(&self.product), product, "{case}: product.json");
        assert_eq!(read(&self.css), css, "{case}: landing.css");
        assert_eq!(
            read(&self.shipped),
            format!("{}\n", space::shipped().id),
            "{case}: the shipped file"
        );
        assert!(!self.evidence.exists(), "{case}: an evidence document");
        let left: BTreeSet<String> = std::fs::read_dir(self.product.parent().expect("a parent"))
            .expect("the directory of the copies")
            .map(|entry| {
                entry
                    .expect("an entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        let expected: BTreeSet<String> = [
            "landing.css",
            "measurements.json",
            "product.json",
            "shipped.txt",
        ]
        .map(str::to_owned)
        .into();
        assert_eq!(left, expected, "{case}: files beside the sources");
    }
}

fn copies(test: &str) -> Copies {
    let directory = crate::state_dir(test);
    if directory.exists() {
        std::fs::remove_dir_all(&directory).expect("the last run's copies removed");
    }
    std::fs::create_dir_all(&directory).expect("a directory for the copies");
    let copy = |name: &str| {
        let to = directory.join(name);
        std::fs::copy(fixture(name), &to).unwrap_or_else(|error| panic!("{name}: {error}"));
        to
    };
    let shipped = directory.join("shipped.txt");
    std::fs::write(&shipped, format!("{}\n", space::shipped().id)).expect("the shipped file");
    Copies {
        measurements: copy("measurements.json"),
        product: copy("product.json"),
        css: copy("landing.css"),
        evidence: directory.join("evidence/promotion.evidence.json"),
        shipped,
    }
}

fn variant(id: &str) -> space::Variant {
    space::find(id).unwrap_or_else(|| panic!("no variant {id}"))
}

// The evidence document.

fn evidence(document: &Value) -> &Value {
    document.get("evidence").expect("an `evidence` member")
}

fn results(document: &Value) -> &[Value] {
    evidence(document)
        .get("results")
        .expect("`evidence.results`")
        .list()
}

fn result<'a>(document: &'a Value, output: &str, item: Option<&str>) -> &'a Value {
    results(document)
        .iter()
        .find(|result| result.get_str("output") == Some(output) && result.get_str("item") == item)
        .unwrap_or_else(|| panic!("no result {output} for {item:?}"))
}

fn keys(value: &Value) -> Vec<&str> {
    value
        .entries()
        .iter()
        .map(|(key, _)| key.as_str())
        .collect()
}

fn is_digest(text: &str) -> bool {
    text.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}

/// Every text of a value, whatever it is nested in.
fn texts<'a>(value: &'a Value, out: &mut Vec<&'a str>) {
    match value {
        Value::Str(text) => out.push(text),
        Value::List(items) => items.iter().for_each(|item| texts(item, out)),
        Value::Map(entries) => entries.iter().for_each(|(_, item)| texts(item, out)),
        _ => {}
    }
}

/// The members `ratchet-evidence/1` knows (ratchet `docs/evidence-format.md`, revision 2, § 3)
/// and the rules of its § 4 that one document can be held to.
fn check_shape(document: &Value) {
    assert_eq!(
        keys(document),
        ["format", "evidence"],
        "exact results: no `run`, so the same inputs give the same file"
    );
    assert_eq!(document.get_str("format"), Some("ratchet-evidence/1"));
    let evidence = evidence(document);
    let known = [
        "project",
        "assessment",
        "assessor",
        "protocol_version",
        "candidate",
        "target",
        "inputs",
        "completion",
        "conditions",
        "coverage",
        "results",
        "source",
    ];
    for key in keys(evidence) {
        assert!(known.contains(&key), "`evidence.{key}` is no member");
    }
    for key in [
        "project",
        "assessment",
        "assessor",
        "target",
        "completion",
        "conditions",
        "coverage",
        "results",
    ] {
        assert!(evidence.get(key).is_some(), "no `evidence.{key}`");
    }
    let example = json(&read(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../static/tests/fixtures/ratchet-example/example.evidence.json"),
    ));
    assert_eq!(
        keys(evidence),
        keys(self::evidence(&example)),
        "the members of `evidence`, in the order of ratchet's example"
    );
    assert_eq!(evidence.get_str("completion"), Some("completed"));

    let target = evidence.get("target").expect("a target");
    for key in keys(target) {
        assert!(
            [
                "kind",
                "class",
                "selection",
                "revision",
                "identifiers",
                "window"
            ]
            .contains(&key),
            "`evidence.target.{key}` is no member"
        );
    }
    assert_eq!(target.get_str("kind"), Some("document"));
    assert!(
        ["local", "ci", "ephemeral", "staging", "production"]
            .contains(&target.get_str("class").expect("a class"))
    );
    assert_eq!(
        evidence.get("candidate").is_some(),
        target.get("selection").is_some(),
        "rule 1: `candidate` exactly when `target.selection`"
    );
    let identifiers = target
        .get("identifiers")
        .expect("identifiers of a document");
    assert!(is_digest(identifiers.get_str("digest").expect("a digest")));
    assert!(identifiers.get_str("digest_form").is_some());
    assert_eq!(evidence.get_str("candidate"), identifiers.get_str("digest"));

    for (name, input) in evidence.get("inputs").expect("inputs").entries() {
        for key in keys(input) {
            assert!(
                ["value", "digest", "digest_form"].contains(&key),
                "`evidence.inputs.{name}.{key}` is no member"
            );
        }
        match input.get_str("digest") {
            Some(digest) => {
                assert!(is_digest(digest), "{name}: {digest}");
                assert_eq!(input.get_str("digest_form"), Some("bytes"), "{name}");
            }
            None => assert!(input.get("value").is_some(), "{name}: no value, no digest"),
        }
    }

    let conditions = evidence.get("conditions").expect("conditions");
    for key in keys(conditions) {
        assert!(
            [
                "runner",
                "host_load_per_cpu",
                "tools",
                "cache_profile",
                "other"
            ]
            .contains(&key),
            "`evidence.conditions.{key}` is no member"
        );
    }
    for part in ["tools", "other"] {
        for (name, value) in conditions.get(part).expect("a map of texts").entries() {
            assert!(
                value.str().is_some(),
                "conditions.{part}.{name} is not text"
            );
        }
    }

    let coverage = evidence.get("coverage").expect("coverage");
    assert_eq!(
        keys(coverage),
        [
            "unit",
            "executed",
            "skipped",
            "not_run",
            "unsupported",
            "passed",
            "failed"
        ]
    );
    let count = |key: &str| match coverage.get(key) {
        Some(Value::Int(n)) if *n >= 0 => *n,
        other => panic!("coverage.{key} is {other:?}"),
    };
    assert!(
        count("passed") + count("failed") <= count("executed"),
        "rule 4"
    );
    for key in ["skipped", "not_run", "unsupported"] {
        assert_eq!(count(key), 0, "no `not_executed`, so nothing is left out");
    }

    let mut seen = BTreeSet::new();
    for result in results(document) {
        let output = result.get_str("output").expect("an output");
        let scope = result.get_str("scope").expect("a scope");
        let item = result.get_str("item");
        assert_eq!(result.get_str("kind"), Some("exact"), "{output}");
        assert_eq!(
            item.is_none(),
            scope == "run",
            "rule 2: `item` absent exactly when the scope is `run` ({output})"
        );
        assert!(
            seen.insert((output, scope, item)),
            "rule 3: two results for {output} {scope} {item:?}"
        );
        assert!(
            result.get("incomplete").is_none() && result.get("at_least").is_none(),
            "rules 5 and 6: a complete result ({output})"
        );
        let own: &[&str] = match result.get_str("value_type") {
            Some("number") => {
                let value = result.get_str("value").expect("a number is text");
                assert!(value.parse::<u64>().is_ok(), "{output}: {value}");
                assert!(result.get_str("unit").is_some(), "{output}: no unit");
                &["value", "unit"]
            }
            Some("digest") => {
                assert!(is_digest(result.get_str("digest").expect("a digest")));
                &["digest"]
            }
            other => panic!("{output}: value type {other:?}"),
        };
        for key in keys(result) {
            assert!(
                ["output", "kind", "scope", "item", "value_type", "details"].contains(&key)
                    || own.contains(&key),
                "{output}: `{key}` is no member of a result of its type"
            );
        }
    }

    let mut all = Vec::new();
    texts(evidence, &mut all);
    for text in all {
        assert!(
            !text.starts_with('/') || text == record::PAGE,
            "rule 9: `{text}` reads as an absolute path"
        );
        assert!(!text.contains(env!("CARGO_MANIFEST_DIR")), "rule 9: {text}");
    }
}

fn golden(text: &str) -> String {
    let version = format!("\"ux-search\":\"{}\"", env!("CARGO_PKG_VERSION"));
    assert_eq!(text.matches(&version).count(), 1, "{text}");
    text.replace(&version, "\"ux-search\":\"{version}\"")
}

// A promotion.

#[test]
fn a_promotion_writes_both_sources_the_evidence_and_the_shipped_variant() {
    let copies = copies("landing-promote");
    let (product, css) = (read(&copies.product), read(&copies.css));
    let outcome =
        promote::promote(&copies.options(CHOSEN)).unwrap_or_else(|error| panic!("{error}"));

    // The sources: what the variant's patch writes, applied here on its own.
    let (patched_product, patched_css) = variant(CHOSEN)
        .patch()
        .apply(&product, &css)
        .unwrap_or_else(|error| panic!("{error}"));
    assert_ne!(patched_product, product, "the variant changes product.json");
    assert_ne!(patched_css, css, "the variant changes landing.css");
    assert_eq!(read(&copies.product), patched_product);
    assert_eq!(read(&copies.css), patched_css);
    // And what its axis values say, read from the files.
    let written = json(&read(&copies.product));
    let actions = written
        .get("product")
        .and_then(|product| product.get("actions"))
        .expect("/product/actions")
        .list();
    assert_eq!(actions.len(), 1, "`1+0`: the primary action alone");
    assert_eq!(actions[0].get_str("kind"), Some("primary"));
    for (property, value) in [
        ("--uxui-density", "0.85"),
        ("--uxui-heading-scale", "0.85"),
        ("--uxui-hero-align", "center"),
        ("--uxui-hero-lede-inline", "auto"),
    ] {
        assert_eq!(
            patch::custom_property(&read(&copies.css), ":root", property).as_deref(),
            Some(value),
            "{property}"
        );
    }

    assert_eq!(read(&copies.shipped), format!("{CHOSEN}\n"));
    assert_eq!(
        outcome.written,
        [
            copies.product.clone(),
            copies.css.clone(),
            copies.evidence.clone(),
            copies.shipped.clone()
        ]
    );

    // The evidence: the committed document, and the same bytes from fresh copies.
    let text = read(&copies.evidence);
    let path = fixture("promote.evidence.json");
    if std::env::var_os("UX_SEARCH_BLESS").is_some() {
        std::fs::write(&path, golden(&text)).expect("the golden document");
    }
    assert_eq!(
        golden(&text),
        read(&path),
        "{} is not the evidence of promoting {CHOSEN}; UX_SEARCH_BLESS=1 rewrites it",
        path.display()
    );
    let again = self::copies("landing-promote-again");
    promote::promote(&again.options(CHOSEN)).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        read(&again.evidence),
        text,
        "a second run writes other bytes"
    );
    assert_eq!(read(&again.product), patched_product);
    assert_eq!(read(&again.css), patched_css);
}

#[test]
fn the_evidence_names_the_campaign_the_variant_and_the_sources() {
    let copies = copies("landing-promote-evidence");
    let (product, css) = (read(&copies.product), read(&copies.css));
    promote::promote(&copies.options(CHOSEN)).unwrap_or_else(|error| panic!("{error}"));
    let text = read(&copies.evidence);
    assert!(text.ends_with("}\n"), "{text}");
    let document = json(&text);
    check_shape(&document);

    let recorded = recorded();
    let ranked = report::rank(&recorded);
    let place = |id: &str| {
        let at = ranked
            .ranking
            .iter()
            .position(|&index| ranked.scored[index].id == id)
            .unwrap_or_else(|| panic!("{id} is not ranked"));
        (at + 1, ranked.ranking[at])
    };
    let evidence = evidence(&document);
    assert_eq!(evidence.get_str("project"), Some("uxui"));
    assert_eq!(
        evidence.get_str("assessment"),
        Some("docs-landing-promotion")
    );
    assert_eq!(evidence.get_str("assessor"), Some("ux-search@1"));

    // The campaign: the measurements file, the adapter and the browser it ran on.
    let input = |name: &str| {
        evidence
            .get("inputs")
            .and_then(|inputs| inputs.get(name))
            .unwrap_or_else(|| panic!("no input {name}"))
    };
    assert_eq!(
        keys(evidence.get("inputs").expect("inputs")),
        ["measurements", "variant", "product", "css", "chosen_by"]
    );
    assert_eq!(
        input("measurements").get_str("digest"),
        Some(record::digest(read(&fixture("measurements.json")).as_bytes()).as_str())
    );
    let tools = evidence
        .get("conditions")
        .and_then(|conditions| conditions.get("tools"))
        .expect("conditions.tools");
    let (adapter, version) = recorded
        .adapter
        .split_once(' ')
        .expect("an adapter and its version");
    assert_eq!(keys(tools), ["ux-search", adapter, "chromium"]);
    assert_eq!(tools.get_str("ux-search"), Some(env!("CARGO_PKG_VERSION")));
    assert_eq!(tools.get_str(adapter), Some(version));
    assert_eq!(tools.get_str("chromium"), Some(recorded.browser.as_str()));
    let other = evidence
        .get("conditions")
        .and_then(|conditions| conditions.get("other"))
        .expect("conditions.other");
    assert_eq!(other.get_str("measurements_format"), Some(record::FORMAT));
    assert_eq!(other.get_str("page"), Some(record::PAGE));
    assert_eq!(
        other.get_str("environments"),
        Some(
            "1440x900 light, 1440x900 dark, 1024x768 light, 1024x768 dark, 390x844 light, 390x844 dark"
        )
    );

    // The chosen variant and who chose it.
    assert_eq!(input("variant").get_str("value"), Some(CHOSEN));
    assert_eq!(input("chosen_by").get_str("value"), Some(CHOSEN_BY));
    let shipped = space::shipped().id;
    for (role, id, on_frontier) in [
        ("chosen", CHOSEN, true),
        ("shipped", shipped.as_str(), false),
    ] {
        let (rank, index) = place(id);
        let found = result(&document, "landing.rank@1", Some(id));
        assert_eq!(found.get_str("scope"), Some("variant"), "{role}");
        assert_eq!(found.get_str("value"), Some(rank.to_string().as_str()));
        assert_eq!(found.get_str("unit"), Some("place"));
        let details = found.get("details").expect("details");
        assert_eq!(details.get_str("role"), Some(role));
        assert_eq!(details.get("of"), Some(&Value::Int(144)));
        if role == "chosen" {
            assert_eq!(details.get("frontier"), Some(&Value::Bool(on_frontier)));
        } else {
            assert_eq!(
                details.get("frontier"),
                Some(&Value::Bool(ranked.frontier.contains(&index)))
            );
        }
        for (term, value) in report::TERMS.iter().zip(&ranked.scored[index].terms) {
            let found = result(&document, &format!("landing.{term}@1"), Some(id));
            assert_eq!(
                found.get_str("value"),
                Some(value.to_string().as_str()),
                "{role} {term}"
            );
            assert_eq!(
                found.get("details").and_then(|d| d.get_str("role")),
                Some(role)
            );
        }
    }
    let units: Vec<&str> = report::TERMS
        .iter()
        .map(|term| {
            result(&document, &format!("landing.{term}@1"), Some(CHOSEN))
                .get_str("unit")
                .expect("a unit")
        })
        .collect();
    assert_eq!(
        units,
        ["rules", "words", "px", "regions", "targets", "actions"]
    );

    // The frontier.
    let frontier = result(&document, "landing.frontier_size@1", None);
    assert_eq!(frontier.get_str("scope"), Some("run"));
    assert_eq!(
        frontier.get_str("value"),
        Some(ranked.frontier.len().to_string().as_str())
    );
    assert_eq!(frontier.get_str("unit"), Some("variants"));
    let on_it: Vec<&str> = frontier
        .get("details")
        .and_then(|details| details.get("variants"))
        .expect("details.variants")
        .list()
        .iter()
        .filter_map(Value::str)
        .collect();
    let expected: Vec<&str> = ranked
        .frontier
        .iter()
        .map(|&index| ranked.scored[index].id.as_str())
        .collect();
    assert_eq!(on_it, expected);

    let coverage = evidence.get("coverage").expect("coverage");
    assert_eq!(coverage.get_str("unit"), Some("variant"));
    assert_eq!(coverage.get("executed"), Some(&Value::Int(144)));
    assert_eq!(coverage.get("passed"), Some(&Value::Int(144)));
    assert_eq!(coverage.get("failed"), Some(&Value::Int(0)));

    // The sources, before and after, and the patch as the report prints it.
    let report_text = read(&fixture("report.txt"));
    let block: Vec<&str> = report_text
        .lines()
        .skip_while(|line| !line.contains(&format!(" {CHOSEN} hard=")))
        .skip(1)
        .take_while(|line| line.starts_with("     "))
        .map(str::trim_start)
        .collect();
    assert!(!block.is_empty(), "the report prints no patch for {CHOSEN}");
    let mut printed = Vec::new();
    let mut after = Vec::new();
    for (name, flag, before, path) in [
        ("product.json", "product", &product, &copies.product),
        ("landing.css", "css", &css, &copies.css),
    ] {
        let found = result(&document, "landing.source@1", Some(name));
        assert_eq!(found.get_str("scope"), Some("source"));
        assert_eq!(found.get_str("value_type"), Some("digest"));
        let before = record::digest(before.as_bytes());
        let written = record::digest(read(path).as_bytes());
        assert_ne!(before, written, "{name}");
        assert_eq!(input(flag).get_str("digest"), Some(before.as_str()));
        assert_eq!(found.get_str("digest"), Some(written.as_str()));
        let details = found.get("details").expect("details");
        assert_eq!(details.get_str("before"), Some(before.as_str()));
        printed.extend(
            details
                .get("patch")
                .expect("details.patch")
                .list()
                .iter()
                .filter_map(Value::str),
        );
        after.push(format!(
            "{}  {name}\n",
            written.trim_start_matches("sha256:")
        ));
    }
    assert_eq!(printed, block, "the patch as the report prints it");
    assert_eq!(
        evidence.get_str("candidate"),
        Some(record::digest(after.concat().as_bytes()).as_str()),
        "the candidate: the two sources after the patch, as `sha256sum` lists them"
    );
    assert_eq!(
        evidence
            .get("target")
            .and_then(|target| target.get("identifiers"))
            .and_then(|identifiers| identifiers.get_str("digest_form")),
        Some("sha256sum_lines")
    );
}

#[test]
fn a_promotion_without_a_name_or_a_shipped_file_leaves_both_out() {
    let copies = copies("landing-promote-bare");
    let options = Options {
        chosen_by: None,
        shipped_file: None,
        ..copies.options(CHOSEN)
    };
    let outcome = promote::promote(&options).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(outcome.written.len(), 3);
    assert_eq!(
        read(&copies.shipped),
        format!("{}\n", space::shipped().id),
        "the shipped file is not named, so it stays"
    );
    let document = json(&read(&copies.evidence));
    check_shape(&document);
    assert_eq!(
        keys(evidence(&document).get("inputs").expect("inputs")),
        ["measurements", "variant", "product", "css"]
    );
}

// Refusals.

#[test]
fn a_refused_promotion_writes_nothing() {
    let measurements = read(&fixture("measurements.json"));
    let chosen_entry = format!("\"id\": \"{CHOSEN}\"");
    assert_eq!(measurements.matches(&chosen_entry).count(), 1);

    // The record with the chosen variant scrolling sideways at 1440x900 light.
    let mut broken = recorded();
    let at = broken
        .variants
        .iter()
        .position(|(id, _)| id == CHOSEN)
        .expect("the chosen variant is recorded");
    broken.variants[at].1[0].h_scroll = true;
    // The record without the chosen variant.
    let mut without = recorded();
    without.variants.remove(at);
    let shipped = space::shipped().id;
    // The rank the refusal names for the variant off the frontier.
    let ranked = report::rank(&recorded());
    let off_rank = ranked
        .ranking
        .iter()
        .position(|&index| ranked.scored[index].id == OFF_THE_FRONTIER)
        .map(|at| format!("rank {} of 144, hard=0 ", at + 1))
        .expect("ranked");

    // A case: its name, the id asked for, the measurements when they are not the fixture's,
    // what it changes in `product.json` and in `landing.css`, and what the refusal says.
    type Change = fn(&str) -> String;
    type Case<'a> = (
        &'a str,
        &'a str,
        Option<String>,
        Change,
        Change,
        &'a [&'a str],
    );
    let same: Change = |text| text.to_owned();
    let cases: [Case; 9] = [
        (
            "an id outside the space",
            "docs-system.left.1+2.features-first.d85.h85",
            None,
            same,
            same,
            &[
                "docs-system.left.1+2.features-first.d85.h85",
                "no variant of the space",
            ],
        ),
        (
            "half an id",
            "docs-system.left",
            None,
            same,
            same,
            &["`docs-system.left`", "no variant of the space"],
        ),
        (
            "off the frontier",
            OFF_THE_FRONTIER,
            None,
            same,
            same,
            &[
                OFF_THE_FRONTIER,
                "not on the frontier",
                &off_rank,
                "is no worse on any term and better on one",
            ],
        ),
        (
            "the page as shipped",
            &shipped,
            None,
            same,
            same,
            &[shipped.as_str(), "the page as shipped"],
        ),
        (
            "another product.json",
            CHOSEN,
            None,
            |text| text.replace("Measure the UI.", "Measure the page."),
            same,
            &[
                "product.json",
                "not the source the measurements were recorded on",
            ],
        ),
        (
            "another landing.css",
            CHOSEN,
            None,
            same,
            |text| text.replace("3.5rem", "3.6rem"),
            &[
                "landing.css",
                "not the source the measurements were recorded on",
            ],
        ),
        (
            "a hard rule broken",
            CHOSEN,
            Some(broken.text()),
            same,
            same,
            &[CHOSEN, "breaks a hard rule", "hard=1"],
        ),
        (
            "not in the measurements",
            CHOSEN,
            Some(without.text()),
            same,
            same,
            &[CHOSEN, "not in the measurements"],
        ),
        (
            "measurements of another format",
            CHOSEN,
            Some(measurements.replace(record::FORMAT, "ux-search-landing/0")),
            same,
            same,
            &["format"],
        ),
    ];
    for (at, (case, id, record, change_product, change_css, says)) in cases.into_iter().enumerate()
    {
        let copies = copies(&format!("landing-promote-refused-{at}"));
        if let Some(record) = record {
            assert_ne!(record, measurements, "{case}: the case changes nothing");
            std::fs::write(&copies.measurements, record).expect("the changed measurements");
        }
        let product = change_product(&read(&copies.product));
        let css = change_css(&read(&copies.css));
        std::fs::write(&copies.product, &product).expect("product.json");
        std::fs::write(&copies.css, &css).expect("landing.css");
        let changed =
            product != read(&fixture("product.json")) || css != read(&fixture("landing.css"));
        assert_eq!(changed, case.starts_with("another"), "{case}");

        let refused = match promote::promote(&copies.options(id)) {
            Ok(outcome) => panic!("{case}: promoted, wrote {:?}", outcome.written),
            Err(refused) => refused,
        };
        for part in says {
            assert!(refused.contains(part), "{case}: {refused}");
        }
        copies.assert_untouched(case, &product, &css);
    }
}

/// A source the patch cannot be applied to (here: `product.json` not written as the patch
/// writes it) refuses the promotion although the measurements name that very file.
#[test]
fn a_patch_that_does_not_apply_writes_nothing() {
    let copies = copies("landing-promote-unpatched");
    let product = read(&copies.product).replace("\n  \"sections\"", "\n\n  \"sections\"");
    assert_ne!(product, read(&copies.product));
    std::fs::write(&copies.product, &product).expect("product.json");
    let css = read(&copies.css);
    let mut recorded = recorded();
    recorded.product_digest = record::digest(product.as_bytes());
    std::fs::write(&copies.measurements, recorded.text()).expect("the measurements");
    let refused = promote::promote(&copies.options(CHOSEN)).expect_err("the patch is refused");
    assert!(refused.contains("JSON.stringify"), "{refused}");
    copies.assert_untouched("another form of product.json", &product, &css);
}

/// An evidence path that cannot be written (its directory is a file) fails the promotion
/// before a source changes.
#[test]
fn an_evidence_file_that_cannot_be_written_leaves_the_sources() {
    let copies = copies("landing-promote-unwritable");
    let (product, css) = (read(&copies.product), read(&copies.css));
    let options = Options {
        evidence: copies.measurements.join("promotion.evidence.json"),
        ..copies.options(CHOSEN)
    };
    let failed = promote::promote(&options).expect_err("the evidence cannot be written");
    assert!(failed.contains("promotion.evidence.json"), "{failed}");
    copies.assert_untouched("an evidence path under a file", &product, &css);
}

// The shipped variant.

/// `src/landing/shipped.txt` is the one place the shipped variant stands: the id and a newline.
#[test]
fn the_shipped_variant_is_the_one_the_shipped_file_names() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/landing/shipped.txt");
    let id = space::shipped().id;
    assert_eq!(read(&path), format!("{id}\n"));
    assert_eq!(space::find(&id), Some(space::shipped()));
    assert!(
        space::anchored(&space::shipped()),
        "{id}: the operations on product.json and the palette rules are written for a shipped \
         page with other actions, section order or theme"
    );
    for (id, anchored) in [
        ("docs-system.centred.1+1.features-first.d115.h100", true),
        ("docs-system.left.1+0.features-first.d100.h85", false),
        ("docs-system.left.1+1.status-first.d100.h85", false),
        ("studio.left.1+1.features-first.d100.h85", false),
    ] {
        assert_eq!(space::anchored(&variant(id)), anchored, "{id}");
    }
    for variant in space::variants() {
        assert_eq!(space::find(&variant.id).as_ref(), Some(&variant));
    }
    for id in [
        "",
        "docs-system",
        "docs-system.left.1+1.features-first.d100",
        "docs-system.left.1+1.features-first.d100.h85.h85",
        "docs-system.left.1+1.features-first.h85.d100",
        " docs-system.left.1+1.features-first.d100.h85",
    ] {
        assert_eq!(space::find(id), None, "`{id}`");
    }
}

/// A promotion that moves the shipped page off the values the space's operations are written
/// for says so.
#[test]
fn a_promotion_off_the_anchored_values_is_noted() {
    let copies = copies("landing-promote-note");
    let outcome =
        promote::promote(&copies.options(CHOSEN)).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(outcome.notes.len(), 1, "{:?}", outcome.notes);
    assert!(outcome.notes[0].contains("space.rs"), "{:?}", outcome.notes);

    let copies = self::copies("landing-promote-no-note");
    let anchored = report::rank(&recorded())
        .frontier
        .iter()
        .map(|&index| space::variants()[index].clone())
        .find(|variant| space::anchored(variant) && !variant.is_shipped());
    let Some(anchored) = anchored else {
        return;
    };
    let outcome =
        promote::promote(&copies.options(&anchored.id)).unwrap_or_else(|error| panic!("{error}"));
    assert!(outcome.notes.is_empty(), "{:?}", outcome.notes);
}

// The command.

fn ux_search(copies: &Copies, id: &str, more: &[&str]) -> std::process::Output {
    let path = |path: &Path| path.to_str().expect("a UTF-8 path").to_owned();
    Command::new(env!("CARGO_BIN_EXE_ux-search"))
        .args(["landing", "promote", "--measurements"])
        .arg(path(&copies.measurements))
        .args(["--variant", id, "--product"])
        .arg(path(&copies.product))
        .arg("--css")
        .arg(path(&copies.css))
        .arg("--evidence")
        .arg(path(&copies.evidence))
        .args(more)
        .output()
        .expect("ux-search runs")
}

#[test]
fn landing_promote_exits_0_and_says_what_it_wrote() {
    let copies = copies("landing-promote-command");
    let shipped = copies.shipped.to_str().expect("a UTF-8 path").to_owned();
    let output = ux_search(
        &copies,
        CHOSEN,
        &["--chosen-by", CHOSEN_BY, "--shipped-file", &shipped],
    );
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let ranked = report::rank(&recorded());
    let rank = ranked
        .ranking
        .iter()
        .position(|&index| ranked.scored[index].id == CHOSEN)
        .expect("ranked")
        + 1;
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!(
            "promoted {CHOSEN}: rank {rank} of 144, on the frontier of {}\n",
            ranked.frontier.len()
        )
    );
    let said = String::from_utf8_lossy(&output.stderr);
    for path in [
        &copies.product,
        &copies.css,
        &copies.evidence,
        &copies.shipped,
    ] {
        assert!(
            said.contains(&format!("ux-search: wrote {}\n", path.display())),
            "{said}"
        );
    }
    assert!(said.contains("ux-search: note: "), "{said}");

    let library = self::copies("landing-promote-command-library");
    promote::promote(&library.options(CHOSEN)).unwrap_or_else(|error| panic!("{error}"));
    for (written, expected) in [
        (&copies.product, &library.product),
        (&copies.css, &library.css),
        (&copies.evidence, &library.evidence),
        (&copies.shipped, &library.shipped),
    ] {
        assert_eq!(read(written), read(expected), "{}", written.display());
    }
}

#[test]
fn landing_promote_exits_1_on_a_refusal_and_2_on_a_wrong_invocation() {
    let copies = copies("landing-promote-command-refused");
    let (product, css) = (read(&copies.product), read(&copies.css));
    let output = ux_search(&copies, OFF_THE_FRONTIER, &[]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    let said = String::from_utf8_lossy(&output.stderr);
    assert!(
        said.starts_with(&format!(
            "ux-search: `{OFF_THE_FRONTIER}` is not on the frontier"
        )),
        "{said}"
    );
    assert!(!said.contains("usage:"), "{said}");
    copies.assert_untouched("the command, off the frontier", &product, &css);

    for more in [
        &["--allow-off-frontier", "true"][..],
        &["--chosen-by", ""],
        &["--variant", CHOSEN],
        &["--param", "a=b"],
    ] {
        let output = ux_search(&copies, CHOSEN, more);
        assert_eq!(output.status.code(), Some(2), "{more:?}: {output:?}");
        assert!(output.stdout.is_empty(), "{more:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("usage:"),
            "{more:?}"
        );
        copies.assert_untouched("a wrong invocation", &product, &css);
    }
    let output = Command::new(env!("CARGO_BIN_EXE_ux-search"))
        .args(["landing", "promote", "--variant", CHOSEN])
        .output()
        .expect("ux-search runs");
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(
            "landing promote needs --measurements, --variant, --product, --css and --evidence"
        ),
        "{output:?}"
    );
}
