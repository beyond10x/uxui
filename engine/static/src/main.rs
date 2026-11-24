use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "usage: ux-static --document <ess-ui/1 file> --journeys <ess-ui-test/1 file> --declarations <uxui-declarations/1 file> --project <project> --assessment <assessment> [--root <repository root, default .>] [--out <file>] [--class <target class, default local>]";

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
            other => {
                eprintln!("ux-static: unknown argument `{other}`\n{USAGE}");
                return ExitCode::from(2);
            }
        };
        *slot = args.next();
    }
    let (Some(document), Some(journeys), Some(declarations), Some(project), Some(assessment)) =
        (document, journeys, declarations, project, assessment)
    else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
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
    let evidence = match ux_static::assess(&inputs) {
        Ok(evidence) => evidence,
        Err(message) => {
            eprintln!("ux-static: {message}");
            return ExitCode::FAILURE;
        }
    };
    let text = ux_static::json(&evidence);
    match out {
        Some(path) => {
            if let Err(e) = std::fs::write(&path, text) {
                eprintln!("ux-static: {path}: {e}");
                return ExitCode::FAILURE;
            }
            eprintln!("evidence written to {path}");
        }
        None => print!("{text}"),
    }
    ExitCode::SUCCESS
}
