//! The live view of a campaign (`ux-search campaign run --events <dir>`): two JSON-lines files
//! the run appends to while it measures, so a page that polls them shows the climb as it goes.
//!
//! - the candidates file ([`CANDIDATES_FILE`]): one line per finished variant, a variant being
//!   finished when every environment has measured it: its terms (`metrics`), its place in the
//!   ranking of the variants finished so far (`score`), whether it stands on the frontier of
//!   those (`accepted`), and its terms against the page as shipped (`deltas`, `worse`);
//! - the phase file ([`PHASE_FILE`]): one line when the run starts, one when a variant starts to
//!   be measured, one after each finished variant with the top three so far (`ranking`), and
//!   one at the end (`end`, or `failed` with the reason).
//!
//! The ranking so far is [`report::rank`] over the variants finished so far, the code
//! `campaign report` ranks with, so the last ranking line names the report's top three. Each
//! line is written with one write to a file opened for appending, so a reader sees whole lines
//! as soon as they are written. Both files are made empty when the run starts.
//!
//! [`Progress`] is what the run feeds from its pages, one per environment, each measuring the
//! variants in order: the page as shipped, then each render. [`replay`] feeds a recorded campaign
//! through the same path.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

use super::declaration::{Declaration, Environment, Variant};
use super::record::{Recorded, Render};
use super::report;
use crate::landing::patch::{self, Value};

/// The candidates file in the events directory.
pub const CANDIDATES_FILE: &str = "hill-climb.jsonl";
/// The phase file in the events directory.
pub const PHASE_FILE: &str = "hill-climb-events.jsonl";
/// The directory under the events directory the screenshots of the live view are written to.
pub const SHOTS_DIR: &str = "shots";

/// The time stamp of a line: local time with its offset, RFC 3339.
pub type Clock = Box<dyn Fn() -> String + Send>;

/// Unix seconds at an offset from UTC (in seconds) as RFC 3339 to the second:
/// `2026-11-27T10:48:05+01:00`.
pub fn rfc3339(unix: i64, offset_seconds: i32) -> String {
    let local = unix + i64::from(offset_seconds);
    let (days, seconds) = (local.div_euclid(86_400), local.rem_euclid(86_400));
    // Days since 1970-01-01 to a civil date (the proleptic Gregorian calendar).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted + 2) / 5 + 1;
    let month = if shifted < 10 {
        shifted + 3
    } else {
        shifted - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    let sign = if offset_seconds < 0 { '-' } else { '+' };
    let offset = offset_seconds.unsigned_abs() / 60;
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}{sign}{:02}:{:02}",
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60,
        offset / 60,
        offset % 60
    )
}

/// The host's offset from UTC now, in seconds, as `date +%z` prints it; 0 when that fails.
fn local_offset() -> i32 {
    let Ok(output) = std::process::Command::new("date").arg("+%z").output() else {
        return 0;
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let text = text.trim();
    let (sign, digits) = match text.split_at_checked(1) {
        Some(("+", digits)) => (1, digits),
        Some(("-", digits)) => (-1, digits),
        _ => return 0,
    };
    if digits.len() != 4 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return 0;
    }
    let (hours, minutes) = digits.split_at(2);
    let minutes = hours.parse::<i32>().unwrap_or(0) * 60 + minutes.parse::<i32>().unwrap_or(0);
    sign * minutes * 60
}

/// The clock of a run: the system's time at the host's offset, read once when the clock is
/// made.
pub fn local_clock() -> Clock {
    let offset = local_offset();
    Box::new(move || {
        let unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| i64::try_from(since.as_secs()).unwrap_or(0));
        rfc3339(unix, offset)
    })
}

fn text(value: &str) -> Value {
    Value::Str(value.to_owned())
}

fn whole(value: u64) -> Value {
    Value::Int(i64::try_from(value).unwrap_or(i64::MAX))
}

fn map(entries: Vec<(&str, Value)>) -> Value {
    Value::Map(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect(),
    )
}

/// What a term where more is better is held as by the report: this less the count.
const MORE_IS_BETTER: u64 = u64::MAX;

