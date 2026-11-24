//! ux-static@1: the static UX metrics of an `ess-ui/1` document, written as ratchet evidence
//! records, without a browser or a layout model. Inputs: the document, its `ess-ui-test/1`
//! journeys and a `uxui-declarations/1` file. Metrics, one record per metric and scope:
//!
//! - `structure.required_present@1`: declared required nodes the document lacks, per journey, and
//!   the always-visible nodes and cue for the variant;
//! - `effort.actions_per_journey@1`: the actions each journey's steps take;
//! - `a11y.accessible_names@1`: interactive nodes without an accessible name, per page, with an ACT
//!   outcome per node.
//!
//! Reference: `docs/ux-static.md`, `docs/uxui-declarations.md`.

mod assess;
pub mod declarations;
pub mod journeys;
mod names;
mod record;

use std::path::Path;
use uxui_doc::yaml::Value;

/// The assessor and the version of its metric set.
pub const ASSESSOR: &str = "ux-static@1";

pub struct Inputs<'a> {
    pub document: &'a Path,
    pub journeys: &'a Path,
    pub declarations: &'a Path,
    /// The target class the evidence is produced in (`local`, `ci`).
    pub class: &'a str,
}

/// The evidence records for `inputs`, in a fixed order: structure, actions, names. An input that is
/// unreadable, or inputs that do not belong together, are an error, not evidence.
pub fn assess(inputs: &Inputs) -> Result<Vec<Value>, String> {
    let document = assess::read(inputs.document)?;
    let journeys = assess::read(inputs.journeys)?;
    let declarations = assess::read(inputs.declarations)?;
    let paths = [inputs.document, inputs.journeys, inputs.declarations];
    assess::run(&document, &journeys, &declarations, paths, inputs.class)?.records()
}

/// The records as JSON lines.
pub fn jsonl(records: &[Value]) -> String {
    records
        .iter()
        .map(|r| uxui_doc::json::write(r) + "\n")
        .collect()
}
