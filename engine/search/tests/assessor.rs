//! The assessor declaration `ratchet-assessor/1` of ux-search@1
//! (`engine/search/ux-search.assessor.yaml`) against the command line of `ux-search landing
//! promote` and the evidence documents a promotion writes, as `engine/browser/tests/assessor.rs`
//! and `engine/static/tests/main.rs` hold theirs.
//!
//! Two things the declaration cannot say in the form ratchet's `docs/evidence-format.md` § 10
//! gives it, held here as they are:
//!
//! - It names no `target_input`. The target is the two sources after the patch, identified by
//!   one digest over both (`digest_form: sha256sum_lines`); `product` and `css` are inputs, read
//!   before the patch, and neither is the target alone.
//! - The command takes `--evidence` where ratchet passes `--out`, and none of ratchet's other
//!   reserved arguments: it is run by hand, once per promotion.

use std::path::Path;
use std::process::Command;

use ux_search::landing::patch::{self, Value};
use ux_search::landing::promote;

use crate::landing_promote::{CHOSEN, copies};

/// The declaration, beside the crate's `Cargo.toml`.
const DECLARATION: &str = "ux-search.assessor.yaml";
/// The arguments ratchet passes to every assessor; no input takes one of these names.
const RESERVED: [&str; 6] = ["project", "assessment", "root", "class", "out", "revision"];

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn declaration() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(DECLARATION);
    uxui_doc::yaml::parse(&read(&path))
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn text(items: &[&str]) -> Value {
    Value::List(
        items
            .iter()
            .map(|item| Value::Str((*item).to_owned()))
            .collect(),
    )
}

fn keys(value: &Value) -> Vec<&str> {
    value
        .entries()
        .iter()
        .map(|(key, _)| key.as_str())
        .collect()
}

/// The arguments of `landing promote` in the usage the binary prints: `(name, required)` per
/// `--name`; an argument in `[ ]` is not required.
fn promote_arguments() -> Vec<(String, bool)> {
    let help = Command::new(env!("CARGO_BIN_EXE_ux-search"))
        .arg("--help")
        .output()
        .expect("ux-search runs");
    assert!(help.status.success());
    let usage = String::from_utf8(help.stdout).expect("the usage is UTF-8");
    let line = usage
        .lines()
        .find_map(|line| line.trim_start().strip_prefix("ux-search landing promote "))
        .expect("the usage has a `landing promote` line");
    line.split_whitespace()
        .filter_map(|token| {
            let name = token.trim_start_matches('[').strip_prefix("--")?;
            Some((name.to_owned(), !token.starts_with('[')))
        })
        .collect()
}

#[test]
fn the_declaration_names_the_assessor_its_target_kind_and_its_counts() {
    let declared = declaration();
    assert_eq!(declared.get_str("format"), Some("ratchet-assessor/1"));
    assert_eq!(
        format!(
            "{}@{}",
            declared.get_str("name").expect("a name"),
            declared.get_str("version").expect("a version")
        ),
        promote::ASSESSOR
    );
    assert_eq!(
        declared.get_str("command"),
        Some("ux-search landing promote")
    );
    assert_eq!(declared.get("targets"), Some(&text(&["document"])));
    assert_eq!(
        declared.get("target_input"),
        None,
        "the target is the two sources after the patch, not one input"
    );
    assert_eq!(declared.get_str("coverage_unit"), Some("variant"));
    assert_eq!(
        declared.get("coverage"),
        Some(&text(&[
            "executed",
            "skipped",
            "not_run",
            "unsupported",
            "passed",
            "failed"
        ]))
    );
}

#[test]
fn the_declared_inputs_are_the_arguments_of_landing_promote() {
    let declared = declaration();
    let inputs = declared.get("inputs").expect("inputs");
    let mut named: Vec<(String, bool)> = inputs
        .entries()
        .iter()
        .map(|(name, input)| {
            assert!(!RESERVED.contains(&name.as_str()), "{name} is reserved");
            (
                name.clone(),
                input.get("required") == Some(&Value::Bool(true)),
            )
        })
        .collect();
    let mut arguments = promote_arguments();
    assert_eq!(arguments.len(), 7, "{arguments:?}");
    for (name, _) in &arguments {
        assert!(
            !RESERVED.contains(&name.as_str()),
            "--{name} is an argument ratchet passes"
        );
    }
    named.sort();
    arguments.sort();
    assert_eq!(named, arguments, "the declared inputs are the command's");

    let kind = |name: &str| {
        let input = inputs
            .get(name)
            .unwrap_or_else(|| panic!("no input {name}"));
        (
            input.get_str("type").expect("a type"),
            input.get("local") == Some(&Value::Bool(true)),
        )
    };
    assert_eq!(kind("measurements"), ("path", false));
    assert_eq!(kind("variant"), ("string", false));
    assert_eq!(kind("product"), ("path", false));
    assert_eq!(kind("css"), ("path", false));
    assert_eq!(kind("chosen-by"), ("string", false));
    assert_eq!(
        kind("evidence"),
        ("path", true),
        "the file the run writes its document to"
    );
    assert_eq!(
        kind("shipped-file"),
        ("path", true),
        "a file the run writes; it changes no value of the document"
    );
}

