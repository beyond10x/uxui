//! Finding and starting Chromium. The binary is `UXUI_CHROME`, else the newest
//! `chromium-<n>/chrome-linux*/chrome` under the Playwright cache (`PLAYWRIGHT_BROWSERS_PATH`, else
//! `~/.cache/ms-playwright`). It runs headless with a profile of its own, and is ended by its PID.

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// The switches a headless run starts with: those of Playwright's headless Chromium that change
/// what a page measures (no scrollbars, the pointer and hover media of a touch-less headless
/// device, sRGB), and those that keep the browser from reaching out on its own.
pub const SWITCHES: &[&str] = &[
    "--headless=new",
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
    pub fn start(binary: &Path, profile: &Path) -> Result<Process, String> {
        std::fs::create_dir_all(profile).map_err(|e| format!("{}: {e}", profile.display()))?;
        let mut child = Command::new(binary)
            .args(SWITCHES)
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
