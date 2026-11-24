//! ux-browser@1, the browser adapter of the engine (`docs/targets.md`, `docs/browser-adapter.md`):
//! any page with a URL, rendered by headless Chromium driven over the DevTools protocol by a client
//! of its own, with no third-party crate.
//!
//! - [`ws`], [`base64`], [`sha1`]: the WebSocket client the protocol runs over;
//! - [`json`]: the JSON reader; writing is `uxui_doc::json`;
//! - [`cdp`]: commands, responses, events and the request guard;
//! - [`chrome`]: finding and starting Chromium; [`session`]: one page driven through a session;
//! - [`serve`]: a static server for a built site;
//! - [`render`]: the render record `ux-browser-render/1`.

pub mod base64;
pub mod cdp;
pub mod checks;
pub mod chrome;
pub mod evidence;
pub mod json;
pub mod measure;
pub mod render;
pub mod serve;
pub mod session;
pub mod sha1;
pub mod value;
pub mod ws;
