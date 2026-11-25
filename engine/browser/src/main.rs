use std::path::{Path, PathBuf};
use std::process::ExitCode;
use ux_browser::checks::Pages;
use ux_browser::evidence::{self, Header};
use ux_browser::measure::{self, RenderInputs};
use ux_browser::render::{self, FileRef};
use ux_browser::session::Browser;
use ux_browser::target::Site;
use ux_browser::values;

const USAGE: &str = "usage:
  ux-browser render --declarations <ux-browser-render/1 file> --url <url, or a path with --serve> --out <record file>
                    [--screenshot <png file, default: the record file with .png>] [--serve <built site> --base-path <path>]
                    [--root <repository root, default .>] [--capture <file>] [--profile <dir>]
  ux-browser checks --config <ux-browser-checks/1 file> --project <project> --assessment <assessment>
                    (--build <built site> | --url <running server: http(s)://host[:port][/where the base path is]>)
                    [--root <repository root, default .>] [--out <file>] [--class <target class, default local>]
                    [--pages <path,path,...>] [--capture <dir>] [--profile <dir>]
                    [--jobs <pages measured at a time, default: the CPUs, at most 4>] [--values <values file>]
                    [--chrome-arg <a Chromium switch, or a JSON list of them; repeatable>]
  ux-browser values --config <ux-browser-checks/1 file with a values section> --evidence <ratchet-evidence/1 file> --out <values file>
                    [--root <repository root, default .>]
  ux-browser -V | --version
  ux-browser -h | --help";

/// Where a checks run measures.
enum Place<'a> {
    /// A built site the run serves itself (`--build`).
    Build(&'a Path),
    /// A running server the run did not start (`--url`).
    Server(Site),
}

/// The target classes ratchet knows (`evidence.target.class`).
const CLASSES: &[&str] = &["local", "ci", "ephemeral", "staging", "production"];

/// 0: the record or document is written.
/// 1: the inputs are refused (unreadable or malformed files, a page that is not served); nothing written.
/// For `values` and `checks --values` also: the values file is written and reports a problem.
const REFUSED: u8 = 1;
/// 2: the invocation is wrong; nothing written.
const WRONG_INVOCATION: u8 = 2;
/// 3: infrastructure: the browser could not be started or driven, or the output not written.
const NOT_WRITTEN: u8 = 3;

fn wrong(message: &str) -> ExitCode {
    eprintln!("ux-browser: {message}\n{USAGE}");
    ExitCode::from(WRONG_INVOCATION)
}

fn fail(code: u8, message: &str) -> ExitCode {
    eprintln!("ux-browser: {message}");
    ExitCode::from(code)
}

/// The arguments that may be given more than once.
const REPEATABLE: &[&str] = &["chrome-arg"];

/// `--name value` pairs; only names from `known`, each at most once unless it is [`REPEATABLE`].
fn options(args: &[String], known: &[&str]) -> Result<Vec<(String, String)>, String> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let Some(name) = arg.strip_prefix("--").filter(|n| known.contains(n)) else {
            return Err(format!("unknown argument `{arg}`"));
        };
        if !REPEATABLE.contains(&name) && out.iter().any(|(n, _)| n == name) {
            return Err(format!("`{arg}` is given twice"));
        }
        let value = it.next().ok_or_else(|| format!("`{arg}` has no value"))?;
        out.push((name.to_string(), value.clone()));
    }
    Ok(out)
}

fn get<'a>(opts: &'a [(String, String)], name: &str) -> Option<&'a str> {
    opts.iter()
        .find(|(n, _)| n == name)
        .map(|(_, v)| v.as_str())
}

/// The path of `file` relative to `root`, with `/`, when it lies under it.
pub fn relative(root: &Path, file: &Path) -> Option<String> {
    let root = root.canonicalize().ok()?;
    let file = file.canonicalize().ok()?;
    let rel = file.strip_prefix(&root).ok()?;
    Some(
        rel.components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join("/"),
    )
}

fn default_profile() -> PathBuf {
    PathBuf::from("target/ux-browser").join(format!("profile-{}", std::process::id()))
}

