//! The one integration-test binary of ux-browser. No test starts a browser or opens a socket: the
//! WebSocket framing runs over an in-memory stream, the DevTools-protocol client over a scripted
//! transport, and every check's computation over geometry recorded from a real run
//! (`fixtures/`, written by `ux-browser checks --capture` and `ux-browser render --capture`). The
//! command-line cases run the binary only on invocations it refuses before it starts a browser.

mod assessor;
mod cdp;
mod checks;
mod codec;
mod evidence;
mod json;
mod layout;
mod pool;
mod render;
mod target;
mod values;
mod ws;

use std::path::{Path, PathBuf};
use uxui_doc::yaml::Value;

pub fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

pub fn fixture_json(name: &str) -> Value {
    let text = std::fs::read_to_string(fixture(name)).expect("fixture is readable");
    ux_browser::json::parse(&text).expect("fixture is JSON")
}
