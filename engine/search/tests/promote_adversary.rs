//! Adversary cases against `ux-search landing promote` (`src/landing/promote.rs`): the claim
//! "the four files are written all or none, a refusal writes nothing", and the lines of
//! `promote.rs` the cases of `landing_promote` do not hold. No test starts a browser or touches
//! `website/`: each promotes on copies of `fixtures/docs-landing/` under cargo's scratch
//! directory for integration tests.
//!
//! Every case compares the whole directory of the copies before and after ([`Scene::changes`]):
//! a file added, changed or removed, a directory or a staged file left behind are all seen.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use ux_search::landing::patch::{self, Value};
use ux_search::landing::promote::{self, Options};
use ux_search::landing::record::{self, Recorded};
use ux_search::landing::space;

/// The first variant of the recorded campaign's frontier; not the page as shipped.
const CHOSEN: &str = "docs-system.centred.1+0.features-first.d85.h85";
/// A variant off the frontier of the recorded campaign.
const OFF_THE_FRONTIER: &str = "docs-system.left.1+3.features-first.d115.h100";

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/docs-landing")
        .join(name)
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// Every entry under a directory: its path from there, and the digest of a file's bytes (a
/// directory has none).
type Snapshot = BTreeMap<String, Option<String>>;

fn walk(root: &Path, at: &Path, out: &mut Snapshot) {
    for entry in std::fs::read_dir(at).unwrap_or_else(|error| panic!("{}: {error}", at.display())) {
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
            let bytes =
                std::fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            out.insert(name, Some(record::digest(&bytes)));
        }
    }
}

/// Copies of the fixture's measurements and sources and a shipped file, in a directory of the
/// test's own, and that directory as it was when the copies were made.
struct Scene {
    directory: PathBuf,
    before: Snapshot,
}

impl Scene {
    fn path(&self, name: &str) -> PathBuf {
        self.directory.join(name)
    }

    /// The promotion of `id` with every file in the directory of the copies, so a promotion
    /// creates no directory.
    fn options(&self, id: &str) -> Options {
        Options {
            measurements: self.path("measurements.json"),
            variant: id.to_owned(),
            product: self.path("product.json"),
            css: self.path("landing.css"),
            evidence: self.path("promotion.evidence.json"),
            chosen_by: Some("the owner, 2026-11-26".to_owned()),
            shipped_file: Some(self.path("shipped.txt")),
        }
    }

    fn snapshot(&self) -> Snapshot {
        let mut out = Snapshot::new();
        walk(&self.directory, &self.directory, &mut out);
        out
    }

    /// Takes the directory as it is now for the state to compare with.
    fn settle(&mut self) {
        self.before = self.snapshot();
    }

    /// What differs from the settled state, one line per entry: empty when nothing was written.
    fn changes(&self) -> Vec<String> {
        let after = self.snapshot();
        let mut out = Vec::new();
        for (name, digest) in &after {
            match self.before.get(name) {
                None => out.push(format!("added {name}")),
                Some(was) if was != digest => out.push(format!("changed {name}")),
                Some(_) => {}
            }
        }
        for name in self.before.keys() {
            if !after.contains_key(name) {
                out.push(format!("removed {name}"));
            }
        }
        out
    }
}

fn scene(test: &str) -> Scene {
    let directory = crate::state_dir(&format!("promote-adversary-{test}"));
    if directory.exists() {
        std::fs::remove_dir_all(&directory).expect("the last run's copies removed");
    }
    std::fs::create_dir_all(&directory).expect("a directory for the copies");
    for name in ["measurements.json", "product.json", "landing.css"] {
        std::fs::copy(fixture(name), directory.join(name))
            .unwrap_or_else(|error| panic!("{name}: {error}"));
    }
    std::fs::write(
        directory.join("shipped.txt"),
        format!("{}\n", space::shipped().id),
    )
    .expect("the shipped file");
    let mut scene = Scene {
        directory,
        before: Snapshot::new(),
    };
    scene.settle();
    scene
}

