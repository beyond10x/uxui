//! The switches Chromium starts with: the defaults (no GPU process), a consumer's own switches from
//! `chrome_args` and `--chrome-arg` after them, the refusal of one that changes a default, and the
//! local input that names them outside the evidence. The one process started here is a shell
//! script standing in for Chromium: it prints the line Chromium prints and records its arguments.
//! The Chrome build (`chrome`): a fake Playwright cache of two builds whose binaries are shell
//! scripts that answer `--version` and nothing else.

use crate::checks::LLM_LIKE;
use std::path::{Path, PathBuf};
use std::process::Command;
use ux_browser::checks;
use ux_browser::chrome::{self, Pin, SWITCHES};
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

// ---------------------------------------------------------------- the Chrome build: `chrome`

/// A stand-in Chrome binary at `dir/chrome`: `--version` prints `version` as Chrome for Testing
/// prints it; anything else fails, so no test starts a browser by mistake.
fn fake_binary(dir: &Path, version: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    std::fs::create_dir_all(dir).unwrap();
    let script = dir.join("chrome");
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\nif [ \"$1\" = --version ]; then echo 'Google Chrome for Testing {version} '; exit 0; fi\nexit 9\n"
        ),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    script
}

/// A fake Playwright cache with two builds, `chromium-1228` (Chrome 149) and `chromium-1254`
/// (Chrome 158), beside what is not a build: a headless shell, a build directory without a binary.
fn two_builds(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let cache = scratch(name).join("cache");
    let old = fake_binary(&cache.join("chromium-1228/chrome-linux64"), "149.0.7827.55");
    let new = fake_binary(&cache.join("chromium-1254/chrome-linux64"), "158.0.8127.18");
    fake_binary(
        &cache.join("chromium_headless_shell-1300/chrome-linux64"),
        "160.0.1.1",
    );
    std::fs::create_dir_all(cache.join("chromium-1400/chrome-linux64")).unwrap();
    (cache, old, new)
}

#[test]
fn without_a_pin_the_newest_build_is_taken() {
    let (cache, _, new) = two_builds("default");
    assert_eq!(chrome::select(&cache, None).unwrap(), new);
    assert_eq!(chrome::newest_in(&cache), Some(new));
}

#[test]
fn a_build_pin_selects_that_build() {
    let (cache, old, new) = two_builds("pick-build");
    assert_eq!(chrome::select(&cache, Some(Pin::Build(1228))).unwrap(), old);
    assert_eq!(chrome::select(&cache, Some(Pin::Build(1254))).unwrap(), new);
}

#[test]
fn a_version_pin_selects_the_newest_build_of_that_major() {
    let (cache, old, new) = two_builds("pick-version");
    assert_eq!(
        chrome::select(&cache, Some(Pin::Version(149))).unwrap(),
        old
    );
    assert_eq!(
        chrome::select(&cache, Some(Pin::Version(158))).unwrap(),
        new
    );
    let newer_149 = fake_binary(&cache.join("chromium-1230/chrome-linux64"), "149.0.7900.2");
    assert_eq!(
        chrome::select(&cache, Some(Pin::Version(149))).unwrap(),
        newer_149
    );
}

#[test]
fn a_missing_build_is_refused_naming_the_wanted_and_the_found_builds() {
    let (cache, _, _) = two_builds("miss");
    let e = chrome::select(&cache, Some(Pin::Build(1300))).unwrap_err();
    for part in ["chromium-1300", "chromium-1254, chromium-1228"] {
        assert!(e.contains(part), "{part}: {e}");
    }
    let e = chrome::select(&cache, Some(Pin::Version(150))).unwrap_err();
    for part in [
        "Chrome 150",
        "chromium-1254 (158.0.8127.18), chromium-1228 (149.0.7827.55)",
    ] {
        assert!(e.contains(part), "{part}: {e}");
    }
    let empty = scratch("miss-empty");
    let e = chrome::select(&empty, Some(Pin::Build(1228))).unwrap_err();
    assert!(
        e.contains("chromium-1228") && e.contains("found: none"),
        "{e}"
    );
    assert!(chrome::select(&empty, None).is_err());
}

#[test]
fn the_version_is_read_from_the_binary_without_starting_a_browser() {
    let (_, old, _) = two_builds("version");
    assert_eq!(chrome::version_of(&old).unwrap(), "149.0.7827.55");
    let dir = scratch("version-bad");
    let bad = fake_binary(&dir, "unknown");
    assert!(chrome::version_of(&bad).is_err());
    assert!(chrome::version_of(&dir.join("no-such-chrome")).is_err());
}

