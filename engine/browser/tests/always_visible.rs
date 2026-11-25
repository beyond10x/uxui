//! Always-visible items located by CSS alone or by `aria-current` in a scope: what the
//! configuration accepts and refuses, which recorded candidate is the item, and the values of a page
//! from recorded candidates. An item with a role computes as before.

use crate::checks::LLM_LIKE;
use ux_browser::checks::{self, Candidate, Status};
use ux_browser::json;

fn with_items(items: &str) -> String {
    let start = LLM_LIKE.find("always_visible:\n").unwrap();
    let end = LLM_LIKE.find("keyboard:\n").unwrap();
    format!(
        "{}always_visible:\n{items}{}",
        &LLM_LIKE[..start],
        LLM_LIKE[end..].replace("must_reach: [search, navbar]", "must_reach: [title]")
    )
}

fn item<'a>(config: &'a checks::Config, name: &str) -> &'a checks::AlwaysVisible {
    config
        .always_visible
        .iter()
        .find(|a| a.name == name)
        .unwrap()
}

#[test]
fn an_item_may_be_located_by_css_alone() {
    let c = checks::config(&with_items(
        "  - {name: title, css: 'a.active', scope: 'nav[aria-label=screens]'}\n",
    ))
    .unwrap();
    let l = &item(&c, "title").locate;
    assert_eq!(l.role, None);
    assert_eq!(l.name, checks::Name::Any);
    assert_eq!(l.css.as_deref(), Some("a.active"));
    assert_eq!(l.scope.as_deref(), Some("nav[aria-label=screens]"));
    assert!(!l.current);
}

#[test]
fn an_item_may_be_the_current_element_of_its_scope() {
    let c = checks::config(&with_items(
        "  - {name: title, current: true, scope: 'nav[aria-label=screens]'}\n  - {name: entry, role: link, current: true, scope: nav}\n  - {name: tab, css: '[role=tab]', current: true}\n",
    ))
    .unwrap();
    let title = &item(&c, "title").locate;
    assert!(title.current);
    assert_eq!((title.role.as_deref(), title.css.as_deref()), (None, None));
    assert_eq!(item(&c, "entry").locate.role.as_deref(), Some("link"));
    assert!(item(&c, "entry").locate.current);
    assert!(item(&c, "tab").locate.current);
}

#[test]
fn refuses_an_item_with_nothing_to_locate_it_by_or_a_name_it_cannot_match() {
    for items in [
        "  - {name: title, scope: nav}\n",
        "  - {name: title, current: false, scope: nav}\n",
        "  - {name: title, current: yes, scope: nav}\n",
        "  - {name: title, current: 'true', scope: nav}\n",
        "  - {name: title, css: a, name_is: Outbox}\n",
        "  - {name: title, current: true, name_contains: [outbox]}\n",
        "  - {name: title, role: link, current: 1}\n",
    ] {
        assert!(checks::config(&with_items(items)).is_err(), "{items}");
    }
    assert!(
        checks::config(&with_items("  - {name: title, role: link}\n")).is_ok(),
        "a role alone still locates an item"
    );
}

#[test]
fn the_declared_items_of_the_examples_read_as_before() {
    let llm =
        checks::config(&std::fs::read_to_string(crate::fixture("checks/llm.checks.yaml")).unwrap())
            .unwrap();
    let located: Vec<(&str, Option<&str>, bool)> = llm
        .always_visible
        .iter()
        .map(|a| (a.name.as_str(), a.locate.role.as_deref(), a.locate.current))
        .collect();
    assert_eq!(
        located,
        [
            ("navbar", Some("navigation"), false),
            ("search", Some("searchbox"), false),
            ("theme toggle", Some("button"), false),
            ("sidebar", Some("navigation"), false),
        ]
    );
}

fn cand(text: &str) -> Candidate {
    Candidate::from_value(&json::parse(text).unwrap()).unwrap()
}

const BOX: &str = r#""box":{"x":10,"y":10,"width":50,"height":20}"#;

fn visible(current: &str) -> Candidate {
    cand(&format!(
        r#"{{"in_scope":true,"visible":true,{BOX},"hit":true,"current":{current}}}"#
    ))
}

