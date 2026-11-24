//! The render record (`ux-browser-render/1`): declarations in, one record out with each declared
//! node's box, computed styles and accessibility-tree entry, a screenshot, and the adapter's name
//! and version.

use crate::fixture_json;
use ux_browser::json;
use ux_browser::render::{self, AxEntry, FileRef, Record, Shot};
use uxui_doc::yaml::Value;

const DECLARATIONS: &str = "format: ux-browser-render/1
viewport: {width: 1440, height: 900}
mode: light
styles: [display, color, background-color]
nodes:
  - name: navbar
    selector: nav.navbar
  - name: hero
    selector: section.b10x-hero
";

#[test]
fn reads_the_declarations() {
    let d = render::declarations(DECLARATIONS).unwrap();
    assert_eq!((d.viewport.width, d.viewport.height), (1440, 900));
    assert_eq!(d.mode, "light");
    assert_eq!(d.styles, ["display", "color", "background-color"]);
    let nodes: Vec<(&str, &str)> = d
        .nodes
        .iter()
        .map(|n| (n.name.as_str(), n.selector.as_str()))
        .collect();
    assert_eq!(
        nodes,
        [("navbar", "nav.navbar"), ("hero", "section.b10x-hero")]
    );
}

#[test]
fn refuses_declarations_without_nodes_or_with_an_unknown_mode() {
    let no_nodes = "format: ux-browser-render/1\nviewport: {width: 1, height: 1}\nmode: light\nstyles: [color]\nnodes: []\n";
    assert!(render::declarations(no_nodes).is_err());
    assert!(render::declarations(&DECLARATIONS.replace("mode: light", "mode: sepia")).is_err());
    assert!(render::declarations(&DECLARATIONS.replace("ux-browser-render/1", "other/1")).is_err());
    assert!(render::declarations(&DECLARATIONS.replace("    selector: nav.navbar\n", "")).is_err());
}

