//! The cost of one evaluation on layout model v0 (`ux-search cost`).
//!
//! A UI state is the page opened (state 0) and then the first `n` keys of a key sequence typed
//! (state `n`). ESS 0.99.5 has no UI state API: `App` cannot be cloned, so every state is reached
//! by opening the document again and replaying its keys from the opened page. Per state and run:
//!
//! - open: `App::from_text` (it loads the fixtures and opens the home page), `open_page`, and
//!   the reads due at time zero;
//! - replay: every key of the state, each followed by the reads due at time zero; `step` is the
//!   last key's share of it;
//! - geometry: `render_text` at the size and `regions`, read into [`crate::drive::Placed`];
//! - peak memory: the process's `VmHWM` from `/proc/self/status` after the state.
//!
//! Each time is the median over the runs. The numbers are the host's: they vary with its load.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::drive::{self, Target};

/// What to measure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// The page and its params.
    pub target: Target,
    /// The keys typed after the open, as ESS writes them (`<tab>jj<enter>`).
    pub keys: String,
    /// The size the geometry is read at, `(width, height)`.
    pub size: (u16, u16),
    /// Runs per state; the median is reported.
    pub repeat: usize,
    /// Where file-placed state is kept.
    pub state_dir: PathBuf,
}

/// The cost of one UI state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateCost {
    /// How many keys were typed after the open.
    pub state: usize,
    /// The last key typed; empty for the opened page.
    pub key: String,
    /// Loading the document and opening the page.
    pub open: Duration,
    /// Typing the state's keys after the open.
    pub replay: Duration,
    /// Typing the last of them.
    pub step: Duration,
    /// Drawing at the size and reading the regions.
    pub geometry: Duration,
    /// How many regions the drawing recorded.
    pub regions: usize,
    /// The process's peak resident memory after the state, in KiB.
    pub peak_kib: Option<u64>,
}

/// A finished measurement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cost {
    /// The document's file name.
    pub file: String,
    /// The page and its params.
    pub target: Target,
    /// The size drawn.
    pub size: (u16, u16),
    /// Runs per state.
    pub runs: usize,
    /// The keys, one step each.
    pub keys: Vec<String>,
    /// The process's peak resident memory before the first open, in KiB.
    pub baseline_kib: Option<u64>,
    /// One entry per state, the opened page first.
    pub states: Vec<StateCost>,
}

/// The special keys ESS's key parser names in angle brackets (`ess_ui_tui::keys`); `<c-x>` is
/// control plus one character.
const NAMED: [&str; 11] = [
    "enter", "esc", "tab", "backtab", "bs", "space", "up", "down", "left", "right", "lt",
];

/// A key spec split into one step per key, as `ess_ui_tui::keys::parse` splits it: a known
/// `<name>` is one key, every other character is one key.
pub fn key_steps(spec: &str) -> Vec<String> {
    let mut steps = Vec::new();
    let mut rest = spec;
    while let Some(first) = rest.chars().next() {
        if first == '<'
            && let Some(end) = rest.find('>')
            && is_named(&rest[1..end])
        {
            steps.push(rest[..=end].to_owned());
            rest = &rest[end + 1..];
            continue;
        }
        steps.push(first.to_string());
        rest = &rest[first.len_utf8()..];
    }
    steps
}

fn is_named(name: &str) -> bool {
    NAMED.contains(&name)
        || name
            .strip_prefix("c-")
            .is_some_and(|letter| letter.chars().count() == 1)
}

/// The process's peak resident memory (`VmHWM`), in KiB; `None` where `/proc` does not say.
pub fn peak_kib() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    status
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))
        .and_then(|rest| rest.trim().trim_end_matches("kB").trim().parse().ok())
}

/// Measures every state of `options.keys` on `document`.
pub fn measure(document: &Path, options: &Options) -> Result<Cost, String> {
    let text = std::fs::read_to_string(document)
        .map_err(|error| format!("cannot read {}: {error}", document.display()))?;
    let base = document.parent().unwrap_or_else(|| Path::new("."));
    let file = document
        .file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().into_owned());
    let steps = key_steps(&options.keys);
    let runs = options.repeat.max(1);
    let baseline_kib = peak_kib();
    let mut states = Vec::with_capacity(steps.len() + 1);
    for state in 0..=steps.len() {
        let mut open = Vec::with_capacity(runs);
        let mut replay = Vec::with_capacity(runs);
        let mut step = Vec::with_capacity(runs);
        let mut geometry = Vec::with_capacity(runs);
        let mut regions = 0;
        for _ in 0..runs {
            let started = Instant::now();
            let mut app = drive::open(&text, base, &options.state_dir, &options.target)?;
            open.push(started.elapsed());
            let mut typed = Duration::ZERO;
            let mut last = Duration::ZERO;
            for key in &steps[..state] {
                let started = Instant::now();
                app.keys(key);
                app.advance(Duration::ZERO);
                last = started.elapsed();
                typed += last;
            }
            replay.push(typed);
            step.push(last);
            let started = Instant::now();
            let drawn = drive::draw(&app, options.size.0, options.size.1);
            geometry.push(started.elapsed());
            regions = drawn.len();
        }
        states.push(StateCost {
            state,
            key: if state == 0 {
                String::new()
            } else {
                steps[state - 1].clone()
            },
            open: median(&mut open),
            replay: median(&mut replay),
            step: median(&mut step),
            geometry: median(&mut geometry),
            regions,
            peak_kib: peak_kib(),
        });
    }
    Ok(Cost {
        file,
        target: options.target.clone(),
        size: options.size,
        runs,
        keys: steps,
        baseline_kib,
        states,
    })
}

