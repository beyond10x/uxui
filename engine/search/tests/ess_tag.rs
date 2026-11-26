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

#[test]
fn the_fixture_and_the_instructions_name_that_tag() {
    for (file, wanted) in [
        (
            "tests/fixtures/partner-portal/README.md",
            format!("`examples/partner-portal/` at tag `{ESS_TAG}`"),
        ),
        (
            "../../AGENTS.md",
            format!("`github.com/beyond10x/ess` tag `{ESS_TAG}`"),
        ),
    ] {
        assert!(
            read(file).contains(&wanted),
            "{file} does not say: {wanted}"
        );
    }
}
