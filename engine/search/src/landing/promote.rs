//! Promotion (`ux-search landing promote`): the variant a person chose from the frontier of a
//! recorded campaign is written to the page's two sources, with one `ratchet-evidence/1`
//! document that names the campaign.
//!
//! A promotion is refused, and writes nothing, when the id is no variant of the space, is the
//! page as shipped, is not in the measurements, breaks a hard rule or is not on the frontier
//! of those measurements; when the measurements are not a campaign over the space; when a
//! source is not the file the measurements were recorded on (they name each source's SHA-256);
//! and when the variant's patch does not apply. It is refused as well when a place it is to
//! write is a directory or has no file name, and when two of its files are one file, by
//! whatever spelling (`..`, a link). All of that is decided before a file is written.
//!
//! A promotion writes five files, all of them or none: the two sources, the evidence document,
//! a copy of the measurements beside it ([`measurements_copy`]) and, when named, the shipped
//! file. Each text is staged beside its place and moved there when all are staged. When a move
//! fails, the files already moved are put back as they were (one that was not there is
//! removed), and the staged files and the directories made for the promotion are removed.
//! Left as they are: a place that is a link is replaced by a file, and a rewritten file gets
//! the mode of a new file.
//!
//! The copy of the measurements is what the evidence names: the campaign's own file is
//! replaced when the campaign is recorded again on the promoted sources, the copy stays.
//!
//! The evidence document (ratchet `docs/evidence-format.md`, revision 2) is written the way
//! `ux-browser checks` writes its own: compact JSON, one result per line. Its results are
//! exact, so it has no `run` member and no time: the same inputs give the same bytes. A file
//! is named by its path from the root (the working directory of the command, the repository's
//! root when run there), with `/`, however the path was written; a file outside the root has
//! no path, as in `ux-static` (`docs/ux-static.md`): its digest identifies it. So the document
//! holds no absolute path.
//!
//! | field | value |
//! |---|---|
//! | `evidence.project`, `.assessment`, `.assessor` | [`PROJECT`], [`ASSESSMENT`], [`ASSESSOR`] |
//! | `evidence.candidate` | the two sources after the patch: SHA-256 over one line `<hex>  <name>` per source, `product.json` then `landing.css`, as `sha256sum` prints them |
//! | `evidence.target` | `{kind: document, class: local, selection: candidate, identifiers: {digest: <the candidate>, digest_form: sha256sum_lines}}` |
//! | `evidence.inputs` | `measurements`, `product`, `css`: each `{value: <path>, digest, digest_form: bytes}`, no `value` for a file outside the root; `measurements` is the copy beside the evidence, the sources are as they were before; `variant`: the chosen id; `chosen_by`, when given |
//! | `evidence.conditions.tools` | `ux-search`, the campaign's adapter (`ux-browser`) with its version, `chromium`: the campaign's browser |
//! | `evidence.conditions.other` | `measurements_format`, `page`, `environments` |
//! | `evidence.coverage` | unit `variant`: the variants ranked; `passed` those that break no hard rule |
//! | `evidence.results[]` | for the chosen and for the shipped variant (`details.role`): [`RANK`] with `details.of` and `details.frontier`, then one number per term, `landing.<term>@1`; [`FRONTIER`] with the frontier's ids in `details.variants`; [`SOURCE`] per source: its digest after the patch, `details.before` and `details.patch`, the lines the report prints for the variant |
//!
//! The declaration `ratchet-assessor/1` of [`ASSESSOR`] is `engine/search/ux-search.assessor.yaml`.

use std::path::{Component, Path, PathBuf};

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

/// The file of the repository that names the shipped variant, from the repository's root.
pub const SHIPPED_FILE: &str = "engine/search/src/landing/shipped.txt";

/// The move of a staged file to its place: `std::fs::rename` in [`promote`]. A test passes one
/// that fails.
pub type Place<'a> = dyn FnMut(&Path, &Path) -> std::io::Result<()> + 'a;

/// The files the evidence names, each by its path from the root with `/`; none for a file
/// outside the root. `measurements` is the copy kept beside the evidence.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Names {
    pub measurements: Option<String>,
    pub product: Option<String>,
    pub css: Option<String>,
}

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
    /// The files written, in the order written: `product.json`, `landing.css`, the evidence,
    /// the copy of the measurements and, when named, the shipped file.
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

