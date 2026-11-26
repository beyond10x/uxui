use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ux_search::landing::{promote, record::Recorded, report, run};
use ux_search::{climb, cost, drive};

const USAGE: &str = "usage: ux-search climb --doc <ui.yaml> --page <page> [--param <name>=<value>]... [--sizes <W>x<H>[,<W>x<H>]...] [--cap <1-8>] [--state-dir <dir>]
       ux-search cost --doc <ui.yaml> --page <page> [--param <name>=<value>]... [--keys <key spec>] [--size <W>x<H>] [--repeat <n>] [--format text|json] [--state-dir <dir>]
       ux-search landing run --build <built site> --product <product.json> --css <landing.css> --out <dir> [--jobs <n>] [--profile <dir>] [--rebuilt <variant id>=<built site>]
       ux-search landing report --measurements <measurements.json>
       ux-search landing promote --measurements <measurements.json> --variant <variant id> --product <product.json> --css <landing.css> --evidence <evidence.json> [--chosen-by <text>] [--shipped-file <shipped.txt>]
       ux-search -V | --version
       ux-search -h | --help
defaults: --sizes 120x40,48x40, --cap 5, --keys <tab><tab><tab><tab><tab>, --size 120x40, --repeat 5, --format text, --state-dir target/ux-search/state, --jobs the CPUs (at most 4), --profile target/ux-search/profile-<pid>
landing run renders the 144 variants of the documentation landing page in 6 environments in Chromium (UXUI_CHROME, else the Playwright cache), writes <dir>/measurements.json and two screenshots per frontier variant, and prints the report; landing report prints the report of a measurements file; landing promote applies the patch of one variant of that file's frontier to the two sources in place and writes one ratchet-evidence/1 document that names the campaign (and, with --shipped-file, the variant's id to that file); it refuses, and writes nothing, a variant off the frontier and sources the measurements were not recorded on";

/// 1: the inputs are refused (a document that does not load, a page it lacks); nothing printed.
const REFUSED: u8 = 1;
/// 2: the invocation is wrong.
const WRONG_INVOCATION: u8 = 2;

const DEFAULT_SIZES: &str = "120x40,48x40";
const DEFAULT_SIZE: &str = "120x40";
const DEFAULT_KEYS: &str = "<tab><tab><tab><tab><tab>";
const DEFAULT_REPEAT: usize = 5;
const DEFAULT_STATE_DIR: &str = "target/ux-search/state";
/// The most sections a climb permutes: 8 give 40,320 orders per theme.
const MAX_CAP: usize = 8;

fn wrong(message: &str) -> ExitCode {
    eprintln!("ux-search: {message}\n{USAGE}");
    ExitCode::from(WRONG_INVOCATION)
}

