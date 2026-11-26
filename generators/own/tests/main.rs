//! The one integration-test binary of uxui-gen.

use std::path::{Path, PathBuf};
use uxui_gen::yaml::{self, Value};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn out(name: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(name)
}

#[test]
fn yaml_reads_block_flow_quotes_and_folded_text() {
    let doc = yaml::parse(
        "# a comment\nformat: ess-ui/1\nsummary: >-\n  one\n  two\nlist:\n  - name: a\n    x: {k: [1, two], \"on\": 'it''s'}\n  - plain text # trailing\n  - \"q: #1\"\nempty:\nnums: [0.5, -3, 500ms, null, true]\n",
    )
    .expect("parses");
    assert_eq!(doc.get_str("format"), Some("ess-ui/1"));
    assert_eq!(doc.get_str("summary"), Some("one two"));
    let list = doc.get("list").unwrap().list();
    assert_eq!(list[0].get_str("name"), Some("a"));
    let x = list[0].get("x").unwrap();
    assert_eq!(
        x.get("k").unwrap().list(),
        &[Value::Int(1), Value::Str("two".into())]
    );
    assert_eq!(x.get_str("on"), Some("it's"));
    assert_eq!(list[1].str(), Some("plain text"));
    assert_eq!(list[2].str(), Some("q: #1"));
    assert!(doc.get("empty").unwrap().is_null());
    assert_eq!(
        doc.get("nums").unwrap().list(),
        &[
            Value::Float(0.5),
            Value::Int(-3),
            Value::Str("500ms".into()),
            Value::Null,
            Value::Bool(true)
        ]
    );
}

#[test]
fn yaml_refuses_a_duplicate_key_and_an_open_flow() {
    assert!(yaml::parse("a: 1\na: 2\n").is_err());
    assert!(yaml::parse("a: {b: 1\n").is_err());
}

#[test]
fn every_studio_and_model_file_parses() {
    let mut files = vec![
        repo().join("ess/ui/studio.ui.yaml"),
        repo().join("ess/ess-inputs.yaml"),
    ];
    for dir in [
        "ess/domains",
        "ess/ui/fixtures",
        "ess/ui/fixtures/scripts",
        "ess/ui/tests",
    ] {
        for entry in std::fs::read_dir(repo().join(dir)).unwrap() {
            files.push(entry.unwrap().path());
        }
    }
    for file in files
        .iter()
        .filter(|f| f.extension().is_some_and(|e| e == "yaml"))
    {
        let text = std::fs::read_to_string(file).unwrap();
        yaml::parse(&text).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
    }
}

#[test]
fn the_climb_page_is_resolved_with_paths_defaults_and_live_semantics() {
    let text = std::fs::read_to_string(repo().join("ess/ui/studio.ui.yaml")).unwrap();
    let doc = yaml::parse(&text).unwrap();
    let page = uxui_gen::resolve::page(&doc, "campaigns.climb").unwrap();
    let sections = page.get("sections").unwrap().list();
    let names: Vec<_> = sections
        .iter()
        .map(|s| s.get_str("name").unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "summary",
            "hill_kpis",
            "hill",
            "progress",
            "points",
            "frontier"
        ]
    );
    let progress = &sections[3];
    let live = progress.get("live").unwrap();
    assert_eq!(live.get_str("match"), Some("climb_point_id"));
    assert_eq!(live.get("coalesce"), Some(&Value::Int(500)));
    let summary = &sections[0];
    let state = &summary.get("fields").unwrap().list()[3];
    assert_eq!(
        state.get_str("path"),
        Some("pages/campaigns.climb/sections/summary/fields/state")
    );
    assert_eq!(state.get("tones").unwrap().get_str("Running"), Some("info"));
    let stop = &page.get("header").unwrap().get("actions").unwrap().list()[0];
    assert_eq!(
        stop.get("confirm").unwrap().get_str("path"),
        Some("pages/campaigns.climb/header/actions/stop/confirm/overlay")
    );
    let points = &sections[4];
    assert_eq!(points.get_str("empty"), Some("Nothing to show yet"));
    assert_eq!(
        points.get("fields").unwrap().list()[4].get_str("label"),
        Some("frontier size")
    );
}

