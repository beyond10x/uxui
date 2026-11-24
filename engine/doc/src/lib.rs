//! uxui-doc: what every uxui tool needs to read an `ess-ui/1` document, with no third-party
//! crates. ESS hands no resolved form over, so the tools read the YAML themselves.
//!
//! - [`yaml`]: the YAML subset ESS documents use;
//! - [`json`]: JSON text from a [`yaml::Value`];
//! - [`sha256`]: the digest a tool records for each input it read;
//! - [`tree`]: every node of a document with its canonical node path.

pub mod json;
pub mod sha256;
pub mod tree;
pub mod yaml;
