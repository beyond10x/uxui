use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "usage: ux-static --document <ess-ui/1 file> --journeys <ess-ui-test/1 file> --declarations <uxui-declarations/1 file> [--out <file>] [--class <target class>]";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let (mut document, mut journeys, mut declarations, mut out, mut class) =
        (None, None, None, None, None);
    while let Some(arg) = args.next() {
        let slot = match arg.as_str() {
            "--document" => &mut document,
            "--journeys" => &mut journeys,
            "--declarations" => &mut declarations,
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
    let (Some(document), Some(journeys), Some(declarations)) = (document, journeys, declarations)
    else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    let (document, journeys, declarations) = (
        PathBuf::from(document),
        PathBuf::from(journeys),
        PathBuf::from(declarations),
    );
    let class = class.unwrap_or_else(|| "local".to_string());
    let inputs = ux_static::Inputs {
        document: &document,
        journeys: &journeys,
        declarations: &declarations,
        class: &class,
    };
    let records = match ux_static::assess(&inputs) {
        Ok(records) => records,
        Err(message) => {
            eprintln!("ux-static: {message}");
            return ExitCode::FAILURE;
        }
    };
    let text = ux_static::jsonl(&records);
    match out {
        Some(path) => {
            if let Err(e) = std::fs::write(&path, text) {
                eprintln!("ux-static: {path}: {e}");
                return ExitCode::FAILURE;
            }
            eprintln!("{} records written to {path}", records.len());
        }
        None => print!("{text}"),
    }
    ExitCode::SUCCESS
}
