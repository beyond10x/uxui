//! A fixed page clock (`clock: {at, advance}`): the configuration and its refusals, the instant it
//! reads, the script that sets `Date` before the page's own scripts, and the condition the document
//! names. Without the key the page reads the host's clock and every value is 0.4.0's.

use crate::checks::LLM_LIKE;
use ux_browser::checks::{self, Clock};
use ux_browser::evidence::{self, Header, Target};
use ux_browser::measure;
use ux_browser::render::FileRef;
use uxui_doc::yaml::Value;

fn with(extra: &str) -> String {
    format!("{LLM_LIKE}{extra}")
}

#[test]
fn without_clock_the_page_reads_the_host_clock() {
    let c = checks::config(LLM_LIKE).unwrap();
    assert_eq!(c.clock, None);
    assert_eq!(checks::clock_condition(&c), "real");
    assert_eq!(measure::clock_init(&c), None, "no script sets the clock");
}

#[test]
fn reads_an_instant_that_runs_on_or_stands_still() {
    let c = checks::config(&with("clock: {at: 2026-01-01T12:00:20Z}\n")).unwrap();
    assert_eq!(
        c.clock,
        Some(Clock {
            at_ms: 1_767_268_820_000,
            advance: true
        })
    );
    assert_eq!(checks::clock_condition(&c), "from 2026-01-01T12:00:20Z");
    let frozen = checks::config(&with(
        "clock:\n  at: '2026-01-01T13:00:20.25+01:00'\n  advance: false\n",
    ))
    .unwrap();
    assert_eq!(
        frozen.clock,
        Some(Clock {
            at_ms: 1_767_268_820_250,
            advance: false
        })
    );
    assert_eq!(
        checks::clock_condition(&frozen),
        "frozen at 2026-01-01T12:00:20.250Z",
        "the instant is written in UTC"
    );
    let running = checks::config(&with("clock: {at: 2026-01-01T12:00:20Z, advance: true}\n"))
        .unwrap()
        .clock;
    assert_eq!(running.map(|c| c.advance), Some(true));
}

#[test]
fn an_rfc_3339_instant_is_milliseconds_since_the_epoch() {
    for (text, ms) in [
        ("1970-01-01T00:00:00Z", 0),
        ("1970-01-01t00:00:00z", 0),
        ("1969-12-31T23:59:59Z", -1_000),
        ("2000-02-29T00:00:00Z", 951_782_400_000),
        ("2024-02-29T23:59:59Z", 1_709_251_199_000),
        ("2026-01-01T12:00:20Z", 1_767_268_820_000),
        ("2026-01-01T12:00:20.5Z", 1_767_268_820_500),
        ("2026-01-01T12:00:20.123Z", 1_767_268_820_123),
        ("2026-01-01T07:30:20-04:30", 1_767_268_820_000),
        ("2026-01-01T12:00:20+00:00", 1_767_268_820_000),
        ("2026-01-01T12:00:20-00:00", 1_767_268_820_000),
        ("0001-01-01T00:00:00Z", -62_135_596_800_000),
        ("9999-12-31T23:59:59Z", 253_402_300_799_000),
    ] {
        assert_eq!(checks::rfc3339_ms(text), Ok(ms), "{text}");
    }
    for (ms, text) in [
        (0, "1970-01-01T00:00:00Z"),
        (-1_000, "1969-12-31T23:59:59Z"),
        (951_782_400_000, "2000-02-29T00:00:00Z"),
        (1_767_268_820_000, "2026-01-01T12:00:20Z"),
        (1_767_268_820_007, "2026-01-01T12:00:20.007Z"),
    ] {
        let clock = Clock {
            at_ms: ms,
            advance: true,
        };
        assert_eq!(clock.instant(), text, "{ms}");
    }
}