#[test]
fn a_candidate_is_current_when_its_aria_current_is_not_false() {
    let recorded_before = cand(&format!(
        r#"{{"in_scope":true,"visible":true,{BOX},"hit":true}}"#
    ));
    assert!(
        !recorded_before.is_current(),
        "a 0.3.0 record has no `current`"
    );
    for (value, current) in [
        ("null", false),
        (r#""""#, false),
        (r#""false""#, false),
        (r#"" FALSE ""#, false),
        (r#""page""#, true),
        (r#""true""#, true),
        (r#""step""#, true),
        (r#""location""#, true),
    ] {
        assert_eq!(visible(value).is_current(), current, "aria-current={value}");
    }
}

fn locate(text: &str) -> checks::Locate {
    let c = checks::config(&with_items(text)).unwrap();
    item(&c, "title").locate.clone()
}

#[test]
fn the_item_is_the_first_visible_candidate_in_scope_and_current_when_it_must_be() {
    let hidden = cand(&format!(
        r#"{{"in_scope":true,"visible":false,{BOX},"hit":false,"current":"page"}}"#
    ));
    let outside = cand(&format!(
        r#"{{"in_scope":false,"visible":true,{BOX},"hit":true,"current":"page"}}"#
    ));
    let other = visible("null");
    let current = visible(r#""page""#);
    let all = [hidden, outside, other, current];

    let by_css = locate("  - {name: title, css: 'nav a'}\n");
    assert_eq!(
        checks::located(&by_css, &all),
        Some(2),
        "its name is not checked"
    );
    let by_current = locate("  - {name: title, current: true, scope: nav}\n");
    assert_eq!(checks::located(&by_current, &all), Some(3));
    assert_eq!(checks::located(&by_current, &all[..3]), None);
    let by_role = locate("  - {name: title, role: link, name_is: Outbox}\n");
    assert_eq!(
        checks::located(&by_role, &all),
        checks::first_visible(&all),
        "an item without `current` takes the first visible candidate, as before"
    );
}

/// One page at one viewport in one mode, with the item's candidates recorded at the top and the
/// bottom.
fn page(candidates: &str) -> ux_browser::checks::PageResult {
    let config = checks::config(&with_items(
        "  - {name: title, current: true, scope: 'nav[aria-label=screens]'}\n",
    ))
    .unwrap();
    let visible = format!(r#"[{{"item":"title","candidates":[{candidates}]}}]"#);
    let record = json::parse(&format!(
        r#"{{"path":"/llm/asks.html","viewports":[{{"viewport":"1440x900",
            "shifts":{{"ready":1,"entries":[]}},
            "units":[{{"mode":"light","mode_applied":true,
              "overflow":{{"vw":1440,"scroll_width":1440,"blocks":[],"items":[]}},
              "visible":{{"top":{visible},"bottom":{visible}}}}}]}}]}}"#
    ))
    .unwrap();
    checks::page_result(&config, &record).unwrap()
}

#[test]
fn a_page_s_current_item_is_found_or_missing() {
    let found = page(&format!(
        r#"{{"in_scope":true,"visible":true,{BOX},"hit":true,"current":null}},
           {{"in_scope":true,"visible":true,{BOX},"hit":true,"current":"page"}}"#
    ));
    let statuses: Vec<Status> = found.units[0].visible.iter().map(|v| v.status).collect();
    assert_eq!(statuses, [Status::Ok, Status::Ok]);
    assert_eq!(found.always_visible_misses(), 0);

    let none_current = page(&format!(
        r#"{{"in_scope":true,"visible":true,{BOX},"hit":true,"current":"false"}}"#
    ));
    assert_eq!(none_current.units[0].visible[0].status, Status::Missing);
    assert_eq!(none_current.always_visible_misses(), 2, "top and bottom");

    let covered = page(&format!(
        r#"{{"in_scope":true,"visible":true,{BOX},"hit":false,"current":"page"}}"#
    ));
    assert_eq!(covered.units[0].visible[0].status, Status::Covered);
}

/// A guide of the first consumer's site recorded by `ux-browser checks --capture` with
/// `fixtures/checks/llm-current.checks.yaml`: the navbar by CSS alone, the navbar's current section
/// link, the sidebar's current entry (by `aria-current` alone and by role `link` with it), and a
/// current element in the footer, which has none.
#[test]
fn the_recorded_guide_page_locates_items_by_css_and_by_aria_current() {
    let config = checks::config(
        &std::fs::read_to_string(crate::fixture("checks/llm-current.checks.yaml")).unwrap(),
    )
    .unwrap();
    let page = crate::fixture_json("checks/llm-current.capture.json");
    let result = checks::page_result(&config, &page).unwrap();
    assert_eq!(result.error, None);
    let status = |vp: &str, item: &str| {
        let seen: Vec<&str> = result
            .units
            .iter()
            .filter(|u| u.viewport == vp)
            .flat_map(|u| &u.visible)
            .filter(|v| v.item == item)
            .map(|v| v.status.word())
            .collect();
        seen
    };
    let four = |word| vec![word; 4];
    // Light and dark, top and bottom.
    assert_eq!(status("1440x900", "navbar"), four("ok"));
    assert_eq!(status("1440x900", "section"), four("ok"));
    assert_eq!(status("1440x900", "page entry"), four("ok"));
    assert_eq!(status("1440x900", "entry link"), four("ok"));
    assert_eq!(status("1440x900", "footer current"), four("missing"));
    assert_eq!(status("390x844", "navbar"), four("ok"));
    // The section link is in the collapsed menu at 390 px: there, but not visible.
    assert_eq!(status("390x844", "section"), four("missing"));
    assert_eq!(status("390x844", "page entry"), four("n/a"));
    assert_eq!(result.always_visible_misses(), 12);

    // The sidebar's current entry is not the first visible link of the sidebar.
    let entry = config
        .always_visible
        .iter()
        .find(|a| a.name == "entry link")
        .unwrap();
    let top = page.get("viewports").unwrap().list()[0]
        .get("units")
        .unwrap()
        .list()[0]
        .get("visible")
        .unwrap()
        .get("top")
        .unwrap()
        .list()
        .iter()
        .find(|e| e.get_str("item") == Some("entry link"))
        .unwrap()
        .get("candidates")
        .unwrap()
        .list()
        .iter()
        .map(|c| Candidate::from_value(c).unwrap())
        .collect::<Vec<_>>();
    let current = checks::located(&entry.locate, &top).unwrap();
    assert!(top[current].is_current());
    assert!(checks::first_visible(&top).unwrap() < current);
}