/// One term as the report prints it, as a number: `text_under_14` a share (thousandths held,
/// three places printed), `rows_in_view` the count, every other term itself.
fn metric(name: &str, value: u64) -> Value {
    match name {
        "text_under_14" => Value::Float(value as f64 / 1000.0),
        "rows_in_view" => whole(MORE_IS_BETTER - value),
        _ => whole(value),
    }
}

/// A term less the same term of the page as shipped, in the unit of [`metric`].
fn delta(name: &str, value: u64, shipped: u64) -> Value {
    let difference = i128::from(value) - i128::from(shipped);
    match name {
        "text_under_14" => Value::Float(difference as f64 / 1000.0),
        // Held as MORE_IS_BETTER less the count: the count's difference is the other way.
        "rows_in_view" => Value::Int(i64::try_from(-difference).unwrap_or(i64::MAX)),
        _ => Value::Int(i64::try_from(difference).unwrap_or(i64::MAX)),
    }
}

/// The two files of a run's live view, and the variants finished so far.
pub struct Events {
    candidates: File,
    phases: File,
    clock: Clock,
    variants: Vec<Variant>,
    environments: Vec<Environment>,
    /// The index of the variant of the shipped values.
    shipped: usize,
    /// The terms of the page as shipped, once every environment measured it.
    baseline: Option<Vec<u64>>,
    /// The variants finished so far, in order, as a record [`report::rank`] ranks.
    so_far: Recorded,
}

impl Events {
    /// Makes the directory and both files in it, empty.
    pub fn create(dir: &Path, declaration: &Declaration, clock: Clock) -> Result<Events, String> {
        std::fs::create_dir_all(dir).map_err(|error| format!("{}: {error}", dir.display()))?;
        let open = |name: &str| -> Result<File, String> {
            let path = dir.join(name);
            File::create(&path)
                .and_then(|_| OpenOptions::new().append(true).open(&path))
                .map_err(|error| format!("{}: {error}", path.display()))
        };
        let variants = declaration.variants();
        let shipped = declaration.shipped();
        let shipped = variants
            .iter()
            .position(|variant| *variant == shipped)
            .ok_or("the space holds no shipped variant")?;
        Ok(Events {
            candidates: open(CANDIDATES_FILE)?,
            phases: open(PHASE_FILE)?,
            clock,
            environments: declaration.environments(),
            variants,
            shipped,
            baseline: None,
            so_far: Recorded {
                format: super::record::FORMAT,
                adapter: String::new(),
                browser: String::new(),
                declaration: declaration.clone(),
                variants: Vec::new(),
                changing: Vec::new(),
                notes: Vec::new(),
            },
        })
    }

    fn append(file: &mut File, line: &Value) -> Result<(), String> {
        let mut text = patch::compact(line);
        text.push('\n');
        file.write_all(text.as_bytes())
            .and_then(|()| file.flush())
            .map_err(|error| format!("the live view: {error}"))
    }

    fn phase(&mut self, iteration: usize, phase: &str, line: &str) -> Result<(), String> {
        let value = map(vec![
            ("at", text(&(self.clock)())),
            ("iteration", whole(iteration as u64)),
            ("phase", text(phase)),
            ("text", text(line)),
        ]);
        Self::append(&mut self.phases, &value)
    }

    /// The phase `start`: the size of the space and the environments.
    pub fn start(&mut self) -> Result<(), String> {
        let names: Vec<String> = self.environments.iter().map(Environment::name).collect();
        let line = format!(
            "{} variants in {} environments: {}; {} renders",
            self.variants.len(),
            names.len(),
            names.join(", "),
            self.variants.len() * names.len()
        );
        self.phase(0, "start", &line)
    }

    /// The page as shipped, one render per environment: what `deltas` and `worse` are against.
    pub fn baseline(&mut self, renders: Vec<Render>) {
        self.baseline = Some(report::terms(&self.so_far.declaration, &renders));
    }

    /// The phase `measuring`: the variant at `index` starts to be measured.
    pub fn measuring(&mut self, index: usize) -> Result<(), String> {
        let line = format!("measuring {}", self.change(index));
        self.phase(index + 1, "measuring", &line)
    }

