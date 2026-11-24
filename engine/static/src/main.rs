use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "usage: ux-static --document <ess-ui/1 file> --journeys <ess-ui-test/1 file> --declarations <uxui-declarations/1 file> --project <project> --assessment <assessment> [--root <repository root, default .>] [--out <file>] [--class <target class, default local>]";

/// Exit statuses, as ratchet's assessor contract (`docs/evidence-format.md` § 10) reads them.
/// 0: the document is written.
/// 1: the assessor refuses its inputs; no document.
const REFUSED: u8 = 1;
/// 2: the invocation is wrong; no document.
const WRONG_INVOCATION: u8 = 2;
/// 3: the run could not write its document (`--out`): the assessor's infrastructure, not the
/// inputs. Ratchet records `stopped_by_infrastructure`.
const NOT_WRITTEN: u8 = 3;

fn wrong(message: &str) -> ExitCode {
    eprintln!("ux-static: {message}\n{USAGE}");
    ExitCode::from(WRONG_INVOCATION)
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let (mut document, mut journeys, mut declarations) = (None, None, None);
    let (mut project, mut assessment, mut root, mut out, mut class) =
        (None, None, None, None, None);
    while let Some(arg) = args.next() {
        let slot = match arg.as_str() {
            "--document" => &mut document,
            "--journeys" => &mut journeys,
            "--declarations" => &mut declarations,
            "--project" => &mut project,
            "--assessment" => &mut assessment,
            "--root" => &mut root,
            "--out" => &mut out,
            "--class" => &mut class,
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            other => return wrong(&format!("unknown argument `{other}`")),
        };
        if slot.is_some() {
            return wrong(&format!("`{arg}` is given twice"));
        }
        let Some(value) = args.next() else {
            return wrong(&format!("`{arg}` has no value"));
        };
        *slot = Some(value);
    }
    let (Some(document), Some(journeys), Some(declarations), Some(project), Some(assessment)) =
        (document, journeys, declarations, project, assessment)
    else {
        return wrong("a required argument is missing");
    };
    let (document, journeys, declarations) = (
        PathBuf::from(document),
        PathBuf::from(journeys),
        PathBuf::from(declarations),
    );
    let root = PathBuf::from(root.unwrap_or_else(|| ".".to_string()));
    let class = class.unwrap_or_else(|| "local".to_string());
    let inputs = ux_static::Inputs {
        document: &document,
        journeys: &journeys,
        declarations: &declarations,
        root: &root,
        project: &project,
        assessment: &assessment,
        class: &class,
    };
    if let Err(message) = ux_static::invocation(&inputs) {
        return wrong(&message);
    }
    let evidence = match ux_static::assess(&inputs) {
        Ok(evidence) => evidence,
        Err(message) => {
            eprintln!("ux-static: {message}");
            return ExitCode::from(REFUSED);
        }
    };
    let text = ux_static::json(&evidence);
    match out {
        Some(path) => {
            if let Err(e) = std::fs::write(&path, text) {
                eprintln!("ux-static: {path}: {e}");
                return ExitCode::from(NOT_WRITTEN);
            }
            eprintln!("evidence written to {path}");
        }
        None => print!("{text}"),
    }
    ExitCode::SUCCESS
}
