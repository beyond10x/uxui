//! Design-token checks (`tokens`, `docs/browser-adapter.md`, Design tokens): font sizes, colours,
//! gaps and paddings, and radii off a declared token set, and the lowest text contrast, per page,
//! viewport and mode.
//!
//! `fixtures/checks/tokens-site/` measured at 400 x 300 in light and dark mode
//! (`ux-browser checks --capture`, a real headless run): `/on-scale/` uses only its tokens,
//! `/offenders/` plants one offender of each kind (a font size of 18 px, the colour `#2e7d32`, a gap
//! of 13 px and a padding of 10 px, a radius of 6 px, faint text). `tokens/` holds the captures
//! with `tokens.checks.yaml`; `tokens-unknown/` the capture of `/on-scale/` with
//! `tokens-unknown.checks.yaml`, which declares no spacing unit and no radii and reads a custom
//! property the page defines in light mode only.

use ux_browser::checks::{self, Config, PageResult, ShiftUnit};
use ux_browser::evidence::{self, Header, Target};
use ux_browser::render::FileRef;
use ux_browser::tokens::{
    self, COLOURS, CONTRAST, FontSizes, Judged, OFFENDERS_CAP, OUTPUTS, Offender, RADII, Radius,
    Rgba, SPACING, TYPE_SCALE, TokenUnit,
};
use ux_browser::values;
use uxui_doc::yaml::Value;

const COUNTS: [&str; 4] = [TYPE_SCALE, COLOURS, SPACING, RADII];

const BASE: &str = "format: ux-browser-checks/1
base_path: /
pages: {paths: [/]}
viewports:
  - {name: 400x300, width: 400, height: 300}
modes: [light, dark]
";

fn config_of(name: &str) -> Config {
    let text = std::fs::read_to_string(crate::fixture(&format!("checks/{name}.checks.yaml")))
        .expect("the configuration is readable");
    checks::config(&text).expect("the configuration reads")
}

fn page(config: &Config, dir: &str, name: &str) -> PageResult {
    let capture = crate::fixture_json(&format!("checks/{dir}/{name}.capture.json"));
    checks::page_result(config, &capture).expect("the capture reads")
}

fn document_of(config: &Config, pages: &[PageResult]) -> Value {
    let file = FileRef {
        path: Some("engine/browser/tests/fixtures/checks/tokens.checks.yaml".to_string()),
        digest: "sha256:aa".to_string(),
    };
    let build = FileRef {
        path: None,
        digest: "sha256:bb".to_string(),
    };
    let header = Header {
        project: "tokens",
        assessment: "page-checks",
        class: "local",
        config: &file,
        target: Target::Build(&build),
        pages: None,
        local_inputs: Vec::new(),
        browser: "Chrome/149.0.7827.55",
        started_at: "2026-11-27T10:00:00Z".to_string(),
        ended_at: "2026-11-27T10:01:00Z".to_string(),
    };
    evidence::document(&header, config, pages)
}

/// The document of the recorded run over both pages of the token site.
pub fn document() -> Value {
    let c = config_of("tokens");
    let pages = [
        page(&c, "tokens", "on-scale"),
        page(&c, "tokens", "offenders"),
    ];
    document_of(&c, &pages)
}

fn result<'a>(doc: &'a Value, output: &str, item: &str) -> &'a Value {
    doc.get("evidence")
        .unwrap()
        .get("results")
        .unwrap()
        .list()
        .iter()
        .find(|r| r.get_str("output") == Some(output) && r.get_str("item") == Some(item))
        .unwrap_or_else(|| panic!("no {output} for {item}"))
}

fn coverage(doc: &Value) -> &Value {
    doc.get("evidence").unwrap().get("coverage").unwrap()
}

fn outcome<'a>(unit: &'a TokenUnit, output: &str) -> &'a Result<Judged, String> {
    let i = OUTPUTS.iter().position(|(o, _)| *o == output).unwrap();
    &unit.outcomes[i]
}

fn judged<'a>(unit: &'a TokenUnit, output: &str) -> &'a Judged {
    outcome(unit, output)
        .as_ref()
        .unwrap_or_else(|e| panic!("{output} {}: not run: {e}", unit.mode))
}

fn has(offenders: &[Offender], node: &str, value: &str) -> bool {
    offenders
        .iter()
        .any(|o| o.node.contains(node) && o.value == value)
}

// ---------------------------------------------------------------- acceptance 1: the section

