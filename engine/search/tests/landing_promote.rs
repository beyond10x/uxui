//! `ux-search landing promote` (`story:docs-site-first-climb`, phase B): one variant of the
//! frontier of a recorded campaign is written to the two sources, with one `ratchet-evidence/1`
//! document that names the campaign. No test starts a browser, runs a campaign or touches
//! `website/`: each promotes on copies of `fixtures/docs-landing/` under cargo's scratch
//! directory for integration tests.
//!
//! `fixtures/docs-landing/promote.evidence.json` is the evidence document of promoting
//! [`CHOSEN`] on the fixture's sources and measurements, every file named from the directory it
//! lies in and the evidence beside the sources, with `{version}` where the version of this crate
//! stands. `UX_SEARCH_BLESS=1` rewrites it: after a new campaign was recorded, or when the
//! document's fields change.
//!
//! No test names a variant a promotion can make the shipped one: [`CHOSEN`] and the variants of
//! [`on_the_frontier`] are worked out from the recorded campaign and the shipped file, and what
//! a variant writes is read off its id ([`said_by`]).
//!
//! The repository has no reader of `ratchet-evidence/1` that checks a document's shape, so
//! [`check_shape`] holds the document to the members and the rules 1, 2, 3, 5, 6 and 9 of
//! ratchet's `docs/evidence-format.md`, revision 2, and to the members of ratchet's own example
//! (`engine/static/tests/fixtures/ratchet-example/example.evidence.json`). The module `assessor`
//! holds it to the declaration of `ux-search@1`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;

use ux_search::landing::patch::{self, Value};
use ux_search::landing::promote::{self, Options, Outcome};
use ux_search::landing::record::{self, Recorded};
use ux_search::landing::report;
use ux_search::landing::space;

/// The variant the tests promote: the first of the recorded campaign's frontier that is not
/// the page as shipped.
pub static CHOSEN: LazyLock<String> = LazyLock::new(|| {
    on_the_frontier(|_| true)
        .expect("a variant on the frontier beside the shipped one")
        .id
});
/// A variant no campaign has on its frontier: three links, the loosest density, the large
/// heading.
const OFF_THE_FRONTIER: &str = "docs-system.left.1+3.features-first.d115.h100";
const CHOSEN_BY: &str = "the owner, 2026-11-26";

/// The first variant of the recorded campaign's frontier, in rank order and the page as shipped
/// left out, that `wanted` takes.
fn on_the_frontier(wanted: impl Fn(&space::Variant) -> bool) -> Option<space::Variant> {
    let variants = space::variants();
    report::rank(&recorded())
        .frontier
        .iter()
        .map(|&index| variants[index].clone())
        .find(|variant| !variant.is_shipped() && wanted(variant))
}

/// What a variant's id says of the two sources, read off the id alone: the number of hero
/// actions, whether the status section comes first, and the four custom properties of `:root`.
fn said_by(id: &str) -> (usize, bool, [(&'static str, &'static str); 4]) {
    let parts: Vec<&str> = id.split('.').collect();
    assert_eq!(parts.len(), 6, "{id}");
    let actions = match parts[2] {
        "1+0" => 1,
        "1+1" => 2,
        "1+3" => 4,
        other => panic!("{id}: actions `{other}`"),
    };
    let status_first = match parts[3] {
        "features-first" => false,
        "status-first" => true,
        other => panic!("{id}: section order `{other}`"),
    };
    let (align, lede) = match parts[1] {
        "left" => ("start", "0"),
        "centred" => ("center", "auto"),
        other => panic!("{id}: hero `{other}`"),
    };
    let density = match parts[4] {
        "d85" => "0.85",
        "d100" => "1",
        "d115" => "1.15",
        other => panic!("{id}: density `{other}`"),
    };
    let heading = match parts[5] {
        "h85" => "0.85",
        "h100" => "1",
        other => panic!("{id}: heading scale `{other}`"),
    };
    (
        actions,
        status_first,
        [
            ("--uxui-density", density),
            ("--uxui-heading-scale", heading),
            ("--uxui-hero-align", align),
            ("--uxui-hero-lede-inline", lede),
        ],
    )
}

/// Whether a variant changes `product.json` and `landing.css` of the page as shipped: the
/// first when its actions or its section order are other ones, the second when its theme, hero,
/// density or heading scale are.
fn changes(id: &str) -> (bool, bool) {
    let shipped = space::shipped().id;
    let (own, shipped): (Vec<&str>, Vec<&str>) =
        (id.split('.').collect(), shipped.split('.').collect());
    let differs = |axes: &[usize]| axes.iter().any(|&at| own[at] != shipped[at]);
    (differs(&[2, 3]), differs(&[0, 1, 4, 5]))
}

fn rename(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::rename(from, to)
}

