use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "usage: uxui-gen --path <ess-ui document> --model <ESS specification dir> --page <page> --out <dir> [--theme <uxui theme file>]...";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let (mut path, mut model, mut page, mut out) = (None, None, None, None);
    let mut themes: Vec<PathBuf> = Vec::new();
    while let Some(arg) = args.next() {
        let slot = match arg.as_str() {
            "--path" => &mut path,
            "--model" => &mut model,
            "--page" => &mut page,
            "--out" => &mut out,
            "--theme" => {
                themes.extend(args.next().map(PathBuf::from));
                continue;
            }
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("uxui-gen: unknown argument `{other}`\n{USAGE}");
                return ExitCode::from(2);
            }
        };
        *slot = args.next();
    }
    let (Some(path), Some(model), Some(page), Some(out)) = (path, model, page, out) else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    match uxui_gen::generate(&PathBuf::from(path), &PathBuf::from(model), &page, &PathBuf::from(&out), &themes.iter().map(PathBuf::as_path).collect::<Vec<_>>()) {
        Ok(report) => {
            for warning in &report.warnings {
                eprintln!("warning {warning}");
            }
            println!("{} files written to {out}", report.files.len());
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("uxui-gen: {message}");
            ExitCode::FAILURE
        }
    }
}