fn recorded() -> Recorded {
    Recorded::parse(&read(&fixture("measurements.json"))).unwrap_or_else(|error| panic!("{error}"))
}

const NOTHING: [&str; 0] = [];

// All or none: red on the tree as handed over.

/// `--evidence` names a directory that exists (`docs/promotions` for
/// `docs/promotions/<date>-docs-landing.evidence.json`). The document says the four files are
/// written "all of them or none". Staging succeeds beside the directory, the two sources take
/// their place, and the third rename fails: the sources are the promoted ones, there is no
/// evidence, the shipped file is the old one, and a second run is refused because the sources
/// are no longer the ones the measurements name.
#[test]
fn an_evidence_path_that_is_a_directory_changes_no_source() {
    let mut scene = scene("evidence-directory");
    std::fs::create_dir(scene.path("promotions")).expect("a directory where the evidence goes");
    scene.settle();
    let options = Options {
        evidence: scene.path("promotions"),
        ..scene.options(CHOSEN)
    };
    let failed = match promote::promote(&options) {
        Ok(outcome) => panic!("promoted, wrote {:?}", outcome.written),
        Err(failed) => failed,
    };
    assert_eq!(
        scene.changes(),
        NOTHING,
        "all or none; the promotion failed with: {failed}"
    );
}

/// The same for `--shipped-file`: the fourth rename fails after three files took their place.
#[test]
fn a_shipped_file_path_that_is_a_directory_changes_no_other_file() {
    let mut scene = scene("shipped-directory");
    std::fs::create_dir(scene.path("landing")).expect("a directory where the shipped file goes");
    scene.settle();
    let options = Options {
        shipped_file: Some(scene.path("landing")),
        ..scene.options(CHOSEN)
    };
    let failed = match promote::promote(&options) {
        Ok(outcome) => panic!("promoted, wrote {:?}", outcome.written),
        Err(failed) => failed,
    };
    assert_eq!(
        scene.changes(),
        NOTHING,
        "all or none; the promotion failed with: {failed}"
    );
}

/// `promote` refuses a path "named for two of the files of a promotion", but compares the
/// paths as written. `product.json` named again for the evidence through `..` passes that
/// check; both stage to one file, and `product.json` ends up holding the evidence document.
#[test]
fn a_source_named_again_for_the_evidence_by_another_spelling_is_refused() {
    let scene = scene("evidence-is-the-product");
    let name = scene.directory.file_name().expect("the directory's name");
    let again = scene.directory.join("..").join(name).join("product.json");
    assert_eq!(
        std::fs::canonicalize(&again).expect("the other spelling resolves"),
        std::fs::canonicalize(scene.path("product.json")).expect("product.json resolves"),
        "the two spellings name one file"
    );
    let options = Options {
        evidence: again,
        ..scene.options(CHOSEN)
    };
    let result = promote::promote(&options);
    assert_eq!(
        scene.changes(),
        NOTHING,
        "one file named for a source and for the evidence; the promotion returned {result:?}"
    );
    assert!(result.is_err(), "{result:?}");
}

// Lines of `promote.rs` no case of `landing_promote` holds: green on the tree as handed over,
// red on the mutant each one names.

