//! Always-visible items located by CSS alone or by `aria-current` in a scope: what the
//! configuration accepts and refuses, which recorded candidate is the item, and the values of a page
//! from recorded candidates. An item with a role computes as before. An item's `max_width` limits
//! the viewports it is checked at from above, as `min_width` does from below.

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

// ---------------------------------------------------------------- max_width

/// The configuration of the narrow-menu fixture page (`fixtures/checks/narrow-menu-site/`): a
/// header whose "Open menu" button is shown below 1024 px and whose "Toggle navigation" button is
/// shown from 1024 px, both by a media query. The first is declared with `max_width: 1023`.
fn narrow_menu_config() -> String {
    std::fs::read_to_string(crate::fixture("checks/narrow-menu.checks.yaml")).unwrap()
}

/// The statuses of `item` at viewport `vp`, top then bottom (one mode).
fn statuses<'a>(result: &'a checks::PageResult, vp: &str, item: &str) -> Vec<&'a str> {
    result
        .units
        .iter()
        .filter(|u| u.viewport == vp)
        .flat_map(|u| &u.visible)
        .filter(|v| v.item == item)
        .map(|v| v.status.word())
        .collect()
}

fn misses_at(result: &checks::PageResult, vp: &str) -> usize {
    result
        .units
        .iter()
        .filter(|u| u.viewport == vp)
        .flat_map(|u| &u.visible)
        .filter(|v| v.status.is_miss())
        .count()
}

/// The page as ux-browser 0.6.0 recorded it (`checks --capture`, the configuration without
/// `max_width`): the "Open menu" item's candidates at all three viewports, none of them at 1024 and
/// 1440 px.
#[test]
fn an_item_with_max_width_is_checked_only_up_to_it() {
    let config = checks::config(&narrow_menu_config()).unwrap();
    let page = crate::fixture_json("checks/narrow-menu-0.6.0.capture.json");
    let result = checks::page_result(&config, &page).unwrap();
    assert_eq!(result.error, None);
    assert_eq!(statuses(&result, "390x844", "open menu"), ["ok", "ok"]);
    assert_eq!(statuses(&result, "1024x768", "open menu"), ["n/a", "n/a"]);
    assert_eq!(statuses(&result, "1440x900", "open menu"), ["n/a", "n/a"]);
    assert_eq!(
        statuses(&result, "390x844", "navigation toggle"),
        ["n/a", "n/a"]
    );
    assert_eq!(
        statuses(&result, "1440x900", "navigation toggle"),
        ["ok", "ok"]
    );
    assert_eq!(misses_at(&result, "390x844"), 0);
    assert_eq!(misses_at(&result, "1440x900"), 0);
    assert_eq!(result.always_visible_misses(), 0);
}

/// The page as this version records it with `max_width: 1023`: the item is measured at 390 px only.
#[test]
fn an_item_past_its_max_width_is_not_measured() {
    let config = checks::config(&narrow_menu_config()).unwrap();
    let page = crate::fixture_json("checks/narrow-menu.capture.json");
    let measured: Vec<(&str, Vec<&str>)> = page
        .get("viewports")
        .unwrap()
        .list()
        .iter()
        .map(|vp| {
            let top = vp.get("units").unwrap().list()[0]
                .get("visible")
                .unwrap()
                .get("top")
                .unwrap()
                .list();
            (
                vp.get_str("viewport").unwrap(),
                top.iter().map(|e| e.get_str("item").unwrap()).collect(),
            )
        })
        .collect();
    assert_eq!(
        measured,
        [
            ("390x844", vec!["open menu"]),
            ("1024x768", vec!["navigation toggle"]),
            ("1440x900", vec!["navigation toggle"]),
        ]
    );
    let result = checks::page_result(&config, &page).unwrap();
    assert_eq!(statuses(&result, "390x844", "open menu"), ["ok", "ok"]);
    assert_eq!(statuses(&result, "1440x900", "open menu"), ["n/a", "n/a"]);
    assert_eq!(result.always_visible_misses(), 0);
}

#[test]
fn without_max_width_the_item_counts_at_every_width_as_in_0_6_0() {
    let text = narrow_menu_config().replace(", max_width: 1023}", "}");
    assert!(!text.contains("max_width"));
    let config = checks::config(&text).unwrap();
    let page = crate::fixture_json("checks/narrow-menu-0.6.0.capture.json");
    let result = checks::page_result(&config, &page).unwrap();
    assert_eq!(statuses(&result, "390x844", "open menu"), ["ok", "ok"]);
    assert_eq!(
        statuses(&result, "1440x900", "open menu"),
        ["missing", "missing"]
    );
    assert_eq!(misses_at(&result, "390x844"), 0);
    assert_eq!(
        misses_at(&result, "1440x900"),
        2,
        "one item, top and bottom"
    );
    assert_eq!(result.always_visible_misses(), 4, "at 1024 and at 1440 px");
}

