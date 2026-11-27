//! `ux-search --version` and `-V`: the version on the first line, `ux-search <version>`, which a
//! release (`tools/release-versions.sh`) and a recorded campaign's document read; then one line
//! per kind of campaign file with the formats the engine writes and the ones it reads, so two
//! engines whose formats differ answer differently. Each format is taken from its constant in the
//! source here, never typed out.

use std::process::Command;

use ux_search::campaign::{declaration, events, record};

fn ask(flag: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_ux-search"))
        .arg(flag)
        .output()
        .unwrap_or_else(|error| panic!("ux-search {flag}: {error}"));
    assert!(
        output.status.success(),
        "ux-search {flag}: {}",
        output.status
    );
    assert!(output.stderr.is_empty(), "ux-search {flag} wrote to stderr");
    String::from_utf8(output.stdout).expect("UTF-8")
}

#[test]
fn the_first_line_is_the_name_and_the_version() {
    for flag in ["--version", "-V"] {
        let answer = ask(flag);
        assert_eq!(
            answer.split_inclusive('\n').next(),
            Some(format!("ux-search {}\n", env!("CARGO_PKG_VERSION")).as_str()),
            "{flag}: {answer}"
        );
    }
}

#[test]
fn the_further_lines_name_the_formats_written_and_read() {
    let expected = format!(
        "ux-search {}\n\
         declaration: reads {}, {}\n\
         measurements: writes {}; reads {}, {}\n\
         renders: writes {}\n\
         events: writes {}, {}\n",
        env!("CARGO_PKG_VERSION"),
        declaration::FORMAT,
        declaration::FORMAT_1,
        record::FORMAT,
        record::FORMAT,
        record::FORMAT_BEFORE_THE_ROWS,
        record::RENDERS_FORMAT,
        events::CANDIDATES_FILE,
        events::PHASE_FILE,
    );
    for flag in ["--version", "-V"] {
        assert_eq!(ask(flag), expected, "{flag}");
    }
}