#[test]
fn takes_the_accessibility_entry_of_the_node_itself() {
    let partial = json::parse(
        r#"{"nodes":[{"nodeId":"12","ignored":false,"role":{"type":"role","value":"navigation"},"name":{"type":"computedString","value":"Main"},"backendDOMNodeId":40}]}"#,
    )
    .unwrap();
    assert_eq!(
        render::ax_entry(&partial),
        Some(AxEntry {
            role: "navigation".to_string(),
            name: "Main".to_string(),
            ignored: false
        })
    );
    let ignored = json::parse(
        r#"{"nodes":[{"nodeId":"3","ignored":true,"ignoredReasons":[],"role":{"type":"role","value":"none"},"backendDOMNodeId":8}]}"#,
    )
    .unwrap();
    assert_eq!(
        render::ax_entry(&ignored),
        Some(AxEntry {
            role: "none".to_string(),
            name: String::new(),
            ignored: true
        })
    );
    assert_eq!(
        render::ax_entry(&json::parse(r#"{"nodes":[]}"#).unwrap()),
        None
    );
}

#[test]
fn reads_the_size_of_a_png() {
    // The signature and the IHDR chunk of a 1440 x 900 PNG.
    let mut png = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 13];
    png.extend(b"IHDR");
    png.extend(1440u32.to_be_bytes());
    png.extend(900u32.to_be_bytes());
    png.extend([8, 6, 0, 0, 0]);
    assert_eq!(render::png_size(&png), Some((1440, 900)));
    assert_eq!(render::png_size(b"GIF89a"), None);
}

fn record_of(probes: &[Value], axes: Vec<Option<AxEntry>>) -> Value {
    let d = render::declarations(DECLARATIONS).unwrap();
    let nodes = d
        .nodes
        .iter()
        .zip(probes)
        .zip(axes)
        .map(|((decl, probe), ax)| render::node_render(decl, &d.styles, probe, ax).unwrap())
        .collect();
    let record = Record {
        url: "http://127.0.0.1:4000/uxui/".to_string(),
        declarations: &d,
        declarations_file: FileRef {
            path: Some("engine/browser/examples/uxui-landing.render.yaml".to_string()),
            digest: "sha256:00".to_string(),
        },
        browser: "Chrome/150.0.0.0".to_string(),
        nodes,
        screenshot: Some(Shot {
            file: "landing.png".to_string(),
            digest: "sha256:11".to_string(),
            width: 1440,
            height: 900,
        }),
    };
    render::record_value(&record)
}

#[test]
fn a_record_names_the_adapter_and_holds_every_declared_node() {
    let found = json::parse(
        r#"{"matches":1,"box":{"x":0,"y":0,"width":1440,"height":60},"styles":{"display":"flex","color":"rgb(28, 30, 33)","background-color":"rgb(255, 255, 255)"}}"#,
    )
    .unwrap();
    let missing = json::parse(r#"{"matches":0}"#).unwrap();
    let nav = AxEntry {
        role: "navigation".to_string(),
        name: "Main".to_string(),
        ignored: false,
    };
    let v = record_of(&[found, missing], vec![Some(nav), None]);
    assert_eq!(v.get_str("format"), Some("ux-browser-render/1"));
    let adapter = v.get("adapter").unwrap();
    assert_eq!(adapter.get_str("name"), Some("browser"));
    assert_eq!(adapter.get_str("tool"), Some("ux-browser"));
    assert_eq!(adapter.get_str("version"), Some(env!("CARGO_PKG_VERSION")));
    assert_eq!(v.get_str("mode"), Some("light"));
    assert_eq!(
        v.get("viewport").unwrap().get("width"),
        Some(&Value::Int(1440))
    );
    let nodes = v.get("nodes").unwrap().list();
    assert_eq!(nodes.len(), 2);
    let navbar = &nodes[0];
    assert_eq!(navbar.get_str("name"), Some("navbar"));
    assert_eq!(navbar.get("matches"), Some(&Value::Int(1)));
    assert_eq!(
        navbar.get("box").unwrap().get("height"),
        Some(&Value::Int(60))
    );
    assert_eq!(
        navbar.get("styles").unwrap().get_str("display"),
        Some("flex")
    );
    let ax = navbar.get("accessibility").unwrap();
    assert_eq!(
        (ax.get_str("role"), ax.get_str("name")),
        (Some("navigation"), Some("Main"))
    );
    let hero = &nodes[1];
    assert_eq!(hero.get("matches"), Some(&Value::Int(0)));
    assert_eq!(hero.get("box"), None);
    let shot = v.get("screenshot").unwrap();
    assert_eq!(shot.get_str("file"), Some("landing.png"));
    assert_eq!(shot.get("width"), Some(&Value::Int(1440)));
}

#[test]
fn a_probe_missing_a_declared_style_is_refused() {
    let d = render::declarations(DECLARATIONS).unwrap();
    let probe = json::parse(
        r#"{"matches":1,"box":{"x":0,"y":0,"width":1,"height":1},"styles":{"display":"block"}}"#,
    )
    .unwrap();
    assert!(render::node_render(&d.nodes[0], &d.styles, &probe, None).is_err());
}

/// The probes and accessibility entries of the acceptance run on the documentation landing page,
/// recorded by `ux-browser render --capture`.
#[test]
fn the_recorded_landing_page_gives_every_declared_node_a_box_styles_and_a_role() {
    let capture = fixture_json("render/landing.capture.json");
    let declarations =
        std::fs::read_to_string(crate::fixture("render/landing.render.yaml")).unwrap();
    let d = render::declarations(&declarations).unwrap();
    let nodes: Vec<render::NodeRender> = d
        .nodes
        .iter()
        .zip(capture.get("nodes").unwrap().list())
        .map(|(decl, raw)| {
            let ax = raw.get("ax").and_then(render::ax_entry);
            render::node_render(decl, &d.styles, raw.get("probe").unwrap(), ax).unwrap()
        })
        .collect();
    assert_eq!(nodes.len(), d.nodes.len());
    for node in &nodes {
        assert_eq!(node.matches, 1, "{} matches one element", node.name);
        let b = node.rect.as_ref().expect("a box");
        assert!(b.width > 0.0 && b.height > 0.0, "{} has a size", node.name);
        assert_eq!(
            node.styles.len(),
            d.styles.len(),
            "{} has every style",
            node.name
        );
        assert!(
            node.ax.is_some(),
            "{} has an accessibility entry",
            node.name
        );
    }
    let navbar = nodes.iter().find(|n| n.name == "navbar").unwrap();
    let ax = navbar.ax.as_ref().unwrap();
    assert_eq!((ax.role.as_str(), ax.name.as_str()), ("navigation", "Main"));
}
