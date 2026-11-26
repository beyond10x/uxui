//! Promotion (`ux-search landing promote`): the variant a person chose from the frontier of a
//! recorded campaign is written to the page's two sources, with one `ratchet-evidence/1`
//! document that names the campaign.
//!
//! A promotion is refused, and writes nothing, when the id is no variant of the space, is the
//! page as shipped, is not in the measurements, breaks a hard rule or is not on the frontier
//! of those measurements; when the measurements are not a campaign over the space; when a
//! source is not the file the measurements were recorded on (they name each source's SHA-256);
//! and when the variant's patch does not apply. Everything is worked out before a file is
//! written, so the sources, the evidence and the shipped file change together or not at all.
//!
//! The evidence document (ratchet `docs/evidence-format.md`, revision 2) is written the way
//! `ux-browser checks` writes its own: compact JSON, one result per line. Its results are
//! exact, so it has no `run` member and no time: the same inputs give the same bytes. It names
//! no path; each file is identified by its digest.
//!
//! | field | value |
//! |---|---|
//! | `evidence.project`, `.assessment`, `.assessor` | [`PROJECT`], [`ASSESSMENT`], [`ASSESSOR`] |
//! | `evidence.candidate` | the two sources after the patch: SHA-256 over one line `<hex>  <name>` per source, `product.json` then `landing.css`, as `sha256sum` prints them |
//! | `evidence.target` | `{kind: document, class: local, selection: candidate, identifiers: {digest: <the candidate>, digest_form: sha256sum_lines}}` |
//! | `evidence.inputs` | `measurements`, `product`, `css`: each `{digest, digest_form: bytes}`, the sources as they were before; `variant`: the chosen id; `chosen_by`, when given |
//! | `evidence.conditions.tools` | `ux-search`, the campaign's adapter (`ux-browser`) with its version, `chromium`: the campaign's browser |
//! | `evidence.conditions.other` | `measurements_format`, `page`, `environments` |
//! | `evidence.coverage` | unit `variant`: the variants ranked; `passed` those that break no hard rule |
//! | `evidence.results[]` | for the chosen and for the shipped variant (`details.role`): [`RANK`] with `details.of` and `details.frontier`, then one number per term, `landing.<term>@1`; [`FRONTIER`] with the frontier's ids in `details.variants`; [`SOURCE`] per source: its digest after the patch, `details.before` and `details.patch`, the lines the report prints for the variant |

use std::path::{Path, PathBuf};

use ux_browser::value::{map, s};

use super::patch::Value;
use super::record::{self, Recorded};
use super::report::{self, Ranked};
use super::space::{self, Variant};

/// `evidence.project`: this repository.
pub const PROJECT: &str = "uxui";
/// `evidence.assessment`.
pub const ASSESSMENT: &str = "docs-landing-promotion";
/// `evidence.assessor`.
pub const ASSESSOR: &str = "ux-search@1";
/// The output of a variant's rank in the campaign, scope `variant`.
pub const RANK: &str = "landing.rank@1";
/// The output of the number of variants on the frontier, scope `run`.
pub const FRONTIER: &str = "landing.frontier_size@1";
/// The output of a source's digest after the patch, scope `source`.
pub const SOURCE: &str = "landing.source@1";
/// The form of the candidate's digest: SHA-256 over `sha256sum` lines of the sources.
pub const DIGEST_FORM: &str = "sha256sum_lines";

/// The unit of each term, in the order of [`report::TERMS`].
const UNITS: [&str; 6] = ["rules", "words", "px", "regions", "targets", "actions"];

/// The two sources as the measurements file and the evidence name them.
const PRODUCT: &str = "product.json";
const CSS: &str = "landing.css";

/// What to promote, and where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// The measurements file of the campaign the variant was chosen from.
    pub measurements: PathBuf,
    /// The id of the chosen variant.
    pub variant: String,
    /// `website/product.json`, rewritten in place.
    pub product: PathBuf,
    /// `website/src/css/landing.css`, rewritten in place.
    pub css: PathBuf,
    /// The evidence document to write.
    pub evidence: PathBuf,
    /// Who chose the variant, as the evidence is to say it.
    pub chosen_by: Option<String>,
    /// The file that names the shipped variant, rewritten to the chosen one.
    pub shipped_file: Option<PathBuf>,
}

/// A finished promotion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// The chosen variant's rank in the campaign.
    pub rank: usize,
    /// The variants of the campaign.
    pub variants: usize,
    /// The variants on the campaign's frontier.
    pub frontier: usize,
    /// The files written, in the order written.
    pub written: Vec<PathBuf>,
    /// What the promotion leaves to do by hand, one line each.
    pub notes: Vec<String>,
}

