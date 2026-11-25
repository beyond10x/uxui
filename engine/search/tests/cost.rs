//! The cost spike (`story:cost-spike`): per UI state, the time of open, of each key step replayed
//! from open, of the geometry read, and the peak memory. The times themselves vary from run to
//! run; these tests hold the shape of the measurement, not its numbers.

use std::time::Duration;

use ux_search::cost::{self, Options, key_steps};

fn options(test: &str, keys: &str) -> Options {
    Options {
        target: crate::ticket_page(),
        keys: keys.to_owned(),
        size: (120, 40),
        repeat: 1,
        state_dir: crate::state_dir(test),
    }
}

#[test]
fn a_key_spec_splits_into_steps_as_ess_types_it() {
    assert_eq!(
        key_steps("<tab>jj<c-s><enter><foo>"),
        [
            "<tab>", "j", "j", "<c-s>", "<enter>", "<", "f", "o", "o", ">"
        ]
    );
    assert!(key_steps("").is_empty());
}

#[test]
fn every_state_has_its_open_its_steps_its_geometry_read_and_the_peak_memory() {
    let measured = cost::measure(&crate::portal(), &options("states", "<tab><tab>"))
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(measured.keys, ["<tab>", "<tab>"]);
    assert_eq!(
        measured.states.len(),
        3,
        "the opened page and one state per key"
    );
    for (index, state) in measured.states.iter().enumerate() {
        assert_eq!(state.state, index);
        assert!(state.open > Duration::ZERO, "state {index}: no open time");
        assert!(
            state.geometry > Duration::ZERO,
            "state {index}: no geometry time"
        );
        assert!(state.regions > 0, "state {index}: nothing drawn");
        assert!(
            state.replay >= state.step,
            "state {index}: a step outlasts its replay"
        );
        if cfg!(target_os = "linux") {
            assert!(
                state.peak_kib.is_some_and(|kib| kib > 0),
                "state {index}: no VmHWM"
            );
        }
    }
    assert_eq!(measured.states[0].replay, Duration::ZERO);
    assert_eq!(measured.states[0].key, "");
    assert_eq!(measured.states[2].key, "<tab>");
    assert!(measured.states[2].step > Duration::ZERO);
}

#[test]
fn the_report_names_every_state_as_text_and_as_json() {
    let measured = cost::measure(&crate::portal(), &options("report", "<tab>"))
        .unwrap_or_else(|error| panic!("{error}"));
    let text = measured.text();
    assert!(text.contains("page tickets.detail"), "{text}");
    assert_eq!(
        text.lines().filter(|line| !line.starts_with('#')).count(),
        3,
        "a header row and two states:\n{text}"
    );
    let json = measured.json();
    assert!(
        json.starts_with("{\"format\":\"ux-search-cost/1\""),
        "{json}"
    );
    assert_eq!(json.matches("\"state\":").count(), 2, "{json}");
    assert!(json.contains("\"key\":\"<tab>\""), "{json}");
    assert!(json.ends_with("}\n"), "{json}");
}

#[test]
fn a_page_the_document_lacks_is_refused() {
    let mut options = options("missing", "");
    options.target.page = "no.such.page".to_owned();
    let error = cost::measure(&crate::portal(), &options).expect_err("no such page");
    assert!(error.contains("no.such.page"), "{error}");
}
