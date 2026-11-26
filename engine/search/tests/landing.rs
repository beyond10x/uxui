//! The campaign over the documentation landing page (`story:docs-site-first-climb`): the space
//! of 144 variants, each a patch to `website/product.json` and `website/src/css/landing.css`,
//! and the ranking of a recorded campaign. No test starts a browser or runs the campaign.
//!
//! `fixtures/docs-landing/` holds the two sources as the campaign of 2026-11-26 measured them
//! (`product.json`, `landing.css`), what one variant writes (`variant.product.json`, made with
//! `jq`, and `variant.landing.css`, made by hand), the measurements of that campaign
//! (`measurements.json`, written by `ux-search landing run`) and the report over them
//! (`report.txt`, written by `ux-search landing report`).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use ux_search::landing::patch::{self, CssRule, Op, Value};
use ux_search::landing::record::{self, Recorded, Render, Route};
use ux_search::landing::report::{self, Terms};
use ux_search::landing::space::{self, AXES, STUDIO_PALETTE};

/// The command that writes the golden report, from the repository root.
const GENERATE: &str = "engine/search/target/debug/ux-search landing report --measurements engine/search/tests/fixtures/docs-landing/measurements.json > engine/search/tests/fixtures/docs-landing/report.txt";

/// The variant `variant.product.json` and `variant.landing.css` are: no axis at its shipped
/// value.
const FAR: &str = "studio.centred.1+3.status-first.d115.h100";

/// The page as shipped: the id `src/landing/shipped.txt` holds, the one place it stands.
fn shipped_id() -> String {
    read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src/landing/shipped.txt"))
        .trim_end()
        .to_owned()
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/docs-landing")
        .join(name)
}