fn render_command(args: &[String]) -> ExitCode {
    let known = [
        "declarations",
        "url",
        "out",
        "screenshot",
        "serve",
        "base-path",
        "root",
        "capture",
        "profile",
    ];
    let opts = match options(args, &known) {
        Ok(o) => o,
        Err(e) => return wrong(&e),
    };
    let (Some(decl_path), Some(url), Some(out)) = (
        get(&opts, "declarations"),
        get(&opts, "url"),
        get(&opts, "out"),
    ) else {
        return wrong("`--declarations`, `--url` and `--out` are required");
    };
    if get(&opts, "serve").is_some() != get(&opts, "base-path").is_some() {
        return wrong("`--serve` and `--base-path` go together");
    }
    let root = PathBuf::from(get(&opts, "root").unwrap_or("."));
    if !root.is_dir() {
        return wrong("`--root` is not a directory");
    }
    let shot = get(&opts, "screenshot")
        .map_or_else(|| Path::new(out).with_extension("png"), PathBuf::from);
    let bytes = match std::fs::read(decl_path) {
        Ok(b) => b,
        Err(e) => return fail(REFUSED, &format!("{decl_path}: {e}")),
    };
    let declarations = match render::declarations(&String::from_utf8_lossy(&bytes)) {
        Ok(d) => d,
        Err(e) => return fail(REFUSED, &format!("{decl_path}: {e}")),
    };
    let full_url = match (get(&opts, "serve"), get(&opts, "base-path")) {
        (Some(build), Some(base)) => match ux_browser::serve::start(Path::new(build), base) {
            Ok(server) => format!("{}{url}", server.origin),
            Err(e) => return fail(REFUSED, &e),
        },
        _ => url.to_string(),
    };
    let profile = get(&opts, "profile").map_or_else(default_profile, PathBuf::from);
    let shot_name = shot
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let inputs = RenderInputs {
        url: &full_url,
        declarations: &declarations,
        declarations_file: FileRef {
            path: relative(&root, Path::new(decl_path)),
            digest: measure::digest(&bytes),
        },
        shot_file: &shot_name,
        settle_ms: declarations.settle_ms,
        profile: &profile,
    };
    let mut rendered = match measure::render_url(&inputs) {
        Ok(r) => r,
        Err(e) if e.contains("HTTP status") => return fail(REFUSED, &e),
        Err(e) => return fail(NOT_WRITTEN, &e),
    };
    if let uxui_doc::yaml::Value::Map(entries) = &mut rendered.record
        && let Some((_, v)) = entries.iter_mut().find(|(k, _)| k == "url")
    {
        *v = uxui_doc::yaml::Value::Str(url.to_string());
    }
    if let Err(e) = std::fs::write(&shot, &rendered.png) {
        return fail(NOT_WRITTEN, &format!("{}: {e}", shot.display()));
    }
    if let Err(e) = std::fs::write(out, render::record_text(&rendered.record)) {
        return fail(NOT_WRITTEN, &format!("{out}: {e}"));
    }
    if let Some(capture) = get(&opts, "capture")
        && let Err(e) = std::fs::write(capture, uxui_doc::json::write(&rendered.capture) + "\n")
    {
        return fail(NOT_WRITTEN, &format!("{capture}: {e}"));
    }
    eprintln!("record written to {out}, screenshot to {}", shot.display());
    ExitCode::SUCCESS
}

