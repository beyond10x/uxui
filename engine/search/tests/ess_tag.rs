//! The ESS release `ux-search` builds on (`story:track-ess-gaps`): [`ux_search::ESS_TAG`] is the
//! one name of it in the source, and the manifest, the lock file, the reports the engine prints,
//! the fixture's provenance and the repository's instructions name the same tag. A bump that
//! leaves one of them behind fails here, naming the file.

use std::path::Path;

use ux_search::ESS_TAG;

/// ESS's repository as the manifest and the lock file write it.
const ESS: &str = "https://github.com/beyond10x/ess";

fn read(path: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// The text between `open` and the next `"` on a line.
fn quoted<'a>(line: &'a str, open: &str) -> Option<&'a str> {
    let rest = &line[line.find(open)? + open.len()..];
    rest.split('"').next()
}

#[test]
fn the_manifest_takes_every_ess_crate_at_the_tag_the_engine_names() {
    let manifest = read("Cargo.toml");
    let tags: Vec<(&str, &str)> = manifest
        .lines()
        .filter(|line| line.contains(ESS))
        .map(|line| {
            let name = line.split(' ').next().unwrap_or(line);
            let tag = quoted(line, "tag = \"")
                .unwrap_or_else(|| panic!("`{line}` names ESS without a tag"));
            (name, tag)
        })
        .collect();
    assert_eq!(
        tags,
        [
            ("ess-ui", ESS_TAG),
            ("ess-ui-check", ESS_TAG),
            ("ess-ui-tui", ESS_TAG)
        ],
        "engine/search/Cargo.toml against ux_search::ESS_TAG"
    );
}

#[test]
fn the_lock_file_holds_every_ess_crate_at_that_tag() {
    let lock = read("Cargo.lock");
    let sources: Vec<&str> = lock
        .lines()
        .filter(|line| line.starts_with("source = ") && line.contains(ESS))
        .collect();
    assert!(
        sources.len() >= 3,
        "the lock file names {} packages from ESS",
        sources.len()
    );
    let wanted = format!("source = \"git+{ESS}?tag={ESS_TAG}#");
    for source in sources {
        assert!(
            source.starts_with(&wanted),
            "engine/search/Cargo.lock: `{source}` is not at tag {ESS_TAG}"
        );
    }
}

#[test]
fn the_reports_name_the_tag_they_were_drawn_on() {
    let wanted = format!("the terminal renderer of ESS {ESS_TAG}, in process");
    for golden in [
        "tests/fixtures/partner-portal.frontier.txt",
        "tests/fixtures/studio.frontier.txt",
    ] {
        let text = read(golden);
        let first = text.lines().next().unwrap_or_default();
        assert!(first.ends_with(&wanted), "{golden}: `{first}`");
    }
    assert!(
        ux_search::cost::header().ends_with(&wanted),
        "{}",
        ux_search::cost::header()
    );
}

/// The texts a document writes between backquotes, in its order.
fn quoted_texts(text: &str) -> Vec<&str> {
    text.split('`').skip(1).step_by(2).collect()
}

/// Whether a text reads as a release: three numbers with dots, as `0.99.5`.
fn is_release(text: &str) -> bool {
    let parts: Vec<&str> = text.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
}

/// Whether a text reads as a commit id: 40 hex digits.
fn is_commit(text: &str) -> bool {
    text.len() == 40 && text.bytes().all(|b| b.is_ascii_hexdigit())
}

/// What is wrong with the provenance a fixture's README gives, against the tag and the commit
/// the lock file holds: empty when it names that tag, that commit, and no other release or
/// commit beside them.
fn provenance_faults(readme: &str, tag: &str, commit: &str) -> Vec<String> {
    let mut faults = Vec::new();
    let wanted = format!("`examples/partner-portal/` at tag `{tag}`");
    if !readme.contains(&wanted) {
        faults.push(format!("does not say: {wanted}"));
    }
    let texts = quoted_texts(readme);
    for release in texts.iter().filter(|text| is_release(text)) {
        if *release != tag {
            faults.push(format!("names the release `{release}`, not `{tag}`"));
        }
    }
    let commits: Vec<&str> = texts
        .iter()
        .copied()
        .filter(|text| is_commit(text))
        .collect();
    if commits.is_empty() {
        faults.push(format!("names no commit; the lock file holds `{commit}`"));
    }
    for named in commits {
        if named != commit {
            faults.push(format!(
                "names the commit `{named}`, the lock file holds `{commit}`"
            ));
        }
    }
    faults
}

/// The one commit the lock file holds every ESS crate at.
fn locked_commit() -> String {
    let lock = read("Cargo.lock");
    let wanted = format!("source = \"git+{ESS}?tag={ESS_TAG}#");
    let mut commits: Vec<&str> = lock
        .lines()
        .filter_map(|line| line.strip_prefix(&wanted))
        .map(|rest| rest.trim_end_matches('"'))
        .collect();
    commits.sort_unstable();
    commits.dedup();
    assert_eq!(
        commits.len(),
        1,
        "engine/search/Cargo.lock holds the ESS crates at {commits:?}"
    );
    assert!(is_commit(commits[0]), "{}", commits[0]);
    commits[0].to_owned()
}

#[test]
fn the_fixture_names_that_tag_and_the_commit_the_lock_file_holds() {
    let file = "tests/fixtures/partner-portal/README.md";
    let (readme, commit) = (read(file), locked_commit());
    assert_eq!(
        provenance_faults(&readme, ESS_TAG, &commit),
        Vec::<String>::new(),
        "{file}"
    );

    // The check itself: each stale mention is a fault, one for one.
    let stale_tag = readme.replacen(&format!("at tag `{ESS_TAG}`"), "at tag `0.0.1`", 1);
    assert_ne!(stale_tag, readme);
    assert_eq!(
        provenance_faults(&stale_tag, ESS_TAG, &commit).len(),
        2,
        "the wanted sentence is gone and a stale release stands"
    );
    let second_tag = format!("{readme}\nThe files were copied at tag `0.0.1`.\n");
    assert_eq!(
        provenance_faults(&second_tag, ESS_TAG, &commit),
        [format!("names the release `0.0.1`, not `{ESS_TAG}`")],
        "a second, stale mention beside the right one"
    );
    let other = "0123456789abcdef0123456789abcdef01234567";
    let stale_commit = readme.replace(&commit, other);
    assert_eq!(
        provenance_faults(&stale_commit, ESS_TAG, &commit),
        [format!(
            "names the commit `{other}`, the lock file holds `{commit}`"
        )]
    );
    let no_commit = readme.replace(&format!("`{commit}`"), "an earlier one");
    assert_eq!(
        provenance_faults(&no_commit, ESS_TAG, &commit),
        [format!("names no commit; the lock file holds `{commit}`")]
    );
}

#[test]
fn the_instructions_name_that_tag_wherever_they_name_one() {
    let file = "../../AGENTS.md";
    let instructions = read(file);
    let named: Vec<&str> = instructions
        .split("`github.com/beyond10x/ess` tag `")
        .skip(1)
        .map(|rest| rest.split('`').next().unwrap_or_default())
        .collect();
    assert!(
        !named.is_empty(),
        "{file} does not say: `github.com/beyond10x/ess` tag `{ESS_TAG}`"
    );
    for tag in named {
        assert_eq!(tag, ESS_TAG, "{file} names ESS at another tag");
    }
}
