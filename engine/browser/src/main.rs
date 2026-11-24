use std::path::{Path, PathBuf};
use std::process::ExitCode;
use ux_browser::evidence::{self, Header};
use ux_browser::measure::{self, RenderInputs};
use ux_browser::render::{self, FileRef};

const USAGE: &str = "usage:
  ux-browser render --declarations <ux-browser-render/1 file> --url <url, or a path with --serve> --out <record file>
                    [--screenshot <png file, default: the record file with .png>] [--serve <built site> --base-path <path>]
                    [--root <repository root, default .>] [--capture <file>] [--profile <dir>]
  ux-browser checks --config <ux-browser-checks/1 file> --build <built site> --project <project> --assessment <assessment>
                    [--root <repository root, default .>] [--out <file>] [--class <target class, default local>]
                    [--pages <path,path,...>] [--capture <dir>] [--profile <dir>]";

/// The target classes ratchet knows (`evidence.target.class`).
const CLASSES: &[&str] = &["local", "ci", "ephemeral", "staging", "production"];

/// 0: the record or document is written.
/// 1: the inputs are refused (unreadable or malformed files, a page that is not served); nothing written.
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

/// `--name value` pairs; each name at most once and only from `known`.
fn options(args: &[String], known: &[&str]) -> Result<Vec<(String, String)>, String> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let Some(name) = arg.strip_prefix("--").filter(|n| known.contains(n)) else {
            return Err(format!("unknown argument `{arg}`"));
        };
        if out.iter().any(|(n, _)| n == name) {
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
        "project",
        "assessment",
        "root",
        "out",
        "class",
        "pages",
        "capture",
        "profile",
    ];
    let opts = match options(args, &known) {
        Ok(o) => o,
        Err(e) => return wrong(&e),
    };
    let (Some(config_path), Some(build), Some(project), Some(assessment)) = (
        get(&opts, "config"),
        get(&opts, "build"),
        get(&opts, "project"),
        get(&opts, "assessment"),
    ) else {
        return wrong("`--config`, `--build`, `--project` and `--assessment` are required");
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
    let build_dir = Path::new(build);
    let build_digest = match evidence::tree_digest(build_dir) {
        Ok(d) => d,
        Err(e) => return fail(REFUSED, &e),
    };
    let mut paths = match measure::pages(&config, build_dir) {
        Ok(p) => p,
        Err(e) => return fail(REFUSED, &e),
    };
    if let Some(only) = get(&opts, "pages") {
        let wanted: Vec<&str> = only.split(',').collect();
        paths.retain(|p| wanted.contains(&p.as_str()));
    }
    let server = match ux_browser::serve::start(build_dir, &config.base_path) {
        Ok(s) => s,
        Err(e) => return fail(REFUSED, &e),
    };
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
    let (recorded, product) =
        match measure::check_pages(&config, &server.origin, &paths, &profile, progress) {
            Ok(r) => r,
            Err(e) => return fail(NOT_WRITTEN, &e),
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
    let config_file = FileRef {
        path: relative(&root, Path::new(config_path)),
        digest: measure::digest(&bytes),
    };
    let build_file = FileRef {
        path: relative(&root, build_dir),
        digest: build_digest,
    };
    let header = Header {
        project,
        assessment,
        class,
        config: &config_file,
        build: &build_file,
        browser: &product,
        started_at,
        ended_at: evidence::now(),
    };
    let text = evidence::text(&evidence::document(&header, &config, &pages));
    match get(&opts, "out") {
        Some(out) => {
            if let Err(e) = std::fs::write(out, text) {
                return fail(NOT_WRITTEN, &format!("{out}: {e}"));
            }
            eprintln!("evidence written to {out}");
        }
        None => print!("{text}"),
    }
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("render") => render_command(&args[1..]),
        Some("checks") => checks_command(&args[1..]),
        Some("-h" | "--help") => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Some(other) => wrong(&format!("unknown command `{other}`")),
        None => wrong("no command"),
    }
}