#[test]
fn reads_the_tokens_section() {
    let c = checks::config(&format!(
        "{BASE}tokens:
  font_sizes: [12, 14, 16]
  colours:
    properties: ['--color-*', --fg]
    values: ['#fff', '#00000080', 'rgb(1, 2, 3)', 'rgba(1, 2, 3, 0.5)', 'rgb(1 2 3 / 50%)']
  spacing_unit_px: 4
  radii: [0, 4, 8.5, full]
"
    ))
    .unwrap();
    let t = c.tokens.expect("a tokens section");
    assert_eq!(t.font_sizes, Some(FontSizes::List(vec![12.0, 14.0, 16.0])));
    let colours = t.colours.expect("colours");
    assert_eq!(colours.properties, ["--color-*", "--fg"]);
    assert_eq!(
        colours.values,
        [
            Rgba::new(255.0, 255.0, 255.0, 1.0),
            Rgba::new(0.0, 0.0, 0.0, 128.0 / 255.0),
            Rgba::new(1.0, 2.0, 3.0, 1.0),
            Rgba::new(1.0, 2.0, 3.0, 0.5),
            Rgba::new(1.0, 2.0, 3.0, 0.5),
        ]
    );
    assert_eq!(t.spacing_unit_px, Some(4.0));
    assert_eq!(
        t.radii,
        Some(vec![
            Radius::Px(0.0),
            Radius::Px(4.0),
            Radius::Px(8.5),
            Radius::Full
        ])
    );
    let scale = checks::config(&format!(
        "{BASE}tokens:\n  font_sizes: {{base: 16, ratio: 1.25}}\n"
    ))
    .unwrap();
    let t = scale.tokens.unwrap();
    assert_eq!(
        t.font_sizes,
        Some(FontSizes::Scale {
            base: 16.0,
            ratio: 1.25
        })
    );
    assert_eq!(t.colours, None, "a kind left out is not declared");
    assert_eq!(t.spacing_unit_px, None);
    assert_eq!(t.radii, None);
    assert!(checks::config(BASE).unwrap().tokens.is_none());
}

#[test]
fn refuses_a_tokens_section_that_does_not_read() {
    for wrong in [
        "tokens: 1\n",
        "tokens: {}\n",
        "tokens: {fonts: [12]}\n",
        "tokens: {font_sizes: []}\n",
        "tokens: {font_sizes: [12, big]}\n",
        "tokens: {font_sizes: [0]}\n",
        "tokens: {font_sizes: 16}\n",
        "tokens: {font_sizes: {base: 16}}\n",
        "tokens: {font_sizes: {base: 16, ratio: 1}}\n",
        "tokens: {font_sizes: {base: 16, ratio: 1.25, step: 2}}\n",
        "tokens: {colours: {}}\n",
        "tokens: {colours: {properties: []}}\n",
        "tokens: {colours: {properties: [color]}}\n",
        "tokens: {colours: {properties: ['--a*b']}}\n",
        "tokens: {colours: {properties: '--x'}}\n",
        "tokens: {colours: {values: [blue]}}\n",
        "tokens: {colours: {values: ['#12']}}\n",
        "tokens: {colours: {values: ['rgb(1, 2)']}}\n",
        "tokens: {colours: {shades: ['#fff']}}\n",
        "tokens: {spacing_unit_px: 0}\n",
        "tokens: {spacing_unit_px: four}\n",
        "tokens: {radii: []}\n",
        "tokens: {radii: [4, round]}\n",
        "tokens: {radii: [-1]}\n",
        "tokens: {radii: 4}\n",
    ] {
        let error = match checks::config(&format!("{BASE}{wrong}")) {
            Ok(_) => panic!("read: {wrong}"),
            Err(e) => e,
        };
        assert!(error.contains("tokens"), "{wrong}: {error}");
    }
}

