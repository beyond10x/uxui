//! uxui-cells <W>x<H> <state-dir> <doc.ui.yaml>...
//!
//! For each document, in one process: `ess ui check` (no model), then open the home page on the
//! terminal renderer, draw one frame of W x H cells and print one JSON line with the check's error
//! count, the drawn screen, every region the renderer recorded (node path and cell box) and the
//! time each step took in microseconds. Path dependencies expect an ESS checkout beside this
//! repository (`../ess`).

use std::path::{Path, PathBuf};
use std::time::Instant;

use serde_json::json;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 3 {
        eprintln!("usage: uxui-cells <W>x<H> <state-dir> <doc.ui.yaml>...");
        std::process::exit(2);
    }
    let (w, h) = args[0]
        .split_once('x')
        .map(|(w, h)| (w.parse::<u16>().unwrap_or(80), h.parse::<u16>().unwrap_or(24)))
        .unwrap_or((80, 24));
    let state = PathBuf::from(&args[1]);
    for doc in &args[2..] {
        let t0 = Instant::now();
        let text = match std::fs::read_to_string(doc) {
            Ok(t) => t,
            Err(e) => {
                println!("{}", json!({"doc": doc, "error": e.to_string()}));
                continue;
            }
        };
        let base = Path::new(doc).parent().unwrap_or(Path::new("."));
        let report = ess_ui_check::check_source(&text, doc, base, None, &ess_ui_check::Options::default());
        let errors: Vec<String> = report
            .findings
            .iter()
            .filter(|f| f.severity == ess_ui_check::Severity::Error)
            .map(|f| format!("{} {}: {}", f.check, f.path, f.message))
            .collect();
        let check_us = t0.elapsed().as_micros();
        let t1 = Instant::now();
        let app = ess_ui_tui::App::from_text(&text, base, ess_ui_tui::Options::new(state.clone()));
        let open_us = t1.elapsed().as_micros();
        match app {
            Err(e) => println!(
                "{}",
                json!({"doc": doc, "check_errors": errors, "check_us": check_us, "error": format!("{e:?}")})
            ),
            Ok(app) => {
                let t2 = Instant::now();
                let screen = app.render_text(w, h);
                let regions: Vec<_> = app
                    .regions()
                    .iter()
                    .map(|r| json!({"path": r.path, "x": r.area.x, "y": r.area.y, "w": r.area.width, "h": r.area.height}))
                    .collect();
                let draw_us = t2.elapsed().as_micros();
                println!(
                    "{}",
                    json!({
                        "doc": doc,
                        "check_errors": errors,
                        "check_us": check_us,
                        "open_us": open_us,
                        "draw_us": draw_us,
                        "size": [w, h],
                        "screen": screen.lines().collect::<Vec<_>>(),
                        "regions": regions,
                    })
                );
            }
        }
    }
}