/// A capture file name for a page path: `home` for the base path, else the path's words.
fn slug(path: &str, base_path: &str) -> String {
    let rest = path
        .strip_prefix(base_path)
        .unwrap_or(path)
        .trim_matches('/');
    if rest.is_empty() {
        return "home".to_string();
    }
    rest.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

fn checks_command(args: &[String]) -> ExitCode {
    let known = [
        "config",
        "build",
        "url",
        "project",
        "assessment",
        "root",
        "out",
        "class",
        "pages",
        "capture",
        "profile",
        "jobs",
        "values",
        "chrome-arg",
    ];
    let opts = match options(args, &known) {
        Ok(o) => o,
        Err(e) => return wrong(&e),
    };
    let mut flag_switches = Vec::new();
    for (_, value) in opts.iter().filter(|(n, _)| n == "chrome-arg") {
        match ux_browser::chrome::flag_switches(value) {
            Ok(s) => flag_switches.extend(s),
            Err(e) => return wrong(&e),
        }
    }
    if let Err(e) = ux_browser::chrome::switches(&flag_switches) {
        return wrong(&format!("`--chrome-arg`: {e}"));
    }
    let jobs = match get(&opts, "jobs") {
        None => ux_browser::pool::default_jobs(ux_browser::pool::cpus()),
        Some(n) => match n.parse::<usize>() {
            Ok(n) if n >= 1 => n,
            _ => return wrong(&format!("`--jobs {n}` is not a whole number of at least 1")),
        },
    };
    let (Some(config_path), Some(project), Some(assessment)) = (
        get(&opts, "config"),
        get(&opts, "project"),
        get(&opts, "assessment"),
    ) else {
        return wrong(
            "`--config`, `--project`, `--assessment` and one of `--build` and `--url` are required",
        );
    };
    let place = match (get(&opts, "build"), get(&opts, "url")) {
        (Some(_), Some(_)) => {
            return wrong(
                "`--build` and `--url` exclude each other: a built site the run serves, or a running server",
            );
        }
        (None, None) => return wrong("one of `--build` and `--url` is required"),
        (Some(build), None) => Place::Build(Path::new(build)),
        (None, Some(url)) => match ux_browser::target::parse_url(url) {
            Ok(site) => Place::Server(site),
            Err(e) => return wrong(&format!("`--url`: {e}")),
        },
    };
    if project.is_empty() || assessment.is_empty() {
        return wrong("`--project` and `--assessment` are not empty");
    }
    let class = get(&opts, "class").unwrap_or("local");
    if !CLASSES.contains(&class) {
        return wrong(&format!("class `{class}` is not one of {CLASSES:?}"));
    }
    let root = PathBuf::from(get(&opts, "root").unwrap_or("."));
    if !root.is_dir() {
        return wrong("`--root` is not a directory");
    }
    let bytes = match std::fs::read(config_path) {
        Ok(b) => b,
        Err(e) => return fail(REFUSED, &format!("{config_path}: {e}")),
    };
    let config = match ux_browser::checks::config(&String::from_utf8_lossy(&bytes)) {
        Ok(c) => c,
        Err(e) => return fail(REFUSED, &format!("{config_path}: {e}")),
    };
    let values_out = get(&opts, "values");
    let values_inputs = match values_out {
        None => None,
        Some(out) => match values_inputs(&config, config_path, &root, out) {
            Ok(i) => Some(i),
            Err(code) => return code,
        },
    };
    // The site, the served build's file reference, and the pages: known before the browser starts,
    // except a running server's sitemap, which the browser reads (`sitemap`).
    let (site, build_file, known, sitemap) = match &place {
        Place::Build(dir) => {
            let digest = match evidence::build_digest(dir) {
                Ok(d) => d,
                Err(e) => return fail(REFUSED, &e),
            };
            let paths = match measure::pages(&config, dir) {
                Ok(p) => p,
                Err(e) => return fail(REFUSED, &e),
            };
            let server = match ux_browser::serve::start(dir, &config.base_path) {
                Ok(s) => s,
                Err(e) => return fail(REFUSED, &e),
            };
            let file = FileRef {
                path: relative(&root, dir),
                digest,
            };
            (Site::at(&server.origin), Some(file), Some(paths), None)
        }
        Place::Server(site) => match &config.pages {
            Pages::Paths(paths) => (site.clone(), None, Some(paths.clone()), None),
            Pages::Sitemap(file) => match site.sitemap_url(&config.base_path, file) {
                Ok(url) => (site.clone(), None, None, Some(url)),
                Err(e) => return fail(REFUSED, &format!("{config_path}: {e}")),
            },
        },
    };
    if let Some(paths) = &known
        && let Some(e) = paths
            .iter()
            .find_map(|p| site.page_url(&config.base_path, p).err())
    {
        return fail(REFUSED, &format!("{config_path}: {e}"));
    }
    if let Some(e) = config.journeys.iter().find_map(|j| {
        site.page_url(&config.base_path, &j.page)
            .err()
            .map(|e| format!("journey `{}`: {e}", j.name))
    }) {
        return fail(REFUSED, &format!("{config_path}: {e}"));
    }
    // The browser, after every refusal of the inputs (exit 1) and before anything is measured: a
    // missing browser or a pinned build that is missing ends the run here (exit 3).
    let chrome = match ux_browser::chrome::find(config.chrome) {
        Ok(binary) => binary,
        Err(e) => return fail(NOT_WRITTEN, &e),
    };
    let only: Option<Vec<&str>> = get(&opts, "pages").map(|p| p.split(',').collect());
    let capture = get(&opts, "capture").map(PathBuf::from);
    if let Some(dir) = &capture
        && let Err(e) = std::fs::create_dir_all(dir)
    {
        return fail(NOT_WRITTEN, &format!("{}: {e}", dir.display()));
    }
    let profile = get(&opts, "profile").map_or_else(default_profile, PathBuf::from);
    let started_at = evidence::now();
    let base_path = config.base_path.clone();
    let progress = |path: &str, page: &uxui_doc::yaml::Value| {
        eprintln!("measured {path}");
        if let Some(dir) = &capture {
            let file = dir.join(format!("{}.capture.json", slug(path, &base_path)));
            let _ = std::fs::write(file, uxui_doc::json::write(page) + "\n");
        }
    };
    let journey_progress = |journey: &ux_browser::journey::Journey,
                            record: &uxui_doc::yaml::Value| {
        eprintln!("measured journey `{}`", journey.name);
        if let Some(dir) = &capture {
            let name = ux_browser::journey::capture_name(&journey.name);
            let file = dir.join(format!("{name}.capture.json"));
            let _ = std::fs::write(file, uxui_doc::json::write(record) + "\n");
        }
    };
    // The configuration's switches, then the flag's; each list was checked alone, so together too.
    let added: Vec<String> = config
        .chrome_args
        .iter()
        .chain(&flag_switches)
        .cloned()
        .collect();
    let mut browser = match Browser::start_binary(&chrome, &profile, &added) {
        Ok(b) => b,
        Err(e) => return fail(NOT_WRITTEN, &e),
    };
    let run = (|| {
        let mut paths = match (known, &sitemap) {
            (Some(paths), _) => paths,
            (None, Some(url)) => {
                let guard = measure::guard(&config, &site);
                match measure::fetch_text(&mut browser.conn, &guard, url) {
                    Ok(Ok(xml)) => ux_browser::checks::sitemap_paths(&xml),
                    Ok(Err(e)) => return Err((REFUSED, format!("the sitemap: {e}"))),
                    Err(e) => return Err((NOT_WRITTEN, e)),
                }
            }
            (None, None) => unreachable!("a page list or a sitemap"),
        };
        if let Some(wanted) = &only {
            paths.retain(|p| wanted.contains(&p.as_str()));
        }
        let pages = measure::check_pages(&browser, &config, &site, &paths, jobs, progress)
            .map_err(|e| (NOT_WRITTEN, e))?;
        // The journeys whose page is measured: every one, or with `--pages` those on its pages.
        let journeys: Vec<&ux_browser::journey::Journey> = config
            .journeys
            .iter()
            .filter(|j| only.as_ref().is_none_or(|w| w.contains(&j.page.as_str())))
            .collect();
        let journeys = if journeys.is_empty() {
            Vec::new()
        } else {
            measure::check_journeys(&browser, &config, &site, &journeys, jobs, journey_progress)
                .map_err(|e| (NOT_WRITTEN, e))?
        };
        Ok((pages, journeys))
    })();
    let product = browser.product.clone();
    browser.close();
    let (recorded, recorded_journeys) = match run {
        Ok(r) => r,
        Err((code, e)) => return fail(code, &e),
    };
    let mut pages = Vec::new();
    for page in &recorded {
        match ux_browser::checks::page_result(&config, page) {
            Ok(p) => pages.push(p),
            Err(e) => {
                return fail(
                    NOT_WRITTEN,
                    &format!("a recorded page does not read back: {e}"),
                );
            }
        }
    }
    let mut journeys = Vec::new();
    for record in &recorded_journeys {
        match ux_browser::journey::journey_result(&config, record) {
            Ok(j) => journeys.push(j),
            Err(e) => {
                return fail(
                    NOT_WRITTEN,
                    &format!("a recorded journey does not read back: {e}"),
                );
            }
        }
    }
    let config_file = FileRef {
        path: relative(&root, Path::new(config_path)),
        digest: measure::digest(&bytes),
    };
    let url = site.url();
    let target = match &build_file {
        Some(file) => evidence::Target::Build(file),
        None => evidence::Target::Server(&url),
    };
    let mut local_inputs = vec![("jobs", jobs.to_string())];
    for name in ["capture", "profile", "values"] {
        if let Some(value) = get(&opts, name) {
            local_inputs.push((name, value.to_string()));
        }
    }
    if let Some(switches) = ux_browser::chrome::local_input(&added) {
        local_inputs.push(("chrome-arg", switches));
    }
    let header = Header {
        project,
        assessment,
        class,
        config: &config_file,
        target,
        pages: get(&opts, "pages"),
        local_inputs,
        browser: &product,
        started_at,
        ended_at: evidence::now(),
    };
    let document = evidence::document_with_journeys(&header, &config, &pages, &journeys);
    let text = evidence::text(&document);
    match get(&opts, "out") {
        Some(out) => {
            if let Err(e) = std::fs::write(out, text) {
                return fail(NOT_WRITTEN, &format!("{out}: {e}"));
            }
            eprintln!("evidence written to {out}");
        }
        None => print!("{text}"),
    }
    match (values_out, values_inputs, &config.values) {
        (Some(out), Some(inputs), Some(section)) => write_values(section, &inputs, &document, out),
        _ => ExitCode::SUCCESS,
    }
}

/// For a run that writes a values file to `out`: the configuration's `values` section, and the
/// scenario files and finding records it names, read under `root`. A stale `out` is removed first.
fn values_inputs(
    config: &ux_browser::checks::Config,
    config_path: &str,
    root: &Path,
    out: &str,
) -> Result<values::Inputs, ExitCode> {
    if let Err(e) = std::fs::remove_file(out)
        && e.kind() != std::io::ErrorKind::NotFound
    {
        return Err(fail(NOT_WRITTEN, &format!("{out}: {e}")));
    }
    let Some(section) = &config.values else {
        return Err(fail(
            REFUSED,
            &format!("{config_path}: the configuration declares no `values` section"),
        ));
    };
    values::inputs(section, root).map_err(|e| fail(REFUSED, &e))
}

/// Writes the values file of `document`: 0, or 1 when it reports a problem, 3 when not written.
fn write_values(
    section: &values::ValuesConfig,
    inputs: &values::Inputs,
    document: &uxui_doc::yaml::Value,
    out: &str,
) -> ExitCode {
    let v = match values::values(section, inputs, document) {
        Ok(v) => v,
        Err(e) => return fail(REFUSED, &e),
    };
    if let Err(e) = std::fs::write(out, &v.text) {
        return fail(NOT_WRITTEN, &format!("{out}: {e}"));
    }
    for w in &v.waived {
        eprintln!("ux-browser: waived: {w}");
    }
    for p in &v.problems {
        eprintln!("ux-browser: {p}");
    }
    eprintln!("values written to {out}");
    if v.problems.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(REFUSED)
    }
}