/// A promotion worked out from the texts of its inputs: nothing is written yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Planned {
    /// The chosen variant's rank in the campaign.
    pub rank: usize,
    /// The variants of the campaign.
    pub variants: usize,
    /// The variants on the campaign's frontier.
    pub frontier: usize,
    /// `product.json` with the variant's patch applied.
    pub product: String,
    /// `landing.css` with the variant's patch applied.
    pub css: String,
    /// The evidence document.
    pub evidence: String,
    /// What the promotion leaves to do by hand.
    pub notes: Vec<String>,
}

fn number(output: &str, item: Option<&str>, value: u64, unit: &str, details: Value) -> Value {
    let mut entries = vec![
        ("output", s(output)),
        ("kind", s("exact")),
        ("scope", s(if item.is_some() { "variant" } else { "run" })),
    ];
    if let Some(item) = item {
        entries.push(("item", s(item)));
    }
    entries.extend([
        ("value_type", s("number")),
        ("value", s(&value.to_string())),
        ("unit", s(unit)),
        ("details", details),
    ]);
    map(entries)
}

fn count(n: usize) -> Value {
    Value::Int(i64::try_from(n).unwrap_or(i64::MAX))
}

/// A file the promotion read, by its digest.
fn file(digest: &str) -> Value {
    map(vec![("digest", s(digest)), ("digest_form", s("bytes"))])
}

/// The digest of the two sources together: over one line `<hex>  <name>` per source.
fn candidate(product: &str, css: &str) -> String {
    let line =
        |digest: &str, name: &str| format!("{}  {name}\n", digest.trim_start_matches("sha256:"));
    record::digest((line(product, PRODUCT) + &line(css, CSS)).as_bytes())
}

/// What the evidence document says.
struct Facts<'a> {
    recorded: &'a Recorded,
    ranked: &'a Ranked,
    /// The digest of the measurements file.
    measurements: String,
    /// The chosen and the shipped variant, as indices of the record.
    chosen: usize,
    shipped: usize,
    /// The lines the report prints for the chosen variant's patch.
    patch: Vec<String>,
    /// The digests of `product.json` and `landing.css`, before and after the patch.
    before: (String, String),
    after: (String, String),
    chosen_by: Option<&'a str>,
}

fn rank_of(ranked: &Ranked, index: usize) -> usize {
    ranked
        .ranking
        .iter()
        .position(|&ranked| ranked == index)
        .map_or(0, |at| at + 1)
}

