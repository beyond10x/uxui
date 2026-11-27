//! The visual ratchet (`docs/browser-adapter.md`, Visual ratchet): with a `visual` section, a checks
//! run takes a screenshot of every page, viewport and mode (the viewport at scroll 0, and the full
//! page when declared), blanks the boxes of the mask selectors, and compares each shot pixel by
//! pixel with the baseline PNG of the same name. Two numbers per page, `visual.diff_share@1` and
//! `visual.diff_px@1`, a failed check and a finding for each unit over its tolerance, a diff image
//! of the changed pixels for the round report; the baseline is written only on `--visual update`.
//!
//! Layout of the directory `dir`: `baseline/` (the consumer's, committed), `shots/` (this run's),
//! `diff/` (this run's diff images); one file per unit, `<page>.<viewport>.<mode>.<shot>.png`.

use crate::checks::{Config, NamedViewport, PageResult};
use crate::png::{self, Image};
use crate::session::{Conn, Page};
use crate::value::{decimal, f64_of, get_strs, map, s};
use std::path::{Path, PathBuf};
use uxui_doc::yaml::Value;

/// The largest share of differing pixels over a page's units.
pub const DIFF_SHARE: &str = "visual.diff_share@1";
/// The number of differing pixels, summed over a page's units.
pub const DIFF_PX: &str = "visual.diff_px@1";
/// The in-page part: `freeze` (caret, animations, scroll, fonts; the mask boxes) and `thaw`.
pub const PROBE: &str = include_str!("probes/visual.js");
/// The colour a masked box is filled with in a shot.
pub const MASK: [u8; 4] = [255, 0, 255, 255];
/// The colour of a changed pixel in a diff image.
pub const MARK: [u8; 4] = [255, 0, 0, 255];
/// The shots of a unit: the viewport at scroll 0, then the full page when declared.
pub const SHOTS: [&str; 2] = ["viewport", "page"];

/// When two shots count as equal: a pixel is equal when no channel differs by more than
/// `threshold` (0 to 255); a unit passes when at most `share` (0 to 1) of its pixels differ.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tolerance {
    pub share: f64,
    pub threshold: u8,
}

/// A tolerance for the units of one page, one viewport, or both.
#[derive(Clone, Debug, PartialEq)]
pub struct ToleranceRule {
    pub page: Option<String>,
    pub viewport: Option<String>,
    pub tolerance: Tolerance,
}

/// The `visual` section of a checks configuration.
#[derive(Clone, Debug, PartialEq)]
pub struct Visual {
    /// The directory of `baseline/`, `shots/` and `diff/`; relative to `--root` as declared, made
    /// absolute by the command before the run.
    pub dir: PathBuf,
    /// Whether the full page is shot besides the viewport.
    pub full_page: bool,
    /// CSS selectors whose boxes are blanked in every shot.
    pub masks: Vec<String>,
    /// The tolerance of a unit no rule names.
    pub tolerance: Tolerance,
    /// The first rule that matches a unit's page and viewport gives its tolerance.
    pub tolerances: Vec<ToleranceRule>,
}

impl Visual {
    pub fn baseline_dir(&self) -> PathBuf {
        self.dir.join("baseline")
    }
    pub fn shots_dir(&self) -> PathBuf {
        self.dir.join("shots")
    }
    pub fn diff_dir(&self) -> PathBuf {
        self.dir.join("diff")
    }

    /// The shots of each unit.
    pub fn shots(&self) -> &'static [&'static str] {
        if self.full_page { &SHOTS } else { &SHOTS[..1] }
    }

    /// The tolerance of the units of `page` at `viewport`.
    pub fn tolerance_for(&self, page: &str, viewport: &str) -> Tolerance {
        self.tolerances
            .iter()
            .find(|r| {
                r.page.as_deref().is_none_or(|p| p == page)
                    && r.viewport.as_deref().is_none_or(|v| v == viewport)
            })
            .map_or(self.tolerance, |r| r.tolerance)
    }
}

fn only(v: &Value, keys: &[&str], what: &str) -> Result<(), String> {
    for (k, _) in v.entries() {
        if !keys.contains(&k.as_str()) {
            return Err(format!("{what}: unknown key `{k}`"));
        }
    }
    Ok(())
}