/// A file the promotion read: its path from the root, when it lies under it, and its digest.
fn file(named: Option<&str>, digest: &str) -> Value {
    let mut entries = Vec::new();
    if let Some(named) = named {
        entries.push(("value", s(named)));
    }
    entries.extend([("digest", s(digest)), ("digest_form", s("bytes"))]);
    map(entries)
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
    /// The paths the document names the files by.
    names: &'a Names,
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
    let names = facts.names;
    let mut inputs = vec![
        (
            "measurements",
            file(names.measurements.as_deref(), &facts.measurements),
        ),
        (
            "variant",
            map(vec![("value", s(&ranked.scored[facts.chosen].id))]),
        ),
        ("product", file(names.product.as_deref(), &facts.before.0)),
        ("css", file(names.css.as_deref(), &facts.before.1)),
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

/// Works a promotion out from the texts of the measurements file and the two sources, and the
/// paths the evidence is to name them by. Every refusal that the texts decide is decided here,
/// before a file is written.
pub fn plan(
    measurements: &str,
    id: &str,
    product: &str,
    css: &str,
    names: &Names,
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
        names,
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

/// Where a promotion keeps the measurements it read: beside the evidence document, under its
/// name with `.measurements.json` in the place of `.evidence.json` (of `.json`, for another
/// name; added to a name without it). None for a path without a file name.
pub fn measurements_copy(evidence: &Path) -> Option<PathBuf> {
    let name = evidence.file_name()?.to_string_lossy();
    let stem = name
        .strip_suffix(".evidence.json")
        .or_else(|| name.strip_suffix(".json"))
        .unwrap_or(&name);
    Some(evidence.with_file_name(format!("{stem}.measurements.json")))
}

/// A path as the file system resolves it, also when the file or its directories are not there
/// yet: the part that exists is resolved (`.`, `..`, links), the rest is taken as written,
/// `..` stepping back. Two spellings of one place give one path.
fn canonical(path: &Path) -> Result<PathBuf, String> {
    let mut resolved = if path.is_absolute() {
        PathBuf::new()
    } else {
        std::env::current_dir()
            .and_then(std::fs::canonicalize)
            .map_err(|error| format!("the working directory: {error}"))?
    };
    let mut exists = true;
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir if !exists => {
                resolved.pop();
            }
            part => {
                resolved.push(part);
                if exists {
                    match std::fs::canonicalize(&resolved) {
                        Ok(real) => resolved = real,
                        Err(_) => exists = false,
                    }
                }
            }
        }
    }
    Ok(resolved)
}

/// A file's path from the root, with `/`, both as [`canonical`] gives them; none for a file
/// that does not lie under the root.
fn named(root: &Path, path: &Path) -> Option<String> {
    let parts: Vec<String> = path
        .strip_prefix(root)
        .ok()?
        .components()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect();
    (!parts.is_empty()).then(|| parts.join("/"))
}

/// The file a text is written to before it takes its place: beside it, so the move is a rename.
fn staged(path: &Path) -> PathBuf {
    let mut name = std::ffi::OsString::from(".");
    name.push(path.file_name().unwrap_or_default());
    name.push(".promote.tmp");
    path.with_file_name(name)
}

/// What a promotion made beside the files themselves: to take away when it fails.
#[derive(Default)]
struct Made {
    /// The directories made, the outermost first.
    directories: Vec<PathBuf>,
    /// The staged files.
    staged: Vec<PathBuf>,
}

impl Made {
    /// Removes the staged files that are still there, then the directories, the innermost
    /// first. A directory that holds something else by now stays.
    fn remove(&self) {
        for path in &self.staged {
            let _ = std::fs::remove_file(path);
        }
        for directory in self.directories.iter().rev() {
            let _ = std::fs::remove_dir(directory);
        }
    }
}

/// Stages one text beside its place, making the directories that are not there.
fn stage(path: &Path, text: &str, made: &mut Made) -> std::io::Result<()> {
    let mut missing = Vec::new();
    let mut at = path.parent();
    while let Some(directory) = at.filter(|directory| !directory.as_os_str().is_empty()) {
        if directory.exists() {
            break;
        }
        missing.push(directory.to_path_buf());
        at = directory.parent();
    }
    for directory in missing.into_iter().rev() {
        match std::fs::create_dir(&directory) {
            Ok(()) => made.directories.push(directory),
            // A directory spelled twice (`new/other/..` after `new`) is there by now.
            Err(error)
                if error.kind() == std::io::ErrorKind::AlreadyExists && directory.is_dir() => {}
            Err(error) => return Err(error),
        }
    }
    let staged = staged(path);
    made.staged.push(staged.clone());
    std::fs::write(staged, text)
}

/// Writes every file or none. Each text is staged beside its place, and only when all are
/// staged does each take its place, through `place`. When a move fails, the files moved before
/// it get back the bytes they had (a file that was not there is removed), and the staged files
/// and the directories made here are removed; the failure says so, or names what could not be
/// put back.
fn write_all(files: &[(&Path, &str)], place: &mut Place<'_>) -> Result<(), String> {
    let failed = |path: &Path, error: std::io::Error| format!("{}: {error}", path.display());
    let mut before = Vec::new();
    for (path, _) in files {
        before.push(match std::fs::read(path) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(failed(path, error)),
        });
    }
    let mut made = Made::default();
    for (path, text) in files {
        if let Err(error) = stage(path, text, &mut made) {
            made.remove();
            return Err(failed(path, error));
        }
    }
    for (at, (path, _)) in files.iter().enumerate() {
        let Err(error) = place(&staged(path), path) else {
            continue;
        };
        let mut left = Vec::new();
        for ((moved, _), was) in files[..at].iter().zip(&before) {
            let put_back = match was {
                Some(bytes) => std::fs::write(moved, bytes),
                None => std::fs::remove_file(moved),
            };
            if let Err(error) = put_back {
                left.push(failed(moved, error));
            }
        }
        made.remove();
        return Err(if left.is_empty() {
            format!(
                "{}; nothing is changed: the files already in place were put back",
                failed(path, error)
            )
        } else {
            format!(
                "{}; could not put back: {}",
                failed(path, error),
                left.join("; ")
            )
        });
    }
    Ok(())
}