#[test]
fn uxui_chrome_wins_and_is_checked_against_the_pin() {
    let (cache, old, _) = two_builds("env");
    let find = |pin| chrome::find_in(Some(&old), &cache, pin);
    assert_eq!(find(None).unwrap(), old, "not the newest build");
    assert_eq!(find(Some(Pin::Build(1228))).unwrap(), old);
    assert_eq!(find(Some(Pin::Version(149))).unwrap(), old);
    let e = find(Some(Pin::Build(1254))).unwrap_err();
    assert!(
        e.contains("UXUI_CHROME") && e.contains("chromium-1228") && e.contains("chromium-1254"),
        "{e}"
    );
    let e = find(Some(Pin::Version(158))).unwrap_err();
    assert!(
        e.contains("UXUI_CHROME") && e.contains("149.0.7827.55") && e.contains("Chrome 158"),
        "{e}"
    );
    let outside = fake_binary(&scratch("env-outside").join("bin"), "149.0.7827.55");
    let find = |pin| chrome::find_in(Some(&outside), &cache, pin);
    assert_eq!(find(Some(Pin::Version(149))).unwrap(), outside);
    let e = find(Some(Pin::Build(1228))).unwrap_err();
    assert!(
        e.contains("UXUI_CHROME") && e.contains("chromium-1228") && e.contains("`version`"),
        "a build pin cannot be checked on a binary outside a build: {e}"
    );
    assert_eq!(
        chrome::find_in(None, &cache, Some(Pin::Build(1228))).unwrap(),
        old
    );
}

#[test]
fn the_configuration_pins_a_build_or_a_version() {
    assert_eq!(checks::config(LLM_LIKE).unwrap().chrome, None);
    let pinned = |key: &str| checks::config(&format!("{LLM_LIKE}chrome: {key}\n"));
    assert_eq!(
        pinned("{build: 1228}").unwrap().chrome,
        Some(Pin::Build(1228))
    );
    assert_eq!(
        pinned("{version: 149}").unwrap().chrome,
        Some(Pin::Version(149))
    );
    for bad in [
        "{build: '1228'}",
        "{build: 12.5}",
        "{build: 0}",
        "{version: -1}",
        "{version: '149'}",
        "{build: 1228, version: 149}",
        "{revision: 1228}",
        "{build: 1228, channel: stable}",
        "{}",
        "1228",
        "[1228]",
    ] {
        let e = pinned(bad)
            .err()
            .unwrap_or_else(|| panic!("{bad} is refused"));
        assert!(e.contains("chrome"), "{bad}: {e}");
    }
}

/// `checks` on a configuration that pins `pin`, with the fake cache and no `UXUI_CHROME`, on the
/// built site `build`. The inputs are refused first (exit 1), then the browser is chosen (exit 3).
fn pinned_call(name: &str, pin: &str, build: &Path) -> (Option<i32>, String) {
    let (cache, _, _) = two_builds(name);
    let config = cache.parent().unwrap().join("checks.yaml");
    let pages = LLM_LIKE.replace("pages: {sitemap: sitemap.xml}", "pages: {paths: [/llm/]}");
    std::fs::write(&config, format!("{pages}chrome: {pin}\n")).unwrap();
    let out = cache.parent().unwrap().join("out.json");
    let o = Command::new(env!("CARGO_BIN_EXE_ux-browser"))
        .env_remove("UXUI_CHROME")
        .env("PLAYWRIGHT_BROWSERS_PATH", &cache)
        .args(["checks", "--config"])
        .arg(&config)
        .arg("--build")
        .arg(build)
        .args(["--project", "p", "--assessment", "a", "--out"])
        .arg(&out)
        .output()
        .unwrap();
    assert!(!out.exists(), "nothing is written");
    (
        o.status.code(),
        String::from_utf8_lossy(&o.stderr).to_string(),
    )
}

#[test]
fn a_missing_pinned_build_fails_the_run_before_anything_is_measured() {
    let site = crate::fixture("site");
    let (code, stderr) = pinned_call("run-miss", "{build: 1300}", &site);
    assert_eq!(code, Some(3), "{stderr}");
    assert!(
        stderr.contains("chromium-1300") && stderr.contains("chromium-1254, chromium-1228"),
        "{stderr}"
    );
    let missing = Path::new("no-such-build");
    let (code, stderr) = pinned_call("run-miss-inputs", "{build: 1300}", missing);
    assert_eq!(code, Some(1), "the inputs are refused first: {stderr}");
    assert!(
        stderr.contains("no-such-build") && !stderr.contains("chromium-1300"),
        "{stderr}"
    );
    let (code, stderr) = pinned_call("run-hit", "{build: 1228}", missing);
    assert_eq!(code, Some(1), "the pinned build is found: {stderr}");
    assert!(stderr.contains("no-such-build"), "{stderr}");
}
