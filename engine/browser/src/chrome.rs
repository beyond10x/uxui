//! Finding and starting Chromium. The binary is `UXUI_CHROME`, else the newest
//! `chromium-<n>/chrome-linux*/chrome` under the Playwright cache (`PLAYWRIGHT_BROWSERS_PATH`, else
//! `~/.cache/ms-playwright`). It runs headless with a profile of its own, and is ended by its PID.
//! It starts with [`SWITCHES`], then the switches a consumer adds ([`switches`]).

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use uxui_doc::yaml::Value;

/// The switches a headless run starts with: those of Playwright's headless Chromium that change
/// what a page measures (no scrollbars, the pointer and hover media of a touch-less headless
/// device, sRGB), those that keep the browser from reaching out on its own, and [`NO_GPU`].
pub const SWITCHES: &[&str] = &[
    "--headless=new",
    "--disable-gpu",
    "--in-process-gpu",
    "--remote-debugging-port=0",
    "--no-first-run",
    "--no-default-browser-check",
    "--disable-extensions",
    "--disable-background-networking",
    "--disable-component-update",
    "--disable-sync",
    "--disable-default-apps",
    "--disable-breakpad",
    "--disable-dev-shm-usage",
    "--metrics-recording-only",
    "--password-store=basic",
    "--use-mock-keychain",
    "--hide-scrollbars",
    "--mute-audio",
    "--force-color-profile=srgb",
    "--blink-settings=primaryHoverType=2,availableHoverTypes=2,primaryPointerType=4,availablePointerTypes=4",
];

/// The switches of [`SWITCHES`] that start no GPU process: no GPU work (hardware or SwiftShader),
/// and the compositing that `--disable-gpu` still leaves to a GPU process run on a thread of the
/// browser process. Measured on Chromium 149: `--disable-gpu` alone keeps a `--type=gpu-process`.
pub const NO_GPU: &[&str] = &["--disable-gpu", "--in-process-gpu"];

/// An added switch that undoes [`NO_GPU`]: the line then starts without them, as earlier versions
/// did. Chromium itself has no switch that cancels another.
pub const ENABLE_GPU: &str = "--enable-gpu";

/// The name of a switch, `--name` of `--name` and `--name=value`; `None` for what is not a switch.
fn switch_name(arg: &str) -> Option<&str> {
    let name = arg.split_once('=').map_or(arg, |(n, _)| n);
    let bare = name.strip_prefix("--")?;
    (!bare.is_empty() && !bare.starts_with('-')).then_some(name)
}

/// The command line's switches: [`SWITCHES`], then `added` in its order. An added switch that
/// `SWITCHES` sets with another value (`--headless`, `--remote-debugging-port=9222`), the profile's
/// `--user-data-dir`, and what is not a switch are refused, by name. One that `SWITCHES` sets with
/// the same value adds nothing; [`ENABLE_GPU`] leaves out [`NO_GPU`].
pub fn switches(added: &[String]) -> Result<Vec<String>, String> {
    for arg in added {
        let Some(name) = switch_name(arg) else {
            return Err(format!(
                "`{arg}` is not a Chromium switch: `--<name>` or `--<name>=<value>`"
            ));
        };
        if name == "--user-data-dir" {
            return Err(format!(
                "Chromium switch `{arg}`: the profile directory is `--profile`"
            ));
        }
        if let Some(set) = SWITCHES
            .iter()
            .find(|s| switch_name(s) == Some(name) && *s != arg)
        {
            return Err(format!(
                "Chromium switch `{arg}`: ux-browser starts Chromium with `{set}`; an added switch may not change it"
            ));
        }
    }
    let undo_gpu = added.iter().any(|a| a == ENABLE_GPU);
    Ok(SWITCHES
        .iter()
        .filter(|s| !(undo_gpu && NO_GPU.contains(s)))
        .map(|s| s.to_string())
        .chain(added.iter().cloned())
        .collect())
}

/// The switches of one `--chrome-arg` value: one switch, or a JSON list of them (how ratchet passes
/// a list input, and how [`local_input`] writes them).
pub fn flag_switches(value: &str) -> Result<Vec<String>, String> {
    if !value.trim_start().starts_with('[') {
        return Ok(vec![value.to_string()]);
    }
    let list = crate::json::parse(value).map_err(|e| format!("`--chrome-arg {value}`: {e}"))?;
    list.list()
        .iter()
        .map(|item| match item {
            Value::Str(s) => Ok(s.clone()),
            _ => Err(format!(
                "`--chrome-arg {value}`: a list of switches is a JSON list of text"
            )),
        })
        .collect()
}

