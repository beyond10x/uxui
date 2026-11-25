//! `ux-browser --version` and `-V`: the one line a consumer reads to prove which ux-browser a
//! binary is before it runs it. The answer starts no browser and reads no file.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The answer: the tool's name, the crate version and a newline.
fn answer() -> String {
    format!("ux-browser {}\n", env!("CARGO_PKG_VERSION"))
}

/// An empty working directory, and in it a stand-in for Chromium that leaves `started` behind
/// when anything starts it.
fn place(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("version")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let work = dir.join("work");
    std::fs::create_dir_all(&work).unwrap();
    let chrome = dir.join("chrome");
    let started = dir.join("started");
    crate::executable(
        &chrome,
        &format!("#!/bin/sh\ntouch '{}'\nexit 9\n", started.display()),
    );
    (work, chrome, started)
}

fn ask(test: &str, flag: &str) -> (Output, PathBuf) {
    let (work, chrome, started) = place(&format!("{test}{flag}"));
    let o = Command::new(env!("CARGO_BIN_EXE_ux-browser"))
        .arg(flag)
        .current_dir(&work)
        .env("UXUI_CHROME", &chrome)
        .env("PLAYWRIGHT_BROWSERS_PATH", &work)
        .output()
        .unwrap();
    assert_eq!(
        std::fs::read_dir(&work).unwrap().count(),
        0,
        "{flag} writes nothing"
    );
    (o, started)
}

#[test]
fn version_and_v_print_the_name_and_the_crate_version_and_nothing_else() {
    for flag in ["--version", "-V"] {
        let (o, started) = ask("print", flag);
        assert_eq!(o.status.code(), Some(0), "{flag}");
        assert_eq!(String::from_utf8_lossy(&o.stdout), answer(), "{flag}");
        assert_eq!(String::from_utf8_lossy(&o.stderr), "", "{flag}");
        assert!(!started.exists(), "{flag} starts no browser");
    }
}

#[test]
fn the_version_is_the_one_a_run_records_in_its_conditions() {
    let (o, _) = ask("record", "--version");
    let stdout = String::from_utf8(o.stdout).unwrap();
    let version = stdout.trim_end().strip_prefix("ux-browser ").unwrap();
    for doc in [
        crate::evidence::document(),
        crate::evidence::server_document(),
    ] {
        let tools = doc
            .get("evidence")
            .unwrap()
            .get("conditions")
            .unwrap()
            .get("tools")
            .unwrap();
        assert_eq!(tools.get_str("ux-browser"), Some(version));
    }
}

#[test]
fn the_usage_lists_version() {
    let help = Command::new(env!("CARGO_BIN_EXE_ux-browser"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(help.status.success());
    let usage = String::from_utf8(help.stdout).unwrap();
    assert!(
        usage
            .lines()
            .any(|l| l.trim() == "ux-browser -V | --version"),
        "{usage}"
    );
}