/// One case of a path named twice: what it is, and the options that name it.
type NamedTwice = (&'static str, fn(&Scene) -> Options);

/// Mutant: `promote`, `let twice = ...` taken out of `if twice || ...` (the check on a path
/// named twice). No case of `landing_promote` names a path twice.
#[test]
fn a_path_named_for_two_files_of_a_promotion_is_refused() {
    let scene = scene("named-twice");
    let cases: [NamedTwice; 3] = [
        ("the evidence is product.json", |scene| Options {
            evidence: scene.path("product.json"),
            ..scene.options(CHOSEN)
        }),
        ("the evidence is the shipped file", |scene| Options {
            shipped_file: Some(scene.path("promotion.evidence.json")),
            ..scene.options(CHOSEN)
        }),
        ("the shipped file is landing.css", |scene| Options {
            shipped_file: Some(scene.path("landing.css")),
            ..scene.options(CHOSEN)
        }),
    ];
    for (case, options) in cases {
        let refused = promote::promote(&options(&scene)).expect_err(case);
        assert!(
            refused.contains("named for two of the files of a promotion"),
            "{case}: {refused}"
        );
        assert_eq!(scene.changes(), NOTHING, "{case}: {refused}");
    }
}

/// Mutant: `promote`, `|| *path == options.measurements` taken out. The promotion then
/// succeeds and the measurements file holds the evidence document.
#[test]
fn the_measurements_file_named_for_the_evidence_is_refused() {
    let scene = scene("evidence-is-the-measurements");
    let options = Options {
        evidence: scene.path("measurements.json"),
        ..scene.options(CHOSEN)
    };
    let refused = promote::promote(&options).expect_err("the measurements would be overwritten");
    assert!(
        refused.contains("named for two of the files of a promotion"),
        "{refused}"
    );
    assert_eq!(scene.changes(), NOTHING, "{refused}");
}

/// Mutant: `write_all`, `if path.file_name().is_none()` written `if false`. The evidence is
/// then staged in the directory above and the third rename fails after two sources took
/// their place.
#[test]
fn an_evidence_path_without_a_file_name_is_refused_before_a_source_changes() {
    let scene = scene("evidence-without-a-name");
    let options = Options {
        evidence: scene.directory.join(".."),
        ..scene.options(CHOSEN)
    };
    let refused = promote::promote(&options).expect_err("`..` is no file");
    assert!(refused.contains("not a file"), "{refused}");
    assert_eq!(scene.changes(), NOTHING, "{refused}");
}

/// Mutant: `document`, the filter of `sound` written `|_| true` (or `failed` written 0): in
/// the recorded campaign every variant has `hard` = 0, so `passed` is 144 either way. Here two
/// variants off the frontier break a hard rule, one of them in the last environment only.
#[test]
fn the_coverage_counts_the_variants_that_break_a_hard_rule() {
    let mut scene = scene("coverage");
    let mut broken = recorded();
    let at = |id: &str| {
        broken
            .variants
            .iter()
            .position(|(recorded, _)| recorded == id)
            .unwrap_or_else(|| panic!("{id} is not recorded"))
    };
    let (first, second) = (
        at(OFF_THE_FRONTIER),
        at("studio.left.1+3.status-first.d115.h100"),
    );
    broken.variants[first].1[0].h_scroll = true;
    broken.variants[second].1[5].unnamed = 2;
    std::fs::write(scene.path("measurements.json"), broken.text()).expect("the measurements");
    scene.settle();

    promote::promote(&scene.options(CHOSEN)).unwrap_or_else(|error| panic!("{error}"));
    let document = patch::parse(&read(&scene.path("promotion.evidence.json")))
        .unwrap_or_else(|error| panic!("{error}"));
    let coverage = document
        .get("evidence")
        .and_then(|evidence| evidence.get("coverage"))
        .expect("evidence.coverage");
    let counts: Vec<(&str, Option<&Value>)> = ["executed", "passed", "failed"]
        .map(|key| (key, coverage.get(key)))
        .into();
    assert_eq!(
        counts,
        [
            ("executed", Some(&Value::Int(144))),
            ("passed", Some(&Value::Int(142))),
            ("failed", Some(&Value::Int(2)))
        ]
    );
}

// Refusals the brief of the attack names; green on the tree as handed over.

/// An id that differs from a variant's only in case or white space, a variant whose one broken
/// hard rule is in the last environment, and records that are not one render per environment
/// of the 144 variants in their order: each is refused and nothing is written.
#[test]
fn near_ids_and_records_that_are_not_the_campaign_are_refused() {
    let last_environment = {
        let mut record = recorded();
        let at = record
            .variants
            .iter()
            .position(|(id, _)| id == CHOSEN)
            .expect("the chosen variant is recorded");
        record.variants[at].1[5].contrast_failures = 1;
        record
    };
    let a_render_twice = {
        let mut record = recorded();
        let render = record.variants[7].1[0];
        record.variants[7].1.push(render);
        record
    };
    let two_variants_swapped = {
        let mut record = recorded();
        record.variants.swap(3, 4);
        record
    };
    let a_variant_twice = {
        let mut record = recorded();
        record.variants[1] = record.variants[0].clone();
        record
    };
    let upper = CHOSEN.to_uppercase();
    let cases: [(&str, String, Option<Recorded>, &str); 8] = [
        ("upper case", upper, None, "no variant of the space"),
        (
            "one axis in upper case",
            CHOSEN.replace("centred", "Centred"),
            None,
            "no variant of the space",
        ),
        (
            "a space after the id",
            format!("{CHOSEN} "),
            None,
            "no variant of the space",
        ),
        (
            "a newline after the id",
            format!("{CHOSEN}\n"),
            None,
            "no variant of the space",
        ),
        (
            "a hard rule broken in the last environment only",
            CHOSEN.to_owned(),
            Some(last_environment),
            "breaks a hard rule: hard=1 ",
        ),
        (
            "a render recorded twice",
            CHOSEN.to_owned(),
            Some(a_render_twice),
            "7 renders, not one per environment",
        ),
        (
            "two variants swapped",
            CHOSEN.to_owned(),
            Some(two_variants_swapped),
            "where the space has",
        ),
        (
            "a variant recorded twice",
            CHOSEN.to_owned(),
            Some(a_variant_twice),
            "where the space has",
        ),
    ];
    for (at, (case, id, record, says)) in cases.into_iter().enumerate() {
        let mut scene = scene(&format!("refused-{at}"));
        if let Some(record) = record {
            std::fs::write(scene.path("measurements.json"), record.text())
                .expect("the measurements");
            scene.settle();
        }
        let refused = match promote::promote(&scene.options(&id)) {
            Ok(outcome) => panic!("{case}: promoted, wrote {:?}", outcome.written),
            Err(refused) => refused,
        };
        assert!(refused.contains(says), "{case}: {refused}");
        assert_eq!(scene.changes(), NOTHING, "{case}: {refused}");
    }
}

/// The command run in the directory of the copies with bare file names, another locale and
/// another time zone writes the committed evidence document byte for byte: no path but the
/// names given, no clock and no locale enters it. The fifth file is the copy of the measurements
/// a promotion keeps beside the evidence.
#[test]
fn the_evidence_is_the_same_bytes_from_bare_names_in_another_locale() {
    let scene = scene("bare-names");
    let output = Command::new(env!("CARGO_BIN_EXE_ux-search"))
        .current_dir(&scene.directory)
        .env("LC_ALL", "de_DE.UTF-8")
        .env("LANG", "de_DE.UTF-8")
        .env("TZ", "Asia/Tokyo")
        .args([
            "landing",
            "promote",
            "--measurements",
            "measurements.json",
            "--variant",
            CHOSEN,
            "--product",
            "product.json",
            "--css",
            "landing.css",
            "--evidence",
            "promotion.evidence.json",
            "--chosen-by",
            "the owner, 2026-11-26",
            "--shipped-file",
            "shipped.txt",
        ])
        .output()
        .expect("ux-search runs");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let golden = read(&fixture("promote.evidence.json")).replace(
        "\"ux-search\":\"{version}\"",
        &format!("\"ux-search\":\"{}\"", env!("CARGO_PKG_VERSION")),
    );
    assert_eq!(read(&scene.path("promotion.evidence.json")), golden);
    assert_eq!(
        scene.changes(),
        [
            "changed landing.css",
            "changed product.json",
            "added promotion.evidence.json",
            "added promotion.measurements.json",
            "changed shipped.txt"
        ],
        "the five files and nothing else, no staged file left"
    );
    assert_eq!(
        read(&scene.path("promotion.measurements.json")),
        read(&fixture("measurements.json")),
        "the copy of the measurements beside the evidence"
    );
}
