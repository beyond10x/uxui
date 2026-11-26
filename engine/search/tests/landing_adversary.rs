//! Adversary cases for `story:docs-site-first-climb`, phase A: what the patch of a variant does
//! on sources shaped otherwise than the kept ones, and the lines of the ranking and the record
//! reader that the module `landing` leaves untested. No test starts a browser.
//!
//! Two kinds of cases. The first (RFC 6902, the style sheet) fail on the code as written. The
//! second (terms, record, space) hold on the code as written and fail on one changed line of
//! `report.rs`, `record.rs` or `space.rs`, named in each case's comment.

use std::path::{Path, PathBuf};
use std::process::Command;

use ux_search::landing::patch::{self, CssRule, Op, Value};
use ux_search::landing::record::{Recorded, Render, Route};
use ux_search::landing::report;
use ux_search::landing::space;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/docs-landing")
        .join(name)
}

fn read(name: &str) -> String {
    let path = fixture(name);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn json(text: &str) -> Value {
    patch::parse(text).unwrap_or_else(|error| panic!("{text}: {error}"))
}

fn variant(id: &str) -> space::Variant {
    space::variants()
        .into_iter()
        .find(|variant| variant.id == id)
        .unwrap_or_else(|| panic!("no variant {id}"))
}

// RFC 6902.

fn test(path: &str, value: &str) -> Op {
    Op::Test {
        path: path.to_owned(),
        value: json(value),
    }
}

/// RFC 6902, 4.6: two objects are equal when they hold the same members, each equal to its
/// counterpart by key; the order of the members is no part of the value.
#[test]
fn a_test_compares_objects_whatever_the_order_of_their_members() {
    let mut document = json(r#"{"a":{"x":1,"y":2}}"#);
    let held = patch::apply_ops(&mut document, &[test("/a", r#"{"y":2,"x":1}"#)]);
    assert_eq!(held, Ok(()));
}

/// RFC 6902, 4.6: two numbers are equal when their values are numerically equal.
#[test]
fn a_test_compares_numbers_by_their_value() {
    let mut document = json(r#"{"n":1}"#);
    let held = patch::apply_ops(&mut document, &[test("/n", "1.0")]);
    assert_eq!(held, Ok(()));
}

/// A patch whose second operation is a `test` that does not hold is refused as a whole: the
/// document is as it was, not with the first operation applied.
#[test]
fn a_refused_patch_leaves_the_document_as_it_was() {
    let before = json(r#"{"foo":["a","b"],"baz":"qux"}"#);
    let mut document = before.clone();
    let refused = patch::apply_ops(
        &mut document,
        &[
            Op::Add {
                path: "/foo/0".to_owned(),
                value: json(r#""z""#),
            },
            test("/baz", r#""other""#),
        ],
    );
    assert!(refused.is_err(), "{refused:?}");
    assert_eq!(
        patch::compact(&document),
        patch::compact(&before),
        "the refused patch left its first operation in the document"
    );
}

// Custom properties. In each case the patch may refuse the sheet; what it may not do is answer
// with a sheet in which the rule's own declaration keeps its old value.

fn set(sheet: &str, property: &str, value: &str) -> Result<String, String> {
    patch::set_custom_properties(
        sheet,
        &[CssRule {
            selector: ":root".to_owned(),
            values: vec![(property.to_owned(), value.to_owned())],
        }],
    )
}

/// Two declarations commented out together: the second starts after a semicolon, inside the
/// comment.
#[test]
fn a_declaration_inside_a_comment_is_not_the_one_set() {
    let sheet = ":root {\n  /* --y: 2;\n  --x: 3; */\n  --x: 1;\n}\n";
    assert_eq!(
        patch::custom_property(sheet, ":root", "--x").as_deref(),
        Some("1"),
        "the value read is the comment's"
    );
    if let Ok(patched) = set(sheet, "--x", "0.85") {
        assert_eq!(patched, sheet.replace("  --x: 1;", "  --x: 0.85;"));
    }
}

/// A comment on the line above a declaration: the property is there, so the shipped value gives
/// the sheet's bytes back and no second declaration is added.
#[test]
fn a_comment_above_a_declaration_does_not_hide_it() {
    let sheet = ":root {\n  /* the density */\n  --x: 1;\n}\n";
    assert_eq!(
        patch::custom_property(sheet, ":root", "--x").as_deref(),
        Some("1")
    );
    assert_eq!(set(sheet, "--x", "1").as_deref(), Ok(sheet));
}

/// A second `:root` rule further down wins the cascade: a sheet in which it still says 2 does
/// not render the value set.
#[test]
fn a_second_root_rule_does_not_keep_the_old_value() {
    let sheet = ":root {\n  --x: 1;\n}\n\n.a {\n  gap: var(--x);\n}\n\n:root {\n  --x: 2;\n}\n";
    if let Ok(patched) = set(sheet, "--x", "0.85") {
        assert!(
            !patched.contains("--x: 2;"),
            "the later rule keeps its value and wins:\n{patched}"
        );
    }
}

/// A `:root` rule inside `@media` sets the property for the narrow viewports, the ones tap
/// targets are counted at: a sheet in which it still says 2 does not render the value set there.
#[test]
fn a_root_rule_inside_a_media_block_does_not_keep_the_old_value() {
    let sheet =
        ":root {\n  --x: 1;\n}\n\n@media (max-width: 720px) {\n  :root {\n    --x: 2;\n  }\n}\n";
    if let Ok(patched) = set(sheet, "--x", "0.85") {
        assert!(
            !patched.contains("--x: 2;"),
            "under 720 px the rule in the media block keeps its value:\n{patched}"
        );
    }
}

/// A rule nested in `:root` (CSS nesting) has declarations of its own: the property of `.a` is
/// not the property of `:root`.
#[test]
fn a_declaration_of_a_nested_rule_is_not_the_one_set() {
    let sheet = ":root {\n  .a {\n    color: red;\n    --x: 2;\n  }\n  --x: 1;\n}\n";
    if let Ok(patched) = set(sheet, "--x", "0.85") {
        assert_eq!(patched, sheet.replace("  --x: 1;\n}", "  --x: 0.85;\n}"));
    }
}

/// A value that is a string with a semicolon in it ends at the declaration's semicolon, not at
/// the string's.
#[test]
fn a_string_value_with_a_semicolon_is_replaced_whole() {
    let sheet = ":root {\n  --sep: \"a;b\";\n  --x: 1;\n}\n";
    assert_eq!(
        patch::custom_property(sheet, ":root", "--sep").as_deref(),
        Some("\"a;b\"")
    );
    if let Ok(patched) = set(sheet, "--sep", "\"c\"") {
        assert_eq!(patched, sheet.replace("\"a;b\"", "\"c\""));
    }
}

// Terms and the report.

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

/// One text under the contrast rule, one element without a name, one card not shown: each
/// breaks its hard rule. Fails on `report.rs` `broken` with `> 0` written `> 1` in any of the
/// three lines; `landing::hard_counts_each_rule_an_environment_breaks` uses 12, 3 and 2.
#[test]
fn one_finding_breaks_its_hard_rule() {
    type Break = fn(&mut Render);
    let cases: [(&str, Break); 3] = [
        ("contrast_failures", |render| render.contrast_failures = 1),
        ("unnamed", |render| render.unnamed = 1),
        ("content_missing", |render| render.content_missing = 1),
    ];
    for (name, break_it) in cases {
        let mut renders = vec![clean(); 6];
        break_it(&mut renders[3]);
        assert_eq!(report::terms(&renders).hard, 1, "{name}");
    }
}

/// The line of the page as shipped when it stands on the frontier: with equal terms everywhere
/// all 144 stand on it, and the page's rank is its id's place among the ids (for
/// `docs-system.left.1+1.features-first.d100.h85` the 50th: 36 ids of `docs-system.centred`,
/// 12 of `docs-system.left.1+0` and `...d100.h100` sort before it). The shipped id is the one
/// `src/landing/shipped.txt` names, so the place is counted here, from the ids. Fails on
/// `report.rs` `report` with `ranked.frontier.contains(&at)` written `false`: a recorded
/// campaign may have the page off the frontier, and its golden report then holds only that word.
#[test]
fn the_page_as_shipped_is_said_to_be_on_the_frontier_when_it_is() {
    let recorded = Recorded {
        adapter: "ux-browser 0.0.0".to_owned(),
        browser: "Chrome/0".to_owned(),
        product_digest: "sha256:00".to_owned(),
        css_digest: "sha256:11".to_owned(),
        variants: space::variants()
            .into_iter()
            .map(|variant| (variant.id, vec![clean(); 6]))
            .collect(),
    };
    let text = report::report(&recorded).unwrap_or_else(|error| panic!("{error}"));
    let shipped = text
        .lines()
        .find(|line| line.starts_with("shipped "))
        .expect("the line of the page as shipped");
    let id = space::shipped().id;
    let place = 1 + space::variants()
        .iter()
        .filter(|variant| variant.id < id)
        .count();
    assert_eq!(
        shipped,
        format!(
            "shipped {id}: rank {place} of 144, 0 variants before it on their terms, 143 equal \
             to it, on the frontier"
        )
    );
    assert!(text.contains("\nfrontier 144\n"), "{text}");
}

// The measurements file.

const FIRST_ROW: &str = "      [1,0,0,0,0,56,448,23,1,1,1,1,2,3],\n";
const FIRST_ID: &str = "\"docs-system.left.1+3.features-first.d85.h85\"";
const SECOND_ID: &str = "\"docs-system.left.1+3.features-first.d85.h100\"";

fn ranked(text: &str) -> Result<String, String> {
    Recorded::parse(text).and_then(|recorded| report::report(&recorded))
}

/// The recorded campaign with one thing changed is refused with a message that names the
/// thing, not ranked. Each case fails on one changed line:
///
/// - "a seventh render": `report.rs` `report`, `renders.len() != ...` written `<`;
/// - "an id outside the space", "two variants exchanged": `report.rs` `report`, the comparison
///   `*id != variant.id` written `false`;
/// - "a seventh environment", "the environments in another order", "the columns in another
///   order": `record.rs` `parse`, `names(&value, key)? != expected` written `false`;
/// - "a flag of 2": `record.rs` `from_row`, `Some(1) => Ok(true)` written `Some(_)`;
/// - "half a route": `record.rs` `from_row`, the last arm of `route` written `Ok(None)`;
/// - "another page": `record.rs` `parse`, `field("page")? != PAGE` written `false`.
#[test]
fn a_measurements_file_that_is_not_the_campaign_is_refused_not_ranked() {
    let recorded = read("measurements.json");
    assert_eq!(ranked(&recorded), Ok(read("report.txt")));
    let exchanged = recorded
        .replace(FIRST_ID, "\"\"")
        .replace(SECOND_ID, FIRST_ID)
        .replace("\"\"", SECOND_ID);
    let reordered = recorded
        .replace("\"1440x900 light\"", "\"\"")
        .replace("\"390x844 light\"", "\"1440x900 light\"")
        .replace("\"\"", "\"390x844 light\"");
    let cases: [(&str, String, &str); 15] = [
        (
            "a missing render",
            recorded.replacen(FIRST_ROW, "", 1),
            "5 renders",
        ),
        (
            "a seventh render",
            recorded.replacen(FIRST_ROW, &FIRST_ROW.repeat(2), 1),
            "7 renders",
        ),
        (
            "an id outside the space",
            recorded.replace(FIRST_ID, "\"docs-system.left.1+2.features-first.d85.h85\""),
            "docs-system.left.1+2.features-first.d85.h85",
        ),
        (
            "two variants exchanged",
            exchanged,
            "docs-system.left.1+3.features-first.d85.h100",
        ),
        (
            "a seventh environment",
            recorded.replace("\"390x844 dark\"]", "\"390x844 dark\", \"320x568 light\"]"),
            "environments",
        ),
        (
            "the environments in another order",
            reordered,
            "environments",
        ),
        (
            "the columns in another order",
            recorded.replace("\"words\", \"primary_px\"", "\"primary_px\", \"words\""),
            "columns",
        ),
        (
            "a number written as text",
            recorded.replacen("[1,0,0,0,0,56,", "[1,0,0,0,0,\"56\",", 1),
            "not a whole number",
        ),
        (
            "a number with a fraction",
            recorded.replacen(",448,", ",448.5,", 1),
            "not a whole number",
        ),
        (
            "a number below 0",
            recorded.replacen(",448,", ",-448,", 1),
            "below 0",
        ),
        (
            "a number left out",
            recorded.replacen("[1,0,0,0,0,56,", "[1,0,0,0,0,null,", 1),
            "words",
        ),
        (
            "a flag of 2",
            recorded.replacen("      [1,0,", "      [2,0,", 1),
            "primary_in_view",
        ),
        (
            "half a route",
            recorded.replacen(",1,1,1,1,2,3]", ",1,null,1,1,2,3]", 1),
            "install_regions",
        ),
        (
            "another page",
            recorded.replace("\"page\": \"/uxui/\"", "\"page\": \"/other/\""),
            "page",
        ),
        (
            "another journey",
            recorded.replace("\"what exists today\"", "\"what existed\""),
            "journeys",
        ),
    ];
    for (name, changed, says) in cases {
        assert_ne!(changed, recorded, "{name}: the case changes nothing");
        match ranked(&changed) {
            Ok(_) => panic!("{name}: ranked"),
            Err(refused) => assert!(refused.contains(says), "{name}: {refused}"),
        }
    }
}

/// The command on a file with a render missing: exit 1, the reason on standard error, no report.
#[test]
fn landing_report_refuses_a_file_with_a_render_missing() {
    let directory = crate::state_dir("landing-adversary");
    std::fs::create_dir_all(&directory).expect("a directory for the changed file");
    let path = directory.join("measurements.json");
    std::fs::write(&path, read("measurements.json").replacen(FIRST_ROW, "", 1))
        .expect("the changed file");
    let output = Command::new(env!("CARGO_BIN_EXE_ux-search"))
        .args(["landing", "report", "--measurements"])
        .arg(&path)
        .output()
        .expect("ux-search runs");
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    let said = String::from_utf8_lossy(&output.stderr);
    assert!(said.contains("5 renders"), "{said}");
}

// The space.

/// The patch of `1+3` is made for the page as shipped, like that of `1+0`: on a page whose link
/// beside the primary action is another one it is refused. Fails on `space.rs` `patch` with the
/// `product.push(link());` of the `"1+3"` arm removed: the patched file is the same with or
/// without the `test`, and no `1+3` variant stands on the recorded frontier.
#[test]
fn the_links_of_1_3_are_refused_on_a_page_with_another_link() {
    let other = read("product.json").replace("\"/docs/status\"", "\"/docs/roadmap\"");
    let refused = variant("docs-system.left.1+3.features-first.d100.h85")
        .patch()
        .apply(&other, &read("landing.css"))
        .expect_err("the link the patch adds before is another one");
    assert!(refused.contains("/product/actions/1/href"), "{refused}");
}