/// Every entry under a directory: its path from there and, for a file, the digest of its bytes.
fn tree(directory: &Path) -> BTreeMap<String, Option<String>> {
    fn walk(root: &Path, at: &Path, out: &mut BTreeMap<String, Option<String>>) {
        for entry in
            std::fs::read_dir(at).unwrap_or_else(|error| panic!("{}: {error}", at.display()))
        {
            let path = entry.expect("an entry").path();
            let name = path
                .strip_prefix(root)
                .expect("an entry under the root")
                .to_string_lossy()
                .into_owned();
            if path.is_dir() {
                out.insert(format!("{name}/"), None);
                walk(root, &path, out);
            } else {
                let bytes = std::fs::read(&path)
                    .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
                out.insert(name, Some(record::digest(&bytes)));
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(directory, directory, &mut out);
    out
}

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
/// files a promotion adds or rewrites there: the evidence document and, beside it, the copy of
/// the measurements, both in a directory `evidence/` that a promotion makes, and the shipped
/// file.
pub struct Copies {
    pub directory: PathBuf,
    pub measurements: PathBuf,
    pub product: PathBuf,
    pub css: PathBuf,
    pub evidence: PathBuf,
    pub kept: PathBuf,
    pub shipped: PathBuf,
}

impl Copies {
    pub fn options(&self, id: &str) -> Options {
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

    /// The copies with the evidence beside the sources, so a promotion makes no directory.
    pub fn beside(mut self) -> Copies {
        self.evidence = self.directory.join("promotion.evidence.json");
        self.kept = self.directory.join("promotion.measurements.json");
        self
    }

    /// Promotes with the directory of the copies as the root the evidence names its files from.
    pub fn promote(&self, options: &Options) -> Result<Outcome, String> {
        promote::promote_in(&self.directory, options, &mut rename)
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
        assert!(!self.kept.exists(), "{case}: a copy of the measurements");
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

pub fn copies(test: &str) -> Copies {
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
        kept: directory.join("evidence/promotion.measurements.json"),
        shipped,
        directory,
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
    let chosen = CHOSEN.as_str();
    let copies = copies("landing-promote");
    let (product, css) = (read(&copies.product), read(&copies.css));
    let measurements = read(&copies.measurements);
    let outcome = copies
        .promote(&copies.options(chosen))
        .unwrap_or_else(|error| panic!("{error}"));

    // The sources: what the variant's patch writes, applied here on its own.
    let (patched_product, patched_css) = variant(chosen)
        .patch()
        .apply(&product, &css)
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        (patched_product != product, patched_css != css),
        changes(chosen),
        "{chosen} against {}: which of product.json and landing.css it changes",
        space::shipped().id
    );
    assert!(
        patched_product != product || patched_css != css,
        "{chosen} changes neither source"
    );
    assert_eq!(read(&copies.product), patched_product);
    assert_eq!(read(&copies.css), patched_css);
    // And what its axis values say, read from the files.
    let (actions, status_first, properties) = said_by(chosen);
    let written = json(&read(&copies.product));
    let hero = written
        .get("product")
        .and_then(|product| product.get("actions"))
        .expect("/product/actions")
        .list();
    assert_eq!(hero.len(), actions, "{chosen}: the hero's actions");
    assert_eq!(hero[0].get_str("kind"), Some("primary"));
    let first_section = written
        .get("sections")
        .map(Value::list)
        .and_then(|sections| sections.first())
        .and_then(|section| section.get_str("kind"));
    assert_eq!(
        first_section == Some("status"),
        status_first,
        "{chosen}: the first section is {first_section:?}"
    );
    for (property, value) in properties {
        assert_eq!(
            patch::custom_property(&read(&copies.css), ":root", property).as_deref(),
            Some(value),
            "{chosen}: {property}"
        );
    }

    assert_eq!(read(&copies.shipped), format!("{chosen}\n"));
    assert_eq!(
        outcome.written,
        [
            copies.product.clone(),
            copies.css.clone(),
            copies.evidence.clone(),
            copies.kept.clone(),
            copies.shipped.clone()
        ]
    );
    // The measurements the choice was made on: kept beside the evidence, the file itself left.
    assert_eq!(read(&copies.kept), measurements);
    assert_eq!(read(&copies.measurements), measurements);
    assert_eq!(
        tree(&copies.directory).into_keys().collect::<Vec<_>>(),
        [
            "evidence/",
            "evidence/promotion.evidence.json",
            "evidence/promotion.measurements.json",
            "landing.css",
            "measurements.json",
            "product.json",
            "shipped.txt"
        ],
        "the five files and nothing else, no staged file left"
    );

    // The evidence: the committed document is the one of a promotion with the evidence beside
    // the sources, and fresh copies in another directory give the same bytes.
    let beside = self::copies("landing-promote-golden").beside();
    beside
        .promote(&beside.options(chosen))
        .unwrap_or_else(|error| panic!("{error}"));
    let text = read(&beside.evidence);
    let path = fixture("promote.evidence.json");
    if std::env::var_os("UX_SEARCH_BLESS").is_some() {
        std::fs::write(&path, golden(&text)).expect("the golden document");
    }
    assert_eq!(
        golden(&text),
        read(&path),
        "{} is not the evidence of promoting {chosen}; UX_SEARCH_BLESS=1 rewrites it",
        path.display()
    );
    let again = self::copies("landing-promote-again").beside();
    again
        .promote(&again.options(chosen))
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        read(&again.evidence),
        text,
        "a second run, in another directory, writes other bytes"
    );
    assert_eq!(read(&again.product), patched_product);
    assert_eq!(read(&again.css), patched_css);
    // With the evidence in a directory of its own the document differs in one text: the path
    // of the copy of the measurements.
    assert_eq!(
        read(&copies.evidence),
        text.replace(
            "\"value\":\"promotion.measurements.json\"",
            "\"value\":\"evidence/promotion.measurements.json\""
        )
    );
}

#[test]
fn the_evidence_names_the_campaign_the_variant_and_the_sources() {
    let chosen = CHOSEN.as_str();
    let copies = copies("landing-promote-evidence");
    let (product, css) = (read(&copies.product), read(&copies.css));
    copies
        .promote(&copies.options(chosen))
        .unwrap_or_else(|error| panic!("{error}"));
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
    // Each file by its path from the root, and the measurements by the copy the promotion keeps
    // beside the evidence: the file that is still there after the campaign is recorded again.
    for (name, value) in [
        ("measurements", "evidence/promotion.measurements.json"),
        ("product", "product.json"),
        ("css", "landing.css"),
    ] {
        assert_eq!(
            keys(input(name)),
            ["value", "digest", "digest_form"],
            "{name}"
        );
        assert_eq!(input(name).get_str("value"), Some(value), "{name}");
        assert_eq!(input(name).get_str("digest_form"), Some("bytes"), "{name}");
    }
    assert_eq!(
        input("measurements").get_str("digest"),
        Some(record::digest(read(&copies.kept).as_bytes()).as_str()),
        "the digest is the copy's"
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
    assert_eq!(input("variant").get_str("value"), Some(chosen));
    assert_eq!(input("chosen_by").get_str("value"), Some(CHOSEN_BY));
    let shipped = space::shipped().id;
    for (role, id, on_frontier) in [
        ("chosen", chosen, true),
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
            result(&document, &format!("landing.{term}@1"), Some(chosen))
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
        .skip_while(|line| !line.contains(&format!(" {chosen} hard=")))
        .skip(1)
        .take_while(|line| line.starts_with("     "))
        .map(str::trim_start)
        .collect();
    assert!(!block.is_empty(), "the report prints no patch for {chosen}");
    let mut printed = Vec::new();
    let mut after = Vec::new();
    for (name, flag, before, path, changed) in [
        (
            "product.json",
            "product",
            &product,
            &copies.product,
            changes(chosen).0,
        ),
        ("landing.css", "css", &css, &copies.css, changes(chosen).1),
    ] {
        let found = result(&document, "landing.source@1", Some(name));
        assert_eq!(found.get_str("scope"), Some("source"));
        assert_eq!(found.get_str("value_type"), Some("digest"));
        let before = record::digest(before.as_bytes());
        let written = record::digest(read(path).as_bytes());
        assert_eq!(before != written, changed, "{name}: {chosen} changes it");
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
        ..copies.options(CHOSEN.as_str())
    };
    let outcome = copies
        .promote(&options)
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        outcome.written,
        [
            copies.product.clone(),
            copies.css.clone(),
            copies.evidence.clone(),
            copies.kept.clone()
        ]
    );
    assert_eq!(
        read(&copies.shipped),
        format!("{}\n", space::shipped().id),
        "the shipped file is not named, so it stays"
    );
    // And the promotion says so, with the file that names the shipped variant.
    let said: Vec<&String> = outcome
        .notes
        .iter()
        .filter(|note| note.contains("the shipped file was not changed"))
        .collect();
    assert_eq!(said.len(), 1, "{:?}", outcome.notes);
    assert!(
        said[0].contains("--shipped-file")
            && said[0].contains("engine/search/src/landing/shipped.txt")
            && said[0].contains(&space::shipped().id),
        "{said:?}"
    );
    assert_eq!(
        promote::SHIPPED_FILE,
        "engine/search/src/landing/shipped.txt"
    );
    assert_eq!(
        read(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(promote::SHIPPED_FILE)
        ),
        format!("{}\n", space::shipped().id),
        "the file the note names is the one the engine is built with"
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
    let chosen = CHOSEN.as_str();
    let measurements = read(&fixture("measurements.json"));
    let chosen_entry = format!("\"id\": \"{chosen}\"");
    assert_eq!(measurements.matches(&chosen_entry).count(), 1);

    // The record with the chosen variant scrolling sideways at 1440x900 light.
    let mut broken = recorded();
    let at = broken
        .variants
        .iter()
        .position(|(id, _)| id == chosen)
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
            chosen,
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
            chosen,
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
            chosen,
            Some(broken.text()),
            same,
            same,
            &[chosen, "breaks a hard rule", "hard=1"],
        ),
        (
            "not in the measurements",
            chosen,
            Some(without.text()),
            same,
            same,
            &[chosen, "not in the measurements"],
        ),
        (
            "measurements of another format",
            chosen,
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

        let refused = match copies.promote(&copies.options(id)) {
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
    // A variant that has operations on product.json: other actions or another section order.
    let with_operations = on_the_frontier(|variant| changes(&variant.id).0)
        .expect("a frontier variant that changes product.json")
        .id;
    let copies = copies("landing-promote-unpatched");
    let product = read(&copies.product).replace("\n  \"sections\"", "\n\n  \"sections\"");
    assert_ne!(product, read(&copies.product));
    std::fs::write(&copies.product, &product).expect("product.json");
    let css = read(&copies.css);
    let mut recorded = recorded();
    recorded.product_digest = record::digest(product.as_bytes());
    std::fs::write(&copies.measurements, recorded.text()).expect("the measurements");
    let refused = copies
        .promote(&copies.options(&with_operations))
        .expect_err("the patch is refused");
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
        ..copies.options(CHOSEN.as_str())
    };
    let failed = copies
        .promote(&options)
        .expect_err("the evidence cannot be written");
    assert!(failed.contains("promotion.evidence.json"), "{failed}");
    copies.assert_untouched("an evidence path under a file", &product, &css);
}

// All or none.

/// What a failed promotion must leave: the directory of the copies as it was, entry for entry.
fn assert_nothing_changed(
    case: &str,
    copies: &Copies,
    before: &BTreeMap<String, Option<String>>,
    failed: &str,
) {
    assert_eq!(
        &tree(&copies.directory),
        before,
        "{case}: all or none; the promotion failed with: {failed}"
    );
}

/// The move of a staged file to its place fails at the first, second, ... fifth file, after the
/// earlier ones took their place: each of those is put back as it was, the evidence, the copy
/// of the measurements and the directory made for them are removed, and no staged file stays.
#[test]
fn a_move_that_fails_puts_back_every_file_already_moved() {
    let chosen = CHOSEN.as_str();
    for fails_at in 1..=5 {
        let case = format!("the move of file {fails_at}");
        let copies = copies(&format!("landing-promote-rollback-{fails_at}"));
        let before = tree(&copies.directory);
        let mut moves = 0;
        let mut failing = |from: &Path, to: &Path| {
            moves += 1;
            if moves == fails_at {
                Err(std::io::Error::other("the move the test fails"))
            } else {
                std::fs::rename(from, to)
            }
        };
        let failed =
            match promote::promote_in(&copies.directory, &copies.options(chosen), &mut failing) {
                Ok(outcome) => panic!("{case}: promoted, wrote {:?}", outcome.written),
                Err(failed) => failed,
            };
        assert_eq!(moves, fails_at, "{case}: no move after the failed one");
        assert!(
            failed.contains("the move the test fails"),
            "{case}: {failed}"
        );
        let target = [
            &copies.product,
            &copies.css,
            &copies.evidence,
            &copies.kept,
            &copies.shipped,
        ][fails_at - 1];
        assert!(
            failed.contains(&target.display().to_string()),
            "{case}: the failure names {}: {failed}",
            target.display()
        );
        assert!(failed.contains("nothing is changed"), "{case}: {failed}");
        assert_nothing_changed(&case, &copies, &before, &failed);
    }
    // With no failure the same call promotes: the five moves are the whole of it.
    let copies = copies("landing-promote-rollback-none");
    let mut moves = 0;
    let mut counting = |from: &Path, to: &Path| {
        moves += 1;
        std::fs::rename(from, to)
    };
    promote::promote_in(&copies.directory, &copies.options(chosen), &mut counting)
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(moves, 5);
}

/// A shipped file that did not exist, in a directory that did not exist: after a failed move
/// both are gone again, and so is a shipped file put in place before the failure.
#[test]
fn a_failed_promotion_removes_the_files_and_directories_it_made() {
    let chosen = CHOSEN.as_str();
    // The last move fails: the new shipped file's directory was made and is removed.
    let copies = copies("landing-promote-rollback-new-shipped");
    std::fs::remove_file(&copies.shipped).expect("no shipped file yet");
    let before = tree(&copies.directory);
    let options = Options {
        shipped_file: Some(copies.directory.join("new/deeper/shipped.txt")),
        ..copies.options(chosen)
    };
    let mut moves = 0;
    let mut failing = |from: &Path, to: &Path| {
        moves += 1;
        if moves == 5 {
            Err(std::io::Error::other("the move the test fails"))
        } else {
            std::fs::rename(from, to)
        }
    };
    let failed = promote::promote_in(&copies.directory, &options, &mut failing)
        .expect_err("the fifth move fails");
    assert_nothing_changed("a new shipped file", &copies, &before, &failed);

    // The evidence and the copy of the measurements are staged in a directory made for them,
    // then the shipped file cannot be staged: a directory lies where its staged file goes.
    let copies = self::copies("landing-promote-rollback-staging");
    std::fs::create_dir(copies.directory.join(".shipped.txt.promote.tmp"))
        .expect("a directory in the way of the staged shipped file");
    let before = tree(&copies.directory);
    let mut moves = 0;
    let mut counting = |from: &Path, to: &Path| {
        moves += 1;
        std::fs::rename(from, to)
    };
    let failed = promote::promote_in(&copies.directory, &copies.options(chosen), &mut counting)
        .expect_err("the shipped file cannot be staged");
    assert!(failed.contains("shipped.txt"), "{failed}");
    assert_eq!(moves, 0, "no file is moved before all are staged: {failed}");
    assert_nothing_changed(
        "a staged file that cannot be written",
        &copies,
        &before,
        &failed,
    );

    // A shipped file under a file: found before a file is staged.
    let copies = self::copies("landing-promote-rollback-under-a-file");
    let before = tree(&copies.directory);
    let options = Options {
        shipped_file: Some(copies.measurements.join("shipped.txt")),
        ..copies.options(chosen)
    };
    let failed = copies
        .promote(&options)
        .expect_err("the shipped file cannot be written");
    assert!(failed.contains("shipped.txt"), "{failed}");
    assert_nothing_changed("a shipped file under a file", &copies, &before, &failed);

    // A staged file of an earlier run that broke off is no obstacle and does not stay.
    let copies = self::copies("landing-promote-rollback-stale");
    std::fs::write(
        copies.directory.join(".product.json.promote.tmp"),
        "left by a run that broke off",
    )
    .expect("a stale staged file");
    copies
        .promote(&copies.options(chosen))
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(!copies.directory.join(".product.json.promote.tmp").exists());
}

/// A target that is a directory, or has no file name, is refused before a file is staged.
#[test]
fn a_target_that_is_a_directory_is_refused_before_a_file_is_staged() {
    type Case = (&'static str, fn(&Copies) -> Options, &'static str);
    let cases: [Case; 4] = [
        (
            "the evidence is a directory",
            |copies| Options {
                evidence: copies.directory.join("there"),
                ..copies.options(CHOSEN.as_str())
            },
            "is a directory",
        ),
        (
            "the shipped file is a directory",
            |copies| Options {
                shipped_file: Some(copies.directory.join("there")),
                ..copies.options(CHOSEN.as_str())
            },
            "is a directory",
        ),
        (
            "the copy of the measurements would be a directory",
            |copies| Options {
                evidence: copies.directory.join("taken.evidence.json"),
                ..copies.options(CHOSEN.as_str())
            },
            "is a directory",
        ),
        (
            "the shipped file has no name",
            |copies| Options {
                shipped_file: Some(copies.directory.join("there").join("..")),
                ..copies.options(CHOSEN.as_str())
            },
            "not a file",
        ),
    ];
    for (at, (case, options, says)) in cases.into_iter().enumerate() {
        let copies = copies(&format!("landing-promote-directory-{at}"));
        std::fs::create_dir(copies.directory.join("there")).expect("a directory");
        std::fs::create_dir(copies.directory.join("taken.measurements.json"))
            .expect("a directory where the copy of the measurements goes");
        let before = tree(&copies.directory);
        let mut moves = 0;
        let mut counting = |from: &Path, to: &Path| {
            moves += 1;
            std::fs::rename(from, to)
        };
        let refused = promote::promote_in(&copies.directory, &options(&copies), &mut counting)
            .expect_err(case);
        assert!(refused.contains(says), "{case}: {refused}");
        assert_eq!(moves, 0, "{case}: {refused}");
        assert_nothing_changed(case, &copies, &before, &refused);
    }
}

/// One file named for two of the six files of a promotion, by whatever spelling, is refused:
/// the measurements, the two sources, the evidence, the copy of the measurements beside it and
/// the shipped file.
#[test]
fn one_file_named_twice_by_any_spelling_is_refused() {
    let chosen = CHOSEN.as_str();
    // The five paths a caller names, and the same file spelled through `..`.
    let named = |copies: &Copies| {
        [
            copies.measurements.clone(),
            copies.product.clone(),
            copies.css.clone(),
            copies.directory.join("promotion.evidence.json"),
            copies.shipped.clone(),
        ]
    };
    let options = |copies: &Copies, paths: &[PathBuf; 5]| Options {
        measurements: paths[0].clone(),
        product: paths[1].clone(),
        css: paths[2].clone(),
        evidence: paths[3].clone(),
        shipped_file: Some(paths[4].clone()),
        ..copies.options(chosen)
    };
    let again = |copies: &Copies, path: &Path| {
        let directory = copies.directory.file_name().expect("the directory's name");
        copies
            .directory
            .join("..")
            .join(directory)
            .join(".")
            .join(path.file_name().expect("a file name"))
    };
    let roles = [
        "the measurements",
        "product.json",
        "landing.css",
        "the evidence",
        "the shipped file",
    ];
    let mut cases = 0;
    for first in 0..5 {
        for second in first + 1..5 {
            let case = format!("{} named for {}", roles[first], roles[second]);
            let copies = copies(&format!("landing-promote-twice-{first}-{second}"));
            let before = tree(&copies.directory);
            let mut paths = named(&copies);
            paths[second] = again(&copies, &paths[first]);
            let refused = match copies.promote(&options(&copies, &paths)) {
                Ok(outcome) => panic!("{case}: promoted, wrote {:?}", outcome.written),
                Err(refused) => refused,
            };
            assert!(
                refused.contains("named for two of the files of a promotion"),
                "{case}: {refused}"
            );
            assert_nothing_changed(&case, &copies, &before, &refused);
            cases += 1;
        }
    }
    assert_eq!(cases, 10);

    // The copy of the measurements goes beside the evidence: its place named for another file.
    type Other = fn(&Copies) -> Options;
    let more: [(&str, Other); 4] = [
        (
            "the shipped file is the copy of the measurements",
            |copies| Options {
                shipped_file: Some(copies.kept.clone()),
                ..copies.options(CHOSEN.as_str())
            },
        ),
        (
            "the measurements are read where their copy goes",
            |copies| {
                std::fs::create_dir_all(copies.kept.parent().expect("a directory"))
                    .expect("the directory of the evidence");
                std::fs::rename(&copies.measurements, &copies.kept).expect("moved");
                Options {
                    measurements: copies.kept.clone(),
                    ..copies.options(CHOSEN.as_str())
                }
            },
        ),
        (
            "two files in a directory that does not exist yet, one spelled through `..`",
            |copies| Options {
                evidence: copies.directory.join("new/promotion.evidence.json"),
                shipped_file: Some(
                    copies
                        .directory
                        .join("new/other/../promotion.evidence.json"),
                ),
                ..copies.options(CHOSEN.as_str())
            },
        ),
        ("the evidence is a link to product.json", |copies| {
            let link = copies.directory.join("link.evidence.json");
            std::os::unix::fs::symlink(&copies.product, &link).expect("a link");
            Options {
                evidence: link,
                ..copies.options(CHOSEN.as_str())
            }
        }),
    ];
    for (at, (case, options)) in more.into_iter().enumerate() {
        let copies = copies(&format!("landing-promote-twice-more-{at}"));
        let options = options(&copies);
        let before = tree(&copies.directory);
        let refused = match copies.promote(&options) {
            Ok(outcome) => panic!("{case}: promoted, wrote {:?}", outcome.written),
            Err(refused) => refused,
        };
        assert!(
            refused.contains("named for two of the files of a promotion"),
            "{case}: {refused}"
        );
        assert_nothing_changed(case, &copies, &before, &refused);
    }
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
    let off = on_the_frontier(|variant| !space::anchored(variant));
    let on = on_the_frontier(space::anchored);
    assert!(
        off.is_some() || on.is_some(),
        "the frontier holds a variant beside the shipped one"
    );
    if let Some(off) = off {
        let copies = copies("landing-promote-note");
        let outcome = copies
            .promote(&copies.options(&off.id))
            .unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(outcome.notes.len(), 1, "{:?}", outcome.notes);
        assert!(outcome.notes[0].contains("space.rs"), "{:?}", outcome.notes);
    }
    if let Some(on) = on {
        let copies = copies("landing-promote-no-note");
        let outcome = copies
            .promote(&copies.options(&on.id))
            .unwrap_or_else(|error| panic!("{error}"));
        assert!(outcome.notes.is_empty(), "{:?}", outcome.notes);
    }
}

// The command.

/// `ux-search landing promote` run in the directory of the copies, every file by its path from
/// there, as the command is run in the repository's root.
fn ux_search(copies: &Copies, id: &str, more: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ux-search"))
        .current_dir(&copies.directory)
        .args([
            "landing",
            "promote",
            "--measurements",
            "measurements.json",
            "--variant",
            id,
            "--product",
            "product.json",
            "--css",
            "landing.css",
            "--evidence",
            "evidence/promotion.evidence.json",
        ])
        .args(more)
        .output()
        .expect("ux-search runs")
}

#[test]
fn landing_promote_exits_0_and_says_what_it_wrote() {
    let chosen = CHOSEN.as_str();
    let copies = copies("landing-promote-command");
    let output = ux_search(
        &copies,
        chosen,
        &["--chosen-by", CHOSEN_BY, "--shipped-file", "shipped.txt"],
    );
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let ranked = report::rank(&recorded());
    let rank = ranked
        .ranking
        .iter()
        .position(|&index| ranked.scored[index].id == chosen)
        .expect("ranked")
        + 1;
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!(
            "promoted {chosen}: rank {rank} of 144, on the frontier of {}\n",
            ranked.frontier.len()
        )
    );
    let said = String::from_utf8_lossy(&output.stderr);
    let library = self::copies("landing-promote-command-library");
    let outcome = library
        .promote(&library.options(chosen))
        .unwrap_or_else(|error| panic!("{error}"));
    let mut lines: Vec<String> = [
        "product.json",
        "landing.css",
        "evidence/promotion.evidence.json",
        "evidence/promotion.measurements.json",
        "shipped.txt",
    ]
    .iter()
    .map(|path| format!("ux-search: wrote {path}"))
    .collect();
    lines.extend(
        outcome
            .notes
            .iter()
            .map(|note| format!("ux-search: note: {note}")),
    );
    assert_eq!(
        said.lines().collect::<Vec<_>>(),
        lines,
        "the five files as they were named, then the notes"
    );
    assert!(!said.contains("the shipped file was not changed"), "{said}");
    for (written, expected) in [
        (&copies.product, &library.product),
        (&copies.css, &library.css),
        (&copies.evidence, &library.evidence),
        (&copies.kept, &library.kept),
        (&copies.shipped, &library.shipped),
    ] {
        assert_eq!(read(written), read(expected), "{}", written.display());
    }

    // Without --shipped-file the command says, in one line, that the shipped file stays and
    // which file names the shipped variant.
    let copies = self::copies("landing-promote-command-no-shipped-file");
    let output = ux_search(&copies, chosen, &[]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let said = String::from_utf8_lossy(&output.stderr);
    let about: Vec<&str> = said
        .lines()
        .filter(|line| line.contains("shipped file"))
        .collect();
    assert_eq!(
        about,
        [format!(
            "ux-search: note: the shipped file was not changed (no --shipped-file): \
             engine/search/src/landing/shipped.txt still names `{}` as the shipped variant",
            space::shipped().id
        )],
        "{said}"
    );
    assert_eq!(
        read(&copies.shipped),
        format!("{}\n", space::shipped().id),
        "the shipped file of the copies stays"
    );
}

/// The usage names every argument of `landing promote`; `--chosen-by` takes who chose, the
/// date being the commit's.
#[test]
fn the_usage_names_the_arguments_of_landing_promote() {
    let help = Command::new(env!("CARGO_BIN_EXE_ux-search"))
        .arg("--help")
        .output()
        .expect("ux-search runs");
    assert_eq!(help.status.code(), Some(0));
    let usage = String::from_utf8_lossy(&help.stdout);
    assert!(
        usage.contains(
            "ux-search landing promote --measurements <measurements.json> --variant <variant id> \
             --product <product.json> --css <landing.css> --evidence <evidence.json> \
             [--chosen-by \"<who chose>\"] [--shipped-file <shipped.txt>]\n"
        ),
        "{usage}"
    );
}

/// The evidence names a file by its path from the working directory, with `/`, when the file
/// lies under it, however the path was written; a file outside it has no path in the document,
/// its digest identifies it. So the document holds no absolute path, and the command run in
/// the repository's root names the files of the tree as the tree does.
#[test]
fn the_evidence_names_each_file_by_its_path_from_the_working_directory() {
    let chosen = CHOSEN.as_str();
    let run = |directory: &Path, paths: [&str; 4]| {
        let output = Command::new(env!("CARGO_BIN_EXE_ux-search"))
            .current_dir(directory)
            .args(["landing", "promote", "--variant", chosen])
            .args(["--measurements", paths[0], "--product", paths[1]])
            .args(["--css", paths[2], "--evidence", paths[3]])
            .output()
            .expect("ux-search runs");
        assert_eq!(output.status.code(), Some(0), "{paths:?}: {output:?}");
    };
    let values = |evidence: &Path| -> Vec<Option<String>> {
        let document = json(&read(evidence));
        check_shape(&document);
        ["measurements", "product", "css"]
            .map(|name| {
                let input = self::evidence(&document)
                    .get("inputs")
                    .and_then(|inputs| inputs.get(name))
                    .unwrap_or_else(|| panic!("no input {name}"));
                assert!(is_digest(input.get_str("digest").expect("a digest")));
                input.get_str("value").map(str::to_owned)
            })
            .into()
    };
    let under = |measurements: &str| -> Vec<Option<String>> {
        [measurements, "product.json", "landing.css"]
            .map(|path| Some(path.to_owned()))
            .into()
    };

    // As written, when the path is relative and stays under the working directory.
    let copies = copies("landing-promote-paths-relative");
    run(
        &copies.directory,
        [
            "measurements.json",
            "product.json",
            "landing.css",
            "evidence/promotion.evidence.json",
        ],
    );
    assert_eq!(
        values(&copies.evidence),
        under("evidence/promotion.measurements.json")
    );
    let relative = read(&copies.evidence);

    // The same paths written with `.` and `..`, and as absolute paths: the same document.
    let copies = self::copies("landing-promote-paths-spelled");
    std::fs::create_dir(copies.directory.join("elsewhere")).expect("a directory to step out of");
    run(
        &copies.directory,
        [
            "./measurements.json",
            "elsewhere/../product.json",
            "./././landing.css",
            "./evidence/promotion.evidence.json",
        ],
    );
    assert_eq!(read(&copies.evidence), relative);
    let copies = self::copies("landing-promote-paths-absolute");
    let absolute = |path: &Path| path.to_str().expect("a UTF-8 path").to_owned();
    run(
        &copies.directory,
        [
            &absolute(&copies.measurements),
            &absolute(&copies.product),
            &absolute(&copies.css),
            &absolute(&copies.evidence),
        ],
    );
    assert_eq!(read(&copies.evidence), relative);

    // Files outside the working directory, by `..` and by absolute paths: no `value`.
    for (case, spelled) in [("dots", false), ("absolute", true)] {
        let copies = self::copies(&format!("landing-promote-paths-outside-{case}"));
        let elsewhere = copies.directory.join("elsewhere");
        std::fs::create_dir(&elsewhere).expect("a working directory beside the files");
        let paths = if spelled {
            [
                absolute(&copies.measurements),
                absolute(&copies.product),
                absolute(&copies.css),
                absolute(&copies.evidence),
            ]
        } else {
            [
                "../measurements.json".to_owned(),
                "../product.json".to_owned(),
                "../landing.css".to_owned(),
                "../evidence/promotion.evidence.json".to_owned(),
            ]
        };
        run(&elsewhere, paths.each_ref().map(String::as_str));
        assert_eq!(values(&copies.evidence), [None, None, None], "{case}");
        let text = read(&copies.evidence);
        assert!(
            !text.contains("elsewhere") && !text.contains("landing-promote-paths"),
            "{case}: {text}"
        );
        // And only the paths are missing.
        let mut without = relative.clone();
        for value in [
            "evidence/promotion.measurements.json",
            "product.json",
            "landing.css",
        ] {
            let member = format!("\"value\":\"{value}\",");
            assert_eq!(without.matches(&member).count(), 1, "{member}");
            without = without.replace(&member, "");
        }
        assert_eq!(text, without, "{case}");
    }

    // A sub-directory of the working directory is part of the path.
    let copies = self::copies("landing-promote-paths-above");
    let above = copies
        .directory
        .parent()
        .expect("the directory above the copies");
    let name = copies
        .directory
        .file_name()
        .and_then(|name| name.to_str())
        .expect("the directory's name");
    run(
        above,
        [
            &format!("{name}/measurements.json"),
            &format!("{name}/product.json"),
            &format!("{name}/landing.css"),
            &format!("{name}/evidence/promotion.evidence.json"),
        ],
    );
    assert_eq!(
        values(&copies.evidence),
        [
            Some(format!("{name}/evidence/promotion.measurements.json")),
            Some(format!("{name}/product.json")),
            Some(format!("{name}/landing.css")),
        ]
    );
}

#[test]
fn landing_promote_exits_1_on_a_refusal_and_2_on_a_wrong_invocation() {
    let chosen = CHOSEN.as_str();
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
        &["--variant", chosen],
        &["--param", "a=b"],
    ] {
        let output = ux_search(&copies, chosen, more);
        assert_eq!(output.status.code(), Some(2), "{more:?}: {output:?}");
        assert!(output.stdout.is_empty(), "{more:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("usage:"),
            "{more:?}"
        );
        copies.assert_untouched("a wrong invocation", &product, &css);
    }
    let output = Command::new(env!("CARGO_BIN_EXE_ux-search"))
        .args(["landing", "promote", "--variant", chosen])
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
