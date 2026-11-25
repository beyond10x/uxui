//! The design space as text: the section orders, a page's sections reordered, the theme chosen.

use std::collections::BTreeSet;

use ux_search::variant::{Sections, orders, permutations, with_theme};

const DOC: &str = "\
format: ess-ui/1
themes:
  light: {}
  dark: {}
theme: {default: light, chosen_by: shell.theme}
pages:
  first:
    kind: list_page
    sections:
      - {name: filters, remove: true}
  second:
    title: Second
    sections:
      - name: alpha
        component: record
        fields: [a, b]

      # beta reads the list
      - name: beta
        component: collection
        columns:
          - {field: x}
      - {name: gamma, component: form}
    overlays:
      edit: {kind: drawer}
";

#[test]
fn the_permutations_of_three_are_the_six_orders_in_lexicographic_order() {
    assert_eq!(
        permutations(3),
        [
            [0, 1, 2],
            [0, 2, 1],
            [1, 0, 2],
            [1, 2, 0],
            [2, 0, 1],
            [2, 1, 0]
        ]
    );
}

#[test]
fn nothing_and_one_have_one_order() {
    assert_eq!(permutations(0), [Vec::<usize>::new()]);
    assert_eq!(permutations(1), [[0]]);
}

#[test]
fn the_permutations_of_five_are_120_distinct_orders_from_identity_to_reversed() {
    let all = permutations(5);
    assert_eq!(all.len(), 120);
    assert_eq!(all.iter().collect::<BTreeSet<_>>().len(), 120);
    assert_eq!(all[0], [0, 1, 2, 3, 4]);
    assert_eq!(all[119], [4, 3, 2, 1, 0]);
    let mut sorted = all.clone();
    sorted.sort();
    assert_eq!(sorted, all, "the orders come in lexicographic order");
}

#[test]
fn sections_past_the_cap_keep_their_place() {
    let capped = orders(7, 5);
    assert_eq!(capped.len(), 120);
    assert!(capped.iter().all(|order| order[5..] == [5, 6]));
    assert_eq!(orders(3, 5).len(), 6);
    assert_eq!(orders(3, 5)[0], [0, 1, 2]);
}

#[test]
fn a_pages_sections_are_found_by_name_and_written_in_another_order() {
    let sections = Sections::find(DOC, "second").unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(sections.names(), ["alpha", "beta", "gamma"]);
    assert_eq!(sections.with_order(&[0, 1, 2]), DOC);
    let reordered = sections.with_order(&[2, 0, 1]);
    let expected = DOC.replace(
        "      - name: alpha
        component: record
        fields: [a, b]

      # beta reads the list
      - name: beta
        component: collection
        columns:
          - {field: x}
      - {name: gamma, component: form}
",
        "      - {name: gamma, component: form}
      - name: alpha
        component: record
        fields: [a, b]

      # beta reads the list
      - name: beta
        component: collection
        columns:
          - {field: x}
",
    );
    assert_ne!(expected, DOC);
    assert_eq!(reordered, expected);
}

#[test]
fn a_missing_page_and_flow_sections_are_refused() {
    assert!(Sections::find(DOC, "third").is_err());
    let flow = "pages:\n  only:\n    sections: [{name: a}, {name: b}]\n";
    let error = Sections::find(flow, "only").expect_err("flow sections are refused");
    assert!(error.contains("flow"), "{error}");
}

#[test]
fn the_theme_default_is_rewritten_in_flow_and_block_style_and_added_when_absent() {
    let flow = with_theme(DOC, "dark").unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        flow,
        DOC.replace(
            "theme: {default: light, chosen_by: shell.theme}",
            "theme: {default: dark, chosen_by: shell.theme}"
        )
    );
    let block =
        "themes:\n  a: {}\ntheme:\n  default: a # the first\n  chosen_by: shell.theme\npages: {}\n";
    assert_eq!(
        with_theme(block, "b").unwrap_or_else(|error| panic!("{error}")),
        "themes:\n  a: {}\ntheme:\n  default: b\n  chosen_by: shell.theme\npages: {}\n"
    );
    let none = "themes:\n  a: {}\npages: {}\n";
    assert_eq!(
        with_theme(none, "a").unwrap_or_else(|error| panic!("{error}")),
        "themes:\n  a: {}\npages: {}\ntheme: {default: a}\n"
    );
}