fn share_of(v: &Value, what: &str) -> Result<f64, String> {
    match f64_of(v) {
        Some(x) if (0.0..=1.0).contains(&x) => Ok(x),
        _ => Err(format!("{what}: `share` is a number from 0 to 1")),
    }
}

fn threshold_of(v: &Value, what: &str) -> Result<u8, String> {
    match v {
        Value::Int(n) if (0..=255).contains(n) => Ok(*n as u8),
        _ => Err(format!(
            "{what}: `threshold` is a whole number from 0 to 255"
        )),
    }
}

/// A tolerance's `share` and `threshold`, each defaulting to `default`'s.
fn tolerance_of(v: &Value, default: Tolerance, what: &str) -> Result<Tolerance, String> {
    Ok(Tolerance {
        share: v
            .get("share")
            .map_or(Ok(default.share), |x| share_of(x, what))?,
        threshold: v
            .get("threshold")
            .map_or(Ok(default.threshold), |x| threshold_of(x, what))?,
    })
}

/// The `visual` section, checked against the configuration's viewport names; `None` without one.
pub fn config(v: Option<&Value>, viewports: &[String]) -> Result<Option<Visual>, String> {
    let Some(v) = v else {
        return Ok(None);
    };
    if !matches!(v, Value::Map(_)) {
        return Err("`visual` is a map".to_string());
    }
    only(
        v,
        &["dir", "full_page", "masks", "tolerance", "tolerances"],
        "visual",
    )?;
    let dir = v
        .get_str("dir")
        .filter(|d| !d.is_empty())
        .ok_or("visual: no `dir`")?;
    let full_page = match v.get("full_page") {
        None => false,
        Some(Value::Bool(b)) => *b,
        Some(_) => return Err("visual: `full_page` is true or false".to_string()),
    };
    if v.get("masks").is_some_and(|m| !matches!(m, Value::List(_))) {
        return Err("visual: `masks` is a list of CSS selectors".to_string());
    }
    let masks = get_strs(v, "masks").map_err(|e| format!("visual: {e}"))?;
    for (i, m) in masks.iter().enumerate() {
        if m.trim().is_empty() {
            return Err("visual: `masks` holds an empty selector".to_string());
        }
        if masks[..i].contains(m) {
            return Err(format!("visual: mask `{m}` is named twice"));
        }
    }
    let none = Tolerance {
        share: 0.0,
        threshold: 0,
    };
    let tolerance = match v.get("tolerance") {
        None => none,
        Some(t) => {
            only(t, &["share", "threshold"], "visual.tolerance")?;
            tolerance_of(t, none, "visual.tolerance")?
        }
    };
    let mut tolerances = Vec::new();
    match v.get("tolerances") {
        None => {}
        Some(Value::List(items)) => {
            for (i, t) in items.iter().enumerate() {
                let what = format!("visual.tolerances[{i}]");
                only(t, &["page", "viewport", "share", "threshold"], &what)?;
                let page = t.get_str("page").map(str::to_string);
                let viewport = t.get_str("viewport").map(str::to_string);
                if page.is_none() && viewport.is_none() {
                    return Err(format!("{what}: names no `page` and no `viewport`"));
                }
                if let Some(p) = &page
                    && !p.starts_with('/')
                {
                    return Err(format!("{what}: page `{p}` does not start with `/`"));
                }
                if let Some(vp) = &viewport
                    && !viewports.contains(vp)
                {
                    return Err(format!("{what}: viewport `{vp}` is not declared"));
                }
                tolerances.push(ToleranceRule {
                    page,
                    viewport,
                    tolerance: tolerance_of(t, tolerance, &what)?,
                });
            }
        }
        Some(_) => return Err("visual: `tolerances` is a list".to_string()),
    }
    Ok(Some(Visual {
        dir: PathBuf::from(dir),
        full_page,
        masks,
        tolerance,
        tolerances,
    }))
}

// ---------------------------------------------------------------- units and shots

/// The file stem of a page path, the one of its captures and its shots: `home` for the base path,
/// else the path's words.
pub fn slug(path: &str, base_path: &str) -> String {
    let rest = path
        .strip_prefix(base_path)
        .unwrap_or(path)
        .trim_matches('/');
    if rest.is_empty() {
        return "home".to_string();
    }
    word(rest)
}