    /// The variant's id and its value of each axis: `tight.unset: spacing=tight unread=unset`.
    fn change(&self, index: usize) -> String {
        let variant = &self.variants[index];
        let values: Vec<String> = self
            .so_far
            .declaration
            .axes
            .iter()
            .zip(&variant.choice)
            .map(|(axis, &value)| format!("{}={}", axis.name, axis.values[value].id))
            .collect();
        format!("{}: {}", variant.id, values.join(" "))
    }

    /// The variant at `index` is measured in every environment (`renders`, one per environment):
    /// its candidates line, then the ranking so far. Variants finish in their order; the page
    /// as shipped is known before the first.
    pub fn finished(
        &mut self,
        index: usize,
        renders: Vec<Render>,
        screenshots: &[(String, String)],
    ) -> Result<(), String> {
        if index != self.so_far.variants.len() {
            return Err(format!(
                "the live view: variant {} finished after {} variants",
                index + 1,
                self.so_far.variants.len()
            ));
        }
        let declaration = &self.so_far.declaration;
        let own = report::terms(declaration, &renders);
        let baseline = self.baseline.clone().unwrap_or_else(|| own.clone());
        self.so_far
            .variants
            .push((self.variants[index].id.clone(), renders));
        let ranked = report::rank(&self.so_far);
        let place = ranked
            .ranking
            .iter()
            .position(|&at| at == index)
            .map_or(0, |at| at + 1);
        let names = report::term_names(declaration);
        let worse: Vec<Value> = names
            .iter()
            .zip(own.iter().zip(&baseline))
            .filter(|(_, (own, shipped))| own > shipped)
            .map(|(name, _)| text(name))
            .collect();
        let mut entries = vec![
            ("iteration", whole(index as u64 + 1)),
            ("at", text(&(self.clock)())),
            (
                "kind",
                text(if index == self.shipped {
                    "baseline"
                } else {
                    "candidate"
                }),
            ),
            ("change", text(&self.change(index))),
            ("accepted", Value::Bool(ranked.frontier.contains(&index))),
            ("score", whole(place as u64)),
            (
                "score_keys",
                Value::List(names.iter().map(|name| text(name)).collect()),
            ),
            (
                "metrics",
                map(names
                    .iter()
                    .zip(&own)
                    .map(|(name, &value)| (*name, metric(name, value)))
                    .collect()),
            ),
            (
                "deltas",
                map(names
                    .iter()
                    .zip(own.iter().zip(&baseline))
                    .map(|(name, (&value, &shipped))| (*name, delta(name, value, shipped)))
                    .collect()),
            ),
            ("worse", Value::List(worse)),
        ];
        if !screenshots.is_empty() {
            entries.push((
                "screenshots",
                map(screenshots
                    .iter()
                    .map(|(environment, path)| (environment.as_str(), text(path)))
                    .collect()),
            ));
        }
        let line = map(entries);
        Self::append(&mut self.candidates, &line)?;
        let top: Vec<&str> = ranked
            .ranking
            .iter()
            .take(3)
            .map(|&at| self.so_far.variants[at].0.as_str())
            .collect();
        let line = format!("top 3: {}", top.join(", "));
        self.phase(index + 1, "ranking", &line)
    }

    /// The phase `end`: every variant is measured and the measurements file is written.
    pub fn end(&mut self) -> Result<(), String> {
        let ranked = report::rank(&self.so_far);
        let finished = self.so_far.variants.len();
        let best = ranked
            .ranking
            .first()
            .map_or("", |&at| self.so_far.variants[at].0.as_str());
        let line = format!(
            "{finished} variants measured in {} environments; frontier {}; best {best}",
            self.environments.len(),
            ranked.frontier.len()
        );
        self.phase(finished, "end", &line)
    }

    /// The phase `failed`: the run stopped, for the reason given.
    pub fn failed(&mut self, why: &str) -> Result<(), String> {
        let finished = self.so_far.variants.len();
        self.phase(finished, "failed", why)
    }
}

