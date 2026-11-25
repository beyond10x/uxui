//! Where a checks run measures: the `--url` of a server it did not start, read into an origin (what
//! the request guard lets through) and the place of the configuration's base path on that server;
//! the URL of each page and of the sitemap there. Pure functions: no socket is opened.

use ux_browser::target::{self, Site};

fn site(origin: &str, mount: Option<&str>) -> Site {
    Site {
        origin: origin.to_string(),
        mount: mount.map(str::to_string),
    }
}

#[test]
fn an_origin_is_scheme_host_and_port_as_the_browser_writes_it() {
    let s = target::parse_url("http://127.0.0.1:4000").unwrap();
    assert_eq!(s.origin, "http://127.0.0.1:4000");
    assert_eq!(s.mount, None);
    let s = target::parse_url("HTTP://Example.ORG:80/").unwrap();
    assert_eq!(
        s.origin, "http://example.org",
        "lower case, no default port"
    );
    assert_eq!(s.mount, None, "the path `/` is no path");
    let s = target::parse_url("https://example.org:443").unwrap();
    assert_eq!(s.origin, "https://example.org");
    let s = target::parse_url("https://example.org:8443").unwrap();
    assert_eq!(s.origin, "https://example.org:8443");
    let s = target::parse_url("http://[::1]:3000/").unwrap();
    assert_eq!(s.origin, "http://[::1]:3000");
    let s = target::parse_url("http://localhost:5173").unwrap();
    assert_eq!(s.origin, "http://localhost:5173");
}

#[test]
fn a_path_after_the_origin_is_where_the_base_path_lies_on_the_server() {
    let s = target::parse_url("http://127.0.0.1:4000/preview/llm").unwrap();
    assert_eq!(s.origin, "http://127.0.0.1:4000");
    assert_eq!(
        s.mount.as_deref(),
        Some("/preview/llm/"),
        "a base path ends in `/`"
    );
    assert_eq!(s.url(), "http://127.0.0.1:4000/preview/llm/");
    let s = target::parse_url("http://127.0.0.1:4000/").unwrap();
    assert_eq!(s.url(), "http://127.0.0.1:4000");
}

#[test]
fn a_url_that_is_not_an_origin_with_a_path_is_refused() {
    for wrong in [
        "127.0.0.1:4000",
        "localhost:3000/",
        "ftp://example.org",
        "file:///srv/site",
        "http://",
        "http:///x",
        "http://user@example.org",
        "http://example.org/?q=1",
        "http://example.org/#top",
        "http://example.org:0",
        "http://example.org:65536",
        "http://example.org:80x",
        "http://example.org:",
        "http://exa mple.org",
        "http://example.org/a/../b",
        "http://example.org/a//b",
        "http://[::1",
        "",
    ] {
        assert!(target::parse_url(wrong).is_err(), "{wrong:?} is refused");
    }
}

#[test]
fn without_a_path_a_page_is_its_configured_path_on_the_origin() {
    let s = site("http://127.0.0.1:4000", None);
    assert_eq!(
        s.page_url("/llm/", "/llm/search").unwrap(),
        "http://127.0.0.1:4000/llm/search"
    );
    assert_eq!(
        s.page_url("/", "/today.html").unwrap(),
        "http://127.0.0.1:4000/today.html"
    );
    assert!(
        s.page_url("/llm/", "search").is_err(),
        "a page path starts with `/`"
    );
}

#[test]
fn with_a_path_a_page_moves_from_the_base_path_to_it() {
    let s = site("https://example.org", Some("/preview/"));
    assert_eq!(
        s.page_url("/llm/", "/llm/search").unwrap(),
        "https://example.org/preview/search"
    );
    assert_eq!(
        s.page_url("/llm/", "/llm/").unwrap(),
        "https://example.org/preview/"
    );
    assert_eq!(
        s.page_url("/llm/", "/llm").unwrap(),
        "https://example.org/preview/",
        "the base path without its last `/`"
    );
    assert_eq!(
        s.page_url("/", "/index.html").unwrap(),
        "https://example.org/preview/index.html"
    );
    let err = s.page_url("/llm/", "/other/page").unwrap_err();
    assert!(err.contains("/other/page"), "{err}");
}

#[test]
fn the_sitemap_is_the_file_under_the_base_path_on_the_server() {
    let s = site("http://127.0.0.1:4000", None);
    assert_eq!(
        s.sitemap_url("/llm/", "sitemap.xml").unwrap(),
        "http://127.0.0.1:4000/llm/sitemap.xml"
    );
    assert_eq!(
        s.sitemap_url("/", "sitemap.xml").unwrap(),
        "http://127.0.0.1:4000/sitemap.xml"
    );
    assert_eq!(
        s.sitemap_url("/llm/", "/sitemap.xml").unwrap(),
        "http://127.0.0.1:4000/llm/sitemap.xml",
        "a file of the site, as with `--build`"
    );
    let mounted = site("http://127.0.0.1:4000", Some("/preview/"));
    assert_eq!(
        mounted.sitemap_url("/llm/", "sitemap.xml").unwrap(),
        "http://127.0.0.1:4000/preview/sitemap.xml"
    );
}

#[test]
fn the_paths_of_a_fetched_sitemap_are_measured_where_the_server_serves_them() {
    // A sitemap names the deployed host; its paths are the configuration's, moved by the URL's path.
    let xml = "<urlset><url><loc>https://docs.example.org/llm/</loc></url><url><loc>https://docs.example.org/llm/search</loc></url></urlset>";
    let paths = ux_browser::checks::sitemap_paths(xml);
    let s = site("http://127.0.0.1:4000", Some("/preview/"));
    let urls: Vec<String> = paths
        .iter()
        .map(|p| s.page_url("/llm/", p).unwrap())
        .collect();
    assert_eq!(
        urls,
        [
            "http://127.0.0.1:4000/preview/",
            "http://127.0.0.1:4000/preview/search"
        ]
    );
}