fn word(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

/// One screenshot a run takes and compares: a page at a viewport in a mode, `viewport` or `page`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unit {
    pub path: String,
    pub viewport: String,
    pub mode: String,
    pub shot: &'static str,
    /// `<page>.<viewport>.<mode>.<shot>.png`, the same in `baseline/`, `shots/` and `diff/`.
    pub file: String,
}

impl Unit {
    /// `<viewport>/<mode>/<shot>`, the unit's node path in the findings.
    pub fn at(&self) -> String {
        format!("{}/{}/{}", self.viewport, self.mode, self.shot)
    }
}

/// The units of `path`, in the order of the configuration's viewports and modes.
pub fn units(config: &Config, visual: &Visual, path: &str) -> Vec<Unit> {
    let stem = slug(path, &config.base_path);
    let mut out = Vec::new();
    for vp in &config.viewports {
        for mode in &config.modes {
            for shot in visual.shots() {
                out.push(Unit {
                    path: path.to_string(),
                    viewport: vp.name.clone(),
                    mode: mode.clone(),
                    shot,
                    file: format!("{stem}.{}.{mode}.{shot}.png", word(&vp.name)),
                });
            }
        }
    }
    out
}

/// Makes `shots/` and `diff/` and removes the PNG files a run before left in them, so that every
/// shot and diff image there is of this run. `baseline/` is not touched.
pub fn prepare(visual: &Visual) -> Result<(), String> {
    for dir in [visual.shots_dir(), visual.diff_dir()] {
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let entries = std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        for entry in entries {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path.is_file() && path.extension().is_some_and(|x| x == "png") {
                std::fs::remove_file(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            }
        }
    }
    Ok(())
}

/// Fills `boxes` ([left, top, right, bottom], CSS pixels, outward to whole pixels) with [`MASK`]
/// in the PNG `bytes`; the bytes as they were when no box falls in the image.
pub fn mask(bytes: &[u8], boxes: &[[f64; 4]]) -> Result<Vec<u8>, String> {
    if boxes.is_empty() {
        return Ok(bytes.to_vec());
    }
    let mut image = png::decode(bytes)?;
    let (w, h) = (f64::from(image.width), f64::from(image.height));
    let inside: Vec<&[f64; 4]> = boxes
        .iter()
        .filter(|b| b[0] < w && b[1] < h && b[2] > 0.0 && b[3] > 0.0)
        .collect();
    if inside.is_empty() {
        return Ok(bytes.to_vec());
    }
    for b in inside {
        image.fill(
            b[0].floor() as i64,
            b[1].floor() as i64,
            b[2].ceil() as i64,
            b[3].ceil() as i64,
            MASK,
        );
    }
    Ok(png::encode(&image))
}

/// Takes the shots of `path` at `vp` in `mode` on an open page and writes them to `shots/`: the
/// page is frozen (`probes/visual.js`), each shot taken and masked, the page thawed.
#[allow(clippy::too_many_arguments)]
pub fn shoot(
    conn: &mut Conn,
    page: &Page,
    config: &Config,
    visual: &Visual,
    path: &str,
    vp: &NamedViewport,
    mode: &str,
) -> Result<(), String> {
    let masks = Value::List(visual.masks.iter().map(|m| s(m)).collect());
    let frozen = page.run(conn, PROBE, &[s("freeze"), masks])?;
    let boxes: Vec<[f64; 4]> = frozen
        .get("boxes")
        .map(Value::list)
        .unwrap_or_default()
        .iter()
        .filter_map(|b| {
            let n: Vec<f64> = b.list().iter().filter_map(f64_of).collect();
            (n.len() == 4).then(|| [n[0], n[1], n[2], n[3]])
        })
        .collect();
    let height = frozen
        .get("height")
        .and_then(f64_of)
        .map_or(vp.height, |h| (h.ceil() as i64).max(vp.height));
    let shots = (|| {
        for unit in units(config, visual, path)
            .into_iter()
            .filter(|u| u.viewport == vp.name && u.mode == mode)
        {
            let png = if unit.shot == "viewport" {
                page.screenshot(conn)?
            } else {
                full_page(conn, page, vp.width, height)?
            };
            let bytes = mask(&png, &boxes)?;
            let file = visual.shots_dir().join(&unit.file);
            std::fs::write(&file, bytes).map_err(|e| format!("{}: {e}", file.display()))?;
        }
        Ok(())
    })();
    page.run(conn, PROBE, &[s("thaw"), Value::List(Vec::new())])?;
    shots
}

fn full_page(conn: &mut Conn, page: &Page, width: i64, height: i64) -> Result<Vec<u8>, String> {
    let clip = map(vec![
        ("x", Value::Int(0)),
        ("y", Value::Int(0)),
        ("width", Value::Int(width)),
        ("height", Value::Int(height)),
        ("scale", Value::Int(1)),
    ]);
    let r = page.call(
        conn,
        "Page.captureScreenshot",
        map(vec![
            ("format", s("png")),
            ("captureBeyondViewport", Value::Bool(true)),
            ("clip", clip),
        ]),
    )?;
    crate::base64::decode(r.get_str("data").ok_or("no screenshot data")?)
}

// ---------------------------------------------------------------- comparison

/// The pixels of `shot` that differ from `baseline` (two images of one size): a pixel differs when
/// one of its four channels differs by more than `threshold`. Returns their number and the diff
/// image: each differing pixel [`MARK`], every other one the baseline's grey, faded.
pub fn compare(shot: &Image, baseline: &Image, threshold: u8) -> (u64, Image) {
    let mut diff = Vec::with_capacity(baseline.rgba.len());
    let mut px = 0;
    for (a, b) in shot
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .zip(baseline.rgba.as_chunks::<4>().0)
    {
        if a.iter().zip(b).any(|(x, y)| x.abs_diff(*y) > threshold) {
            px += 1;
            diff.extend_from_slice(&MARK);
        } else {
            let lum =
                (u32::from(b[0]) * 299 + u32::from(b[1]) * 587 + u32::from(b[2]) * 114) / 1000;
            let grey = (255 - (255 - lum) / 4) as u8;
            diff.extend_from_slice(&[grey, grey, grey, 255]);
        }
    }
    (
        px,
        Image {
            width: baseline.width,
            height: baseline.height,
            rgba: diff,
        },
    )
}

/// A unit compared with its baseline.
#[derive(Clone, Debug, PartialEq)]
pub struct Compared {
    /// The shot's size.
    pub width: u32,
    pub height: u32,
    /// The differing pixels; for shots of two sizes, the pixels of the larger.
    pub px: u64,
    /// The pixels compared; for shots of two sizes, `px`.
    pub total: u64,
    /// For shots of two sizes: `size <w>x<h> against the baseline's <w>x<h>`.
    pub size: Option<String>,
    /// The SHA-256 of the baseline file and of the shot file.
    pub baseline: String,
    pub shot: String,
    /// The diff image's file name in `diff/`, written when a pixel differs.
    pub diff: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    /// Not compared: no baseline (or no shot), with the reason.
    NotRun(String),
    Compared(Compared),
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnitResult {
    pub unit: Unit,
    pub tolerance: Tolerance,
    pub outcome: Outcome,
}

impl UnitResult {
    /// The share of differing pixels in millionths, rounded up: a unit with one differing pixel
    /// never reads 0.
    pub fn share_micro(&self) -> Option<u64> {
        match &self.outcome {
            Outcome::Compared(c) if c.total > 0 => Some((c.px * 1_000_000).div_ceil(c.total)),
            Outcome::Compared(_) => Some(0),
            Outcome::NotRun(_) => None,
        }
    }

    /// Whether the unit is within its tolerance; `None` when it was not compared.
    pub fn passed(&self) -> Option<bool> {
        match &self.outcome {
            Outcome::Compared(c) => {
                Some(c.size.is_none() && c.px as f64 <= self.tolerance.share * c.total as f64)
            }
            Outcome::NotRun(_) => None,
        }
    }

    /// The reason of a failed unit's finding.
    pub fn reason(&self) -> Option<String> {
        let Outcome::Compared(c) = &self.outcome else {
            return None;
        };
        if self.passed() == Some(true) {
            return None;
        }
        Some(match &c.size {
            Some(size) => size.clone(),
            None => format!(
                "{} of {} pixels differ ({}), above the tolerance {} at threshold {}",
                c.px,
                c.total,
                micro(self.share_micro().unwrap_or(0)),
                decimal(self.tolerance.share, 6),
                self.tolerance.threshold
            ),
        })
    }
}

/// Millionths as a decimal.
fn micro(n: u64) -> String {
    decimal(n as f64 / 1_000_000.0, 6)
}

/// Why comparing failed: a baseline or shot that does not read (exit 1), or a diff image that
/// could not be written (exit 3).
#[derive(Debug, PartialEq)]
pub enum JudgeError {
    Refused(String),
    NotWritten(String),
}

/// Compares every unit's shot in `shots/` with its baseline in `baseline/`, and writes the diff
/// image of each unit with a differing pixel to `diff/`.
pub fn judge(visual: &Visual, units: &[Unit]) -> Result<Vec<UnitResult>, JudgeError> {
    let mut out = Vec::new();
    for unit in units {
        let tolerance = visual.tolerance_for(&unit.path, &unit.viewport);
        let shot_file = visual.shots_dir().join(&unit.file);
        let base_file = visual.baseline_dir().join(&unit.file);
        let outcome = match (std::fs::read(&shot_file), std::fs::read(&base_file)) {
            (Err(_), _) => Outcome::NotRun(format!("no shot shots/{}", unit.file)),
            (_, Err(_)) => Outcome::NotRun(format!("no baseline baseline/{}", unit.file)),
            (Ok(shot_bytes), Ok(base_bytes)) => {
                let read = |bytes: &[u8], file: &Path| {
                    png::decode(bytes)
                        .map_err(|e| JudgeError::Refused(format!("{}: {e}", file.display())))
                };
                let shot = read(&shot_bytes, &shot_file)?;
                let base = read(&base_bytes, &base_file)?;
                let digests = (
                    uxui_doc::sha256::hex(&base_bytes),
                    uxui_doc::sha256::hex(&shot_bytes),
                );
                if (shot.width, shot.height) != (base.width, base.height) {
                    let area = |i: &Image| u64::from(i.width) * u64::from(i.height);
                    let px = area(&shot).max(area(&base));
                    Outcome::Compared(Compared {
                        width: shot.width,
                        height: shot.height,
                        px,
                        total: px,
                        size: Some(format!(
                            "size {}x{} against the baseline's {}x{}",
                            shot.width, shot.height, base.width, base.height
                        )),
                        baseline: digests.0,
                        shot: digests.1,
                        diff: None,
                    })
                } else {
                    let (px, diff_image) = compare(&shot, &base, tolerance.threshold);
                    let diff = if px > 0 {
                        let file = visual.diff_dir().join(&unit.file);
                        std::fs::create_dir_all(visual.diff_dir())
                            .and_then(|()| std::fs::write(&file, png::encode(&diff_image)))
                            .map_err(|e| {
                                JudgeError::NotWritten(format!("{}: {e}", file.display()))
                            })?;
                        Some(unit.file.clone())
                    } else {
                        None
                    };
                    Outcome::Compared(Compared {
                        width: shot.width,
                        height: shot.height,
                        px,
                        total: u64::from(shot.width) * u64::from(shot.height),
                        size: None,
                        baseline: digests.0,
                        shot: digests.1,
                        diff,
                    })
                }
            }
        };
        out.push(UnitResult {
            unit: unit.clone(),
            tolerance,
            outcome,
        });
    }
    Ok(out)
}

/// Writes the shot of every unit that has one as its new baseline; returns how many. Baseline
/// files of units this run did not take stay.
pub fn update_baseline(visual: &Visual, units: &[Unit]) -> Result<usize, String> {
    let dir = visual.baseline_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut written = 0;
    for unit in units {
        let shot = visual.shots_dir().join(&unit.file);
        let Ok(bytes) = std::fs::read(&shot) else {
            continue;
        };
        let file = dir.join(&unit.file);
        std::fs::write(&file, bytes).map_err(|e| format!("{}: {e}", file.display()))?;
        written += 1;
    }
    Ok(written)
}

// ---------------------------------------------------------------- evidence

fn number(
    output: &str,
    item: &str,
    value: Option<(String, bool)>,
    unit: &str,
    incomplete: Option<Value>,
    details: Option<Value>,
) -> Value {
    let mut e = vec![
        ("output", s(output)),
        ("kind", s("exact")),
        ("scope", s(crate::evidence::SCOPE_PAGE)),
        ("item", s(item)),
        ("value_type", s("number")),
    ];
    if let Some((v, at_least)) = value {
        e.push(("value", s(&v)));
        if at_least {
            e.push(("at_least", Value::Bool(true)));
        }
    }
    e.push(("unit", s(unit)));
    if let Some(i) = incomplete {
        e.push(("incomplete", i));
    }
    if let Some(d) = details {
        e.push(("details", d));
    }
    map(e)
}

fn incomplete(cause: &str, detail: &str) -> Value {
    map(vec![("cause", s(cause)), ("detail", s(detail))])
}

fn unit_detail(r: &UnitResult) -> Value {
    let u = &r.unit;
    let mut e = vec![
        ("viewport", s(&u.viewport)),
        ("mode", s(&u.mode)),
        ("shot", s(u.shot)),
    ];
    match &r.outcome {
        Outcome::NotRun(reason) => {
            e.push(("result", s("not_run")));
            e.push(("reason", s(reason)));
        }
        Outcome::Compared(c) => {
            let passed = r.passed() == Some(true);
            e.push(("result", s(if passed { "passed" } else { "failed" })));
            e.push(("width", Value::Int(i64::from(c.width))));
            e.push(("height", Value::Int(i64::from(c.height))));
            e.push(("px", Value::Int(c.px as i64)));
            e.push(("share", s(&micro(r.share_micro().unwrap_or(0)))));
            e.push(("tolerance", s(&decimal(r.tolerance.share, 6))));
            e.push(("threshold", Value::Int(i64::from(r.tolerance.threshold))));
            e.push(("baseline", s(&c.baseline)));
            e.push(("shot_digest", s(&c.shot)));
            if let Some(d) = &c.diff {
                e.push(("diff", s(&format!("diff/{d}"))));
            }
            if let Some(size) = &c.size {
                e.push(("reason", s(size)));
            }
        }
    }
    map(e)
}

/// The two numbers of one page, from its units' results; the not-run units and the findings go
/// to `not_run` and `findings`.
fn page_numbers(
    path: &str,
    results: &[&UnitResult],
    not_run: &mut Vec<(String, String)>,
    findings: &mut Vec<Value>,
    counts: &mut [i64; 4],
) -> Vec<Value> {
    let mut px_total = 0u64;
    let mut share_max: Option<u64> = None;
    let mut px_by: Vec<(String, u64)> = Vec::new();
    let mut share_by: Vec<(String, u64)> = Vec::new();
    let mut skipped = Vec::new();
    for r in results {
        let u = &r.unit;
        match (&r.outcome, r.passed(), r.share_micro()) {
            (Outcome::Compared(c), Some(passed), Some(share)) => {
                counts[0] += 1;
                counts[if passed { 2 } else { 3 }] += 1;
                px_total += c.px;
                share_max = Some(share_max.map_or(share, |m| m.max(share)));
                match px_by.iter_mut().find(|(v, _)| *v == u.viewport) {
                    Some((_, n)) => *n += c.px,
                    None => px_by.push((u.viewport.clone(), c.px)),
                }
                match share_by.iter_mut().find(|(v, _)| *v == u.viewport) {
                    Some((_, n)) => *n = (*n).max(share),
                    None => share_by.push((u.viewport.clone(), share)),
                }
                if let Some(reason) = r.reason() {
                    findings.push(map(vec![
                        (
                            "identity",
                            map(vec![("rule", s(DIFF_SHARE)), ("node_path", s(&u.at()))]),
                        ),
                        ("fields", map(vec![("reason", s(&reason))])),
                    ]));
                }
            }
            (Outcome::NotRun(reason), _, _) => {
                counts[1] += 1;
                not_run.push((
                    format!("{path}/{}/{}/visual/{}", u.viewport, u.mode, u.shot),
                    reason.clone(),
                ));
                skipped.push(format!("{}: {reason}", u.at()));
            }
            _ => {}
        }
    }
    let complete = skipped.is_empty() && share_max.is_some();
    let why = (!complete).then(|| {
        incomplete(
            "coverage_below_plan",
            &if skipped.is_empty() {
                "no unit compared".to_string()
            } else {
                format!("not compared: {}", skipped.join("; "))
            },
        )
    });
    // Incomplete: a value above 0 is a lower bound; anything else has no value.
    let value = |v: Option<u64>, text: &dyn Fn(u64) -> String| match v {
        Some(n) if complete => Some((text(n), false)),
        Some(n) if n > 0 => Some((text(n), true)),
        _ => None,
    };
    let viewports = |by: &[(String, u64)], text: &dyn Fn(u64) -> String| {
        Value::Map(by.iter().map(|(v, n)| (v.clone(), s(&text(*n)))).collect())
    };
    let whole = |n: u64| n.to_string();
    let units: Vec<Value> = results.iter().map(|r| unit_detail(r)).collect();
    vec![
        number(
            DIFF_SHARE,
            path,
            value(share_max, &micro),
            "ratio",
            why.clone(),
            Some(map(vec![
                ("viewports", viewports(&share_by, &micro)),
                ("units", Value::List(units)),
            ])),
        ),
        number(
            DIFF_PX,
            path,
            value((share_max.is_some()).then_some(px_total), &whole),
            "px",
            why,
            Some(map(vec![("viewports", viewports(&px_by, &whole))])),
        ),
    ]
}

fn entry_mut<'a>(v: &'a mut Value, key: &str) -> Option<&'a mut Value> {
    match v {
        Value::Map(entries) => entries.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v),
        _ => None,
    }
}

