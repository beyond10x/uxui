//! The assessor declaration `ratchet-assessor/1` of ux-browser@1 against the command line and the
//! documents the assessor writes, and the invocations the command refuses before it starts a
//! browser.

use std::path::Path;
use std::process::Command;
use ux_browser::evidence;
use uxui_doc::yaml::{self, Value};

/// The declaration, beside the crate's `Cargo.toml`.
const DECLARATION: &str = "ux-browser.assessor.yaml";
/// The arguments ratchet passes to every assessor; no input takes one of these names.
const RESERVED: [&str; 6] = ["project", "assessment", "root", "class", "out", "revision"];

fn declaration() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(DECLARATION);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    yaml::parse(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn text(items: &[&str]) -> Value {
    Value::List(items.iter().map(|s| Value::Str(s.to_string())).collect())
}

/// The `checks` part of the usage the binary prints: `(name, required)` per `--name`.
fn checks_arguments() -> Vec<(String, bool)> {
    let help = Command::new(env!("CARGO_BIN_EXE_ux-browser"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(help.status.success());
    let usage = String::from_utf8(help.stdout).unwrap();
    let checks = usage
        .split_once("ux-browser checks")
        .expect("the usage has a checks command")
        .1;
    let mut out = Vec::new();
    for token in checks.split_whitespace() {
        if token == "ux-browser" {
            break;
        }
        let required = !token.starts_with('[');
        if let Some(name) = token.trim_start_matches('[').strip_prefix("--") {
            out.push((name.to_string(), required));
        }
    }
    out
}

#[test]
fn the_declaration_names_the_assessor_its_target_kind_and_its_counts() {
    let decl = declaration();
    assert_eq!(decl.get_str("format"), Some("ratchet-assessor/1"));
    assert_eq!(
        format!(
            "{}@{}",
            decl.get_str("name").unwrap(),
            decl.get_str("version").unwrap()
        ),
        evidence::ASSESSOR
    );
    assert_eq!(decl.get_str("command"), Some("ux-browser checks"));
    assert_eq!(decl.get("targets"), Some(&text(&["build"])));
    assert_eq!(
        decl.get("target_input"),
        None,
        "only a document target is an input"
    );
    assert_eq!(decl.get_str("coverage_unit"), Some("check"));
    assert_eq!(
        decl.get("coverage"),
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
fn the_declared_inputs_are_the_arguments_of_the_checks_command() {
    let decl = declaration();
    let mut cli = checks_arguments();
    for (name, required) in cli.iter().filter(|(n, _)| RESERVED.contains(&n.as_str())) {
        let optional_by_default = ["root", "out", "class"].contains(&name.as_str());
        assert_eq!(*required, !optional_by_default, "--{name}");
    }
    cli.retain(|(n, _)| !RESERVED.contains(&n.as_str()));
    let inputs = decl.get("inputs").expect("inputs");
    let mut declared: Vec<(String, bool)> = inputs
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
    cli.sort();
    declared.sort();
    assert_eq!(declared, cli, "the declared inputs are the CLI's");
    let kind = |name: &str| {
        let input = inputs.get(name).unwrap();
        (
            input.get_str("type").unwrap().to_string(),
            input.get("local") == Some(&Value::Bool(true)),
        )
    };
    assert_eq!(kind("config"), ("path".to_string(), false));
    assert_eq!(kind("build"), ("path".to_string(), false));
    assert_eq!(kind("pages"), ("string".to_string(), false));
    assert_eq!(kind("capture"), ("path".to_string(), true));
    assert_eq!(kind("profile"), ("path".to_string(), true));
    assert_eq!(kind("jobs"), ("integer".to_string(), true));
    assert_eq!(
        kind("values"),
        ("path".to_string(), true),
        "a file the run writes, as capture"
    );
}

/// Every result of `doc` is one the declaration allows; returns the outputs it wrote.
fn agrees(decl: &Value, doc: &Value) -> Vec<String> {
    let e = doc.get("evidence").unwrap();
    assert_eq!(e.get_str("assessor"), Some(evidence::ASSESSOR));
    assert_eq!(
        e.get("coverage").unwrap().get_str("unit"),
        decl.get_str("coverage_unit")
    );
    let kind = Value::Str(
        e.get("target")
            .unwrap()
            .get_str("kind")
            .unwrap()
            .to_string(),
    );
    assert!(decl.get("targets").unwrap().list().contains(&kind));
    let other: Vec<&str> = e
        .get("conditions")
        .unwrap()
        .get("other")
        .unwrap()
        .entries()
        .iter()
        .map(|(k, _)| k.as_str())
        .collect();
    assert_eq!(
        decl.get("conditions"),
        Some(&text(&other)),
        "the declared conditions are the ones a document holds"
    );
    let outputs = decl.get("outputs").expect("outputs");
    let mut written = Vec::new();
    for result in e.get("results").unwrap().list() {
        let name = result.get_str("output").unwrap();
        let output = outputs
            .get(name)
            .unwrap_or_else(|| panic!("output {name} is not declared"));
        for key in ["value_type", "kind", "unit"] {
            assert_eq!(result.get(key), output.get(key), "{name}: {key}");
        }
        let scope = Value::Str(result.get_str("scope").unwrap().to_string());
        assert!(
            output.get("scopes").unwrap().list().contains(&scope),
            "{name}: {scope:?}"
        );
        assert_eq!(result.get("key"), output.get("identity_key"), "{name}");
        for item in result.get("items").map(Value::list).unwrap_or(&[]) {
            for (field, _) in item.get("fields").map(Value::entries).unwrap_or(&[]) {
                let allowed = output.get("item_fields").map(Value::list).unwrap_or(&[]);
                assert!(
                    allowed.contains(&Value::Str(field.clone())),
                    "{name}: {field}"
                );
            }
        }
        written.push(name.to_string());
    }
    written
}

#[test]
fn the_documents_write_every_declared_output_and_nothing_else() {
    let decl = declaration();
    let example = crate::fixture_json("../../examples/llm-docs.evidence.json");
    let mut written = agrees(&decl, &example);
    written.extend(agrees(&decl, &crate::evidence::document()));
    written.extend(agrees(&decl, &crate::values::round_document()));
    for (name, _) in decl.get("outputs").unwrap().entries() {
        assert!(
            written.contains(name),
            "{name} is declared and never written"
        );
    }
}

fn checks_call(extra: &[&str]) -> Option<i32> {
    let missing = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/no-such-config.yaml");
    Command::new(env!("CARGO_BIN_EXE_ux-browser"))
        .args(["checks", "--config"])
        .arg(&missing)
        .args([
            "--build",
            "no-such-build",
            "--project",
            "p",
            "--assessment",
            "a",
        ])
        .args(extra)
        .output()
        .unwrap()
        .status
        .code()
}

#[test]
fn jobs_is_a_whole_number_of_at_least_one() {
    assert_eq!(
        checks_call(&["--jobs", "2"]),
        Some(1),
        "a good invocation reaches the missing configuration"
    );
    assert_eq!(checks_call(&["--jobs", "0"]), Some(2));
    assert_eq!(checks_call(&["--jobs", "two"]), Some(2));
    assert_eq!(checks_call(&[]), Some(1));
}
