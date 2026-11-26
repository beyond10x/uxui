//! The one integration-test binary of ux-browser. No test starts a browser or opens a socket: the
//! WebSocket framing runs over an in-memory stream, the DevTools-protocol client over a scripted
//! transport, and every check's computation over geometry recorded from a real run
//! (`fixtures/`, written by `ux-browser checks --capture` and `ux-browser render --capture`). The
//! command-line cases run the binary only on invocations it refuses before it starts a browser, and
//! the start of Chromium runs a shell script that stands in for it (`chrome`).

mod always_visible;
mod assessor;
mod cdp;
mod checks;
mod chrome;
mod clock;
mod codec;
mod evidence;
mod journey;
mod json;
mod keyboard;
mod layout;
mod pool;
mod ready;
mod render;
mod target;
mod timing;
mod values;
mod version;
mod ws;

use std::path::{Path, PathBuf};
use uxui_doc::yaml::Value;

pub fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

pub fn fixture_json(name: &str) -> Value {
    let text = std::fs::read_to_string(fixture(name)).expect("fixture is readable");
    ux_browser::json::parse(&text).expect("fixture is JSON")
}

/// Writes `text` as an executable at `path`, for a test that runs it. The test process never
/// opens `path` for writing: the text goes to `path.src`, and `install` copies it in its own
/// process. A file this process held open for writing could be inherited by a child that another
/// test thread forks at that moment, and running the file then fails with ETXTBSY ("Text file
/// busy"; forge run 45568883701, `chrome::chromium_starts_with_the_defaults_then_the_added_switches`).
pub fn executable(path: &Path, text: &str) {
    let mut source = path.as_os_str().to_owned();
    source.push(".src");
    let source = PathBuf::from(source);
    std::fs::write(&source, text).unwrap();
    let status = std::process::Command::new("install")
        .args(["-m", "0755"])
        .arg(&source)
        .arg(path)
        .status()
        .unwrap();
    assert!(
        status.success(),
        "install {} exited {status}",
        path.display()
    );
}

/// Every executable a test writes goes through [`executable`]: no other file of this binary sets
/// an executable mode itself.
#[test]
fn every_test_executable_is_written_by_install() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mode = ["from_mode(0o7", "from_mode(0o5"];
    let mut found = Vec::new();
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "rs")
            && path.file_name().is_some_and(|n| n != "main.rs")
        {
            let text = std::fs::read_to_string(&path).unwrap();
            if mode.iter().any(|m| text.contains(m)) {
                found.push(path.display().to_string());
            }
        }
    }
    assert!(
        found.is_empty(),
        "set an executable mode without `executable`: {found:?}"
    );
}
