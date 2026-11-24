//! ux-static@1: the static UX metrics of an `ess-ui/1` document, written as one
//! `ratchet-evidence/1` document per run, without a browser or a layout model. Inputs: the
//! document, its `ess-ui-test/1` journeys and a `uxui-declarations/1` file. Outputs, one result per
//! scope item:
//!
//! - `structure.required_present@1`: declared required nodes the document lacks, per journey, and
//!   the always-visible nodes and cue for the variant;
//! - `effort.actions_per_journey@1`: the actions each journey's steps take;
//! - `a11y.accessible_names@1`: interactive nodes without an accessible name, per page, with an ACT
//!   outcome per node;
//! - `findings`: the failed checks of the first and the third, an identity set per scope item.
//!
//! Reference: `docs/ux-static.md`, `docs/uxui-declarations.md`, ratchet `docs/evidence-format.md`.

mod assess;
pub mod declarations;
pub mod journeys;
mod names;
mod record;

use std::path::Path;
use uxui_doc::yaml::Value;

/// The assessor and the version of its metric set.
pub const ASSESSOR: &str = "ux-static@1";
pub use record::FORMAT;

pub struct Inputs<'a> {
    pub document: &'a Path,
    pub journeys: &'a Path,
    pub declarations: &'a Path,
    /// The repository root; every path in the evidence is relative to it.
    pub root: &'a Path,
    /// The `project` of the quality specification that holds the plan.
    pub project: &'a str,
    /// The key of the plan under `assessments`.
    pub assessment: &'a str,
    /// The target class the evidence is produced in (`local`, `ci`, ...).
    pub class: &'a str,
}

/// The evidence document for `inputs`. An input that is unreadable or outside the root, or inputs
/// that do not belong together, are an error, not evidence.
pub fn assess(inputs: &Inputs) -> Result<Value, String> {
    let document = assess::read(inputs.document, inputs.root)?;
    let journeys = assess::read(inputs.journeys, inputs.root)?;
    let declarations = assess::read(inputs.declarations, inputs.root)?;
    let plan = assess::Plan {
        project: inputs.project,
        assessment: inputs.assessment,
        class: inputs.class,
    };
    let files = [&document, &journeys, &declarations];
    let paths = [inputs.document, inputs.journeys, inputs.declarations];
    assess::run(plan, files, paths)?.document()
}

/// The document as JSON text: one result per line, the same bytes for the same document.
pub fn json(document: &Value) -> String {
    record::text(document)
}