/// One render of a variant as an environment told it, with its screenshot there (the
/// environment's name and the path) if one was taken.
type Told = (Render, Option<(String, String)>);

/// What the run tells the live view as its pages measure: one page per environment, each
/// measuring the page as shipped, then the variants in order. A variant is finished, and its
/// lines written, when every environment measured it; the variants finish in their order.
pub struct Progress {
    pub events: Events,
    environments: usize,
    built: Vec<Option<Render>>,
    /// Per variant, per environment: the render, and the screenshot's environment name and path.
    renders: Vec<Vec<Option<Told>>>,
    started: Vec<bool>,
    /// The next variant to finish.
    next: usize,
}

impl Progress {
    pub fn new(events: Events, variants: usize, environments: usize) -> Progress {
        Progress {
            events,
            environments,
            built: vec![None; environments],
            renders: vec![vec![None; environments]; variants],
            started: vec![false; variants],
            next: 0,
        }
    }

    /// An environment measured the page as shipped, with no sheet injected.
    pub fn built(&mut self, environment: usize, render: Render) -> Result<(), String> {
        self.built[environment] = Some(render);
        if self.built.iter().all(Option::is_some) {
            self.events
                .baseline(self.built.iter().flatten().copied().collect());
        }
        self.drain()
    }

    /// An environment starts to measure the variant at `index`: the first to do so writes the
    /// phase line.
    pub fn starting(&mut self, index: usize) -> Result<(), String> {
        if std::mem::replace(&mut self.started[index], true) {
            return Ok(());
        }
        self.events.measuring(index)
    }

    /// An environment measured the variant at `index`, with its screenshot there if one was
    /// taken.
    pub fn measured(
        &mut self,
        environment: usize,
        index: usize,
        render: Render,
        shot: Option<(String, String)>,
    ) -> Result<(), String> {
        self.renders[index][environment] = Some((render, shot));
        self.drain()
    }

    /// Writes the lines of every variant that is finished, in order.
    fn drain(&mut self) -> Result<(), String> {
        if self.built.iter().any(Option::is_none) {
            return Ok(());
        }
        while self.next < self.renders.len() && self.renders[self.next].iter().all(Option::is_some)
        {
            let measured: Vec<Told> = self.renders[self.next]
                .iter_mut()
                .flat_map(Option::take)
                .collect();
            let renders = measured.iter().map(|(render, _)| *render).collect();
            let shots: Vec<(String, String)> =
                measured.into_iter().filter_map(|(_, shot)| shot).collect();
            self.events.finished(self.next, renders, &shots)?;
            self.next += 1;
        }
        debug_assert!(self.environments == self.built.len());
        Ok(())
    }
}

/// The path of a variant's live screenshot in an environment, from the events directory.
pub fn shot_path(id: &str, environment: &Environment) -> String {
    format!(
        "{SHOTS_DIR}/{}",
        super::run::screenshot_name(id, environment)
    )
}

/// Feeds a recorded campaign through [`Progress`] as a run would, environment after environment
/// for each variant, and writes both files into `dir`. The page as shipped is the variant of the
/// shipped values: the run measures the two the same, or refuses the campaign.
pub fn replay(recorded: &Recorded, dir: &Path, clock: Clock) -> Result<(), String> {
    let declaration = &recorded.declaration;
    let mut events = Events::create(dir, declaration, clock)?;
    events.start()?;
    let environments = declaration.environments().len();
    let shipped = events.shipped;
    let mut progress = Progress::new(events, recorded.variants.len(), environments);
    let renders = |index: usize| -> Result<&Vec<Render>, String> {
        recorded
            .variants
            .get(index)
            .map(|(_, renders)| renders)
            .filter(|renders| renders.len() == environments)
            .ok_or_else(|| format!("the record has no renders of variant {}", index + 1))
    };
    for (environment, &render) in renders(shipped)?.iter().enumerate() {
        progress.built(environment, render)?;
    }
    for index in 0..recorded.variants.len() {
        progress.starting(index)?;
        for (environment, &render) in renders(index)?.iter().enumerate() {
            progress.measured(environment, index, render, None)?;
        }
    }
    progress.events.end()
}