/// Every member of `doc` the declaration speaks of is one it allows; returns the outputs the
/// document wrote.
fn agrees(declared: &Value, doc: &Value) -> Vec<String> {
    let evidence = doc.get("evidence").expect("an `evidence` member");
    assert_eq!(evidence.get_str("assessor"), Some(promote::ASSESSOR));
    assert_eq!(
        evidence
            .get("coverage")
            .and_then(|coverage| coverage.get_str("unit")),
        declared.get_str("coverage_unit")
    );
    let counts = keys(evidence.get("coverage").expect("coverage"));
    assert_eq!(
        Some(&text(&counts[1..])),
        declared.get("coverage"),
        "the counts the declaration names, after `unit`"
    );
    let kind = Value::Str(
        evidence
            .get("target")
            .and_then(|target| target.get_str("kind"))
            .expect("a target kind")
            .to_owned(),
    );
    assert!(
        declared
            .get("targets")
            .expect("targets")
            .list()
            .contains(&kind),
        "{kind:?}"
    );
    let other = keys(
        evidence
            .get("conditions")
            .and_then(|conditions| conditions.get("other"))
            .expect("conditions.other"),
    );
    assert_eq!(
        declared.get("conditions"),
        Some(&text(&other)),
        "the declared conditions are the ones a document holds"
    );

    // The inputs of the document are declared ones that are not local, under the argument's
    // name with `_` for `-`; a required one is there.
    let inputs = declared.get("inputs").expect("inputs");
    let written: Vec<&str> = keys(evidence.get("inputs").expect("inputs"));
    for (name, input) in inputs.entries() {
        let member = name.replace('-', "_");
        let local = input.get("local") == Some(&Value::Bool(true));
        let required = input.get("required") == Some(&Value::Bool(true));
        let found = evidence
            .get("inputs")
            .and_then(|inputs| inputs.get(&member));
        if local {
            assert!(found.is_none(), "{name} is local and no evidence");
            continue;
        }
        assert!(found.is_some() || !required, "no input {member}");
        if let Some(found) = found {
            let is_path = input.get_str("type") == Some("path");
            assert_eq!(
                found.get("digest").is_some(),
                is_path,
                "{member}: a digest exactly for a path that was read"
            );
        }
    }
    for member in written {
        assert!(
            inputs
                .entries()
                .iter()
                .any(|(name, _)| name.replace('-', "_") == member),
            "the input {member} is not declared"
        );
    }

    let outputs = declared.get("outputs").expect("outputs");
    let mut wrote = Vec::new();
    for result in evidence.get("results").expect("results").list() {
        let name = result.get_str("output").expect("an output");
        let output = outputs
            .get(name)
            .unwrap_or_else(|| panic!("output {name} is not declared"));
        for key in ["value_type", "kind", "unit"] {
            assert_eq!(result.get(key), output.get(key), "{name}: {key}");
        }
        let scope = result.get_str("scope").expect("a scope");
        let scopes = output.get("scopes").expect("scopes").list();
        assert!(
            scope == "run" || scopes.contains(&Value::Str(scope.to_owned())),
            "{name}: scope {scope} is neither `run`, which every assessor has, nor declared"
        );
        assert_eq!(result.get("key"), None, "{name}: no identity set");
        assert_eq!(output.get("identity_key"), None, "{name}");
        wrote.push(format!("{name} {scope}"));
    }
    wrote
}

#[test]
fn the_documents_write_every_declared_output_and_nothing_else() {
    let declared = declaration();
    // The committed document, and one a promotion writes here: with `--chosen-by`, and without.
    let golden = patch::parse(
        &read(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/docs-landing/promote.evidence.json"),
        )
        .replace("{version}", env!("CARGO_PKG_VERSION")),
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let mut wrote = agrees(&declared, &golden);
    for (test, chosen_by) in [("assessor-named", true), ("assessor-bare", false)] {
        let copies = copies(test);
        let mut options = copies.options(CHOSEN.as_str());
        if !chosen_by {
            options.chosen_by = None;
            options.shipped_file = None;
        }
        copies
            .promote(&options)
            .unwrap_or_else(|error| panic!("{error}"));
        let document =
            patch::parse(&read(&copies.evidence)).unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(
            document
                .get("evidence")
                .and_then(|evidence| evidence.get("inputs"))
                .and_then(|inputs| inputs.get("chosen_by"))
                .is_some(),
            chosen_by
        );
        wrote.extend(agrees(&declared, &document));
    }
    // Every declared output is written, in each of its declared scopes, or in `run` when it
    // declares none.
    for (name, output) in declared.get("outputs").expect("outputs").entries() {
        let scopes = output.get("scopes").expect("scopes").list();
        if scopes.is_empty() {
            assert!(
                wrote.contains(&format!("{name} run")),
                "{name} is declared and never written"
            );
        }
        for scope in scopes {
            let scope = scope.str().expect("a scope is text");
            assert!(
                wrote.contains(&format!("{name} {scope}")),
                "{name} is declared for {scope} and never written there"
            );
        }
    }
    // The scopes and units the declaration is to name, read off the declaration itself.
    let outputs = declared.get("outputs").expect("outputs");
    let scopes: std::collections::BTreeSet<&str> = outputs
        .entries()
        .iter()
        .flat_map(|(_, output)| output.get("scopes").expect("scopes").list())
        .filter_map(Value::str)
        .collect();
    assert_eq!(
        scopes.into_iter().collect::<Vec<_>>(),
        ["source", "variant"]
    );
    assert_eq!(
        outputs
            .get("landing.source@1")
            .and_then(|output| output.get("unit")),
        None,
        "a digest has no unit"
    );
}
