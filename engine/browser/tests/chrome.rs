//! The switches Chromium starts with: the defaults (no GPU process), a consumer's own switches from
//! `chrome_args` and `--chrome-arg` after them, the refusal of one that changes a default, and the
//! local input that names them outside the evidence. The one process started here is a shell
//! script standing in for Chromium: it prints the line Chromium prints and records its arguments.

use crate::checks::LLM_LIKE;
use std::path::{Path, PathBuf};
use std::process::Command;
use ux_browser::checks;
use ux_browser::chrome::{self, SWITCHES};
use ux_browser::evidence::{self, Header, Target};
use ux_browser::render::FileRef;

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/test-chrome")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn by_default_no_gpu_process_runs() {
    assert!(
        SWITCHES.contains(&"--disable-gpu"),
        "no hardware or SwiftShader GPU work: {SWITCHES:?}"
    );
    assert!(
        SWITCHES.contains(&"--in-process-gpu"),
        "`--disable-gpu` alone still starts a GPU process, for compositing: {SWITCHES:?}"
    );
}

#[test]
fn a_consumers_switches_come_after_the_defaults_in_their_order() {
    let extra = strings(&[
        "--use-gl=angle",
        "--remote-debugging-address=127.0.0.1",
        "--user-agent=a b",
    ]);
    let line = chrome::switches(&extra).unwrap();
    assert_eq!(&line[..SWITCHES.len()], SWITCHES);
    assert_eq!(&line[SWITCHES.len()..], extra.as_slice());
    assert_eq!(chrome::switches(&[]).unwrap(), SWITCHES);
}

#[test]
fn a_switch_the_defaults_set_to_another_value_is_refused_by_its_name() {
    for switch in [
        "--headless",
        "--headless=old",
        "--remote-debugging-port=9222",
        "--remote-debugging-port",
        "--force-color-profile=display-p3-d65",
        "--user-data-dir=/somewhere",
    ] {
        let e = chrome::switches(&strings(&["--enable-gpu", switch])).unwrap_err();
        assert!(e.contains(&format!("`{switch}`")), "{switch}: {e}");
    }
    let e = chrome::switches(&strings(&["--headless=old"])).unwrap_err();
    assert!(e.contains("--headless=new"), "names the default: {e}");
}

#[test]
fn a_switch_the_defaults_set_to_the_same_value_or_do_not_set_is_allowed() {
    for switch in [
        "--headless=new",
        "--hide-scrollbars",
        "--disable-gpu",
        "--enable-gpu",
        "--use-gl=egl",
        "--use-angle=swiftshader",
        "--enable-unsafe-swiftshader",
        "--remote-debugging-address=127.0.0.1",
    ] {
        assert!(chrome::switches(&strings(&[switch])).is_ok(), "{switch}");
    }
}

#[test]
fn enable_gpu_undoes_the_gpu_defaults() {
    let line = chrome::switches(&strings(&["--use-gl=egl", "--enable-gpu"])).unwrap();
    let mut want: Vec<String> = SWITCHES
        .iter()
        .filter(|s| !chrome::NO_GPU.contains(s))
        .map(|s| s.to_string())
        .collect();
    want.extend(strings(&["--use-gl=egl", "--enable-gpu"]));
    assert_eq!(
        line, want,
        "the line of earlier versions, then the added switches"
    );
    assert_eq!(chrome::NO_GPU, ["--disable-gpu", "--in-process-gpu"]);
}

#[test]
fn what_is_not_a_switch_is_refused() {
    for arg in ["about:blank", "-x", "--", "--=1", "", "enable-gpu"] {
        let e = chrome::switches(&strings(&[arg])).unwrap_err();
        assert!(e.contains(&format!("`{arg}`")), "{arg:?}: {e}");
    }
}