fn document(facts: &Facts) -> Value {
    let Facts {
        recorded, ranked, ..
    } = facts;
    let mut results = Vec::new();
    for (role, index) in [("chosen", facts.chosen), ("shipped", facts.shipped)] {
        let scored = &ranked.scored[index];
        let item = Some(scored.id.as_str());
        results.push(number(
            RANK,
            item,
            rank_of(ranked, index) as u64,
            "place",
            map(vec![
                ("role", s(role)),
                ("of", count(ranked.ranking.len())),
                ("frontier", Value::Bool(ranked.frontier.contains(&index))),
            ]),
        ));
        for ((term, unit), value) in report::TERMS.iter().zip(UNITS).zip(&scored.terms) {
            results.push(number(
                &format!("landing.{term}@1"),
                item,
                *value,
                unit,
                map(vec![("role", s(role))]),
            ));
        }
    }
    let frontier = ranked
        .frontier
        .iter()
        .map(|&index| s(&ranked.scored[index].id))
        .collect();
    results.push(number(
        FRONTIER,
        None,
        ranked.frontier.len() as u64,
        "variants",
        map(vec![("variants", Value::List(frontier))]),
    ));
    for (name, before, after) in [
        (PRODUCT, &facts.before.0, &facts.after.0),
        (CSS, &facts.before.1, &facts.after.1),
    ] {
        let lines = facts
            .patch
            .iter()
            .filter(|line| line.starts_with(&format!("{name} ")))
            .map(|line| s(line))
            .collect();
        results.push(map(vec![
            ("output", s(SOURCE)),
            ("kind", s("exact")),
            ("scope", s("source")),
            ("item", s(name)),
            ("value_type", s("digest")),
            ("digest", s(after)),
            (
                "details",
                map(vec![("before", s(before)), ("patch", Value::List(lines))]),
            ),
        ]));
    }

    let candidate = candidate(&facts.after.0, &facts.after.1);
    let mut inputs = vec![
        ("measurements", file(&facts.measurements)),
        (
            "variant",
            map(vec![("value", s(&ranked.scored[facts.chosen].id))]),
        ),
        ("product", file(&facts.before.0)),
        ("css", file(&facts.before.1)),
    ];
    if let Some(chosen_by) = facts.chosen_by {
        inputs.push(("chosen_by", map(vec![("value", s(chosen_by))])));
    }
    // The adapter as the record names it, `ux-browser 0.10.0`: the tool and its version.
    let (adapter, version) = recorded
        .adapter
        .split_once(' ')
        .unwrap_or(("adapter", recorded.adapter.as_str()));
    let environments: Vec<String> = record::ENVIRONMENTS
        .iter()
        .map(record::Environment::name)
        .collect();
    let sound = ranked
        .scored
        .iter()
        .filter(|scored| scored.terms[0] == 0)
        .count();
    let evidence = map(vec![
        ("project", s(PROJECT)),
        ("assessment", s(ASSESSMENT)),
        ("assessor", s(ASSESSOR)),
        ("candidate", s(&candidate)),
        (
            "target",
            map(vec![
                ("kind", s("document")),
                ("class", s("local")),
                ("selection", s("candidate")),
                (
                    "identifiers",
                    map(vec![
                        ("digest", s(&candidate)),
                        ("digest_form", s(DIGEST_FORM)),
                    ]),
                ),
            ]),
        ),
        ("inputs", map(inputs)),
        ("completion", s("completed")),
        (
            "conditions",
            map(vec![
                (
                    "tools",
                    map(vec![
                        ("ux-search", s(env!("CARGO_PKG_VERSION"))),
                        (adapter, s(version)),
                        ("chromium", s(&recorded.browser)),
                    ]),
                ),
                (
                    "other",
                    map(vec![
                        ("measurements_format", s(record::FORMAT)),
                        ("page", s(record::PAGE)),
                        ("environments", s(&environments.join(", "))),
                    ]),
                ),
            ]),
        ),
        (
            "coverage",
            map(vec![
                ("unit", s("variant")),
                ("executed", count(ranked.scored.len())),
                ("skipped", Value::Int(0)),
                ("not_run", Value::Int(0)),
                ("unsupported", Value::Int(0)),
                ("passed", count(sound)),
                ("failed", count(ranked.scored.len() - sound)),
            ]),
        ),
        ("results", Value::List(results)),
    ]);
    map(vec![
        ("format", s(ux_browser::evidence::FORMAT)),
        ("evidence", evidence),
    ])
}

/// Works a promotion out from the texts of the measurements file and the two sources. Every
/// refusal is decided here, before a file is written.
pub fn plan(
    measurements: &str,
    id: &str,
    product: &str,
    css: &str,
    chosen_by: Option<&str>,
) -> Result<Planned, String> {
    let Some(chosen) = space::find(id) else {
        return Err(format!("`{id}` is no variant of the space"));
    };
    if chosen.is_shipped() {
        return Err(format!(
            "`{id}` is the page as shipped: a promotion would change nothing"
        ));
    }
    let recorded =
        Recorded::parse(measurements).map_err(|why| format!("the measurements: {why}"))?;
    if !recorded.variants.iter().any(|(recorded, _)| recorded == id) {
        return Err(format!("`{id}` is not in the measurements"));
    }
    let variants = report::check(&recorded).map_err(|why| format!("the measurements: {why}"))?;
    let at = |variant: &Variant| {
        variants
            .iter()
            .position(|known| known == variant)
            .ok_or_else(|| format!("the space holds no variant `{}`", variant.id))
    };
    let (index, shipped) = (at(&chosen)?, at(&space::shipped())?);
    let ranked = report::rank(&recorded);
    let terms = &ranked.scored[index].terms;
    if terms[0] > 0 {
        return Err(format!(
            "`{id}` breaks a hard rule: {}; `hard` counts one per rule and environment",
            report::terms_text(terms)
        ));
    }
    let rank = rank_of(&ranked, index);
    if !ranked.frontier.contains(&index) {
        let beaten_by = ranked
            .ranking
            .iter()
            .map(|&other| &ranked.scored[other])
            .find(|other| other.terms[0] == 0 && crate::rank::dominates(&other.terms, terms))
            .map_or(String::new(), |other| {
                format!(
                    "; `{}` ({}) is no worse on any term and better on one",
                    other.id,
                    report::terms_text(&other.terms)
                )
            });
        return Err(format!(
            "`{id}` is not on the frontier of these measurements: rank {rank} of {}, {}{beaten_by}",
            ranked.ranking.len(),
            report::terms_text(terms)
        ));
    }
    let before = (
        record::digest(product.as_bytes()),
        record::digest(css.as_bytes()),
    );
    for (name, found, named) in [
        (PRODUCT, &before.0, &recorded.product_digest),
        (CSS, &before.1, &recorded.css_digest),
    ] {
        if found != named {
            return Err(format!(
                "{name} is not the source the measurements were recorded on: it is {found}, the \
                 measurements name {named}"
            ));
        }
    }
    let patch = chosen.patch();
    let (patched_product, patched_css) = patch.apply(product, css)?;
    let facts = Facts {
        recorded: &recorded,
        ranked: &ranked,
        measurements: record::digest(measurements.as_bytes()),
        chosen: index,
        shipped,
        patch: report::patch_lines(&patch),
        before,
        after: (
            record::digest(patched_product.as_bytes()),
            record::digest(patched_css.as_bytes()),
        ),
        chosen_by,
    };
    let mut notes = Vec::new();
    if !space::anchored(&chosen) {
        let values: Vec<&str> = space::ANCHOR.iter().map(|&(_, value)| value).collect();
        notes.push(format!(
            "the shipped page leaves {}: the operations on product.json and the palette rules \
             of engine/search/src/landing/space.rs are written for these values (`ANCHOR`); \
             write them for the new sources before the next campaign",
            values.join(", ")
        ));
    }
    Ok(Planned {
        rank,
        variants: ranked.ranking.len(),
        frontier: ranked.frontier.len(),
        product: patched_product,
        css: patched_css,
        evidence: ux_browser::evidence::text(&document(&facts)),
        notes,
    })
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
}

