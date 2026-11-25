//! Finding and starting Chromium. The binary is `UXUI_CHROME`, else the newest
//! `chromium-<n>/chrome-linux*/chrome` under the Playwright cache (`PLAYWRIGHT_BROWSERS_PATH`, else
//! `~/.cache/ms-playwright`), or the build a checks file pins ([`Pin`]; `UXUI_CHROME` is then
//! checked against the pin). It runs headless with a profile of its own, and is ended by its PID.
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

/// The `chromium-<n>` builds under `cache` that have a `chrome-linux*/chrome`, newest first: the
/// build number and the binary.
pub fn builds_in(cache: &Path) -> Vec<(u64, PathBuf)> {
    let Ok(entries) = std::fs::read_dir(cache) else {
        return Vec::new();
    };
    let mut builds: Vec<(u64, PathBuf)> = entries
        .filter_map(Result::ok)
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let n = name.strip_prefix("chromium-")?.parse::<u64>().ok()?;
            let mut subs: Vec<PathBuf> = std::fs::read_dir(e.path())
                .ok()?
                .filter_map(Result::ok)
                .filter(|s| s.file_name().to_string_lossy().starts_with("chrome-linux"))
                .map(|s| s.path().join("chrome"))
                .filter(|p| p.is_file())
                .collect();
            subs.sort();
            Some((n, subs.pop()?))
        })
        .collect();
    builds.sort_by_key(|b| std::cmp::Reverse(b.0));
    builds
}

/// The newest `chromium-<n>` build under `cache` that has a `chrome-linux*/chrome`.
pub fn newest_in(cache: &Path) -> Option<PathBuf> {
    builds_in(cache).into_iter().next().map(|(_, p)| p)
}

/// The Chrome build a checks file pins (`chrome`): a Playwright build, `chromium-<n>`, or the
/// major version of Chrome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin {
    /// `chrome: {build: <n>}`: the build `chromium-<n>`.
    Build(u64),
    /// `chrome: {version: <major>}`: the newest build whose Chrome has that major version.
    Version(u64),
}

impl std::fmt::Display for Pin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pin::Build(n) => write!(f, "chromium-{n}"),
            Pin::Version(major) => write!(f, "Chrome {major}"),
        }
    }
}

/// `chrome`: a map of exactly one of `build` and `version`, a whole number above 0.
pub fn pin(value: &Value) -> Result<Pin, String> {
    let entries = match value {
        Value::Map(entries) if entries.len() == 1 => entries,
        _ => return Err("`chrome` is `{build: <n>}` or `{version: <major>}`".to_string()),
    };
    let (key, n) = &entries[0];
    let n = match n {
        Value::Int(n) if *n > 0 => *n as u64,
        _ => return Err(format!("chrome: `{key}` is a whole number above 0")),
    };
    match key.as_str() {
        "build" => Ok(Pin::Build(n)),
        "version" => Ok(Pin::Version(n)),
        _ => Err(format!("chrome: unknown key `{key}`; `build` or `version`")),
    }
}

/// The version Chrome prints for `--version` (`Google Chrome for Testing 149.0.7827.55`):
/// `149.0.7827.55`. The binary prints it and exits; no browser starts.
pub fn version_of(binary: &Path) -> Result<String, String> {
    let o = Command::new(binary)
        .arg("--version")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map_err(|e| format!("{} --version: {e}", binary.display()))?;
    let text = String::from_utf8_lossy(&o.stdout);
    text.split_whitespace()
        .find(|t| major(t).is_some() && t.contains('.'))
        .map(str::to_string)
        .ok_or_else(|| {
            format!(
                "{} --version printed no version: `{}`",
                binary.display(),
                text.trim()
            )
        })
}

/// The major version of `149.0.7827.55`: 149.
fn major(version: &str) -> Option<u64> {
    if !version.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return None;
    }
    version.split('.').next()?.parse().ok()
}

/// Whether the binary is the pinned build: by its `chromium-<n>` directory for a build, by
/// `--version` for a version.
fn is_pinned(binary: &Path, pin: Pin) -> Result<bool, String> {
    match pin {
        Pin::Build(n) => Ok(build_of(binary) == Some(n)),
        Pin::Version(want) => Ok(major(&version_of(binary)?) == Some(want)),
    }
}

/// The build of a binary that lies in a `chromium-<n>` directory, read from its path.
fn build_of(binary: &Path) -> Option<u64> {
    let real = binary
        .canonicalize()
        .unwrap_or_else(|_| binary.to_path_buf());
    [binary, real.as_path()].into_iter().find_map(|p| {
        p.ancestors().find_map(|a| {
            a.file_name()?
                .to_str()?
                .strip_prefix("chromium-")?
                .parse()
                .ok()
        })
    })
}

/// The binary under `cache` for `pin`: the pinned build, or the newest without a pin. A pin no
/// build meets is refused, naming it and the builds found (a version pin: with their versions).
pub fn select(cache: &Path, pin: Option<Pin>) -> Result<PathBuf, String> {
    let builds = builds_in(cache);
    let Some(pin) = pin else {
        return builds.into_iter().next().map(|(_, p)| p).ok_or_else(|| {
            format!(
                "no chromium-*/chrome-linux*/chrome under {}; set UXUI_CHROME",
                cache.display()
            )
        });
    };
    let mut found = Vec::new();
    for (n, binary) in &builds {
        let hit = match pin {
            Pin::Build(want) => {
                found.push(format!("chromium-{n}"));
                *n == want
            }
            Pin::Version(want) => {
                let version = version_of(binary).unwrap_or_else(|_| "no version".to_string());
                found.push(format!("chromium-{n} ({version})"));
                major(&version) == Some(want)
            }
        };
        if hit {
            return Ok(binary.clone());
        }
    }
    let found = if found.is_empty() {
        "none".to_string()
    } else {
        found.join(", ")
    };
    Err(format!(
        "chrome: the configuration pins {pin}, which is not under {}; found: {found}",
        cache.display()
    ))
}

