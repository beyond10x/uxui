//! Adversary cases for `story:studio-climb`: layout shapes valid in YAML that the column reader
//! does not read as the renderer and the checker do.

use std::fs;
use std::path::{Path, PathBuf};

use ux_search::climb::{self, Options};
use ux_search::drive::Target;
use ux_search::variant::Columns;

/// A block sequence written at the indent of its key (YAML's compact form, which the page's
/// `sections:` reader accepts) is a layout with columns, not a page without them.
#[test]
fn columns_written_at_the_indent_of_their_key_are_found() {
    let text = "pages:\n  live:\n    layout:\n      columns:\n      - {name: side, sections: [a]}\n      - {name: main, sections: [b, c]}\n    sections:\n      - {name: a}\n      - {name: b}\n      - {name: c}\n";
    let found = Columns::find(text, "live").unwrap_or_else(|error| panic!("{error}"));
    let names: Option<Vec<String>> = found.map(|layout| {
        layout
            .columns
            .into_iter()
            .map(|column| column.name)
            .collect()
    });
    assert_eq!(names, Some(vec!["side".to_owned(), "main".to_owned()]));
}

/// A page whose layout is a scalar is a stack; a comment after it is not a flow mapping.
#[test]
fn a_stack_layout_with_a_comment_naming_columns_is_not_refused() {
    let text = "pages:\n  live:\n    layout: stack # no columns on this page\n    sections:\n      - {name: a}\n";
    assert_eq!(Columns::find(text, "live"), Ok(None));
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap_or_else(|error| panic!("{}: {error}", to.display()));
    for entry in fs::read_dir(from).unwrap_or_else(|error| panic!("{}: {error}", from.display())) {
        let entry = entry.expect("a directory entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a file type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target)
                .unwrap_or_else(|error| panic!("{}: {error}", target.display()));
        }
    }
}

/// The studio with its columns in the compact form climbs inside each column: `summary` stays
/// in `side`, first in every variant. Two variants (cap 2).
#[test]
fn the_studio_with_compact_columns_keeps_summary_in_its_column() {
    let ui = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ess/ui");
    let text = fs::read_to_string(ui.join("studio.ui.yaml")).expect("the studio document");
    let compact = text
        .replace(
            "      columns:\n        - {name: side, sections: [summary], width: narrow}\n        - {name: main,",
            "      columns:\n      - {name: side, sections: [summary], width: narrow}\n      - {name: main,",
        );
    assert_ne!(
        compact, text,
        "the studio's columns are no longer written as expected"
    );
    let dir: PathBuf = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ux-search")
        .join("adversary-compact");
    let _ = fs::remove_dir_all(&dir);
    copy_dir(&ui.join("fixtures"), &dir.join("fixtures"));
    let document = dir.join("studio.ui.yaml");
    fs::write(&document, compact).expect("the compact copy");
    let options = Options {
        target: Target {
            page: "campaigns.climb".to_owned(),
            params: vec![("campaign_id".to_owned(), "cp-01".to_owned())],
        },
        sizes: vec![(120, 40), (48, 40)],
        state_dir: dir.join("state"),
        cap: 2,
    };
    let climb = climb::climb(&document, &options).unwrap_or_else(|error| panic!("{error}"));
    for variant in &climb.variants {
        assert_eq!(
            variant.order.first().map(String::as_str),
            Some("summary"),
            "{} moves `summary` out of column `side`:\n{}",
            variant.id,
            climb.render()
        );
    }
}