#[test]
fn generation_writes_four_files_and_warns_about_the_thin_progress_event() {
    let dir = out("climb");
    let report = uxui_gen::generate(
        &repo().join("ess/ui/studio.ui.yaml"),
        &repo().join("ess"),
        "campaigns.climb",
        &dir,
        &[],
    )
    .unwrap();
    assert_eq!(
        report.files,
        ["index.html", "page.js", "runtime.js", "style.css"]
    );
    let html = std::fs::read_to_string(dir.join("index.html")).unwrap();
    for path in [
        "pages/campaigns.climb/sections/progress",
        "pages/campaigns.climb/layout/columns/side",
        "pages/campaigns.climb/header/actions/stop",
    ] {
        assert!(html.contains(&format!("data-ui-path=\"{path}\"")), "{path}");
    }
    let data = std::fs::read_to_string(dir.join("page.js")).unwrap();
    assert!(
        data.contains("\"climb_point_id\":\"pt-04\""),
        "the script is embedded"
    );
    assert!(data.contains("\"at\":12000"), "durations are milliseconds");
    assert!(
        data.contains("\"type\":\"List\\u003cuxsearch.search.KpiReading>\""),
        "fields carry their model type"
    );
    let warning = report
        .warnings
        .iter()
        .find(|w| w.contains("sections/progress/live"))
        .expect("a warning");
    assert!(warning.contains("`insufficient`"), "{warning}");
    let css = std::fs::read_to_string(dir.join("style.css")).unwrap();
    assert!(
        css.contains("--ui-color-accent: #2f5bd3;"),
        "tokens use the stock app's variable names, so a uxui theme applies"
    );
    assert!(!html.contains("theme/"));
}

#[test]
fn a_view_the_model_does_not_have_is_refused_with_its_path() {
    let text = std::fs::read_to_string(repo().join("ess/ui/studio.ui.yaml"))
        .unwrap()
        .replace(
            "view: uxsearch.search.Climb, key",
            "view: uxsearch.search.Nowhere, key",
        );
    let dir = out("refused");
    std::fs::create_dir_all(&dir).unwrap();
    let doc = dir.join("studio.ui.yaml");
    std::fs::write(&doc, text).unwrap();
    let error = uxui_gen::generate(
        &doc,
        &repo().join("ess"),
        "campaigns.climb",
        &dir.join("out"),
        &[],
    )
    .unwrap_err();
    assert!(
        error.contains(
            "pages/campaigns.climb/sections/progress/reads: view `uxsearch.search.Nowhere`"
        ),
        "{error}"
    );
}

#[test]
fn a_path_through_an_object_binds_through_its_first_member() {
    let text = std::fs::read_to_string(repo().join("ess/ui/studio.ui.yaml"))
        .unwrap()
        .replace("field: budget.max_variants", "field: nowhere.max_variants");
    let dir = out("member");
    std::fs::create_dir_all(&dir).unwrap();
    let doc = dir.join("studio.ui.yaml");
    std::fs::write(&doc, text).unwrap();
    let error = uxui_gen::generate(
        &doc,
        &repo().join("ess"),
        "campaigns.climb",
        &dir.join("out"),
        &[],
    )
    .unwrap_err();
    assert!(
        error.contains("fields/max_variants: `nowhere.max_variants` is not a field of"),
        "{error}"
    );
    assert!(!error.contains("max_wall_time_s"), "{error}");
}

#[test]
fn a_theme_is_copied_beside_the_page_its_stylesheet_after_the_page_s_and_its_script_in_the_head() {
    let dir = out("themed");
    std::fs::create_dir_all(&dir).unwrap();
    let (css, js) = (dir.join("brand.css"), dir.join("mode-toggle.js"));
    std::fs::write(&css, ":root { --ui-color-accent: #5b5bd6; }\n").unwrap();
    std::fs::write(
        &js,
        "document.documentElement.dataset.uxuiMode = \"dark\";\n",
    )
    .unwrap();
    let page = dir.join("out");
    let report = uxui_gen::generate(
        &repo().join("ess/ui/studio.ui.yaml"),
        &repo().join("ess"),
        "campaigns.climb",
        &page,
        &[css.as_path(), js.as_path()],
    )
    .unwrap();
    assert!(report.files.ends_with(&[
        "theme/brand.css".to_string(),
        "theme/mode-toggle.js".to_string()
    ]));
    let html = std::fs::read_to_string(page.join("index.html")).unwrap();
    let at = |needle: &str| html.find(needle).unwrap_or_else(|| panic!("{needle}"));
    assert!(
        at("href=\"style.css\"") < at("href=\"theme/brand.css\""),
        "the theme wins over the base stylesheet"
    );
    assert!(
        at("src=\"theme/mode-toggle.js\"") < at("</head>"),
        "the mode is set before the body is drawn"
    );
    assert_eq!(
        std::fs::read_to_string(page.join("theme/brand.css")).unwrap(),
        ":root { --ui-color-accent: #5b5bd6; }\n"
    );
}