#[test]
fn the_width_range_holds_both_its_ends() {
    let c = checks::config(&with_items(
        "  - {name: title, css: a, min_width: 400, max_width: 1023}\n",
    ))
    .unwrap();
    let title = item(&c, "title");
    for (width, applies) in [(399, false), (400, true), (1023, true), (1024, false)] {
        assert_eq!(
            checks::applies(title, "/llm/", width),
            applies,
            "{width} px"
        );
    }
    let only_narrow =
        checks::config(&with_items("  - {name: title, css: a, max_width: 1023}\n")).unwrap();
    let title = item(&only_narrow, "title");
    assert!(checks::applies(title, "/llm/", 1));
    assert!(!checks::applies(title, "/llm/", 1440));
    let one_width = checks::config(&with_items(
        "  - {name: title, css: a, min_width: 1024, max_width: 1024}\n",
    ))
    .unwrap();
    assert!(checks::applies(item(&one_width, "title"), "/llm/", 1024));
}

#[test]
fn refuses_a_max_width_below_min_width_zero_or_not_a_whole_number() {
    for items in [
        "  - {name: title, css: a, min_width: 1024, max_width: 1023}\n",
        "  - {name: title, css: a, max_width: 0}\n",
        "  - {name: title, css: a, max_width: -1}\n",
        "  - {name: title, css: a, max_width: 1023.5}\n",
        "  - {name: title, css: a, max_width: '1023'}\n",
        "  - {name: title, css: a, max_width: wide}\n",
        "  - {name: title, css: a, max_width: true}\n",
        "  - {name: title, css: a, max_width: [1023]}\n",
    ] {
        let Err(err) = checks::config(&with_items(items)) else {
            panic!("{items}: accepted");
        };
        assert!(
            err.contains("always_visible `title`") && err.contains("max_width"),
            "{items}: {err}"
        );
    }
}

// ---------------------------------------------------------------- the points of the cover test
//
// `fixtures/checks/cover-site/` measured with `cover.checks.yaml` (400 x 300, one mode; each page
// holds one item, `button#item`, declared only on its page): `cover/` holds the captures of this
// probe, `cover-0.9.1/` those ux-browser 0.9.1 recorded of the same pages. The pages do not scroll,
// so an item is checked twice, at the top and at the bottom, at the same place.

/// The statuses of the one item of `page` as recorded in `dir`, top then bottom, and the page's
/// `always_visible.misses@1`.
fn cover(dir: &str, page: &str) -> (Vec<&'static str>, i64) {
    let config = checks::config(
        &std::fs::read_to_string(crate::fixture("checks/cover.checks.yaml")).unwrap(),
    )
    .unwrap();
    let capture = crate::fixture_json(&format!("checks/{dir}/{page}.capture.json"));
    let result = checks::page_result(&config, &capture).unwrap();
    assert_eq!(result.error, None, "{dir}/{page}");
    let statuses = result
        .units
        .iter()
        .flat_map(|u| &u.visible)
        .filter(|v| v.item == page)
        .map(|v| v.status.word())
        .collect();
    (statuses, result.always_visible_misses())
}

/// The sample points of the item of `page` that hit another element, at the top, as this probe
/// recorded them (`missed`).
fn missed(page: &str) -> Vec<String> {
    let capture = crate::fixture_json(&format!("checks/cover/{page}.capture.json"));
    let top = capture.get("viewports").unwrap().list()[0]
        .get("units")
        .unwrap()
        .list()[0]
        .get("visible")
        .unwrap()
        .get("top")
        .unwrap()
        .list();
    let candidate = &top[0].get("candidates").unwrap().list()[0];
    candidate
        .get("missed")
        .expect("the probe records the points that missed")
        .list()
        .iter()
        .map(|p| p.str().unwrap().to_string())
        .collect()
}

const COVERED: [&str; 2] = ["covered", "covered"];
const OK: [&str; 2] = ["ok", "ok"];

#[test]
fn an_item_under_a_sticky_bar_at_one_corner_is_covered() {
    // `button#item` at 20,10 160 x 60 under a sticky bar from x 170, 30 px high: its top right
    // corner lies under the bar, its centre does not.
    assert_eq!(cover("cover-0.9.1", "corner"), (OK.to_vec(), 0), "0.9.1");
    assert_eq!(
        cover("cover", "corner"),
        (COVERED.to_vec(), 2),
        "one item, top and bottom"
    );
    assert_eq!(missed("corner"), ["top-right"]);
}

#[test]
fn an_item_under_a_dialog_short_of_its_centre_is_covered() {
    // A dialog from x 110 over the right side of the item (20 to 180): both right corners.
    assert_eq!(cover("cover-0.9.1", "dialog"), (OK.to_vec(), 0), "0.9.1");
    assert_eq!(cover("cover", "dialog"), (COVERED.to_vec(), 2));
    assert_eq!(missed("dialog"), ["top-right", "bottom-right"]);
}

#[test]
fn an_item_covered_at_its_centre_is_covered_as_before() {
    assert_eq!(
        cover("cover-0.9.1", "centre"),
        (COVERED.to_vec(), 2),
        "0.9.1"
    );
    assert_eq!(cover("cover", "centre"), (COVERED.to_vec(), 2));
    assert_eq!(missed("centre"), ["centre"]);
}

#[test]
fn a_free_item_and_a_free_pill_are_not_covered() {
    // The pill (160 x 40, `border-radius: 999px`) has corners outside its curve: a point 1 px
    // inside its box there is no part of it.
    for page in ["free", "pill"] {
        assert_eq!(cover("cover-0.9.1", page), (OK.to_vec(), 0), "0.9.1 {page}");
        assert_eq!(cover("cover", page), (OK.to_vec(), 0), "{page}");
        assert!(missed(page).is_empty(), "{page}");
    }
}