fn repository(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn variant(id: &str) -> space::Variant {
    space::variants()
        .into_iter()
        .find(|variant| variant.id == id)
        .unwrap_or_else(|| panic!("no variant {id}"))
}

fn json(text: &str) -> Value {
    patch::parse(text).unwrap_or_else(|error| panic!("{text}: {error}"))
}

fn recorded() -> Recorded {
    let path = fixture("measurements.json");
    Recorded::parse(&read(&path)).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

// The space.

#[test]
fn the_space_is_144_variants_each_id_once() {
    let sizes: Vec<usize> = AXES.iter().map(|axis| axis.ids.len()).collect();
    assert_eq!(sizes, [2, 2, 3, 2, 3, 2]);
    for (axis, shipped) in AXES.iter().zip(space::shipped().choice) {
        assert_eq!(axis.ids.len(), axis.labels.len(), "{}", axis.name);
        assert!(shipped < axis.ids.len(), "{}", axis.name);
    }
    let variants = space::variants();
    assert_eq!(variants.len(), 144);
    let ids: BTreeSet<&str> = variants.iter().map(|variant| variant.id.as_str()).collect();
    assert_eq!(ids.len(), 144, "an id names two variants");
    let choices: BTreeSet<[usize; 6]> = variants.iter().map(|variant| variant.choice).collect();
    assert_eq!(choices.len(), 144, "a choice is enumerated twice");
    assert_eq!(
        variants[0].id,
        "docs-system.left.1+3.features-first.d85.h85"
    );
    assert_eq!(
        variants[1].id,
        "docs-system.left.1+3.features-first.d85.h100"
    );
    assert_eq!(
        variants[143].id,
        "studio.centred.1+0.status-first.d115.h100"
    );
}

#[test]
fn one_variant_is_the_page_as_shipped() {
    let shipped: Vec<space::Variant> = space::variants()
        .into_iter()
        .filter(space::Variant::is_shipped)
        .collect();
    assert_eq!(shipped.len(), 1);
    assert_eq!(shipped[0].id, shipped_id());
    assert_eq!(space::shipped(), shipped[0]);
    assert!(shipped[0].patch().product.is_empty());
}

// A patch on the sources.

#[test]
fn the_shipped_variant_gives_the_sources_back_byte_for_byte() {
    let pairs = [
        (fixture("product.json"), fixture("landing.css")),
        (
            repository("website/product.json"),
            repository("website/src/css/landing.css"),
        ),
    ];
    for (product, css) in pairs {
        let (product_text, css_text) = (read(&product), read(&css));
        let (patched_product, patched_css) = space::shipped()
            .patch()
            .apply(&product_text, &css_text)
            .unwrap_or_else(|error| panic!("{}: {error}", product.display()));
        assert_eq!(patched_product, product_text, "{}", product.display());
        assert_eq!(patched_css, css_text, "{}", css.display());
    }
}

#[test]
fn a_variant_writes_the_sources_its_axis_values_name() {
    let (product, css) = variant(FAR)
        .patch()
        .apply(
            &read(&fixture("product.json")),
            &read(&fixture("landing.css")),
        )
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(product, read(&fixture("variant.product.json")));
    assert_eq!(css, read(&fixture("variant.landing.css")));
}

/// The site the campaign builds a second time is of a variant with no axis at its shipped
/// value, so the check of the probe's render covers every axis. A promotion that moves the
/// shipped page onto one of that variant's values needs another variant here, in
/// `tools/docs-landing-campaign.sh` and in the two `variant.*` files.
#[test]
fn the_rebuilt_variant_has_no_axis_at_its_shipped_value() {
    let (far, shipped) = (variant(FAR), space::shipped());
    for (at, axis) in AXES.iter().enumerate() {
        assert_ne!(
            far.choice[at], shipped.choice[at],
            "{FAR} has the shipped value of the axis `{}`",
            axis.name
        );
    }
}

/// The campaign script builds a site from `variant.product.json` and `variant.landing.css` and
/// hands it to `landing run --rebuilt` under a variant id: the id of the variant those files are.
#[test]
fn the_campaign_script_names_the_variant_its_second_site_is_built_from() {
    let script = read(&repository("tools/docs-landing-campaign.sh"));
    assert!(
        script.lines().any(|line| line == format!("variant={FAR}")),
        "tools/docs-landing-campaign.sh does not set variant={FAR}"
    );
    for name in ["variant.product.json", "variant.landing.css"] {
        assert!(
            script.contains(&format!("cp \"$fixtures/{name}\" ")),
            "{name}"
        );
    }
    assert_eq!(
        ux_search::landing::run::screenshot_name(FAR, &record::ENVIRONMENTS[5]),
        format!("{FAR}.390x844-dark.png")
    );
}

/// The lines of a shell script that are commands: no comment, no blank line.
fn commands(script: &str) -> impl Iterator<Item = &str> {
    script
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
}

/// The script records into the places these tests read and writes the golden evidence of the
/// module `landing_promote` through that module's own switch, so a recording leaves no file to
/// move and no golden to bless by hand.
#[test]
fn the_campaign_script_records_into_the_places_the_tests_read() {
    let script = read(&repository("tools/docs-landing-campaign.sh"));
    for part in [
        "fixtures=engine/search/tests/fixtures/docs-landing",
        "shots=docs/campaign-2026-11-26-docs-landing-engine",
        "manifest=engine/search/Cargo.toml",
        "cp \"$run/measurements.json\" \"$fixtures/measurements.json\"",
        "\"$bin\" landing report --measurements \"$fixtures/measurements.json\" >\"$fixtures/report.txt\"",
        "rm -f \"$shots\"/*.png",
        "cp \"$run\"/*.png \"$shots/\"",
        "UX_SEARCH_BLESS=1 cargo test --manifest-path \"$manifest\" --locked landing_promote >&2",
    ] {
        assert!(
            commands(&script).any(|line| line == part),
            "tools/docs-landing-campaign.sh has no line `{part}`"
        );
    }
}

/// The second site is built from a copy of `website/`: the script writes neither of the site's
/// two sources, so no other reader of the checkout sees a variant's files in their place.
#[test]
fn the_campaign_script_writes_no_source_of_the_site() {
    let script = read(&repository("tools/docs-landing-campaign.sh"));
    for line in commands(&script) {
        for source in ["\"$product\"", "\"$css\""] {
            assert!(
                !(line.starts_with("cp ") && line.ends_with(source))
                    && !line.contains(&format!(">{source}")),
                "tools/docs-landing-campaign.sh writes {source}: `{line}`"
            );
        }
    }
    assert!(
        commands(&script)
            .any(|line| line == "cp \"$fixtures/variant.product.json\" \"$site/product.json\""),
        "the variant's product.json does not go into the copy of the site"
    );
}

/// A release changes versions and nothing else: no source of the campaign names a version, so a
/// release leaves their digests, and with them the recorded measurements, as they are.
#[test]
fn the_sources_of_the_campaign_name_no_version() {
    for path in [
        repository("website/product.json"),
        repository("website/src/css/landing.css"),
        fixture("product.json"),
        fixture("landing.css"),
        fixture("variant.product.json"),
        fixture("variant.landing.css"),
    ] {
        for line in read(&path).lines() {
            let version = line
                .split(|c: char| !c.is_ascii_digit() && c != '.')
                .map(|word| word.trim_matches('.'))
                .find(|word| crate::is_semver(word));
            assert_eq!(
                version,
                None,
                "{}: `{}` names a version, which the next release would change",
                path.display(),
                line.trim()
            );
        }
    }
}

/// A record names the adapter it was recorded with, as `ux-browser <version>`. The tests hold
/// that form and never the version of this build: a release records nothing again.
#[test]
fn an_adapter_line_is_ux_browser_and_a_bare_semver() {
    for (line, version) in [
        ("ux-browser 0.11.0", "0.11.0"),
        ("ux-browser 10.0.3", "10.0.3"),
        ("ux-browser 0.0.0", "0.0.0"),
    ] {
        assert_eq!(crate::adapter_version(line), Some(version), "{line}");
    }
    for line in [
        "ux-browser",
        "ux-browser ",
        "ux-browser 0.11",
        "ux-browser 0.11.0.1",
        "ux-browser 0.11.0-rc.1",
        "ux-browser v0.11.0",
        "ux-browser 0.11.0 ",
        "ux-browser 00.11.0",
        "ux-browser 0..0",
        "ux-search 0.11.0",
        " ux-browser 0.11.0",
    ] {
        assert_eq!(crate::adapter_version(line), None, "{line:?}");
    }
}

/// The screenshots kept beside the campaign's document are those of the recorded campaign's
/// frontier, one per variant and screenshot environment, and nothing else lies there.
#[test]
fn the_kept_screenshots_are_the_frontier_of_the_recorded_campaign() {
    let recorded = recorded();
    let ranked = report::rank(&recorded);
    let mut expected = BTreeSet::new();
    for &index in &ranked.frontier {
        for at in ux_search::landing::run::SCREENSHOT_ENVIRONMENTS {
            expected.insert(ux_search::landing::run::screenshot_name(
                &ranked.scored[index].id,
                &record::ENVIRONMENTS[at],
            ));
        }
    }
    let directory = repository("docs/campaign-2026-11-26-docs-landing-engine");
    let kept: BTreeSet<String> = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("{}: {error}", directory.display()))
        .map(|entry| {
            entry
                .expect("a directory entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(kept, expected);
}

#[test]
fn every_variant_writes_sources_no_other_variant_writes() {
    let (product, css) = (
        read(&fixture("product.json")),
        read(&fixture("landing.css")),
    );
    let mut written = BTreeSet::new();
    for variant in space::variants() {
        let pair = variant
            .patch()
            .apply(&product, &css)
            .unwrap_or_else(|error| panic!("{}: {error}", variant.id));
        assert!(
            written.insert(pair),
            "{} writes another's sources",
            variant.id
        );
    }
    assert_eq!(written.len(), 144);
}

#[test]
fn a_patch_is_refused_on_sources_it_was_not_made_for() {
    let css = read(&fixture("landing.css"));
    let other = read(&fixture("product.json")).replace("\"/docs/status\"", "\"/docs/roadmap\"");
    let refused = variant("docs-system.left.1+0.features-first.d100.h85")
        .patch()
        .apply(&other, &css)
        .expect_err("the link the patch removes is another one");
    assert!(refused.contains("/product/actions/1/href"), "{refused}");
    let reordered = read(&fixture("variant.product.json"));
    let refused = variant("docs-system.left.1+1.status-first.d100.h85")
        .patch()
        .apply(&reordered, &css)
        .expect_err("the status section is already first");
    assert!(refused.contains("/sections/2/kind"), "{refused}");
}

#[test]
fn the_studio_palette_is_the_colours_of_the_studio_theme() {
    let studio = read(&repository("themes/studio.css"));
    assert_eq!(STUDIO_PALETTE.len(), 22);
    let tokens: BTreeSet<&str> = STUDIO_PALETTE.iter().map(|colour| colour.token).collect();
    assert_eq!(tokens.len(), 22, "a token is written twice");
    for colour in &STUDIO_PALETTE {
        assert!(
            colour.token.starts_with("--b10x-color-"),
            "{}",
            colour.token
        );
        assert_eq!(
            patch::custom_property(&studio, ":root", colour.studio).as_deref(),
            Some(colour.light),
            "{} in light mode",
            colour.studio
        );
        assert_eq!(
            patch::custom_property(&studio, ":root[data-uxui-mode=\"dark\"]", colour.studio)
                .as_deref(),
            Some(colour.dark),
            "{} in dark mode",
            colour.studio
        );
    }
}

// RFC 6902.

fn patched(document: &str, ops: &[Op]) -> Result<String, String> {
    let mut value = json(document);
    patch::apply_ops(&mut value, ops)?;
    Ok(patch::compact(&value))
}

fn add(path: &str, value: &str) -> Op {
    Op::Add {
        path: path.to_owned(),
        value: json(value),
    }
}

fn test(path: &str, value: &str) -> Op {
    Op::Test {
        path: path.to_owned(),
        value: json(value),
    }
}

/// The examples of RFC 6902, appendix A, by their numbers.
#[test]
fn the_operations_do_what_rfc_6902_says() {
    let remove = |path: &str| Op::Remove {
        path: path.to_owned(),
    };
    let moved = |from: &str, path: &str| Op::Move {
        from: from.to_owned(),
        path: path.to_owned(),
    };
    let cases: [(&str, &str, Vec<Op>, &str); 12] = [
        (
            "A.1 add a member",
            r#"{"foo":"bar"}"#,
            vec![add("/baz", r#""qux""#)],
            r#"{"foo":"bar","baz":"qux"}"#,
        ),
        (
            "A.2 add an element",
            r#"{"foo":["bar","baz"]}"#,
            vec![add("/foo/1", r#""qux""#)],
            r#"{"foo":["bar","qux","baz"]}"#,
        ),
        (
            "A.3 remove a member",
            r#"{"baz":"qux","foo":"bar"}"#,
            vec![remove("/baz")],
            r#"{"foo":"bar"}"#,
        ),
        (
            "A.4 remove an element",
            r#"{"foo":["bar","qux","baz"]}"#,
            vec![remove("/foo/1")],
            r#"{"foo":["bar","baz"]}"#,
        ),
        (
            "A.5 replace a value",
            r#"{"baz":"qux","foo":"bar"}"#,
            vec![Op::Replace {
                path: "/baz".to_owned(),
                value: json(r#""boo""#),
            }],
            r#"{"baz":"boo","foo":"bar"}"#,
        ),
        (
            "A.6 move a value",
            r#"{"foo":{"bar":"baz","waldo":"fred"},"qux":{"corge":"grault"}}"#,
            vec![moved("/foo/waldo", "/qux/thud")],
            r#"{"foo":{"bar":"baz"},"qux":{"corge":"grault","thud":"fred"}}"#,
        ),
        (
            "A.7 move an element",
            r#"{"foo":["all","grass","cows","eat"]}"#,
            vec![moved("/foo/1", "/foo/3")],
            r#"{"foo":["all","cows","eat","grass"]}"#,
        ),
        (
            "A.8 a test that holds",
            r#"{"baz":"qux","foo":["a",2,"c"]}"#,
            vec![test("/baz", r#""qux""#), test("/foo/1", "2")],
            r#"{"baz":"qux","foo":["a",2,"c"]}"#,
        ),
        (
            "A.10 add a nested member",
            r#"{"foo":"bar"}"#,
            vec![add("/child", r#"{"grandchild":{}}"#)],
            r#"{"foo":"bar","child":{"grandchild":{}}}"#,
        ),
        (
            "A.14 ~0 and ~1 in a path",
            r#"{"/":9,"~1":10}"#,
            vec![test("/~01", "10"), test("/~1", "9")],
            r#"{"/":9,"~1":10}"#,
        ),
        (
            "A.16 add after the last element",
            r#"{"foo":["bar"]}"#,
            vec![add("/foo/-", r#"["abc","def"]"#)],
            r#"{"foo":["bar",["abc","def"]]}"#,
        ),
        (
            "copy a value",
            r#"{"foo":["bar"]}"#,
            vec![Op::Copy {
                from: "/foo/0".to_owned(),
                path: "/baz".to_owned(),
            }],
            r#"{"foo":["bar"],"baz":"bar"}"#,
        ),
    ];
    for (name, document, ops, expected) in cases {
        assert_eq!(patched(document, &ops).as_deref(), Ok(expected), "{name}");
    }
}

#[test]
fn an_operation_that_cannot_apply_refuses_the_patch() {
    let cases: [(&str, &str, Op); 7] = [
        (
            "A.9 a test that does not hold",
            r#"{"baz":"qux"}"#,
            test("/baz", r#""bar""#),
        ),
        (
            "A.12 add into a member that does not exist",
            r#"{"foo":"bar"}"#,
            add("/baz/bat", r#""qux""#),
        ),
        (
            "A.15 a test compares the value, not its text",
            r#"{"/":9,"~1":10}"#,
            test("/~01", r#""10""#),
        ),
        (
            "remove what is not there",
            r#"{"foo":["bar"]}"#,
            Op::Remove {
                path: "/foo/1".to_owned(),
            },
        ),
        (
            "replace what is not there",
            r#"{"foo":"bar"}"#,
            Op::Replace {
                path: "/baz".to_owned(),
                value: json("1"),
            },
        ),
        (
            "add past the end of a list",
            r#"{"foo":["bar"]}"#,
            add("/foo/2", "1"),
        ),
        (
            "an index with a leading zero",
            r#"{"foo":["bar","baz"]}"#,
            add("/foo/01", "1"),
        ),
    ];
    for (name, document, op) in cases {
        let refused = patched(document, std::slice::from_ref(&op));
        assert!(refused.is_err(), "{name}: {refused:?}");
    }
}

#[test]
fn json_text_is_written_as_the_sources_write_it() {
    let text = "{\n  \"a\": \"x · \\\"y\\\" \\\\ <z>\",\n  \"b\": [\n    1,\n    -2,\n    true,\n    null\n  ],\n  \"c\": {},\n  \"d\": [],\n  \"e\": {\n    \"f\": \"line\\nbreak\"\n  }\n}\n";
    let value = json(text);
    assert_eq!(patch::pretty(&value), text);
    assert_eq!(
        patch::compact(&value),
        r#"{"a":"x · \"y\" \\ <z>","b":[1,-2,true,null],"c":{},"d":[],"e":{"f":"line\nbreak"}}"#
    );
    assert_eq!(
        patch::ops_json(&[
            test("/a/0", r#""x""#),
            Op::Move {
                from: "/a/2".to_owned(),
                path: "/a/0".to_owned()
            },
            Op::Remove {
                path: "/b".to_owned()
            },
        ]),
        r#"[{"op":"test","path":"/a/0","value":"x"},{"op":"move","from":"/a/2","path":"/a/0"},{"op":"remove","path":"/b"}]"#
    );
}

// Custom properties.

const SHEET: &str = "/* a { --x: 9; } */\n:root {\n  --x: 1;\n  --y: calc(2 * var(--x));\n}\n\n.a .b {\n  gap: var(--x);\n}\n\n@media (max-width: 9px) {\n  .c {\n    --x: 3;\n  }\n}\n";

fn rule(selector: &str, values: &[(&str, &str)]) -> CssRule {
    CssRule {
        selector: selector.to_owned(),
        values: values
            .iter()
            .map(|(property, value)| ((*property).to_owned(), (*value).to_owned()))
            .collect(),
    }
}

#[test]
fn a_custom_property_is_set_where_the_sheet_has_it_and_added_where_it_does_not() {
    assert_eq!(
        patch::custom_property(SHEET, ":root", "--x").as_deref(),
        Some("1")
    );
    assert_eq!(
        patch::custom_property(SHEET, ":root", "--y").as_deref(),
        Some("calc(2 * var(--x))")
    );
    assert_eq!(patch::custom_property(SHEET, ":root", "--z"), None);
    assert_eq!(
        patch::custom_property(SHEET, ".c", "--x"),
        None,
        "a rule inside an at-rule is not a top-level rule"
    );
    assert_eq!(patch::custom_property(SHEET, "a", "--x"), None, "a comment");

    let same = patch::set_custom_properties(
        SHEET,
        &[rule(
            ":root",
            &[("--x", "1"), ("--y", "calc(2 * var(--x))")],
        )],
    );
    assert_eq!(same.as_deref(), Ok(SHEET));

    let set = patch::set_custom_properties(SHEET, &[rule(":root", &[("--x", "0.85")])]);
    assert_eq!(
        set.as_deref(),
        Ok(SHEET.replace("  --x: 1;", "  --x: 0.85;").as_str())
    );

    let added = patch::set_custom_properties(
        SHEET,
        &[
            rule(":root", &[("--z", "auto")]),
            rule(".a .b", &[("--w", "0")]),
            rule("html:has(.c)", &[("--x", "#fff"), ("--v", "2")]),
        ],
    );
    let expected = SHEET
        .replace(
            "  --y: calc(2 * var(--x));\n}",
            "  --y: calc(2 * var(--x));\n  --z: auto;\n}",
        )
        .replace("  gap: var(--x);\n}", "  gap: var(--x);\n  --w: 0;\n}")
        + "\nhtml:has(.c) {\n  --x: #fff;\n  --v: 2;\n}\n";
    assert_eq!(added.as_deref(), Ok(expected.as_str()));
}

#[test]
fn only_a_custom_property_with_a_plain_value_is_set() {
    for (property, value) in [
        ("gap", "1rem"),
        ("--x", "1; color: red"),
        ("--x", "1 } a {"),
        ("--x", "1\n"),
        ("--x", ""),
        ("--", "1"),
    ] {
        let refused = patch::set_custom_properties(SHEET, &[rule(":root", &[(property, value)])]);
        assert!(refused.is_err(), "{property}: {value:?} gave {refused:?}");
    }
    assert!(patch::set_custom_properties(SHEET, &[rule("a { b", &[("--x", "1")])]).is_err());
}

#[test]
fn a_declaration_is_read_as_the_style_sheet_writes_it() {
    let sheet = ":root {\n  --x: 1 /* was 2 */;\n  --u: url(a;b:c);\n  --last: 3\n}\n";
    for (property, value) in [("--x", "1"), ("--u", "url(a;b:c)"), ("--last", "3")] {
        assert_eq!(
            patch::custom_property(sheet, ":root", property).as_deref(),
            Some(value),
            "{property}"
        );
    }
    let set = patch::set_custom_properties(sheet, &[rule(":root", &[("--x", "0.85")])]);
    assert_eq!(
        set.as_deref(),
        Ok(sheet.replace("--x: 1 /*", "--x: 0.85 /*").as_str()),
        "the comment behind the value stays"
    );
    let added = patch::set_custom_properties(sheet, &[rule(":root", &[("--new", "4")])]);
    assert_eq!(
        added.as_deref(),
        Ok(sheet
            .replace("  --last: 3\n", "  --last: 3;\n  --new: 4;\n")
            .as_str()),
        "the last declaration gets the semicolon it lacks"
    );
}

#[test]
fn a_property_the_sheet_declares_elsewhere_for_the_selector_is_refused() {
    let cases = [
        (
            "only inside an at-rule",
            "@media (max-width: 9px) {\n  :root {\n    --x: 2;\n  }\n}\n",
            "inside another rule",
        ),
        (
            "in an at-rule inside the rule",
            ":root {\n  --x: 1;\n  @media (max-width: 9px) {\n    --x: 2;\n  }\n}\n",
            "2 times",
        ),
        (
            "twice in one rule",
            ":root {\n  --x: 1;\n  --x: 2;\n}\n",
            "2 times",
        ),
        (
            "a string that does not end",
            ":root {\n  --x: \"a;\n}\n",
            "string",
        ),
    ];
    for (name, sheet, why) in cases {
        let refused = patch::set_custom_properties(sheet, &[rule(":root", &[("--x", "0.85")])]);
        assert!(
            refused.as_ref().is_err_and(|error| error.contains(why)),
            "{name}: {refused:?}"
        );
    }
    let other = "@media (max-width: 9px) {\n  :root {\n    --x: 2;\n  }\n}\n";
    let added = patch::set_custom_properties(other, &[rule(":root", &[("--y", "1")])]);
    assert_eq!(
        added.as_deref(),
        Ok(format!("{other}\n:root {{\n  --y: 1;\n}}\n").as_str()),
        "a property the sheet declares nowhere for the selector is added"
    );
}

// Terms and ranking, on hand-built renders.

fn clean() -> Render {
    Render {
        primary_in_view: true,
        h_scroll: false,
        contrast_failures: 0,
        unnamed: 0,
        content_missing: 0,
        words: 40,
        primary_px: 400,
        small_targets: 1,
        journeys: [Some(Route {
            actions: 1,
            regions: 2,
        }); 3],
    }
}

fn record_of(variants: Vec<(String, Vec<Render>)>) -> Recorded {
    Recorded {
        adapter: "ux-browser 0.0.0".to_owned(),
        browser: "Chrome/0".to_owned(),
        product_digest: "sha256:00".to_owned(),
        css_digest: "sha256:11".to_owned(),
        variants,
    }
}

#[test]
fn a_term_is_the_worst_value_of_the_six_environments() {
    let mut renders = vec![clean(); 6];
    assert_eq!(
        report::terms(&renders),
        Terms {
            hard: 0,
            words: 40,
            primary_px: 400,
            regions: 6,
            small_targets: 1,
            actions: 3,
        }
    );
    assert_eq!(report::terms(&renders).values(), [0, 40, 400, 6, 1, 3]);
    renders[2].words = 55;
    renders[1].primary_px = 512;
    renders[3].journeys[1] = Some(Route {
        actions: 2,
        regions: 1,
    });
    renders[5].journeys[2] = Some(Route {
        actions: 1,
        regions: 4,
    });
    assert_eq!(
        report::terms(&renders).values(),
        [0, 55, 512, 8, 1, 4],
        "regions and actions are summed over the journeys, then the worst environment counts"
    );
}

#[test]
fn tap_targets_count_at_the_narrow_viewport_only() {
    assert_eq!(record::TAP_WIDTH, 390);
    let narrow: Vec<usize> = (0..6)
        .filter(|&at| record::ENVIRONMENTS[at].width == record::TAP_WIDTH)
        .collect();
    assert_eq!(narrow, [4, 5]);
    let mut renders = vec![clean(); 6];
    renders[0].small_targets = 9;
    renders[3].small_targets = 7;
    assert_eq!(report::terms(&renders).small_targets, 1);
    renders[5].small_targets = 4;
    assert_eq!(report::terms(&renders).small_targets, 4);
}

#[test]
fn hard_counts_each_rule_an_environment_breaks() {
    let mut renders = vec![clean(); 6];
    renders[0].primary_in_view = false;
    renders[0].h_scroll = true;
    renders[1].contrast_failures = 12;
    renders[2].unnamed = 3;
    renders[3].content_missing = 2;
    renders[4].journeys[0] = None;
    renders[4].journeys[2] = None;
    let terms = report::terms(&renders);
    assert_eq!(
        terms.hard, 6,
        "one per rule and environment, however often the rule breaks there"
    );
    assert_eq!(
        (terms.regions, terms.actions),
        (6, 3),
        "a journey without a route adds nothing to the sums; hard carries it"
    );
}

#[test]
fn the_frontier_is_the_non_dominated_set_of_the_variants_that_break_no_hard_rule() {
    let mut fast = vec![clean(); 6];
    for render in &mut fast {
        render.primary_px = 300;
        render.words = 50;
    }
    let mut broken = vec![clean(); 6];
    for render in &mut broken {
        render.primary_px = 100;
        render.words = 10;
    }
    broken[4].h_scroll = true;
    let mut dominated = vec![clean(); 6];
    dominated[0].small_targets = 5;
    dominated[5].small_targets = 2;
    let ranked = report::rank(&record_of(vec![
        ("dominated".to_owned(), dominated),
        ("broken".to_owned(), broken),
        ("tie-b".to_owned(), vec![clean(); 6]),
        ("fast".to_owned(), fast),
        ("tie-a".to_owned(), vec![clean(); 6]),
    ]));
    let ids = |order: &[usize]| -> Vec<&str> {
        order
            .iter()
            .map(|&index| ranked.scored[index].id.as_str())
            .collect()
    };
    assert_eq!(
        ids(&ranking_of(&ranked)),
        ["tie-a", "tie-b", "dominated", "fast", "broken"],
        "lexicographic over the terms, hard first, ties by id"
    );
    assert_eq!(
        ids(&ranked.frontier),
        ["tie-a", "tie-b", "fast"],
        "a trade-off and a tie stand on it; a broken rule keeps the best terms off it"
    );
}

fn ranking_of(ranked: &report::Ranked) -> Vec<usize> {
    ranked.ranking.clone()
}

#[test]
fn a_record_of_other_variants_is_refused() {
    let few = record_of(vec![(shipped_id(), vec![clean(); 6])]);
    assert!(
        report::report(&few).is_err(),
        "one variant is not the space"
    );
    let mut short: Vec<(String, Vec<Render>)> = space::variants()
        .into_iter()
        .map(|variant| (variant.id, vec![clean(); 6]))
        .collect();
    assert!(report::report(&record_of(short.clone())).is_ok());
    short[13].1.pop();
    let refused = report::report(&record_of(short)).expect_err("five renders");
    assert!(
        refused.contains("docs-system.left.1+1.features-first.d85.h100"),
        "{refused}"
    );
}

#[test]
fn a_measurements_file_reads_back_what_was_written() {
    let mut renders = vec![clean(); 6];
    renders[1].primary_in_view = false;
    renders[2].h_scroll = true;
    renders[3].journeys[1] = None;
    renders[5] = Render {
        primary_in_view: true,
        h_scroll: false,
        contrast_failures: 1,
        unnamed: 2,
        content_missing: 3,
        words: 4,
        primary_px: 5,
        small_targets: 6,
        journeys: [
            Some(Route {
                actions: 7,
                regions: 8,
            }),
            Some(Route {
                actions: 9,
                regions: 10,
            }),
            Some(Route {
                actions: 11,
                regions: 12,
            }),
        ],
    };
    let written = record_of(vec![
        ("a".to_owned(), renders),
        ("b".to_owned(), vec![clean(); 6]),
    ]);
    let text = written.text();
    assert_eq!(Recorded::parse(&text).as_ref(), Ok(&written), "{text}");
    assert_eq!(
        text.lines().count(),
        10 + 2 * 8 + 2,
        "ten lines of header, a line per render between two per variant, two to close: {text}"
    );
    assert!(text.contains("[1,0,1,2,3,4,5,6,7,8,9,10,11,12]"), "{text}");
    assert!(
        text.contains("[1,0,0,0,0,40,400,1,1,2,null,null,1,2]"),
        "{text}"
    );
    assert!(Recorded::parse(&text.replace(record::FORMAT, "ux-search-landing/0")).is_err());
    assert!(Recorded::parse(&text.replace(",12]", ",12,13]")).is_err());
}

// The recorded campaign.

#[test]
fn the_recorded_campaign_is_every_variant_in_every_environment_on_the_kept_sources() {
    let recorded = recorded();
    let ids: Vec<&str> = recorded
        .variants
        .iter()
        .map(|(id, _)| id.as_str())
        .collect();
    let space: Vec<String> = space::variants()
        .into_iter()
        .map(|variant| variant.id)
        .collect();
    assert_eq!(ids, space);
    assert!(
        recorded
            .variants
            .iter()
            .all(|(_, renders)| renders.len() == record::ENVIRONMENTS.len())
    );
    let digest = |name: &str| ux_search::landing::record::digest(read(&fixture(name)).as_bytes());
    assert_eq!(recorded.product_digest, digest("product.json"));
    assert_eq!(recorded.css_digest, digest("landing.css"));
    assert!(
        crate::adapter_version(&recorded.adapter).is_some(),
        "the adapter line `{}` is not `ux-browser <version>`",
        recorded.adapter
    );
    assert!(recorded.browser.contains("Chrome/"), "{}", recorded.browser);
    assert_eq!(
        recorded.text(),
        read(&fixture("measurements.json")),
        "the file is not what the record writes"
    );
}

#[test]
fn the_report_over_the_recorded_campaign_is_the_committed_one() {
    let recorded = recorded();
    let first = report::report(&recorded).unwrap_or_else(|error| panic!("{error}"));
    let second = report::report(&self::recorded()).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(first, second, "a second run prints other bytes");
    let path = fixture("report.txt");
    assert_eq!(
        first,
        read(&path),
        "{} is not the report; write it from the repository root with:\n{GENERATE}",
        path.display()
    );
}

#[test]
fn the_report_prints_the_frontier_first_then_every_variant() {
    let recorded = recorded();
    let ranked = report::rank(&recorded);
    let text = report::report(&recorded).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(ranked.ranking.len(), 144);
    assert!(!ranked.frontier.is_empty());
    for &index in &ranked.frontier {
        let terms = &ranked.scored[index].terms;
        assert_eq!(
            terms[0], 0,
            "{} breaks a hard rule",
            ranked.scored[index].id
        );
        assert!(
            !ranked
                .scored
                .iter()
                .any(|other| other.terms[0] == 0
                    && ux_search::rank::dominates(&other.terms, terms)),
            "{} is dominated",
            ranked.scored[index].id
        );
    }
    let frontier = text
        .find(&format!("\nfrontier {}\n", ranked.frontier.len()))
        .expect("the frontier's heading");
    let ranking = text.find("\nranking 144\n").expect("the ranking's heading");
    assert!(frontier < ranking);
    let shipped = text
        .lines()
        .find(|line| line.starts_with("shipped "))
        .expect("the line of the page as shipped");
    assert!(
        shipped.starts_with(&format!("shipped {}: rank ", shipped_id())),
        "{shipped}"
    );
    let between = &text[frontier..ranking];
    assert_eq!(
        between.matches("\n     product.json ").count(),
        ranked.frontier.len(),
        "each frontier variant prints its operations on product.json"
    );
    assert!(
        between
            .matches("\n     landing.css :root { --uxui-density: ")
            .count()
            == ranked.frontier.len(),
        "each frontier variant prints its custom properties"
    );
    assert_eq!(text[ranking..].lines().count(), 2 + 144 - 1 + 1);
}

// The command.

fn ux_search(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ux-search"))
        .args(args)
        .output()
        .expect("ux-search runs")
}

#[test]
fn landing_report_prints_the_report_of_a_measurements_file() {
    let measurements = fixture("measurements.json");
    let output = ux_search(&[
        "landing",
        "report",
        "--measurements",
        measurements.to_str().expect("a UTF-8 path"),
    ]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        read(&fixture("report.txt"))
    );
}

#[test]
fn landing_refuses_a_file_it_cannot_read_and_a_wrong_invocation() {
    let missing = fixture("no-such.json");
    let output = ux_search(&[
        "landing",
        "report",
        "--measurements",
        missing.to_str().expect("a UTF-8 path"),
    ]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty());
    for args in [
        &["landing"][..],
        &["landing", "report"],
        &["landing", "climb"],
        &["landing", "run", "--build", "website/build"],
        &["landing", "run", "--jobs", "0"],
    ] {
        let output = ux_search(args);
        assert_eq!(output.status.code(), Some(2), "{args:?}: {output:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
    }
}

#[test]
fn landing_run_without_a_built_site_starts_no_browser() {
    let out = crate::state_dir("landing-run-no-build");
    let output = ux_search(&[
        "landing",
        "run",
        "--build",
        fixture("no-build").to_str().expect("a UTF-8 path"),
        "--product",
        fixture("product.json").to_str().expect("a UTF-8 path"),
        "--css",
        fixture("landing.css").to_str().expect("a UTF-8 path"),
        "--out",
        out.to_str().expect("a UTF-8 path"),
    ]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let said = String::from_utf8_lossy(&output.stderr);
    assert!(said.contains("no built site"), "{said}");
    assert!(!out.join("measurements.json").exists());
}
