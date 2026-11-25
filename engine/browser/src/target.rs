//! Where a checks run measures: a server, named by its origin (what the request guard lets through)
//! and, for a server it did not start (`--url`), the place of the configuration's base path on it.
//! Pure functions: nothing here opens a socket.

/// A server the pages of a run are measured on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Site {
    /// `scheme://host[:port]` as the browser writes a request URL: lower case, no default port, no
    /// trailing `/`.
    pub origin: String,
    /// Where the configuration's base path lies on the server, starting and ending with `/`; `None`:
    /// where the configuration says (a page is its configured path on the origin).
    pub mount: Option<String>,
}

fn host_ok(host: &str) -> bool {
    if let Some(inner) = host.strip_prefix('[') {
        return inner.strip_suffix(']').is_some_and(|ip| {
            !ip.is_empty()
                && ip
                    .chars()
                    .all(|c| c.is_ascii_hexdigit() || c == ':' || c == '.')
        });
    }
    !host.is_empty()
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
}

/// Reads `--url`: `http` or `https`, a host, an optional port and an optional path; no user, query
/// or fragment. A path other than `/` is where the configuration's base path lies on the server.
pub fn parse_url(text: &str) -> Result<Site, String> {
    let wrong = |why: &str| {
        Err(format!(
            "`{text}` is not an origin with an optional path: {why}"
        ))
    };
    let Some((scheme, rest)) = text.split_once("://") else {
        return wrong("no `http://` or `https://`");
    };
    let scheme = scheme.to_ascii_lowercase();
    let default_port = match scheme.as_str() {
        "http" => 80,
        "https" => 443,
        _ => return wrong("the scheme is not `http` or `https`"),
    };
    if rest.contains(['?', '#']) {
        return wrong("a query or fragment");
    }
    let (authority, path) = match rest.find('/') {
        Some(i) => rest.split_at(i),
        None => (rest, ""),
    };
    if authority.contains('@') {
        return wrong("a user");
    }
    let (host, port) = match authority.rfind(':') {
        Some(i) if !authority[i..].contains(']') => {
            let digits = &authority[i + 1..];
            if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
                return wrong("the port is not a number");
            }
            match digits.parse::<u32>() {
                Ok(p) if (1..=65_535).contains(&p) => (&authority[..i], Some(p)),
                _ => return wrong("the port is not between 1 and 65535"),
            }
        }
        _ => (authority, None),
    };
    let host = host.to_ascii_lowercase();
    if !host_ok(&host) {
        return wrong("no host, or a host with characters a host does not have");
    }
    let origin = match port {
        Some(p) if p != default_port => format!("{scheme}://{host}:{p}"),
        _ => format!("{scheme}://{host}"),
    };
    let mount = if path.is_empty() || path == "/" {
        None
    } else {
        let segments: Vec<&str> = path[1..].trim_end_matches('/').split('/').collect();
        if segments
            .iter()
            .any(|s| s.is_empty() || *s == "." || *s == "..")
        {
            return wrong("an empty, `.` or `..` segment in the path");
        }
        if path.contains(char::is_whitespace) {
            return wrong("white space in the path");
        }
        Some(format!("/{}/", segments.join("/")))
    };
    Ok(Site { origin, mount })
}

impl Site {
    /// A served build or a server at the configuration's base path: no mount.
    pub fn at(origin: &str) -> Site {
        Site {
            origin: origin.to_string(),
            mount: None,
        }
    }

    /// The URL the site was named by, normalized: the origin, and the mount when there is one.
    pub fn url(&self) -> String {
        format!(
            "{}{}",
            self.origin,
            self.mount.as_deref().unwrap_or_default()
        )
    }

    /// The URL of the page at the configured `path` (it starts with `/`): the path on the origin, or
    /// with a mount, the path's part after `base_path` under the mount.
    pub fn page_url(&self, base_path: &str, path: &str) -> Result<String, String> {
        if !path.starts_with('/') {
            return Err(format!("page `{path}` does not start with `/`"));
        }
        let Some(mount) = &self.mount else {
            return Ok(format!("{}{path}", self.origin));
        };
        let rest = if let Some(rest) = path.strip_prefix(base_path) {
            rest
        } else if format!("{path}/") == base_path {
            ""
        } else {
            return Err(format!(
                "page `{path}` is not under `base_path` `{base_path}`, so it has no place under `{mount}`"
            ));
        };
        Ok(format!("{}{mount}{rest}", self.origin))
    }

    /// The URL of the sitemap `file`, a file of the site under its base path, as `--build` reads it.
    pub fn sitemap_url(&self, base_path: &str, file: &str) -> Result<String, String> {
        self.page_url(
            base_path,
            &format!("{base_path}{}", file.trim_start_matches('/')),
        )
    }
}