/// The document of a checks run with the visual results: per page its two numbers, after the
/// numbers of the pages and journeys and before the findings sets; each failed unit a finding in
/// its page's set (rule `visual.diff_share@1`, node path `<viewport>/<mode>/<shot>`); each unit one
/// check in the coverage, a unit not compared a `not_run` entry. A page that could not be measured
/// has its two numbers incomplete with the page's error, as its other numbers; its units count
/// nothing (the page is one `not_run` entry already).
pub fn with_visual(mut document: Value, pages: &[PageResult], results: &[UnitResult]) -> Value {
    let mut numbers = Vec::new();
    let mut not_run = Vec::new();
    let mut findings: Vec<(String, Vec<Value>)> = Vec::new();
    // executed, not_run, passed, failed
    let mut counts = [0i64; 4];
    for p in pages {
        if let Some(error) = &p.error {
            let why = incomplete("incompatible_target", error);
            numbers.push(number(
                DIFF_SHARE,
                &p.path,
                None,
                "ratio",
                Some(why.clone()),
                None,
            ));
            numbers.push(number(DIFF_PX, &p.path, None, "px", Some(why), None));
            continue;
        }
        let mine: Vec<&UnitResult> = results.iter().filter(|r| r.unit.path == p.path).collect();
        let mut found = Vec::new();
        numbers.extend(page_numbers(
            &p.path,
            &mine,
            &mut not_run,
            &mut found,
            &mut counts,
        ));
        if !found.is_empty() {
            findings.push((p.path.clone(), found));
        }
    }
    let Some(evidence) = entry_mut(&mut document, "evidence") else {
        return document;
    };
    if let Some(Value::List(list)) = entry_mut(evidence, "results") {
        let at = list
            .iter()
            .position(|r| r.get_str("value_type") == Some("identity_set"))
            .unwrap_or(list.len());
        list.splice(at..at, numbers);
        for (path, found) in findings {
            let set = list.iter_mut().find(|r| {
                r.get_str("output") == Some(crate::evidence::FINDINGS)
                    && r.get_str("scope") == Some(crate::evidence::SCOPE_PAGE)
                    && r.get_str("item") == Some(path.as_str())
            });
            if let Some(Value::List(items)) = set.and_then(|s| entry_mut(s, "items")) {
                items.extend(found);
            }
        }
    }
    if let Some(Value::Map(coverage)) = entry_mut(evidence, "coverage") {
        for (key, n) in [
            ("executed", counts[0]),
            ("not_run", counts[1]),
            ("passed", counts[2]),
            ("failed", counts[3]),
        ] {
            if let Some((_, Value::Int(v))) = coverage.iter_mut().find(|(k, _)| k == key) {
                *v += n;
            }
        }
        if !not_run.is_empty() {
            let entries = not_run.into_iter().map(|(id, reason)| {
                map(vec![
                    ("id", s(&id)),
                    ("state", s("not_run")),
                    ("reason", s(&reason)),
                ])
            });
            match coverage.iter_mut().find(|(k, _)| k == "not_executed") {
                Some((_, Value::List(list))) => list.extend(entries),
                _ => coverage.push(("not_executed".to_string(), Value::List(entries.collect()))),
            }
        }
    }
    document
}