/// Promotes the variant, the files of the evidence named from the working directory: reads the
/// measurements and the two sources, works the promotion out ([`plan`]) and writes the sources,
/// the evidence document, the copy of the measurements beside it and, when named, the shipped
/// file.
pub fn promote(options: &Options) -> Result<Outcome, String> {
    promote_in(Path::new("."), options, &mut |from, to| {
        std::fs::rename(from, to)
    })
}

/// [`promote`] with the root the evidence names its files from, and the move that puts a
/// staged file in its place.
pub fn promote_in(
    root: &Path,
    options: &Options,
    place: &mut Place<'_>,
) -> Result<Outcome, String> {
    // The places first: nothing is read for a promotion that has nowhere to go.
    let not_a_file = |path: &Path| format!("{}: not a file", path.display());
    let kept = measurements_copy(&options.evidence).ok_or_else(|| not_a_file(&options.evidence))?;
    let mut targets: Vec<(&str, &Path)> = vec![
        ("--product", &options.product),
        ("--css", &options.css),
        ("--evidence", &options.evidence),
        ("the copy of the measurements beside the evidence", &kept),
    ];
    if let Some(path) = &options.shipped_file {
        targets.push(("--shipped-file", path));
    }
    for (_, path) in &targets {
        if path.file_name().is_none() {
            return Err(not_a_file(path));
        }
        if path.is_dir() {
            return Err(format!("{}: is a directory, not a file", path.display()));
        }
    }
    let mut places = vec![(
        "--measurements",
        options.measurements.as_path(),
        canonical(&options.measurements)?,
    )];
    for (role, path) in &targets {
        places.push((*role, *path, canonical(path)?));
    }
    for (at, (role, path, resolved)) in places.iter().enumerate() {
        let twice = places[..at]
            .iter()
            .find(|(_, _, earlier)| earlier == resolved);
        if let Some((other, _, _)) = twice {
            return Err(format!(
                "{}: named for two of the files of a promotion ({other} and {role})",
                path.display()
            ));
        }
    }
    let root = canonical(root)?;
    let names = Names {
        measurements: named(&root, &places[4].2),
        product: named(&root, &places[1].2),
        css: named(&root, &places[2].2),
    };

    let measurements = read(&options.measurements)?;
    let product = read(&options.product)?;
    let css = read(&options.css)?;
    let planned = plan(
        &measurements,
        &options.variant,
        &product,
        &css,
        &names,
        options.chosen_by.as_deref(),
    )?;
    let shipped = format!("{}\n", options.variant);
    let mut files: Vec<(&Path, &str)> = vec![
        (&options.product, &planned.product),
        (&options.css, &planned.css),
        (&options.evidence, &planned.evidence),
        (&kept, &measurements),
    ];
    if let Some(path) = &options.shipped_file {
        files.push((path, &shipped));
    }
    write_all(&files, place)?;
    let mut notes = planned.notes;
    if options.shipped_file.is_none() {
        notes.push(format!(
            "the shipped file was not changed (no --shipped-file): {SHIPPED_FILE} still names \
             `{}` as the shipped variant",
            space::shipped().id
        ));
    }
    Ok(Outcome {
        rank: planned.rank,
        variants: planned.variants,
        frontier: planned.frontier,
        written: files.iter().map(|(path, _)| path.to_path_buf()).collect(),
        notes,
    })
}