/// The first line of the report: the layout model and the ESS release it was measured on.
pub fn header() -> String {
    format!(
        "# ux-search cost: layout model v0, the terminal renderer of ESS {}, in process",
        crate::ESS_TAG
    )
}

impl Cost {
    /// The mean open, step and geometry times over the states: the step over the states that
    /// typed a key.
    pub fn means(&self) -> (Duration, Duration, Duration) {
        let mean = |durations: Vec<Duration>| {
            let count = u32::try_from(durations.len()).unwrap_or(u32::MAX);
            if count == 0 {
                return Duration::ZERO;
            }
            durations.iter().sum::<Duration>() / count
        };
        (
            mean(self.states.iter().map(|state| state.open).collect()),
            mean(
                self.states
                    .iter()
                    .filter(|state| state.state > 0)
                    .map(|state| state.step)
                    .collect(),
            ),
            mean(self.states.iter().map(|state| state.geometry).collect()),
        )
    }

    /// The cost of one evaluation of a state `depth` keys deep, each state replayed from open:
    /// open + depth x step + geometry, from [`Cost::means`].
    pub fn evaluation(&self, depth: u32) -> Duration {
        let (open, step, geometry) = self.means();
        open + step * depth + geometry
    }

    /// The report as text: a header, one row per state, and the means.
    pub fn text(&self) -> String {
        let mut out = vec![
            header(),
            format!(
                "# document {}, page {}, params {}, size {}x{}, median of {} runs, times in µs",
                self.file,
                self.target.page,
                self.target.params_text(),
                self.size.0,
                self.size.1,
                self.runs
            ),
            format!(
                "# a state is the opened page and its first n keys, replayed from open: ESS {} has no UI state API, so the app cannot be forked",
                crate::ESS_TAG
            ),
            format!(
                "# peak memory before the first open: VmHWM {} KiB",
                optional(self.baseline_kib)
            ),
            format!(
                "{:>5} {:<10} {:>9} {:>9} {:>9} {:>11} {:>7} {:>9}",
                "state", "key", "open", "replay", "step", "geometry", "regions", "peak_kib"
            ),
        ];
        for state in &self.states {
            out.push(format!(
                "{:>5} {:<10} {:>9} {:>9} {:>9} {:>11} {:>7} {:>9}",
                state.state,
                if state.key.is_empty() {
                    "-"
                } else {
                    state.key.as_str()
                },
                state.open.as_micros(),
                state.replay.as_micros(),
                state.step.as_micros(),
                state.geometry.as_micros(),
                state.regions,
                optional(state.peak_kib)
            ));
        }
        let (open, step, geometry) = self.means();
        let depth = u32::try_from(self.keys.len()).unwrap_or(u32::MAX);
        out.push(format!(
            "# mean: open {} µs, step {} µs, geometry {} µs",
            open.as_micros(),
            step.as_micros(),
            geometry.as_micros()
        ));
        out.push(format!(
            "# one evaluation, replayed from open: depth 0 {} µs ({} per 120 s on one core), depth {depth} {} µs ({} per 120 s)",
            self.evaluation(0).as_micros(),
            per_budget(self.evaluation(0)),
            self.evaluation(depth).as_micros(),
            per_budget(self.evaluation(depth))
        ));
        out.push(format!(
            "# one evaluation with a forkable state (step + geometry): {} µs ({} per 120 s)",
            (step + geometry).as_micros(),
            per_budget(step + geometry)
        ));
        let mut text = out.join("\n");
        text.push('\n');
        text
    }

    /// The report as one JSON object, `ux-search-cost/1`, times in µs.
    pub fn json(&self) -> String {
        let params = self
            .target
            .params
            .iter()
            .map(|(name, value)| format!("{}:{}", json_string(name), json_string(value)))
            .collect::<Vec<_>>()
            .join(",");
        let states = self
            .states
            .iter()
            .map(|state| {
                let key = if state.key.is_empty() {
                    "null".to_owned()
                } else {
                    json_string(&state.key)
                };
                format!(
                    "{{\"state\":{},\"key\":{key},\"open_us\":{},\"replay_us\":{},\"step_us\":{},\"geometry_us\":{},\"regions\":{},\"peak_kib\":{}}}",
                    state.state,
                    state.open.as_micros(),
                    state.replay.as_micros(),
                    state.step.as_micros(),
                    state.geometry.as_micros(),
                    state.regions,
                    optional_json(state.peak_kib)
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let (open, step, geometry) = self.means();
        format!(
            "{{\"format\":\"ux-search-cost/1\",\"document\":{},\"page\":{},\"params\":{{{params}}},\"size\":[{},{}],\"runs\":{},\"baseline_peak_kib\":{},\"states\":[{states}],\"mean\":{{\"open_us\":{},\"step_us\":{},\"geometry_us\":{}}}}}\n",
            json_string(&self.file),
            json_string(&self.target.page),
            self.size.0,
            self.size.1,
            self.runs,
            optional_json(self.baseline_kib),
            open.as_micros(),
            step.as_micros(),
            geometry.as_micros()
        )
    }
}

fn median(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples.get(samples.len() / 2).copied().unwrap_or_default()
}

/// How many evaluations of `cost` fit in 120 s on one core.
fn per_budget(cost: Duration) -> u128 {
    let nanos = cost.as_nanos();
    if nanos == 0 {
        return 0;
    }
    Duration::from_secs(120).as_nanos() / nanos
}

fn optional(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_owned(), |value| value.to_string())
}

fn optional_json(value: Option<u64>) -> String {
    value.map_or_else(|| "null".to_owned(), |value| value.to_string())
}

fn json_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            character if u32::from(character) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", u32::from(character));
            }
            character => out.push(character),
        }
    }
    out.push('"');
    out
}