#[test]
fn refuses_a_clock_that_does_not_read() {
    for extra in [
        "clock: 2026-01-01T12:00:20Z\n",
        "clock: {}\n",
        "clock: [2026-01-01T12:00:20Z]\n",
        "clock: {advance: false}\n",
        "clock: {at: 1767268820000}\n",
        "clock: {at: 2026-01-01T12:00:20Z, zone: Europe/Berlin}\n",
        "clock: {at: 2026-01-01T12:00:20Z, advance: 'no'}\n",
        "clock: {at: 2026-01-01T12:00:20}\n",
        "clock: {at: 2026-01-01}\n",
        "clock: {at: '2026-01-01 12:00:20Z'}\n",
        "clock: {at: 2026-01-01T12:00Z}\n",
        "clock: {at: 2026-1-01T12:00:20Z}\n",
        "clock: {at: 2026-13-01T12:00:20Z}\n",
        "clock: {at: 2026-00-01T12:00:20Z}\n",
        "clock: {at: 2026-02-29T12:00:20Z}\n",
        "clock: {at: 1900-02-29T12:00:20Z}\n",
        "clock: {at: 2026-04-31T12:00:20Z}\n",
        "clock: {at: 2026-01-00T12:00:20Z}\n",
        "clock: {at: 2026-01-01T24:00:00Z}\n",
        "clock: {at: 2026-01-01T12:60:00Z}\n",
        "clock: {at: 2026-12-31T23:59:60Z}\n",
        "clock: {at: 2026-01-01T12:00:20.Z}\n",
        "clock: {at: 2026-01-01T12:00:20.1234Z}\n",
        "clock: {at: 2026-01-01T12:00:20+01}\n",
        "clock: {at: 2026-01-01T12:00:20+0100}\n",
        "clock: {at: 2026-01-01T12:00:20+24:00}\n",
        "clock: {at: 2026-01-01T12:00:20+01:60}\n",
        "clock: {at: 2026-01-01T12:00:20Z now}\n",
        "clock: {at: ''}\n",
    ] {
        let refused = checks::config(&with(extra));
        assert!(refused.is_err(), "{extra:?} is refused");
        let message = refused.err().unwrap();
        assert!(message.contains("clock"), "{extra:?}: {message}");
        assert!(
            !message.contains("unknown key `clock`"),
            "{extra:?} is refused for its value, not as an unknown key: {message}"
        );
    }
}

#[test]
fn the_script_sets_date_to_the_instant_before_the_page_s_own_scripts() {
    let c = checks::config(&with("clock: {at: 2026-01-01T12:00:20Z}\n")).unwrap();
    let init = measure::clock_init(&c).expect("a script for a clock");
    assert!(
        init.trim_end().ends_with("(1767268820000,true)"),
        "the instant in milliseconds and whether it runs on: {init}"
    );
    assert!(init.trim_start().starts_with("// "), "a commented probe");
    let frozen =
        checks::config(&with("clock: {at: 2026-01-01T12:00:20Z, advance: false}\n")).unwrap();
    let init_frozen = measure::clock_init(&frozen).unwrap();
    assert!(init_frozen.trim_end().ends_with("(1767268820000,false)"));
    let probe = measure::CLOCK_INIT;
    assert!(
        probe.contains("(function (at, advance)"),
        "called with the instant"
    );
    for part in [
        "window.Date =",
        "Reflect.construct(",
        "now()",
        "performance.now()",
        "'timeOrigin'",
        "Intl.DateTimeFormat.prototype",
    ] {
        assert!(probe.contains(part), "the probe sets {part}");
    }
    for left in [
        "window.setTimeout =",
        "window.setInterval =",
        "requestAnimationFrame =",
        "performance.now =",
    ] {
        assert!(
            !probe.contains(left),
            "timers and frames run as before: {left}"
        );
    }
}

fn document(config: &checks::Config) -> Value {
    let file = FileRef {
        path: Some("checks.yaml".to_string()),
        digest: "sha256:aa".to_string(),
    };
    let build = FileRef {
        path: None,
        digest: "sha256:bb".to_string(),
    };
    let header = Header {
        project: "p",
        assessment: "a",
        class: "local",
        config: &file,
        target: Target::Build(&build),
        pages: None,
        local_inputs: Vec::new(),
        browser: "Chrome/150.0.0.0",
        started_at: "2026-11-25T10:00:00Z".to_string(),
        ended_at: "2026-11-25T10:01:00Z".to_string(),
    };
    evidence::document(&header, config, &[])
}

fn clock_of(doc: &Value) -> Option<String> {
    doc.get("evidence")?
        .get("conditions")?
        .get("other")?
        .get_str("clock")
        .map(str::to_string)
}

#[test]
fn the_document_names_the_clock() {
    let real = checks::config(LLM_LIKE).unwrap();
    assert_eq!(clock_of(&document(&real)).as_deref(), Some("real"));
    let from = checks::config(&with("clock: {at: 2026-01-01T12:00:20Z}\n")).unwrap();
    assert_eq!(
        clock_of(&document(&from)).as_deref(),
        Some("from 2026-01-01T12:00:20Z")
    );
    let frozen =
        checks::config(&with("clock: {at: 2026-01-01T12:00:20Z, advance: false}\n")).unwrap();
    assert_eq!(
        clock_of(&document(&frozen)).as_deref(),
        Some("frozen at 2026-01-01T12:00:20Z")
    );
}

#[test]
fn the_fixture_page_prints_the_time_and_its_configuration_declares_a_clock() {
    let text = std::fs::read_to_string(crate::fixture("clock/clock.checks.yaml")).unwrap();
    let with_clock = checks::config(&text).unwrap();
    assert_eq!(
        with_clock.clock,
        Some(Clock {
            at_ms: 1_767_268_820_000,
            advance: true
        })
    );
    let page = std::fs::read_to_string(crate::fixture("clock/site/index.html")).unwrap();
    assert!(page.contains("new Date().toISOString()"));
    assert!(page.contains("Date.now()"));
    assert!(!page.contains("http://") && !page.contains("https://"));
}
