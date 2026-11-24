//! A static server for a built site under its base path, on a free loopback port: a URL path
//! resolves to the file, the file plus `.html`, or its `index.html`, as a static host with
//! `trailingSlash: false` serves a Docusaurus build; anything else is the build's `404.html` with
//! status 404. One thread per connection, one response per connection.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};

pub struct Server {
    /// `http://127.0.0.1:<port>`.
    pub origin: String,
}

pub fn content_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
    {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "xml" => "application/xml",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn percent_decode(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = std::str::from_utf8(bytes.get(i + 1..i + 3)?).ok()?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// The file a URL path names under `root`, served at `base_path` (`/llm/`).
pub fn resolve(root: &Path, base_path: &str, url_path: &str) -> Option<PathBuf> {
    let path = url_path.split(['?', '#']).next().unwrap_or_default();
    let rest = if let Some(rest) = path.strip_prefix(base_path) {
        rest
    } else if format!("{path}/") == base_path {
        ""
    } else {
        return None;
    };
    let rel = PathBuf::from(percent_decode(rest)?);
    if rel.components().any(|c| !matches!(c, Component::Normal(_))) {
        return None;
    }
    let base = root.join(&rel);
    let mut with_html = base.clone().into_os_string();
    with_html.push(".html");
    [
        base.clone(),
        PathBuf::from(with_html),
        base.join("index.html"),
    ]
    .into_iter()
    .find(|c| c.is_file())
}

fn respond(mut stream: TcpStream, root: &Path, base_path: &str) {
    let mut head = Vec::new();
    let mut buf = [0u8; 4096];
    while !head.windows(4).any(|w| w == b"\r\n\r\n") && head.len() < 64 * 1024 {
        match stream.read(&mut buf) {
            Ok(0) | Err(_) => return,
            Ok(n) => head.extend_from_slice(&buf[..n]),
        }
    }
    let text = String::from_utf8_lossy(&head);
    let mut parts = text.lines().next().unwrap_or_default().split_whitespace();
    let (method, target) = (
        parts.next().unwrap_or_default(),
        parts.next().unwrap_or("/"),
    );
    let (status, file) = match resolve(root, base_path, target) {
        Some(f) => ("200 OK", Some(f)),
        None => (
            "404 Not Found",
            Some(root.join("404.html")).filter(|p| p.is_file()),
        ),
    };
    let (body, kind) = match &file {
        Some(f) => (std::fs::read(f).unwrap_or_default(), content_type(f)),
        None => (b"not found".to_vec(), "text/plain; charset=utf-8"),
    };
    let header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(header.as_bytes());
    if method != "HEAD" {
        let _ = stream.write_all(&body);
    }
    let _ = stream.flush();
}

/// Serves `root` at `base_path` until the process ends.
pub fn start(root: &Path, base_path: &str) -> Result<Server, String> {
    if !root.join("index.html").is_file() {
        return Err(format!("{}: no built site (no index.html)", root.display()));
    }
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let (root, base_path) = (root.to_path_buf(), base_path.to_string());
    std::thread::spawn(move || {
        for stream in listener.incoming().map_while(Result::ok) {
            let (root, base_path) = (root.clone(), base_path.clone());
            std::thread::spawn(move || respond(stream, &root, &base_path));
        }
    });
    Ok(Server {
        origin: format!("http://127.0.0.1:{port}"),
    })
}