/// The file a text is written to before it takes its place: beside it, so the move is a rename.
fn staged(path: &Path) -> PathBuf {
    let mut name = std::ffi::OsString::from(".");
    name.push(path.file_name().unwrap_or_default());
    name.push(".promote.tmp");
    path.with_file_name(name)
}

/// Writes every file or none: each text goes to a file beside its place first, and only when
/// all of them are written does each take its place.
fn write_all(files: &[(&Path, &str)]) -> Result<(), String> {
    let stage = |(path, text): &(&Path, &str)| -> Result<(), String> {
        let failed = |error: std::io::Error| format!("{}: {error}", path.display());
        if path.file_name().is_none() {
            return Err(format!("{}: not a file", path.display()));
        }
        if let Some(directory) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(directory).map_err(failed)?;
        }
        std::fs::write(staged(path), text).map_err(failed)
    };
    if let Err(failed) = files.iter().try_for_each(stage) {
        for (path, _) in files {
            // Whatever was staged goes; a file that was not staged is not there.
            let _ = std::fs::remove_file(staged(path));
        }
        return Err(failed);
    }
    for (at, (path, _)) in files.iter().enumerate() {
        if let Err(error) = std::fs::rename(staged(path), path) {
            let placed: Vec<String> = files[..at]
                .iter()
                .map(|(path, _)| path.display().to_string())
                .collect();
            for (path, _) in &files[at..] {
                let _ = std::fs::remove_file(staged(path));
            }
            return Err(format!(
                "{}: {error}; already written: {}",
                path.display(),
                if placed.is_empty() {
                    "nothing".to_owned()
                } else {
                    placed.join(", ")
                }
            ));
        }
    }
    Ok(())
}

/// Promotes the variant: reads the measurements and the two sources, works the promotion out
/// ([`plan`]) and writes the sources, the evidence document and, when named, the shipped file.
pub fn promote(options: &Options) -> Result<Outcome, String> {
    let measurements = read(&options.measurements)?;
    let product = read(&options.product)?;
    let css = read(&options.css)?;
    let planned = plan(
        &measurements,
        &options.variant,
        &product,
        &css,
        options.chosen_by.as_deref(),
    )?;
    let shipped = format!("{}\n", options.variant);
    let mut files: Vec<(&Path, &str)> = vec![
        (&options.product, &planned.product),
        (&options.css, &planned.css),
        (&options.evidence, &planned.evidence),
    ];
    if let Some(path) = &options.shipped_file {
        files.push((path, &shipped));
    }
    for (at, (path, _)) in files.iter().enumerate() {
        let twice = files[..at].iter().any(|(other, _)| other == path);
        if twice || *path == options.measurements {
            return Err(format!(
                "{}: named for two of the files of a promotion",
                path.display()
            ));
        }
    }
    write_all(&files)?;
    Ok(Outcome {
        rank: planned.rank,
        variants: planned.variants,
        frontier: planned.frontier,
        written: files.iter().map(|(path, _)| path.to_path_buf()).collect(),
        notes: planned.notes,
    })
}