/// A refused section exits 1 before a browser starts, as any configuration that does not read.
#[test]
fn a_refused_section_exits_1_before_a_browser_starts() {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("tokens-refused");
    std::fs::create_dir_all(&dir).unwrap();
    let config = dir.join("checks.yaml");
    std::fs::write(&config, format!("{BASE}tokens: {{radii: []}}\n")).unwrap();
    let o = std::process::Command::new(env!("CARGO_BIN_EXE_ux-browser"))
        .args(["checks", "--config"])
        .arg(&config)
        .args([
            "--build",
            crate::fixture("checks/tokens-site").to_str().unwrap(),
            "--project",
            "p",
            "--assessment",
            "a",
            "--out",
        ])
        .arg(dir.join("out.json"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("tokens"), "{stderr}");
}

// ---------------------------------------------------------------- acceptance 2: on the tokens

#[test]
fn a_page_on_its_tokens_counts_nothing_off_them_in_both_modes() {
    let c = config_of("tokens");
    let p = page(&c, "tokens", "on-scale");
    let modes: Vec<&str> = p.tokens.iter().map(|u| u.mode.as_str()).collect();
    assert_eq!(modes, ["light", "dark"]);
    for u in &p.tokens {
        for output in COUNTS {
            let j = judged(u, output);
            assert_eq!(j.value, 0.0, "{output} {}: {:?}", u.mode, j.offenders);
            assert!(j.offenders.is_empty());
        }
        let contrast = judged(u, CONTRAST);
        assert!(contrast.value >= 4.5, "{}: {}", u.mode, contrast.value);
        assert!(contrast.offenders.is_empty(), "{:?}", contrast.offenders);
        assert_eq!(contrast.unjudged, 0, "every text has a known background");
    }
    let doc = document_of(&c, &[p]);
    for output in COUNTS {
        let r = result(&doc, output, "/on-scale/");
        assert_eq!(r.get_str("value"), Some("0"), "{output}");
        assert!(r.get("incomplete").is_none(), "{output}");
        assert_eq!(r.get_str("kind"), Some("exact"));
    }
    let r = result(&doc, CONTRAST, "/on-scale/");
    let ratio: f64 = r.get_str("value").unwrap().parse().unwrap();
    assert!(ratio >= 4.5, "{ratio}");
    assert_eq!(r.get_str("unit"), Some("ratio"));
    let cov = coverage(&doc);
    assert_eq!(cov.get("failed"), Some(&Value::Int(0)));
    assert_eq!(cov.get("not_run"), Some(&Value::Int(0)));
}

// ---------------------------------------------------------------- acceptance 3: offenders

#[test]
fn each_planted_offender_is_counted_and_named_in_both_modes() {
    let c = config_of("tokens");
    let p = page(&c, "tokens", "offenders");
    assert_eq!(p.tokens.len(), 2);
    for u in &p.tokens {
        let m = &u.mode;
        let sizes = judged(u, TYPE_SCALE);
        assert_eq!(sizes.value, 1.0, "{m}: {:?}", sizes.offenders);
        assert!(
            has(&sizes.offenders, "off-size", "18px"),
            "{m}: {:?}",
            sizes.offenders
        );
        let colours = judged(u, COLOURS);
        assert_eq!(colours.value, 1.0, "{m}: {:?}", colours.offenders);
        assert!(
            has(&colours.offenders, "off-colour", "#2e7d32"),
            "{m}: {:?}",
            colours.offenders
        );
        assert_eq!(colours.offenders[0].what, "color");
        let spacing = judged(u, SPACING);
        assert_eq!(spacing.value, 2.0, "{m}: {:?}", spacing.offenders);
        assert!(
            has(&spacing.offenders, "off-gap", "13px"),
            "{m}: {:?}",
            spacing.offenders
        );
        assert!(
            has(&spacing.offenders, "off-padding", "10px"),
            "{m}: {:?}",
            spacing.offenders
        );
        let radii = judged(u, RADII);
        assert_eq!(radii.value, 1.0, "{m}: {:?}", radii.offenders);
        assert!(
            has(&radii.offenders, "off-radius", "6px"),
            "{m}: {:?}",
            radii.offenders
        );
        let contrast = judged(u, CONTRAST);
        assert!(contrast.value < 4.5, "{m}: {}", contrast.value);
        assert!(
            contrast.offenders.iter().any(|o| o.node.contains("faint")),
            "{m}: {:?}",
            contrast.offenders
        );
    }
    let doc = document_of(&c, &[p]);
    for output in COUNTS.into_iter().chain([CONTRAST]) {
        let r = result(&doc, output, "/offenders/");
        assert!(r.get("incomplete").is_none(), "{output}");
        let offenders = r
            .get("details")
            .and_then(|d| d.get("offenders"))
            .map(Value::list)
            .unwrap_or_default();
        assert!(
            !offenders.is_empty(),
            "{output}: details name the offenders"
        );
        for mode in ["light", "dark"] {
            assert!(
                offenders.iter().any(|o| o.get_str("mode") == Some(mode)),
                "{output}: an offender in {mode}"
            );
        }
        for o in offenders {
            for key in ["viewport", "mode", "node", "value"] {
                assert!(
                    o.get_str(key).is_some_and(|v| !v.is_empty()),
                    "{output}: {key}"
                );
            }
        }
    }
    assert_eq!(
        result(&doc, TYPE_SCALE, "/offenders/").get_str("value"),
        Some("2")
    );
    assert_eq!(
        result(&doc, SPACING, "/offenders/").get_str("value"),
        Some("4")
    );
    let cov = coverage(&doc);
    assert_eq!(cov.get("not_run"), Some(&Value::Int(0)));
    assert!(
        matches!(cov.get("failed"), Some(Value::Int(n)) if *n >= 10),
        "five token checks fail in each of the two modes"
    );
}

#[test]
fn the_offenders_of_a_result_are_capped() {
    let off = |i: usize| Offender {
        node: format!("p.n{i}"),
        value: format!("{}px", 100 + i),
        what: String::new(),
    };
    let judged = |n: usize| Judged {
        value: n as f64,
        offenders: (0..n).map(off).collect(),
        unjudged: 0,
    };
    let unit = |mode: &str, n: usize| TokenUnit {
        viewport: "400x300".to_string(),
        mode: mode.to_string(),
        outcomes: OUTPUTS
            .iter()
            .map(|(o, _)| {
                if *o == CONTRAST {
                    Ok(Judged {
                        value: 7.0,
                        offenders: Vec::new(),
                        unjudged: 0,
                    })
                } else {
                    Ok(judged(n))
                }
            })
            .collect(),
    };
    let c = checks::config(&format!("{BASE}tokens: {{spacing_unit_px: 4}}\n")).unwrap();
    let mut p = page(&config_of("tokens"), "tokens", "on-scale");
    p.path = "/".to_string();
    p.tokens = vec![unit("light", 18), unit("dark", 12)];
    let doc = document_of(&c, &[p]);
    let r = result(&doc, TYPE_SCALE, "/");
    assert_eq!(r.get_str("value"), Some("30"));
    let d = r.get("details").unwrap();
    assert_eq!(d.get("offenders").unwrap().list().len(), OFFENDERS_CAP);
    assert_eq!(d.get("more"), Some(&Value::Int(30 - OFFENDERS_CAP as i64)));
    let few = result(&doc, CONTRAST, "/").get("details").unwrap();
    assert_eq!(few.get("more"), None, "nothing left out, no `more`");
}

// ---------------------------------------------------------------- acceptance 4: never a pass

#[test]
fn an_undeclared_kind_or_an_undefined_property_is_not_run_never_a_pass() {
    let c = config_of("tokens-unknown");
    let p = page(&c, "tokens-unknown", "on-scale");
    let (light, dark) = (&p.tokens[0], &p.tokens[1]);
    assert_eq!(judged(light, TYPE_SCALE).value, 0.0, "a list of sizes");
    assert_eq!(
        judged(light, COLOURS).value,
        0.0,
        "explicit values beside the property: {:?}",
        judged(light, COLOURS).offenders
    );
    let not_run = |u: &TokenUnit, output: &str| match outcome(u, output) {
        Err(reason) => reason.clone(),
        Ok(j) => panic!("{output} {}: run: {}", u.mode, j.value),
    };
    assert!(not_run(dark, COLOURS).contains("--only-light"));
    for u in [light, dark] {
        assert!(not_run(u, SPACING).contains("spacing"));
        assert!(not_run(u, RADII).contains("radii"));
    }
    let doc = document_of(&c, &[p]);
    for output in [SPACING, RADII, COLOURS] {
        let r = result(&doc, output, "/on-scale/");
        assert_eq!(
            r.get("incomplete").and_then(|i| i.get_str("cause")),
            Some("coverage_below_plan"),
            "{output}"
        );
        assert_eq!(r.get("value"), None, "{output}: no complete 0");
    }
    assert_eq!(
        result(&doc, TYPE_SCALE, "/on-scale/").get_str("value"),
        Some("0")
    );
    assert!(
        result(&doc, CONTRAST, "/on-scale/")
            .get("incomplete")
            .is_none()
    );
    let cov = coverage(&doc);
    assert_eq!(cov.get("not_run"), Some(&Value::Int(5)));
    let ids: Vec<(&str, &str)> = cov
        .get("not_executed")
        .unwrap()
        .list()
        .iter()
        .map(|e| (e.get_str("id").unwrap(), e.get_str("state").unwrap()))
        .collect();
    assert!(
        ids.contains(&("/on-scale//400x300/dark/proxy.colours", "not_run")),
        "{ids:?}"
    );
    assert!(ids.contains(&("/on-scale//400x300/light/proxy.radii", "not_run")));
}

/// A count some unit did not run is a lower bound when it is above 0.
#[test]
fn a_count_some_unit_did_not_run_is_a_lower_bound() {
    let c = config_of("tokens");
    let mut p = page(&c, "tokens", "offenders");
    let i = OUTPUTS.iter().position(|(o, _)| *o == RADII).unwrap();
    p.tokens[1].outcomes[i] = Err("the page defines no radius".to_string());
    let doc = document_of(&c, &[p]);
    let r = result(&doc, RADII, "/offenders/");
    assert_eq!(r.get_str("value"), Some("1"));
    assert_eq!(r.get("at_least"), Some(&Value::Bool(true)));
    assert!(r.get("incomplete").is_some());
}

// ---------------------------------------------------------------- acceptance 5: values

fn two_viewports() -> Config {
    checks::config(
        "format: ux-browser-checks/1
base_path: /
pages: {paths: [/p/]}
viewports:
  - {name: wide, width: 1200, height: 800}
  - {name: narrow, width: 400, height: 800}
modes: [light]
tokens: {font_sizes: [16], spacing_unit_px: 4}
values:
  scenarios: {files: none/*.json}
  waivers: {files: none/*.md}
  gates: {layout.page_scroll@1: 1}
  gating: [wide, narrow]
",
    )
    .unwrap()
}

fn token_unit(viewport: &str, count: f64, contrast: f64) -> TokenUnit {
    TokenUnit {
        viewport: viewport.to_string(),
        mode: "light".to_string(),
        outcomes: OUTPUTS
            .iter()
            .map(|(o, _)| {
                Ok(Judged {
                    value: if *o == CONTRAST { contrast } else { count },
                    offenders: Vec::new(),
                    unjudged: 0,
                })
            })
            .collect(),
    }
}

fn values_of(gating: Option<Vec<String>>) -> Value {
    let c = two_viewports();
    let mut p = page(&config_of("tokens"), "tokens", "on-scale");
    p.path = "/p/".to_string();
    p.units.clear();
    p.shifts = ["wide", "narrow"]
        .map(|v| ShiftUnit {
            viewport: v.to_string(),
            after_ready: Some(0.0),
            load: Some(0.0),
        })
        .into();
    p.tokens = vec![token_unit("wide", 1.0, 7.0), token_unit("narrow", 2.0, 3.2)];
    let doc = document_of(&c, &[p]);
    let mut vc = c.values.unwrap();
    vc.gating = gating;
    let inputs = values::Inputs {
        scenarios: vec![("/p/".to_string(), "page-p".to_string())],
        waivers: Vec::new(),
    };
    let out = values::values(&vc, &inputs, &doc).unwrap();
    ux_browser::json::parse(&out.text).unwrap()
}

#[test]
fn the_values_file_carries_the_token_outputs_and_the_smallest_contrast() {
    let both = values_of(Some(vec!["wide".to_string(), "narrow".to_string()]));
    let v = both.get("values").unwrap().get("page-p").unwrap();
    assert_eq!(v.get("proxy.type_scale"), Some(&Value::Int(3)));
    assert_eq!(v.get("proxy.colours"), Some(&Value::Int(3)));
    assert_eq!(v.get("proxy.spacing_units"), Some(&Value::Int(3)));
    assert_eq!(v.get("proxy.radii"), Some(&Value::Int(3)));
    assert_eq!(
        v.get("a11y.contrast_min"),
        Some(&Value::Float(3.2)),
        "the smallest over the gating viewports, not their sum"
    );
    let narrow = both.get("viewports").unwrap().get("narrow").unwrap();
    assert_eq!(narrow.get("a11y.contrast_min"), Some(&Value::Float(3.2)));
    let wide_only = values_of(Some(vec!["wide".to_string()]));
    let v = wide_only.get("values").unwrap().get("page-p").unwrap();
    assert_eq!(v.get("a11y.contrast_min"), Some(&Value::Int(7)));
    assert_eq!(v.get("proxy.type_scale"), Some(&Value::Int(1)));
    let ungated = values_of(None);
    let v = ungated.get("values").unwrap().get("page-p").unwrap();
    assert_eq!(v.get("a11y.contrast_min"), Some(&Value::Float(3.2)));
}

// ---------------------------------------------------------------- the computation

#[test]
fn a_size_is_on_a_scale_of_base_and_ratio_within_half_a_pixel() {
    let scale = FontSizes::Scale {
        base: 16.0,
        ratio: 1.25,
    };
    for on in [16.0, 20.0, 25.0, 31.0, 12.8, 10.5, 39.06] {
        assert!(tokens::size_on(&scale, on), "{on}");
    }
    for off in [18.0, 14.0, 22.5, 28.0] {
        assert!(!tokens::size_on(&scale, off), "{off}");
    }
    let list = FontSizes::List(vec![12.0, 16.0]);
    assert!(tokens::size_on(&list, 12.4));
    assert!(!tokens::size_on(&list, 12.6));
}

#[test]
fn contrast_is_wcag_s_ratio_with_the_text_colour_composited_on_its_background() {
    let white = Rgba::new(255.0, 255.0, 255.0, 1.0);
    let black = Rgba::new(0.0, 0.0, 0.0, 1.0);
    assert!((tokens::contrast(black, white) - 21.0).abs() < 1e-9);
    let grey = Rgba::new(119.0, 119.0, 119.0, 1.0);
    let r = tokens::contrast(grey, white);
    assert!(r > 4.47 && r < 4.49, "{r}");
    let half = Rgba::new(0.0, 0.0, 0.0, 0.5);
    let r = tokens::contrast(half, white);
    assert!(
        r > 3.9 && r < 4.0,
        "black at half alpha on white is #808080: {r}"
    );
}

#[test]
fn colours_match_within_two_of_255_per_channel() {
    let a = Rgba::new(16.0, 20.0, 24.0, 1.0);
    assert!(tokens::colour_matches(a, Rgba::new(18.0, 18.0, 26.0, 1.0)));
    assert!(!tokens::colour_matches(a, Rgba::new(19.0, 20.0, 24.0, 1.0)));
    assert!(!tokens::colour_matches(a, Rgba::new(16.0, 20.0, 24.0, 0.9)));
}

// ---------------------------------------------------------------- acceptance 6: no section

/// The documents of the test runs that declare no tokens, as text, with `{version}` where the
/// tool's version stands.
fn documents_without_tokens() -> String {
    let mut out = String::new();
    for doc in [
        crate::evidence::document(),
        crate::journey::document(),
        crate::values::round_document(),
    ] {
        out.push_str(&evidence::text(&doc));
    }
    out.replace(
        &format!("\"ux-browser\":\"{}\"", ux_browser::VERSION),
        "\"ux-browser\":\"{version}\"",
    )
}

/// A configuration without `tokens` writes the documents it wrote before. The golden file was
/// written by the build before the `tokens` section existed; `UX_BROWSER_BLESS=1` writes it again.
#[test]
fn without_tokens_the_documents_are_the_bytes_of_before() {
    let golden = crate::fixture("checks/tokens/no-tokens.evidence.txt");
    let text = documents_without_tokens();
    if std::env::var_os("UX_BROWSER_BLESS").is_some() {
        std::fs::create_dir_all(golden.parent().unwrap()).unwrap();
        std::fs::write(&golden, &text).unwrap();
    }
    let want = std::fs::read_to_string(&golden).expect("the golden file is readable");
    assert!(
        text == want,
        "the documents without tokens changed; UX_BROWSER_BLESS=1 writes {} again",
        golden.display()
    );
}

/// A capture of a run without tokens reads with a configuration without tokens, and gives no token
/// result; a configuration with tokens refuses a capture recorded without them.
#[test]
fn a_capture_without_tokens_gives_no_token_result() {
    let c = config_of("hidden");
    let capture = crate::fixture_json("checks/hidden/drawn.capture.json");
    let p = checks::page_result(&c, &capture).unwrap();
    assert!(p.tokens.is_empty());
    let doc = document_of(&c, &[p]);
    for output in COUNTS.into_iter().chain([CONTRAST]) {
        assert!(
            !doc.get("evidence")
                .unwrap()
                .get("results")
                .unwrap()
                .list()
                .iter()
                .any(|r| r.get_str("output") == Some(output)),
            "{output}"
        );
    }
    let text = std::fs::read_to_string(crate::fixture("checks/hidden.checks.yaml")).unwrap();
    let with = checks::config(&format!("{text}tokens: {{spacing_unit_px: 4}}\n")).unwrap();
    let error = match checks::page_result(&with, &capture) {
        Ok(_) => panic!("read a capture without tokens"),
        Err(e) => e,
    };
    assert!(error.contains("tokens"), "{error}");
}