fn refused(message: &str) -> ExitCode {
    eprintln!("ux-search: {message}");
    ExitCode::from(REFUSED)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("-h" | "--help") => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Some("-V" | "--version") => {
            println!("ux-search {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("climb") => run_climb(&args[1..]),
        Some("cost") => run_cost(&args[1..]),
        Some("landing") => match args.get(1).map(String::as_str) {
            Some("run") => run_landing(&args[2..]),
            Some("report") => report_landing(&args[2..]),
            Some("promote") => promote_landing(&args[2..]),
            Some(other) => wrong(&format!("unknown command `landing {other}`")),
            None => wrong("landing needs `run`, `report` or `promote`"),
        },
        Some(other) => wrong(&format!("unknown command `{other}`")),
        None => wrong("no command"),
    }
}

/// The flags of one command: each named once, and `--param` as often as needed.
struct Parsed {
    flags: BTreeMap<String, String>,
    params: Vec<(String, String)>,
}

impl Parsed {
    fn get(&self, flag: &str) -> Option<&str> {
        self.flags.get(flag).map(String::as_str)
    }

    fn target(&self) -> Option<drive::Target> {
        Some(drive::Target {
            page: self.get("--page")?.to_owned(),
            params: self.params.clone(),
        })
    }

    fn state_dir(&self) -> PathBuf {
        PathBuf::from(self.get("--state-dir").unwrap_or(DEFAULT_STATE_DIR))
    }
}

fn parse(args: &[String], allowed: &[&str]) -> Result<Parsed, String> {
    let mut flags = BTreeMap::new();
    let mut params = Vec::new();
    let mut rest = args.iter();
    while let Some(flag) = rest.next() {
        let Some(value) = rest.next() else {
            return Err(format!("`{flag}` has no value"));
        };
        if flag == "--param" {
            let Some((name, value)) = value.split_once('=') else {
                return Err(format!("--param `{value}` is not <name>=<value>"));
            };
            params.push((name.to_owned(), value.to_owned()));
        } else if allowed.contains(&flag.as_str()) {
            if flags.insert(flag.clone(), value.clone()).is_some() {
                return Err(format!("`{flag}` is given twice"));
            }
        } else {
            return Err(format!("unknown argument `{flag}`"));
        }
    }
    Ok(Parsed { flags, params })
}

fn run_climb(args: &[String]) -> ExitCode {
    let parsed = match parse(
        args,
        &["--doc", "--page", "--sizes", "--cap", "--state-dir"],
    ) {
        Ok(parsed) => parsed,
        Err(message) => return wrong(&message),
    };
    let (Some(document), Some(target)) = (parsed.get("--doc"), parsed.target()) else {
        return wrong("climb needs --doc and --page");
    };
    let Some(sizes) = drive::parse_sizes(parsed.get("--sizes").unwrap_or(DEFAULT_SIZES)) else {
        return wrong("--sizes is not <W>x<H>[,<W>x<H>]..., each side from 1 to 1000");
    };
    let cap = match parsed.get("--cap").map(str::parse::<usize>) {
        None => climb::SECTION_CAP,
        Some(Ok(cap)) if (1..=MAX_CAP).contains(&cap) => cap,
        Some(_) => return wrong(&format!("--cap is not a whole number from 1 to {MAX_CAP}")),
    };
    let options = climb::Options {
        target,
        sizes,
        state_dir: parsed.state_dir(),
        cap,
    };
    match climb::climb(Path::new(document), &options) {
        Ok(climb) => {
            print!("{}", climb.render());
            ExitCode::SUCCESS
        }
        Err(message) => refused(&message),
    }
}

fn run_cost(args: &[String]) -> ExitCode {
    let parsed = match parse(
        args,
        &[
            "--doc",
            "--page",
            "--keys",
            "--size",
            "--repeat",
            "--format",
            "--state-dir",
        ],
    ) {
        Ok(parsed) => parsed,
        Err(message) => return wrong(&message),
    };
    let (Some(document), Some(target)) = (parsed.get("--doc"), parsed.target()) else {
        return wrong("cost needs --doc and --page");
    };
    let Some(size) = drive::parse_size(parsed.get("--size").unwrap_or(DEFAULT_SIZE)) else {
        return wrong("--size is not <W>x<H>, each side from 1 to 1000");
    };
    let repeat = match parsed.get("--repeat").map(str::parse::<usize>) {
        None => DEFAULT_REPEAT,
        Some(Ok(repeat)) if repeat > 0 => repeat,
        Some(_) => return wrong("--repeat is not a positive whole number"),
    };
    let json = match parsed.get("--format") {
        None | Some("text") => false,
        Some("json") => true,
        Some(other) => return wrong(&format!("--format `{other}` is not text or json")),
    };
    let options = cost::Options {
        target,
        keys: parsed.get("--keys").unwrap_or(DEFAULT_KEYS).to_owned(),
        size,
        repeat,
        state_dir: parsed.state_dir(),
    };
    match cost::measure(Path::new(document), &options) {
        Ok(measured) => {
            print!(
                "{}",
                if json {
                    measured.json()
                } else {
                    measured.text()
                }
            );
            ExitCode::SUCCESS
        }
        Err(message) => refused(&message),
    }
}

fn run_landing(args: &[String]) -> ExitCode {
    let parsed = match parse(
        args,
        &[
            "--build",
            "--product",
            "--css",
            "--out",
            "--jobs",
            "--profile",
            "--rebuilt",
        ],
    ) {
        Ok(parsed) if parsed.params.is_empty() => parsed,
        Ok(_) => return wrong("landing run takes no --param"),
        Err(message) => return wrong(&message),
    };
    let jobs = match parsed.get("--jobs").map(str::parse::<usize>) {
        None => ux_browser::pool::default_jobs(ux_browser::pool::cpus()),
        Some(Ok(jobs)) if jobs > 0 => jobs,
        Some(_) => return wrong("--jobs is not a positive whole number"),
    };
    let rebuilt = match parsed.get("--rebuilt").map(|value| value.split_once('=')) {
        None => None,
        Some(Some((id, build))) if !id.is_empty() && !build.is_empty() => {
            Some((id.to_owned(), PathBuf::from(build)))
        }
        Some(_) => return wrong("--rebuilt is not <variant id>=<built site>"),
    };
    let (Some(build), Some(product), Some(css), Some(out)) = (
        parsed.get("--build"),
        parsed.get("--product"),
        parsed.get("--css"),
        parsed.get("--out"),
    ) else {
        return wrong("landing run needs --build, --product, --css and --out");
    };
    let options = run::Options {
        build: PathBuf::from(build),
        product: PathBuf::from(product),
        css: PathBuf::from(css),
        out: PathBuf::from(out),
        profile: parsed.get("--profile").map_or_else(
            || PathBuf::from(format!("target/ux-search/profile-{}", std::process::id())),
            PathBuf::from,
        ),
        jobs,
        rebuilt,
    };
    let started = std::time::Instant::now();
    let outcome = match run::run(&options) {
        Ok(outcome) => outcome,
        Err(message) => return refused(&message),
    };
    let text = match report::report(&outcome.recorded) {
        Ok(text) => text,
        Err(message) => return refused(&message),
    };
    for note in &outcome.notes {
        eprintln!("ux-search: hard rule: {note}");
    }
    for check in &outcome.checks {
        eprintln!("ux-search: checked: {check}");
    }
    for path in &outcome.written {
        eprintln!("ux-search: wrote {}", path.display());
    }
    eprintln!(
        "ux-search: {} renders in {}, {:.1} s with {} pages at once",
        outcome.renders,
        outcome.recorded.browser,
        started.elapsed().as_secs_f64(),
        options.jobs
    );
    print!("{text}");
    ExitCode::SUCCESS
}

fn report_landing(args: &[String]) -> ExitCode {
    let parsed = match parse(args, &["--measurements"]) {
        Ok(parsed) if parsed.params.is_empty() => parsed,
        Ok(_) => return wrong("landing report takes no --param"),
        Err(message) => return wrong(&message),
    };
    let Some(measurements) = parsed.get("--measurements") else {
        return wrong("landing report needs --measurements");
    };
    let text = std::fs::read_to_string(measurements)
        .map_err(|error| format!("{measurements}: {error}"))
        .and_then(|text| Recorded::parse(&text).map_err(|why| format!("{measurements}: {why}")))
        .and_then(|recorded| report::report(&recorded));
    match text {
        Ok(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Err(message) => refused(&message),
    }
}

fn promote_landing(args: &[String]) -> ExitCode {
    let parsed = match parse(
        args,
        &[
            "--measurements",
            "--variant",
            "--product",
            "--css",
            "--evidence",
            "--chosen-by",
            "--shipped-file",
        ],
    ) {
        Ok(parsed) if parsed.params.is_empty() => parsed,
        Ok(_) => return wrong("landing promote takes no --param"),
        Err(message) => return wrong(&message),
    };
    let (Some(measurements), Some(variant), Some(product), Some(css), Some(evidence)) = (
        parsed.get("--measurements"),
        parsed.get("--variant"),
        parsed.get("--product"),
        parsed.get("--css"),
        parsed.get("--evidence"),
    ) else {
        return wrong(
            "landing promote needs --measurements, --variant, --product, --css and --evidence",
        );
    };
    if parsed
        .get("--chosen-by")
        .is_some_and(|name| name.trim().is_empty())
    {
        return wrong("--chosen-by is empty");
    }
    let options = promote::Options {
        measurements: PathBuf::from(measurements),
        variant: variant.to_owned(),
        product: PathBuf::from(product),
        css: PathBuf::from(css),
        evidence: PathBuf::from(evidence),
        chosen_by: parsed.get("--chosen-by").map(str::to_owned),
        shipped_file: parsed.get("--shipped-file").map(PathBuf::from),
    };
    match promote::promote(&options) {
        Ok(outcome) => {
            for path in &outcome.written {
                eprintln!("ux-search: wrote {}", path.display());
            }
            for note in &outcome.notes {
                eprintln!("ux-search: note: {note}");
            }
            println!(
                "promoted {variant}: rank {} of {}, on the frontier of {}",
                outcome.rank, outcome.variants, outcome.frontier
            );
            ExitCode::SUCCESS
        }
        Err(message) => refused(&message),
    }
}