/// The binary: `uxui_chrome` (`UXUI_CHROME`) when set, which must then be the pinned build; else
/// [`select`] under `cache`.
pub fn find_in(
    uxui_chrome: Option<&Path>,
    cache: &Path,
    pin: Option<Pin>,
) -> Result<PathBuf, String> {
    let Some(binary) = uxui_chrome else {
        return select(cache, pin);
    };
    let Some(pin) = pin else {
        return Ok(binary.to_path_buf());
    };
    if is_pinned(binary, pin)? {
        return Ok(binary.to_path_buf());
    }
    Err(match (pin, build_of(binary)) {
        (Pin::Build(_), Some(n)) => format!(
            "chrome: UXUI_CHROME {} is chromium-{n}; the configuration pins {pin}",
            binary.display()
        ),
        (Pin::Build(_), None) => format!(
            "chrome: UXUI_CHROME {} lies in no chromium-<n> directory, so the pin {pin} cannot be checked; pin `version` or unset UXUI_CHROME",
            binary.display()
        ),
        (Pin::Version(_), _) => format!(
            "chrome: UXUI_CHROME {} is Chrome {}; the configuration pins {pin}",
            binary.display(),
            version_of(binary)?
        ),
    })
}

/// The binary for `pin` from the environment: `UXUI_CHROME`, else the Playwright cache
/// (`PLAYWRIGHT_BROWSERS_PATH`, else `~/.cache/ms-playwright`); [`find_in`].
pub fn find(pin: Option<Pin>) -> Result<PathBuf, String> {
    let uxui_chrome = std::env::var_os("UXUI_CHROME").map(PathBuf::from);
    let cache = match std::env::var_os("PLAYWRIGHT_BROWSERS_PATH") {
        Some(p) => PathBuf::from(p),
        None => match std::env::var_os("HOME") {
            Some(home) => PathBuf::from(home).join(".cache/ms-playwright"),
            None if uxui_chrome.is_some() => PathBuf::new(),
            None => return Err("no HOME and no UXUI_CHROME".to_string()),
        },
    };
    find_in(uxui_chrome.as_deref(), &cache, pin)
}

/// The DevTools endpoint from the line Chromium prints on start.
pub fn devtools_url(line: &str) -> Option<String> {
    let at = line.find("DevTools listening on ")?;
    let url = line[at + "DevTools listening on ".len()..].trim();
    url.starts_with("ws://").then(|| url.to_string())
}

/// What the reader of Chromium's stderr passes on: a line before the endpoint, the endpoint, or
/// the end of the stream.
enum Said {
    Line(String),
    Url(String),
    End,
}

/// Chromium's last lines on stderr before it listened, for the error when it does not: the cause
/// (a sandbox it cannot set up on a hosted runner, a missing library) is in them, not in the status.
#[derive(Default)]
struct Last(std::collections::VecDeque<String>);

impl Last {
    const KEPT: usize = 20;

    fn push(&mut self, line: String) {
        if self.0.len() == Last::KEPT {
            self.0.pop_front();
        }
        self.0.push_back(line);
    }

    /// `first`, then the lines one per line and indented, or that there were none.
    fn told(&self, first: &str) -> String {
        if self.0.is_empty() {
            return format!("{first}; it printed nothing on stderr");
        }
        let mut text = format!("{first}; its last {} lines on stderr:", self.0.len());
        for line in &self.0 {
            text.push_str("\n  ");
            text.push_str(line);
        }
        text
    }
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
        Process::start_within(binary, profile, added, Duration::from_secs(30))
    }

    /// [`Process::start`] with the time Chromium has to print its DevTools endpoint.
    pub fn start_within(
        binary: &Path,
        profile: &Path,
        added: &[String],
        within: Duration,
    ) -> Result<Process, String> {
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
                if sent {
                    continue;
                }
                let url = devtools_url(&line);
                sent = url.is_some();
                if tx.send(url.map_or(Said::Line(line), Said::Url)).is_err() {
                    return;
                }
            }
            let _ = tx.send(Said::End);
        });
        let mut process = Process {
            child,
            ws_url: String::new(),
            binary: binary.to_path_buf(),
            profile: profile.to_path_buf(),
        };
        let mut last = Last::default();
        let deadline = Instant::now() + within;
        loop {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(Said::Url(url)) => {
                    process.ws_url = url;
                    return Ok(process);
                }
                Ok(Said::Line(line)) => last.push(line),
                Ok(Said::End) | Err(_) => {}
            }
            if let Ok(Some(status)) = process.child.try_wait() {
                // What Chromium printed before it ended is still in the pipe: read up to its end.
                let until = Instant::now() + Duration::from_secs(1);
                while let Ok(Said::Line(line)) =
                    rx.recv_timeout(until.saturating_duration_since(Instant::now()))
                {
                    last.push(line);
                }
                return Err(last.told(&format!("Chromium exited before it listened: {status}")));
            }
            if Instant::now() >= deadline {
                while let Ok(Said::Line(line)) = rx.try_recv() {
                    last.push(line);
                }
                return Err(last.told(&format!(
                    "Chromium printed no DevTools endpoint within {} s",
                    within.as_secs_f64()
                )));
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