fn values_command(args: &[String]) -> ExitCode {
    let opts = match options(args, &["config", "evidence", "out", "root"]) {
        Ok(o) => o,
        Err(e) => return wrong(&e),
    };
    let (Some(config_path), Some(evidence_path), Some(out)) = (
        get(&opts, "config"),
        get(&opts, "evidence"),
        get(&opts, "out"),
    ) else {
        return wrong("`--config`, `--evidence` and `--out` are required");
    };
    let root = PathBuf::from(get(&opts, "root").unwrap_or("."));
    if !root.is_dir() {
        return wrong("`--root` is not a directory");
    }
    if let Err(e) = std::fs::remove_file(out)
        && e.kind() != std::io::ErrorKind::NotFound
    {
        return fail(NOT_WRITTEN, &format!("{out}: {e}"));
    }
    let config = match std::fs::read_to_string(config_path)
        .map_err(|e| e.to_string())
        .and_then(|t| ux_browser::checks::config(&t))
    {
        Ok(c) => c,
        Err(e) => return fail(REFUSED, &format!("{config_path}: {e}")),
    };
    let inputs = match values_inputs(&config, config_path, &root, out) {
        Ok(i) => i,
        Err(code) => return code,
    };
    let document = match std::fs::read_to_string(evidence_path)
        .map_err(|e| e.to_string())
        .and_then(|t| ux_browser::json::parse(&t))
    {
        Ok(d) => d,
        Err(e) => return fail(REFUSED, &format!("{evidence_path}: {e}")),
    };
    if document.get_str("format") != Some(evidence::FORMAT) {
        return fail(
            REFUSED,
            &format!("{evidence_path}: `format` is not `{}`", evidence::FORMAT),
        );
    }
    let section = config.values.as_ref().expect("values_inputs checked it");
    write_values(section, &inputs, &document, out)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("render") => render_command(&args[1..]),
        Some("checks") => checks_command(&args[1..]),
        Some("values") => values_command(&args[1..]),
        Some("-h" | "--help") => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Some("-V" | "--version") => {
            println!("ux-browser {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some(other) => wrong(&format!("unknown command `{other}`")),
        None => wrong("no command"),
    }
}
