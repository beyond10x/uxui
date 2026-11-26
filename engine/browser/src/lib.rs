//! ux-browser@1, the browser adapter of the engine (`docs/targets.md`, `docs/browser-adapter.md`):
//! any page with a URL, rendered by headless Chromium driven over the DevTools protocol by a client
//! of its own, with no third-party crate.
//!
//! - [`ws`], [`base64`], [`sha1`]: the WebSocket client the protocol runs over;
//! - [`json`]: the JSON reader; writing is `uxui_doc::json`;
//! - [`cdp`]: commands, responses, events and the request guard;
//! - [`chrome`]: finding and starting Chromium; [`session`]: one page driven through a session;
//! - [`serve`]: a static server for a built site; [`target`]: a running server (`checks --url`), its
//!   origin and the URL of each page on it;
//! - [`render`]: the render record `ux-browser-render/1`;
//! - [`journey`]: journeys, steps by CSS selector to a page state, and the checks of each state;
//! - [`pool`]: the worker threads a checks run measures its pages with;
//! - [`values`]: the values file a consumer's ratchet and hard gates read, from a checks document.

pub mod base64;
pub mod cdp;
pub mod checks;
pub mod chrome;
pub mod evidence;
pub mod journey;
pub mod json;
pub mod measure;
pub mod pool;
pub mod render;
pub mod serve;
pub mod session;
pub mod sha1;
pub mod target;
pub mod value;
pub mod values;
pub mod ws;

/// The adapter's version, for a consumer of the library that names the adapter in what it records.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