#[test]
fn a_flag_value_is_one_switch_or_a_json_list_of_switches() {
    assert_eq!(
        chrome::flag_switches("--enable-gpu").unwrap(),
        ["--enable-gpu"]
    );
    assert_eq!(
        chrome::flag_switches(r#"["--enable-gpu", "--user-agent=a b"]"#).unwrap(),
        ["--enable-gpu", "--user-agent=a b"]
    );
    assert_eq!(chrome::flag_switches("[]").unwrap(), Vec::<String>::new());
    for bad in ["[1]", "[\"--a\"", "[[\"--a\"]]"] {
        assert!(chrome::flag_switches(bad).is_err(), "{bad}");
    }
}

#[test]
fn the_local_input_is_the_added_switches_as_a_json_list() {
    assert_eq!(chrome::local_input(&[]), None);
    let added = strings(&["--enable-gpu", "--user-agent=a \"b\""]);
    let text = chrome::local_input(&added).unwrap();
    assert_eq!(text, r#"["--enable-gpu","--user-agent=a \"b\""]"#);
    assert_eq!(
        chrome::flag_switches(&text).unwrap(),
        added,
        "it reads back"
    );
}

#[test]
fn the_configuration_adds_switches_with_chrome_args() {
    assert!(checks::config(LLM_LIKE).unwrap().chrome_args.is_empty());
    let c = checks::config(&format!(
        "{LLM_LIKE}chrome_args: [--enable-gpu, '--use-gl=egl']\n"
    ))
    .unwrap();
    assert_eq!(c.chrome_args, ["--enable-gpu", "--use-gl=egl"]);
    let e = checks::config(&format!("{LLM_LIKE}chrome_args: ['--headless=old']\n"))
        .err()
        .unwrap();
    assert!(e.contains("`--headless=old`"), "{e}");
    for bad in [
        "chrome_args: --enable-gpu\n",
        "chrome_args: [1]\n",
        "chrome_args: [about:blank]\n",
    ] {
        assert!(
            checks::config(&format!("{LLM_LIKE}{bad}")).is_err(),
            "{bad}"
        );
    }
}

/// `checks` with a configuration that does not exist: every call is refused before a browser
/// starts.
fn checks_call(extra: &[&str]) -> (Option<i32>, String) {
    let missing = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/no-such-config.yaml");
    let o = Command::new(env!("CARGO_BIN_EXE_ux-browser"))
        .args(["checks", "--config"])
        .arg(&missing)
        .args([
            "--build",
            "no-such-build",
            "--project",
            "p",
            "--assessment",
            "a",
        ])
        .args(extra)
        .output()
        .unwrap();
    (
        o.status.code(),
        String::from_utf8_lossy(&o.stderr).to_string(),
    )
}

#[test]
fn chrome_arg_is_repeatable_and_refused_when_it_changes_a_default() {
    let (code, stderr) = checks_call(&[
        "--chrome-arg",
        "--enable-gpu",
        "--chrome-arg",
        r#"["--use-gl=egl"]"#,
    ]);
    assert_eq!(code, Some(1), "reaches the missing configuration: {stderr}");
    let (code, stderr) = checks_call(&["--chrome-arg", "--remote-debugging-port=9222"]);
    assert_eq!(code, Some(2), "{stderr}");
    assert!(
        stderr.contains("`--remote-debugging-port=9222`"),
        "{stderr}"
    );
    let (code, stderr) = checks_call(&["--chrome-arg", "[1]"]);
    assert_eq!(code, Some(2), "{stderr}");
    let (code, stderr) = checks_call(&["--chrome-arg"]);
    assert_eq!(code, Some(2), "{stderr}");
}

#[test]
fn documents_of_runs_that_differ_only_in_switches_have_equal_evidence() {
    let config = checks::config(LLM_LIKE).unwrap();
    let config_file = FileRef {
        path: Some("engine/browser/examples/llm.checks.yaml".to_string()),
        digest: "sha256:aa".to_string(),
    };
    let build = FileRef {
        path: None,
        digest: "sha256:bb".to_string(),
    };
    let doc = |added: &[&str]| {
        let mut local_inputs = vec![("jobs", "1".to_string())];
        if let Some(text) = chrome::local_input(&strings(added)) {
            local_inputs.push(("chrome-arg", text));
        }
        let header = Header {
            project: "llm-docs",
            assessment: "page-checks",
            class: "local",
            config: &config_file,
            target: Target::Build(&build),
            pages: None,
            local_inputs,
            browser: "Chrome/149.0.7827.55",
            started_at: "2026-11-25T10:00:00Z".to_string(),
            ended_at: "2026-11-25T10:01:00Z".to_string(),
        };
        evidence::document(&header, &config, &[])
    };
    let plain = doc(&[]);
    let gpu = doc(&["--enable-gpu", "--use-gl=egl"]);
    assert_eq!(plain.get("evidence"), gpu.get("evidence"));
    let local = gpu.get("run").unwrap().get("local_inputs").unwrap();
    assert_eq!(
        local.get_str("chrome-arg"),
        Some(r#"["--enable-gpu","--use-gl=egl"]"#)
    );
    assert_eq!(
        plain
            .get("run")
            .unwrap()
            .get("local_inputs")
            .unwrap()
            .get("chrome-arg"),
        None
    );
}

/// A stand-in for Chromium: writes its arguments one per line to `args`, prints the line Chromium
/// prints when it listens, and waits to be ended.
fn fake_chrome(dir: &Path) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let script = dir.join("chrome");
    let args = dir.join("args");
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\nfor a in \"$@\"; do printf '%s\\n' \"$a\"; done > '{}'\necho 'DevTools listening on ws://127.0.0.1:9/devtools/browser/x' >&2\nexec sleep 30\n",
            args.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    script
}

#[test]
fn chromium_starts_with_the_defaults_then_the_added_switches() {
    let dir = scratch("start");
    let binary = fake_chrome(&dir);
    let profile = dir.join("profile");
    let added = strings(&["--use-gl=egl", "--user-agent=a b"]);
    let mut process = chrome::Process::start(&binary, &profile, &added).unwrap();
    assert_eq!(process.ws_url, "ws://127.0.0.1:9/devtools/browser/x");
    let args = std::fs::read_to_string(dir.join("args")).unwrap();
    process.end(std::time::Duration::from_millis(0));
    let args: Vec<&str> = args.lines().collect();
    let mut want: Vec<String> = SWITCHES.iter().map(|s| s.to_string()).collect();
    want.extend(added);
    want.push(format!("--user-data-dir={}", profile.display()));
    want.push("about:blank".to_string());
    assert_eq!(args, want);
}
