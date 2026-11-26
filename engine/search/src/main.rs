use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ux_search::landing::{promote, record::Recorded, report, run};
use ux_search::profile::Profile;
use ux_search::{campaign, climb, cost, drive};

const USAGE: &str = "usage: ux-search climb --doc <ui.yaml> --page <page> [--param <name>=<value>]... [--sizes <W>x<H>[,<W>x<H>]...] [--cap <1-8>] [--state-dir <dir>]
       ux-search cost --doc <ui.yaml> --page <page> [--param <name>=<value>]... [--keys <key spec>] [--size <W>x<H>] [--repeat <n>] [--format text|json] [--state-dir <dir>]
       ux-search landing run --build <built site> --product <product.json> --css <landing.css> --out <dir> [--jobs <n>] [--profile <dir>] [--rebuilt <variant id>=<built site>]
       ux-search landing report --measurements <measurements.json>
       ux-search landing promote --measurements <measurements.json> --variant <variant id> --product <product.json> --css <landing.css> --evidence <evidence.json> [--chosen-by \"<who chose>\"] [--shipped-file <shipped.txt>]
       ux-search campaign run --declaration <campaign.yaml> --url <origin> --out <measurements.json> [--shots <dir>] [--jobs <n>] [--profile <dir>]
       ux-search campaign report --measurements <measurements.json> [--format text|markdown] [--links <prefix>] [--notes]
       ux-search -V | --version
       ux-search -h | --help, also after a command
defaults: --sizes 120x40,48x40, --cap 5, --keys <tab><tab><tab><tab><tab>, --size 120x40, --repeat 5, --format text, --state-dir target/ux-search/state, --jobs the CPUs (at most 4), --profile a directory of the run's own under the system's temp directory (TMPDIR), removed when the run ends
landing run renders the 144 variants of the documentation landing page in 6 environments in Chromium (UXUI_CHROME, else the Playwright cache), writes <dir>/measurements.json and two screenshots per frontier variant, and prints the report; landing report prints the report of a measurements file; landing promote applies the patch of one variant of that file's frontier to the two sources in place and writes, all or none, one ratchet-evidence/1 document that names the campaign, a copy of the measurements beside it (<name>.measurements.json beside <name>.evidence.json) and, with --shipped-file, the variant's id to that file; the document names each file by its path from the working directory; it refuses, and writes nothing, a variant off the frontier, sources the measurements were not recorded on, a place that is a directory and one file named twice
campaign run reads a declaration ux-search-campaign/1 (the page, the environments, the axes of custom-property values per selector, the hard rules and the ordered terms), loads the page from the server at <origin> (an origin, with a path when the page's paths lie under one), renders every variant in every environment in Chromium as an injected style sheet, with the pointer parked at page.pointer or in the bottom-right corner of the viewport, writes the measurements file and, with --shots, per frontier variant one screenshot in every mode at the first viewport and one in the last environment, and prints the report; it reaches only that origin, with GET and HEAD, and refuses a declaration it cannot read (naming the file and the key), a selector that matches nothing, a property that is not in force and a mode whose expectation (expect) the page does not show; it names a mode that is set by a query and expects nothing as not read back, and an axis none of whose values changes the page; campaign report prints the report of a measurements file, with --notes followed by the first elements behind the counts of the page as shipped and of the frontier, with --format markdown the run and the frontier as two tables whose screenshot links start with <prefix>; it refuses a file whose content is not what its digest names";

/// The flags that take no value.
const SWITCHES: &[&str] = &["--notes"];

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

/// Whether the arguments ask for the usage: `-h` or `--help` as the first word, after a
/// command's words, or where a flag of the command stands. The value of a flag is not a
/// question, and neither is anything after words that name no command.
fn asks_for_usage(args: &[String]) -> bool {
    let word = |at: usize| args.get(at).map(String::as_str);
    let help = |word: Option<&str>| matches!(word, Some("-h" | "--help"));
    let flags = match (word(0), word(1)) {
        (first, _) if help(first) => return true,
        (Some("climb" | "cost"), _) => &args[1..],
        (Some("landing"), Some("run" | "report" | "promote"))
        | (Some("campaign"), Some("run" | "report")) => &args[2..],
        (Some("landing" | "campaign"), second) => return help(second),
        _ => return false,
    };
    let mut rest = flags.iter();
    while let Some(flag) = rest.next() {
        if help(Some(flag)) {
            return true;
        }
        if !SWITCHES.contains(&flag.as_str()) {
            rest.next();
        }
    }
    false
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if asks_for_usage(&args) {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    match args.first().map(String::as_str) {
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
        Some("campaign") => match args.get(1).map(String::as_str) {
            Some("run") => run_campaign(&args[2..]),
            Some("report") => report_campaign(&args[2..]),
            Some(other) => wrong(&format!("unknown command `campaign {other}`")),
            None => wrong("campaign needs `run` or `report`"),
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
        if SWITCHES.contains(&flag.as_str()) {
            if !allowed.contains(&flag.as_str()) {
                return Err(format!("unknown argument `{flag}`"));
            }
            if flags.insert(flag.clone(), String::new()).is_some() {
                return Err(format!("`{flag}` is given twice"));
            }
            continue;
        }
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
    // Held to the end of the run: a profile of the run's own is removed when it is dropped.
    let profile = match Profile::of(parsed.get("--profile")) {
        Ok(profile) => profile,
        Err(message) => return refused(&message),
    };
    let options = run::Options {
        build: PathBuf::from(build),
        product: PathBuf::from(product),
        css: PathBuf::from(css),
        out: PathBuf::from(out),
        profile: profile.path().to_path_buf(),
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

fn run_campaign(args: &[String]) -> ExitCode {
    let parsed = match parse(
        args,
        &[
            "--declaration",
            "--url",
            "--out",
            "--shots",
            "--jobs",
            "--profile",
        ],
    ) {
        Ok(parsed) if parsed.params.is_empty() => parsed,
        Ok(_) => return wrong("campaign run takes no --param"),
        Err(message) => return wrong(&message),
    };
    let jobs = match parsed.get("--jobs").map(str::parse::<usize>) {
        None => ux_browser::pool::default_jobs(ux_browser::pool::cpus()),
        Some(Ok(jobs)) if jobs > 0 => jobs,
        Some(_) => return wrong("--jobs is not a positive whole number"),
    };
    let (Some(declaration), Some(url), Some(out)) = (
        parsed.get("--declaration"),
        parsed.get("--url"),
        parsed.get("--out"),
    ) else {
        return wrong("campaign run needs --declaration, --url and --out");
    };
    let site = match ux_browser::target::parse_url(url) {
        Ok(site) => site,
        Err(message) => return wrong(&format!("--url {message}")),
    };
    // Held to the end of the run: a profile of the run's own is removed when it is dropped.
    let profile = match Profile::of(parsed.get("--profile")) {
        Ok(profile) => profile,
        Err(message) => return refused(&message),
    };
    let options = campaign::run::Options {
        declaration: PathBuf::from(declaration),
        site,
        out: PathBuf::from(out),
        shots: parsed.get("--shots").map(PathBuf::from),
        profile: profile.path().to_path_buf(),
        jobs,
    };
    let started = std::time::Instant::now();
    let outcome = match campaign::run::run(&options) {
        Ok(outcome) => outcome,
        Err(message) => return refused(&message),
    };
    let text = match campaign::report::report(&outcome.recorded) {
        Ok(text) => text,
        Err(message) => return refused(&message),
    };
    for remark in &outcome.remarks {
        eprintln!("ux-search: {remark}");
    }
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

fn report_campaign(args: &[String]) -> ExitCode {
    let parsed = match parse(args, &["--measurements", "--format", "--links", "--notes"]) {
        Ok(parsed) if parsed.params.is_empty() => parsed,
        Ok(_) => return wrong("campaign report takes no --param"),
        Err(message) => return wrong(&message),
    };
    let Some(measurements) = parsed.get("--measurements") else {
        return wrong("campaign report needs --measurements");
    };
    let markdown = match parsed.get("--format") {
        None | Some("text") => false,
        Some("markdown") => true,
        Some(other) => return wrong(&format!("--format `{other}` is not text or markdown")),
    };
    if !markdown && parsed.get("--links").is_some() {
        return wrong("--links goes with --format markdown");
    }
    let notes = parsed.get("--notes").is_some();
    if markdown && notes {
        return wrong("--notes goes with --format text");
    }
    let text = std::fs::read_to_string(measurements)
        .map_err(|error| format!("{measurements}: {error}"))
        .and_then(|text| {
            campaign::record::Recorded::parse(&text).map_err(|why| format!("{measurements}: {why}"))
        })
        .and_then(|recorded| {
            if markdown {
                campaign::report::markdown(&recorded, parsed.get("--links").unwrap_or_default())
            } else if notes {
                Ok(campaign::report::report(&recorded)? + &campaign::report::notes(&recorded)?)
            } else {
                campaign::report::report(&recorded)
            }
        });
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