/// `run.local_inputs.chrome-arg`: the switches added to [`SWITCHES`], as a JSON list; none without.
pub fn local_input(added: &[String]) -> Option<String> {
    (!added.is_empty()).then(|| {
        uxui_doc::json::write(&Value::List(
            added.iter().map(|s| Value::Str(s.clone())).collect(),
        ))
    })
}

/// The newest `chromium-<n>` build under `cache` that has a `chrome-linux*/chrome`.
pub fn newest_in(cache: &Path) -> Option<PathBuf> {
    let mut builds: Vec<(u64, PathBuf)> = std::fs::read_dir(cache)
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let n = name.strip_prefix("chromium-")?.parse::<u64>().ok()?;
            Some((n, e.path()))
        })
        .collect();
    builds.sort_by_key(|b| std::cmp::Reverse(b.0));
    for (_, dir) in builds {
        let mut subs: Vec<PathBuf> = std::fs::read_dir(&dir)
            .ok()?
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().starts_with("chrome-linux"))
            .map(|e| e.path().join("chrome"))
            .filter(|p| p.is_file())
            .collect();
        subs.sort();
        if let Some(p) = subs.pop() {
            return Some(p);
        }
    }
    None
}

pub fn find() -> Result<PathBuf, String> {
    if let Some(p) = std::env::var_os("UXUI_CHROME") {
        return Ok(PathBuf::from(p));
    }
    let cache = match std::env::var_os("PLAYWRIGHT_BROWSERS_PATH") {
        Some(p) => PathBuf::from(p),
        None => {
            let home = std::env::var_os("HOME").ok_or("no HOME and no UXUI_CHROME")?;
            PathBuf::from(home).join(".cache/ms-playwright")
        }
    };
    newest_in(&cache).ok_or_else(|| {
        format!(
            "no chromium-*/chrome-linux*/chrome under {}; set UXUI_CHROME",
            cache.display()
        )
    })
}

/// The DevTools endpoint from the line Chromium prints on start.
pub fn devtools_url(line: &str) -> Option<String> {
    let at = line.find("DevTools listening on ")?;
    let url = line[at + "DevTools listening on ".len()..].trim();
    url.starts_with("ws://").then(|| url.to_string())
}

/// A running Chromium. Dropping it ends the process by its PID and removes its profile.
pub struct Process {
    child: Child,
    pub ws_url: String,
    pub binary: PathBuf,
    profile: PathBuf,
}

impl Process {
    /// Starts `binary` with [`switches`] of `added`, the profile and a blank page.
    pub fn start(binary: &Path, profile: &Path, added: &[String]) -> Result<Process, String> {
        let line = switches(added)?;
        std::fs::create_dir_all(profile).map_err(|e| format!("{}: {e}", profile.display()))?;
        let mut child = Command::new(binary)
            .args(&line)
            .arg(format!("--user-data-dir={}", profile.display()))
            .arg("about:blank")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("{}: {e}", binary.display()))?;
        let stderr = child.stderr.take().expect("stderr is piped");
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut sent = false;
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                if !sent && let Some(url) = devtools_url(&line) {
                    let _ = tx.send(url);
                    sent = true;
                }
            }
        });
        let mut process = Process {
            child,
            ws_url: String::new(),
            binary: binary.to_path_buf(),
            profile: profile.to_path_buf(),
        };
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if let Ok(url) = rx.recv_timeout(Duration::from_millis(100)) {
                process.ws_url = url;
                return Ok(process);
            }
            if let Ok(Some(status)) = process.child.try_wait() {
                return Err(format!("Chromium exited before it listened: {status}"));
            }
            if Instant::now() >= deadline {
                return Err("Chromium printed no DevTools endpoint within 30 s".to_string());
            }
        }
    }

    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    /// Waits up to `grace` for the process to end on its own (after `Browser.close`), then kills it.
    pub fn end(&mut self, grace: Duration) {
        let deadline = Instant::now() + grace;
        while Instant::now() < deadline {
            if let Ok(Some(_)) = self.child.try_wait() {
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        if !matches!(self.child.try_wait(), Ok(Some(_))) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        let _ = std::fs::remove_dir_all(&self.profile);
    }
}
