//! A declared campaign in Chromium, through the browser adapter (`ux_browser`).
//!
//! One browser, one page per environment. The page is the declared page of a server somebody
//! else started (`--url`), loaded with its own scripts on and measured once its ready selector
//! matches. In it the probe (`probes/campaign.js`) renders each variant by injecting the
//! variant's style sheet of custom properties after the page's own, and measures it. So V
//! variants in E environments are V x E renders in E page loads.
//!
//! What keeps a render honest:
//!
//! - before the first variant is rendered, each mode with an attribute or an expectation is
//!   read back on one page load, so a mode the page does not take is refused at once and not
//!   after the renders of the modes before it;
//! - the page is first measured with no sheet injected; the variant of the shipped values,
//!   measured with its sheet (an empty one when no shipped value has a rule), must give the
//!   same numbers, in every environment;
//! - the first variant is measured again after the last and must give the same numbers, so no
//!   render leaves a trace in the next and the page does not change while it is measured;
//! - every property a sheet sets is read back from every element its selector matches: a
//!   selector that matches nothing and a value another rule of the page overrides are refused
//!   with the key of the declaration that set it;
//! - each frontier variant is measured once more on a page loaded anew, in the environments of
//!   its screenshots, and must measure there what it measured in the campaign;
//! - a mode's attribute is set before the page's scripts run and again when the page is ready,
//!   and must still be there at every measurement;
//! - what a mode expects of the page (`expect`: an attribute of the root element, a selector)
//!   is read back at every measurement, and a miss is refused: a query the page does not know
//!   leaves it in its default mode, and the numbers would be those of that mode twice;
//! - the pointer is parked before every measurement and before every screenshot, at
//!   `page.pointer` or in the bottom-right corner of the viewport, by a real mouse move: a
//!   headless pointer rests at 0, 0 otherwise, and a page with something there that opens on
//!   hover is measured open.
//!
//! What the run says beside that: a mode that is set by a query and expects nothing is named
//! as not read back; per axis the number of values that change a box or a computed style on
//! the page at all, and an axis none of whose values does is named as one that changes nothing
//! here; and a mode or a viewport whose renders measure the same numbers as another one's for
//! every variant is named, one line each.
//!
//! A screenshot is taken with the caret hidden: a field the page focused blinks, and two runs
//! would write different bytes.
//!
//! A run restricted to one variant (`--variant`) or one environment (`--environment`) measures
//! those renders with the same checks, loads each anew, shoots it with `--shots`, and writes the
//! renders as `ux-search-campaign-renders/1`, which no report ranks.
//!
//! The page reaches nothing but its own origin, with `GET` and `HEAD` only
//! (`ux_browser::cdp::Guard`). A request the guard failed fails the run and is named: a page
//! that was denied something is not the page a visitor sees. An app that reads its fixtures
//! from another origin or with another method is served so that it does not, or is not
//! measured.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use ux_browser::cdp::{Connection, Guard};
use ux_browser::render::Viewport;
use ux_browser::session::{Browser, Conn, Page, Setup};
use ux_browser::target::Site;
use ux_browser::ws::WebSocket;

use super::declaration::{Declaration, Environment, Mode};
use super::events::{self, Events, Progress};
use super::record::{self, NOTED, Note, Recorded, Render, Renders};
use super::report;
use crate::landing::patch::{self, Value};

/// The probe, evaluated once in each page.
pub const PROBE: &str = include_str!("../probes/campaign.js");

/// What to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// The declaration file.
    pub declaration: PathBuf,
    /// The server the page is on.
    pub site: Site,
    /// The measurements file to write.
    pub out: PathBuf,
    /// The directory the frontier's screenshots are written to; `None`: no screenshot.
    pub shots: Option<PathBuf>,
    /// The directory the browser's profile is made in and removed from.
    pub profile: PathBuf,
    /// How many pages are measured at once.
    pub jobs: usize,
    /// The one variant to measure, by id; `None`: every variant.
    pub variant: Option<String>,
    /// The one environment to measure, by name (`800x600 dark`); `None`: every environment.
    pub environment: Option<String>,
    /// The directory of the live view (`events`): the candidates file and the phase file, and
    /// with `shots` a screenshot of each variant as it is measured; `None`: no live view.
    pub events: Option<PathBuf>,
}

/// What a run measured: a campaign, every variant in every environment, or the renders of a run
/// restricted to one variant or one environment, which no report ranks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Measured {
    Campaign(Recorded),
    Restricted(Renders),
}

impl Measured {
    /// The browser's product.
    pub fn browser(&self) -> &str {
        match self {
            Measured::Campaign(recorded) => &recorded.browser,
            Measured::Restricted(renders) => &renders.browser,
        }
    }

    /// The file `--out` names.
    pub fn text(&self) -> String {
        match self {
            Measured::Campaign(recorded) => recorded.text(),
            Measured::Restricted(renders) => renders.text(),
        }
    }

    /// What the run prints: the report of a campaign, the renders of a restricted run.
    pub fn printed(&self) -> Result<String, String> {
        match self {
            Measured::Campaign(recorded) => report::report(recorded),
            Measured::Restricted(renders) => Ok(report::renders(renders)),
        }
    }
}

/// A finished run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// What was measured.
    pub measured: Measured,
    /// The files written, in the order written.
    pub written: Vec<PathBuf>,
    /// The renders measured for the record.
    pub renders: usize,
    /// What broke a declared hard rule, one line per environment and finding, with the number
    /// of variants it was found in and the first of them.
    pub notes: Vec<String>,
    /// The checks the run made beside the record, one line each.
    pub checks: Vec<String>,
    /// What the run says about the declaration on this page, one line each: `note: ...` for a
    /// mode that is not read back, for an axis that changes nothing and for a mode or a
    /// viewport that measures what another one measures, `axis ...` for every other axis. A
    /// restricted run says nothing of axes and environments.
    pub remarks: Vec<String>,
}

/// The environments a frontier variant's screenshot is taken in, as indices of the
/// declaration's environments (every mode of the first viewport, then of the next): every mode
/// at the first viewport, and the last environment. So each mode is seen once, and the last
/// viewport once.
pub fn screenshot_environments(viewports: usize, modes: usize) -> Vec<usize> {
    let mut environments: Vec<usize> = (0..modes.min(viewports * modes)).collect();
    if viewports * modes > modes {
        environments.push(viewports * modes - 1);
    }
    environments
}

/// [`screenshot_environments`] of a declaration.
pub fn screenshots_of(declaration: &Declaration) -> Vec<usize> {
    screenshot_environments(declaration.viewports.len(), declaration.modes.len())
}

/// The file name of a variant's screenshot in an environment.
pub fn screenshot_name(id: &str, environment: &Environment) -> String {
    format!(
        "{id}.{}x{}-{}.png",
        environment.width, environment.height, environment.mode
    )
}

fn text(value: &str) -> Value {
    Value::Str(value.to_owned())
}

fn map(entries: Vec<(&str, Value)>) -> Value {
    Value::Map(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect(),
    )
}

fn json(value: &str) -> String {
    patch::compact(&text(value))
}

/// Everything a page needs to measure the variants, made before a browser starts.
struct Plan {
    /// The declaration file, as every refusal names it.
    file: String,
    declaration: Declaration,
    /// The environments the run measures: every one, or the one of `--environment`.
    environments: Vec<Environment>,
    /// The index of each of them among the declaration's environments.
    at: Vec<usize>,
    /// The variants the run measures, as indices of `ids`: every one, or the one of
    /// `--variant`.
    selected: Vec<usize>,
    /// Whether the run is restricted to one variant or one environment.
    restricted: bool,
    ids: Vec<String>,
    /// Each variant's choice, in the order of `ids`: the index of each axis's value.
    choices: Vec<Vec<usize>>,
    /// Per mode, in the order of the declaration's modes: each variant's sheet and checks as
    /// JSON text, in the order of `ids`.
    specs: Vec<Vec<String>>,
    /// Per mode: the probe's configuration as JSON text.
    configs: Vec<String>,
    /// Per mode: the page's URL.
    urls: Vec<String>,
    /// The index of the variant of the shipped values.
    shipped: usize,
    guard: Guard,
}

impl Plan {
    /// A refusal of something the declaration says, with the file and the key.
    fn declared(&self, key: &str, why: impl std::fmt::Display) -> String {
        format!("{}: `{key}`: {why}", self.file)
    }

    /// The index of an environment's mode among the declaration's.
    fn mode(&self, environment: &Environment) -> usize {
        self.declaration
            .modes
            .iter()
            .position(|mode| mode.name == environment.mode)
            .unwrap_or(0)
    }
}

fn plan(options: &Options) -> Result<Plan, String> {
    let declaration = Declaration::read(&options.declaration)?;
    let variants = declaration.variants();
    let shipped = declaration.shipped();
    let shipped = variants
        .iter()
        .position(|variant| *variant == shipped)
        .ok_or("the space holds no shipped variant")?;
    let mut specs = Vec::with_capacity(declaration.modes.len());
    let mut configs = Vec::with_capacity(declaration.modes.len());
    let mut urls = Vec::with_capacity(declaration.modes.len());
    for mode in &declaration.modes {
        specs.push(
            variants
                .iter()
                .map(|variant| {
                    let checks = declaration
                        .checks(variant, &mode.name)
                        .into_iter()
                        .map(|check| {
                            map(vec![
                                ("key", text(&check.key)),
                                ("selector", text(&check.selector)),
                                ("property", text(&check.property)),
                                ("value", text(&check.value)),
                            ])
                        })
                        .collect();
                    patch::compact(&map(vec![
                        ("css", text(&declaration.sheet(variant, &mode.name))),
                        ("checks", Value::List(checks)),
                    ]))
                })
                .collect(),
        );
        let items = declaration
            .always_visible
            .iter()
            .map(|item| {
                map(vec![
                    ("name", text(&item.name)),
                    ("selector", text(&item.selector)),
                    (
                        "container",
                        item.container
                            .as_ref()
                            .map_or(Value::Null, |container| text(container)),
                    ),
                ])
            })
            .collect();
        configs.push(patch::compact(&map(vec![
            ("primary", text(&declaration.primary)),
            (
                "regions",
                declaration
                    .regions
                    .as_ref()
                    .map_or(Value::Null, |regions| text(regions)),
            ),
            (
                "rows",
                declaration
                    .rows
                    .as_ref()
                    .map_or(Value::Null, |rows| text(rows)),
            ),
            ("always_visible", Value::List(items)),
            (
                "small_target_px",
                Value::Int(i64::try_from(declaration.small_target_px).unwrap_or(i64::MAX)),
            ),
            (
                "attribute",
                mode.attribute
                    .as_ref()
                    .map_or(Value::Null, |(name, value)| {
                        map(vec![("name", text(name)), ("value", text(value))])
                    }),
            ),
            (
                "expect",
                mode.expect.as_ref().map_or(Value::Null, |expect| {
                    map(vec![
                        (
                            "attribute",
                            expect
                                .attribute
                                .as_ref()
                                .map_or(Value::Null, |(name, value)| {
                                    map(vec![("name", text(name)), ("value", text(value))])
                                }),
                        ),
                        (
                            "selector",
                            expect
                                .selector
                                .as_ref()
                                .map_or(Value::Null, |selector| text(selector)),
                        ),
                    ])
                }),
            ),
        ])));
        urls.push(declaration.url(&options.site, &mode.name)?);
    }
    let file = options.declaration.display().to_string();
    if options.events.is_some() && (options.variant.is_some() || options.environment.is_some()) {
        return Err(
            "--events goes with a whole campaign: a run restricted to one variant or one \
             environment ranks nothing"
                .to_owned(),
        );
    }
    let selected: Vec<usize> = match &options.variant {
        None => (0..variants.len()).collect(),
        Some(id) => match variants.iter().position(|variant| variant.id == *id) {
            Some(index) => vec![index],
            None => {
                let known: Vec<&str> = variants.iter().map(|variant| variant.id.as_str()).collect();
                return Err(format!(
                    "{file}: --variant `{id}` is no variant of the declaration ({})",
                    known.join(", ")
                ));
            }
        },
    };
    let all = declaration.environments();
    let at: Vec<usize> = match &options.environment {
        None => (0..all.len()).collect(),
        Some(name) => match all
            .iter()
            .position(|environment| environment.name() == *name)
        {
            Some(index) => vec![index],
            None => {
                let known: Vec<String> = all.iter().map(Environment::name).collect();
                return Err(format!(
                    "{file}: --environment `{name}` is no environment of the declaration ({})",
                    known.join(", ")
                ));
            }
        },
    };
    Ok(Plan {
        file,
        environments: at.iter().map(|&index| all[index].clone()).collect(),
        at,
        selected,
        restricted: options.variant.is_some() || options.environment.is_some(),
        choices: variants
            .iter()
            .map(|variant| variant.choice.clone())
            .collect(),
        ids: variants.into_iter().map(|variant| variant.id).collect(),
        specs,
        configs,
        urls,
        shipped,
        guard: Guard::new(&options.site.origin, &["GET".to_owned(), "HEAD".to_owned()]),
        declaration,
    })
}

fn count(value: &Value, key: &str) -> Result<u64, String> {
    match value.get(key) {
        Some(Value::Int(n)) => {
            u64::try_from(*n).map_err(|_| format!("the probe's `{key}` is {n}, below 0"))
        }
        _ => Err(format!("the probe's `{key}` is not a whole number")),
    }
}

fn flag(value: &Value, key: &str) -> Result<bool, String> {
    match value.get(key) {
        Some(Value::Bool(flag)) => Ok(*flag),
        _ => Err(format!("the probe's `{key}` is not true or false")),
    }
}

/// One render as the probe returns it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Sample {
    render: Render,
    /// What broke a hard rule, each line starting with the rule.
    hard: Vec<String>,
    /// The first elements behind a count, per count of [`NOTED`] that is above 0.
    elements: Vec<(String, Vec<String>)>,
    /// The probe's digest of every element's box and colours.
    digest: String,
}

fn texts(value: Option<&Value>) -> Vec<String> {
    value
        .map(Value::list)
        .unwrap_or_default()
        .iter()
        .filter_map(|text| text.str().map(str::to_owned))
        .collect()
}

fn sample_of(value: &Value) -> Result<Sample, String> {
    let named = value.get("elements");
    let elements = NOTED
        .iter()
        .filter_map(|&term| {
            let elements = texts(named.and_then(|named| named.get(term)));
            (!elements.is_empty()).then(|| (term.to_owned(), elements))
        })
        .collect();
    Ok(Sample {
        render: Render {
            primary_in_view: flag(value, "primary_in_view")?,
            h_scroll: flag(value, "h_scroll")?,
            unnamed: count(value, "unnamed")?,
            primary_px: count(value, "primary_px")?,
            small_targets: count(value, "small_targets")?,
            regions: count(value, "regions")?,
            text_runs: count(value, "text_runs")?,
            text_under_14: count(value, "text_under_14")?,
            low_contrast: count(value, "low_contrast")?,
            overflow: count(value, "overflow")?,
            always_visible_misses: count(value, "always_visible_misses")?,
            rows_in_view: count(value, "rows_in_view")?,
            primary_bottom_px: count(value, "primary_bottom_px")?,
        },
        hard: texts(value.get("notes")),
        elements,
        digest: value
            .get_str("digest")
            .ok_or("the probe's `digest` is not a text")?
            .to_owned(),
    })
}

/// The script that sets a mode's attribute on the root element as soon as the document has
/// one, before any script of the page runs.
fn attribute_script(name: &str, value: &str) -> String {
    format!(
        "(() => {{ const set = () => {{ const root = document.documentElement; if (!root) return \
         false; root.setAttribute({}, {}); return true; }}; if (!set()) new \
         MutationObserver((_, observer) => {{ if (set()) observer.disconnect(); \
         }}).observe(document, {{ childList: true }}); }})()",
        json(name),
        json(value)
    )
}

/// How often the ready selector is looked for.
const READY_WAIT_MS: u64 = 25;

/// What a declared selector matches on the page as shipped: refused when the browser does not
/// read it or it matches nothing.
fn matched(plan: &Plan, key: &str, selector: &str, found: &Value, url: &str) -> Result<(), String> {
    match found {
        Value::Int(n) if *n > 0 => Ok(()),
        Value::Int(0) => Err(plan.declared(
            key,
            format!("`{selector}` matches nothing on the page as shipped ({url})"),
        )),
        _ => Err(plan.declared(
            key,
            format!("the browser does not read the selector `{selector}`"),
        )),
    }
}

/// Parks the pointer where the declaration rests it in the environment, by a mouse move.
fn park(
    conn: &mut Conn,
    page: &Page,
    plan: &Plan,
    environment: &Environment,
) -> Result<(), String> {
    let (x, y) = plan.declaration.pointer(environment);
    page.move_pointer(conn, f64::from(x), f64::from(y))
}

/// Opens the declared page in an environment, waits until it is ready, parks the pointer, and
/// prepares the probe in it.
fn open(conn: &mut Conn, plan: &Plan, environment: &Environment) -> Result<Page, String> {
    let at = plan.mode(environment);
    let mode: &Mode = &plan.declaration.modes[at];
    let declaration = &plan.declaration;
    let viewport = Viewport {
        width: i64::from(environment.width),
        height: i64::from(environment.height),
    };
    let script = mode
        .attribute
        .as_ref()
        .map(|(name, value)| attribute_script(name, value));
    let scripts: Vec<&str> = script.iter().map(String::as_str).collect();
    let page = Page::open(
        conn,
        &Setup {
            viewport: &viewport,
            mode: mode.media.as_deref().unwrap_or("light"),
            reduced_motion: true,
            guard: Some(&plan.guard),
            init_scripts: &scripts,
        },
    )?;
    let url = &plan.urls[at];
    let status = page.navigate(conn, url)?;
    if status != 200 {
        return Err(format!("{url} answered {status}"));
    }
    let ready = format!(
        "(() => {{ try {{ return document.querySelector({}) ? 1 : 0; }} catch {{ return -1; }} }})()",
        json(&declaration.page.ready)
    );
    let started = Instant::now();
    loop {
        match page.eval(conn, &ready)? {
            Value::Int(1) => break,
            Value::Int(0) => {}
            _ => {
                return Err(plan.declared(
                    "page.ready",
                    format!(
                        "the browser does not read the selector `{}`",
                        declaration.page.ready
                    ),
                ));
            }
        }
        if started.elapsed() >= Duration::from_millis(declaration.page.ready_timeout_ms) {
            return Err(plan.declared(
                "page.ready",
                format!(
                    "`{}` matched nothing within {} ms on {url}",
                    declaration.page.ready, declaration.page.ready_timeout_ms
                ),
            ));
        }
        page.wait(conn, READY_WAIT_MS)?;
    }
    // Before the settle wait: what the page closes when the pointer leaves has that time.
    park(conn, &page, plan, environment)?;
    if declaration.page.settle_ms > 0 {
        page.wait(conn, declaration.page.settle_ms)?;
    }
    page.eval(conn, PROBE)?;
    let prepared = page.eval(
        conn,
        &format!("__uxuiCampaign.prepare({})", plan.configs[at]),
    )?;
    let size = (count(&prepared, "width")?, count(&prepared, "height")?);
    if size != (u64::from(environment.width), u64::from(environment.height)) {
        return Err(format!(
            "the viewport is {}x{}, not {}x{}",
            size.0, size.1, environment.width, environment.height
        ));
    }
    let found = |key: &str| prepared.get(key).cloned().unwrap_or(Value::Null);
    matched(
        plan,
        "primary",
        &declaration.primary,
        &found("primary"),
        url,
    )?;
    if let Some(regions) = &declaration.regions {
        matched(plan, "regions", regions, &found("regions"), url)?;
    }
    if let Some(rows) = &declaration.rows {
        matched(plan, "rows", rows, &found("rows"), url)?;
    }
    let items = found("always_visible");
    let containers = found("containers");
    for (index, item) in declaration.always_visible.iter().enumerate() {
        matched(
            plan,
            &format!("always_visible[{index}].selector"),
            &item.selector,
            items.list().get(index).unwrap_or(&Value::Null),
            url,
        )?;
        if let Some(container) = &item.container {
            matched(
                plan,
                &format!("always_visible[{index}].container"),
                container,
                containers.list().get(index).unwrap_or(&Value::Null),
                url,
            )?;
        }
    }
    Ok(page)
}

/// How often a render is measured again while a font it uses is still loading, and the wait
/// between two tries.
const FONT_TRIES: usize = 200;
const FONT_WAIT_MS: u64 = 25;

/// Whether the page shows what the mode expects of it, from what the probe read back.
fn expected(plan: &Plan, at: usize, read: Option<&Value>) -> Result<(), String> {
    let mode = &plan.declaration.modes[at];
    let Some(expect) = &mode.expect else {
        return Ok(());
    };
    let read = |key: &str| read.and_then(|read| read.get(key));
    if let Some((name, value)) = &expect.attribute {
        let held = read("attribute").and_then(Value::str);
        if held != Some(value.as_str()) {
            return Err(plan.declared(
                &format!("environments.modes[{at}].expect.attribute"),
                format!(
                    "the mode `{}` expects `{name}` `{value}` on the root element, and the page \
                     has it {} when it is measured; the mode is set by {}, and the page does \
                     not show it",
                    mode.name,
                    held.map_or("unset".to_owned(), |held| format!("`{held}`")),
                    mode.set_by()
                ),
            ));
        }
    }
    if let Some(selector) = &expect.selector {
        let key = format!("environments.modes[{at}].expect.selector");
        match read("selector") {
            Some(Value::Int(n)) if *n > 0 => {}
            Some(Value::Int(0)) => {
                return Err(plan.declared(
                    &key,
                    format!(
                        "the mode `{}` expects the page to hold an element for it, and \
                         `{selector}` matches nothing on the page when it is measured; the mode \
                         is set by {}, and the page does not show it",
                        mode.name,
                        mode.set_by()
                    ),
                ));
            }
            _ => {
                return Err(plan.declared(
                    &key,
                    format!("the browser does not read the selector `{selector}`"),
                ));
            }
        }
    }
    Ok(())
}

/// Renders a variant (`null`: the page with no sheet injected), parks the pointer, and measures
/// the page. Refused: a property of the sheet that is not in force, a mode's attribute the page
/// took away, and a page that does not show what its mode expects.
fn measure(
    conn: &mut Conn,
    page: &Page,
    plan: &Plan,
    environment: &Environment,
    spec: &str,
) -> Result<Sample, String> {
    let mode = plan.mode(environment);
    page.eval(conn, &format!("__uxuiCampaign.apply({spec})"))?;
    park(conn, page, plan, environment)?;
    let expression = format!("__uxuiCampaign.measure({spec})");
    for _ in 0..FONT_TRIES {
        let value = page.eval(conn, &expression)?;
        if value.get_str("fonts") != Some("loaded") {
            page.wait(conn, FONT_WAIT_MS)?;
            continue;
        }
        if let Some(failed) = value
            .get("failed")
            .map(Value::list)
            .unwrap_or_default()
            .first()
        {
            return Err(plan.declared(
                failed.get_str("key").unwrap_or("axes"),
                failed
                    .get_str("why")
                    .unwrap_or("a property is not in force"),
            ));
        }
        if let Some((name, expected)) = &plan.declaration.modes[mode].attribute
            && value.get_str("attribute") != Some(expected)
        {
            return Err(plan.declared(
                &format!("environments.modes[{mode}].attribute"),
                format!(
                    "the page has `{name}` {} when it is measured, not `{expected}`: the page \
                     changed the attribute after it was ready; the run set it before the page's \
                     scripts ran and again when the page was ready",
                    value
                        .get_str("attribute")
                        .map_or("unset".to_owned(), |held| format!("`{held}`"))
                ),
            ));
        }
        expected(plan, mode, value.get("expect"))?;
        return sample_of(&value);
    }
    Err(format!(
        "a font was still loading after {FONT_TRIES} tries {FONT_WAIT_MS} ms apart"
    ))
}

/// Fails when the guard failed a request of the page.
fn unblocked(conn: &Conn, plan: &Plan) -> Result<(), String> {
    match conn.blocked.first() {
        None => Ok(()),
        Some(url) => Err(format!(
            "the page asked for {url}, which the guard failed ({} requests in all): a campaign \
             page reaches only {} with GET and HEAD, and a page that was denied something is not \
             measured",
            conn.blocked.len(),
            plan.guard.origin
        )),
    }
}

/// One environment's renders, in the order of the plan's selected variants.
type Samples = Vec<Sample>;

/// The live view of a run (`--events`): what the pages tell it, and where its screenshots go.
struct Live<'a> {
    progress: Mutex<Progress>,
    dir: &'a Path,
    /// Whether each variant is shot as it is measured, in the environments of the frontier's
    /// screenshots: with `--shots`.
    shots: bool,
}

impl Live<'_> {
    fn lock(&self) -> Result<MutexGuard<'_, Progress>, String> {
        self.progress
            .lock()
            .map_err(|_| "the live view: another page failed while it wrote".to_owned())
    }

    /// The screenshot of the variant at `index` as it is rendered on `page` in the environment
    /// at `at` of the plan, written under the events directory, when one is taken there.
    fn shoot(
        &self,
        conn: &mut Conn,
        page: &Page,
        plan: &Plan,
        at: usize,
        index: usize,
    ) -> Result<Option<(String, String)>, String> {
        if !self.shots || !screenshots_of(&plan.declaration).contains(&plan.at[at]) {
            return Ok(None);
        }
        let environment = &plan.environments[at];
        let bytes = page.screenshot(conn)?;
        let path = events::shot_path(&plan.ids[index], environment);
        let file = self.dir.join(&path);
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("{}: {error}", parent.display()))?;
        }
        std::fs::write(&file, bytes).map_err(|error| format!("{}: {error}", file.display()))?;
        Ok(Some((environment.name(), path)))
    }
}

/// The renders of the environment at `at` of the plan, on one page; with a live view, each told
/// to it as it is measured.
fn measure_environment(
    conn: &mut Conn,
    plan: &Plan,
    at: usize,
    live: Option<&Live>,
) -> Result<Samples, String> {
    let environment = &plan.environments[at];
    let mode = plan.mode(environment);
    let page = open(conn, plan, environment)?;
    let built = measure(conn, &page, plan, environment, "null")?.render;
    if let Some(live) = live {
        live.lock()?.built(at, built)?;
    }
    let specs = &plan.specs[mode];
    let mut measured: Samples = Vec::with_capacity(plan.selected.len());
    for &index in &plan.selected {
        let id = &plan.ids[index];
        if let Some(live) = live {
            live.lock()?.starting(index)?;
        }
        let sample = measure(conn, &page, plan, environment, &specs[index])
            .map_err(|why| format!("{why} (variant {id})"))?;
        let render = sample.render;
        if index == plan.shipped && render != built {
            return Err(format!(
                "{id}: the variant of the shipped values measures {render:?} with its sheet \
                 injected, the page without a sheet {built:?}"
            ));
        }
        if let Some(live) = live {
            let shot = live.shoot(conn, &page, plan, at, index)?;
            live.lock()?.measured(at, index, render, shot)?;
        }
        measured.push(sample);
    }
    let first = plan.selected[0];
    let again = measure(conn, &page, plan, environment, &specs[first])?.render;
    if again != measured[0].render {
        return Err(format!(
            "{}: measured again after the last variant it gives {again:?}, at first {:?}: a \
             render leaves a trace, or the page changes while it is measured",
            plan.ids[first], measured[0].render
        ));
    }
    unblocked(conn, plan)?;
    page.close(conn)?;
    Ok(measured)
}

/// The environments whose mode is read back before the campaign: for each mode with an
/// attribute or an expectation, the first environment the run measures it in.
fn read_back_in(plan: &Plan) -> Vec<&Environment> {
    plan.declaration
        .modes
        .iter()
        .filter(|mode| mode.attribute.is_some() || mode.expect.is_some())
        .filter_map(|mode| {
            plan.environments
                .iter()
                .find(|environment| environment.mode == mode.name)
        })
        .collect()
}

/// One load of the page in an environment, measured with no sheet injected: a mode whose
/// attribute the page does not keep or whose expectation it does not show is refused here,
/// before any variant is rendered, and not after the renders of the modes before it.
fn read_back(conn: &mut Conn, plan: &Plan, environment: &Environment) -> Result<(), String> {
    let page = open(conn, plan, environment)?;
    measure(conn, &page, plan, environment, "null")?;
    unblocked(conn, plan)?;
    page.close(conn)
}

/// One variant on a page loaded anew: it must measure what the campaign recorded. Returns its
/// screenshot when one is asked for.
fn measure_anew(
    conn: &mut Conn,
    plan: &Plan,
    environment: &Environment,
    index: usize,
    recorded: &Render,
    shoot: bool,
) -> Result<Option<Vec<u8>>, String> {
    let mode = plan.mode(environment);
    let page = open(conn, plan, environment)?;
    let render = measure(conn, &page, plan, environment, &plan.specs[mode][index])?.render;
    if render != *recorded {
        return Err(format!(
            "{}: on a page loaded anew it measures {render:?}, in the campaign {recorded:?}",
            plan.ids[index]
        ));
    }
    let shot = if shoot {
        // A caret blinks in a field the page focused: hidden, it is not in one shot and gone
        // from the next.
        page.eval(conn, "__uxuiCampaign.still()")?;
        park(conn, &page, plan, environment)?;
        Some(page.screenshot(conn)?)
    } else {
        None
    };
    unblocked(conn, plan)?;
    page.close(conn)?;
    Ok(shot)
}

fn connect(ws_url: &str) -> Result<Conn, String> {
    Ok(Connection::new(WebSocket::connect(
        ws_url,
        Duration::from_secs(10),
    )?))
}

fn close(mut conn: Conn) {
    conn.transport_mut().close();
}

/// What broke a declared hard rule, grouped: one line per environment and finding. The probe
/// starts each note with the rule it is about.
fn notes(plan: &Plan, measured: &[Samples]) -> Vec<String> {
    let mut lines = Vec::new();
    for (environment, renders) in plan.environments.iter().zip(measured) {
        let mut found: BTreeMap<&str, (usize, &str)> = BTreeMap::new();
        let ids = plan.selected.iter().map(|&index| &plan.ids[index]);
        for (id, sample) in ids.zip(renders) {
            for note in &sample.hard {
                let declared = note
                    .split_once(": ")
                    .is_some_and(|(rule, _)| plan.declaration.hard.iter().any(|hard| hard == rule));
                if declared {
                    found.entry(note).or_insert((0, id)).0 += 1;
                }
            }
        }
        for (note, (variants, first)) in found {
            lines.push(format!(
                "{}: {note} ({variants} variants, the first {first})",
                environment.name()
            ));
        }
    }
    lines
}

/// Per axis, how many of its values beside the shipped one change the page: a value changes it
/// when one of its renders differs from the render of the shipped value beside the same choice
/// of the other axes, in the same environment, in a measured number or in the probe's digest.
/// Of a run that is not restricted: every variant in every environment.
fn changing(plan: &Plan, measured: &[Samples]) -> Vec<u64> {
    let axes = &plan.declaration.axes;
    // The step between two variants that differ by one value of an axis: the last axis changes
    // fastest.
    let mut steps = vec![1usize; axes.len()];
    for at in (0..axes.len().saturating_sub(1)).rev() {
        steps[at] = steps[at + 1] * axes[at + 1].values.len();
    }
    axes.iter()
        .enumerate()
        .map(|(at, axis)| {
            let differs = |value: usize| {
                plan.choices.iter().enumerate().any(|(index, choice)| {
                    choice[at] == value && {
                        let beside = index - value * steps[at] + axis.shipped * steps[at];
                        measured.iter().any(|renders| {
                            renders[index].render != renders[beside].render
                                || renders[index].digest != renders[beside].digest
                        })
                    }
                })
            };
            let count = (0..axis.values.len())
                .filter(|&value| value != axis.shipped && differs(value))
                .count();
            u64::try_from(count).unwrap_or(u64::MAX)
        })
        .collect()
}

/// What the run says about each axis, one line: how many of its values change the page, or
/// that none does.
fn axis_lines(declaration: &Declaration, changing: &[u64]) -> Vec<String> {
    declaration
        .axes
        .iter()
        .zip(changing)
        .map(|(axis, &changing)| {
            let others = axis.values.len().saturating_sub(1);
            if changing == 0 {
                format!(
                    "note: axis {}: no value changes a box or a computed style on this page",
                    axis.name
                )
            } else {
                format!(
                    "axis {}: {changing} of the {others} values beside the shipped one change a \
                     box or a computed style on this page",
                    axis.name
                )
            }
        })
        .collect()
}

/// The modes that are set by a query and expect nothing, one line each: the run cannot know
/// that the page took the query.
fn unread_modes(declaration: &Declaration) -> Vec<String> {
    declaration
        .modes
        .iter()
        .filter(|mode| mode.expect.is_none())
        .filter_map(|mode| {
            let query = mode.query.as_ref()?;
            Some(format!(
                "note: mode `{}`: set by the query `{query}` and not read back: a page that \
                 does not know the query is measured in its default mode under this name; \
                 `expect` names an attribute of the root element or a selector the page shows \
                 in the mode",
                mode.name
            ))
        })
        .collect()
}

/// The first elements behind the counts of the variant at `position` of the selected ones, per
/// environment; a note's `environment` is an index of the declaration's environments.
fn notes_of(plan: &Plan, measured: &[Samples], position: usize) -> Vec<Note> {
    measured
        .iter()
        .zip(&plan.at)
        .flat_map(|(renders, &environment)| {
            renders[position]
                .elements
                .iter()
                .map(move |(term, elements)| Note {
                    environment,
                    term: term.clone(),
                    elements: elements.clone(),
                })
        })
        .collect()
}

/// Everything the browser is asked: the record, the screenshots, the checks.
struct Rendered {
    measured: Measured,
    shots: Vec<(String, Vec<u8>)>,
    notes: Vec<String>,
    checks: Vec<String>,
    remarks: Vec<String>,
}

/// The checks every run makes, one line each, for the renders it measured.
fn checks_of(plan: &Plan) -> Vec<String> {
    let environments = &plan.environments;
    let mut checks = Vec::new();
    if plan.selected.contains(&plan.shipped) {
        checks.push(format!(
            "{}: with its sheet injected it measures what the page measures without one, in {} \
             environments",
            plan.ids[plan.shipped],
            environments.len()
        ));
    }
    checks.extend([
        format!(
            "{}: measured again after the last variant, the same numbers in {} environments",
            plan.ids[plan.selected[0]],
            environments.len()
        ),
        format!(
            "every property a sheet sets is in force on every element its selector matches, in \
             {} renders",
            plan.selected.len() * environments.len()
        ),
        format!(
            "no request left {} or had another method than GET and HEAD",
            plan.guard.origin
        ),
        format!(
            "the pointer rests at {} while a render is measured and shot, moved there as a \
             mouse moves",
            pointers(plan)
        ),
    ]);
    let read_back = read_back_in(plan);
    if !read_back.is_empty() {
        let names: Vec<String> = read_back
            .iter()
            .map(|environment| environment.name())
            .collect();
        checks.push(format!(
            "each mode with an attribute or an expectation was read back on one page load before \
             the first variant was rendered: {}",
            names.join(", ")
        ));
    }
    for mode in &plan.declaration.modes {
        let Some(expect) = &mode.expect else {
            continue;
        };
        let mut shown = Vec::new();
        if let Some((name, value)) = &expect.attribute {
            shown.push(format!("`{name}` `{value}` on the root element"));
        }
        if let Some(selector) = &expect.selector {
            shown.push(format!("an element that matches `{selector}`"));
        }
        let renders = environments
            .iter()
            .filter(|environment| environment.mode == mode.name)
            .count()
            * plan.selected.len();
        checks.push(format!(
            "the mode `{}` shows what it expects in {renders} renders: {}",
            mode.name,
            shown.join(" and ")
        ));
    }
    checks
}

/// Renders the variants of `wanted` (indices of the plan's variants, each with its index among
/// the selected ones) in the environments of `shot_in` (indices of the plan's environments), each
/// on a page loaded anew, measured again and, with `--shots`, shot. Returns the shots by file
/// name.
fn anew(
    ws_url: &str,
    options: &Options,
    plan: &Plan,
    measured: &[Samples],
    wanted: &[(usize, usize)],
    shot_in: &[usize],
) -> Result<Vec<(String, Vec<u8>)>, String> {
    let environments = &plan.environments;
    let shots: Vec<Option<Vec<u8>>> = ux_browser::pool::run(
        options.jobs,
        wanted.len() * shot_in.len(),
        || connect(ws_url),
        |conn: &mut Conn, at| {
            let (index, position) = wanted[at / shot_in.len()];
            let environment = shot_in[at % shot_in.len()];
            measure_anew(
                conn,
                plan,
                &environments[environment],
                index,
                &measured[environment][position].render,
                options.shots.is_some(),
            )
            .map_err(|why| format!("{why} ({})", environments[environment].name()))
        },
        close,
    )?;
    let mut files = Vec::new();
    for (at, shot) in shots.into_iter().enumerate() {
        if let Some(bytes) = shot {
            let (index, _) = wanted[at / shot_in.len()];
            let environment = &environments[shot_in[at % shot_in.len()]];
            files.push((screenshot_name(&plan.ids[index], environment), bytes));
        }
    }
    Ok(files)
}

fn render(
    browser: &Browser,
    options: &Options,
    plan: &Plan,
    live: Option<&Live>,
) -> Result<Rendered, String> {
    let ws_url = browser.process.ws_url.as_str();
    let environments = &plan.environments;
    let read_back = read_back_in(plan);
    ux_browser::pool::run(
        options.jobs,
        read_back.len(),
        || connect(ws_url),
        |conn: &mut Conn, at| {
            self::read_back(conn, plan, read_back[at])
                .map_err(|why| format!("{why} ({}); no variant was rendered", read_back[at].name()))
        },
        close,
    )?;
    let measured: Vec<Samples> = ux_browser::pool::run(
        options.jobs,
        environments.len(),
        || connect(ws_url),
        |conn: &mut Conn, at| {
            measure_environment(conn, plan, at, live)
                .map_err(|why| format!("{why} ({})", environments[at].name()))
        },
        close,
    )?;
    let mut checks = checks_of(plan);
    let notes = notes(plan, &measured);
    let renders_of = |position: usize| -> Vec<Render> {
        measured
            .iter()
            .map(|renders| renders[position].render)
            .collect()
    };
    if plan.restricted {
        let wanted: Vec<(usize, usize)> = plan
            .selected
            .iter()
            .enumerate()
            .map(|(position, &index)| (index, position))
            .collect();
        let shot_in: Vec<usize> = (0..environments.len()).collect();
        let shots = anew(ws_url, options, plan, &measured, &wanted, &shot_in)?;
        checks.push(format!(
            "{} variants: each measures on a page loaded anew what it measured in the run, in {} \
             environments",
            wanted.len(),
            shot_in.len()
        ));
        let renders = Renders {
            adapter: format!("ux-browser {}", ux_browser::VERSION),
            browser: browser.product.clone(),
            declaration: plan.declaration.clone(),
            variant: options.variant.clone(),
            environment: options.environment.clone(),
            environments: plan.at.clone(),
            variants: wanted
                .iter()
                .map(|&(index, position)| (plan.ids[index].clone(), renders_of(position)))
                .collect(),
            notes: wanted
                .iter()
                .map(|&(_, position)| notes_of(plan, &measured, position))
                .collect(),
        };
        return Ok(Rendered {
            measured: Measured::Restricted(renders),
            shots,
            notes,
            checks,
            remarks: unread_modes(&plan.declaration),
        });
    }
    let mut recorded = Recorded {
        format: record::FORMAT,
        adapter: format!("ux-browser {}", ux_browser::VERSION),
        browser: browser.product.clone(),
        declaration: plan.declaration.clone(),
        variants: plan
            .ids
            .iter()
            .enumerate()
            .map(|(index, id)| (id.clone(), renders_of(index)))
            .collect(),
        changing: changing(plan, &measured),
        notes: vec![Vec::new(); plan.ids.len()],
    };
    let frontier = report::rank(&recorded).frontier;
    for &index in std::iter::once(&plan.shipped).chain(&frontier) {
        recorded.notes[index] = notes_of(plan, &measured, index);
    }
    recorded.check()?;
    let mut remarks = unread_modes(&plan.declaration);
    remarks.extend(axis_lines(&plan.declaration, &recorded.changing));
    remarks.extend(
        report::idle_environments(&recorded)
            .into_iter()
            .map(|line| format!("note: {line}")),
    );

    let shot_in = screenshots_of(&plan.declaration);
    let wanted: Vec<(usize, usize)> = frontier.iter().map(|&index| (index, index)).collect();
    let shots = anew(ws_url, options, plan, &measured, &wanted, &shot_in)?;
    checks.push(format!(
        "{} frontier variants: each measures on a page loaded anew what it measured in the \
         campaign, in {} environments",
        frontier.len(),
        shot_in.len()
    ));
    Ok(Rendered {
        measured: Measured::Campaign(recorded),
        shots,
        notes,
        checks,
        remarks,
    })
}

/// Where the pointer rests, for a line of the run: the declared place, or the corner of each
/// viewport.
fn pointers(plan: &Plan) -> String {
    match plan.declaration.page.pointer {
        Some((x, y)) => format!("[{x}, {y}], the declared `page.pointer`, in every environment"),
        None => {
            let mut corners: Vec<String> = Vec::new();
            for &(width, height) in &plan.declaration.viewports {
                corners.push(format!(
                    "[{}, {}] in {width}x{height}",
                    width.saturating_sub(1),
                    height.saturating_sub(1)
                ));
            }
            format!(
                "the bottom-right corner of the viewport, {}",
                corners.join(", ")
            )
        }
    }
}

/// Runs the campaign: each mode with an attribute or an expectation read back on one page load,
/// every variant in every environment, then the frontier on pages loaded anew. Restricted to
/// one variant or one environment, it measures those renders, loads each anew, and writes them
/// as renders, not as a campaign. A declaration or a restriction that is refused starts no
/// browser. Nothing is written unless all of it succeeds, but the live view (`events`): its two
/// files are made when the plan is accepted and appended to while the run goes, and end with the
/// phase `end`, or `failed` and the reason.
pub fn run(options: &Options) -> Result<Outcome, String> {
    let plan = plan(options)?;
    let live = match &options.events {
        None => None,
        Some(dir) => {
            let mut events = Events::create(dir, &plan.declaration, events::local_clock())?;
            events.start()?;
            Some(Live {
                progress: Mutex::new(Progress::new(
                    events,
                    plan.ids.len(),
                    plan.environments.len(),
                )),
                dir,
                shots: options.shots.is_some(),
            })
        }
    };
    match measured_and_written(options, &plan, live.as_ref()) {
        Ok(mut outcome) => {
            if let Some(live) = &live {
                live.lock()?.events.end()?;
                for name in [events::CANDIDATES_FILE, events::PHASE_FILE] {
                    outcome.written.push(live.dir.join(name));
                }
            }
            Ok(outcome)
        }
        Err(why) => {
            if let Some(live) = &live {
                // The run's own reason is the one returned; a live view that cannot say it
                // adds nothing to it.
                let _ = live
                    .progress
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .events
                    .failed(&why);
            }
            Err(why)
        }
    }
}

fn measured_and_written(
    options: &Options,
    plan: &Plan,
    live: Option<&Live>,
) -> Result<Outcome, String> {
    let chrome = ux_browser::chrome::find(None)?;
    let browser = Browser::start_binary(&chrome, &options.profile, &[])?;
    let rendered = render(&browser, options, plan, live);
    browser.close();
    let rendered = rendered?;

    let write = |path: PathBuf, bytes: &[u8]| -> Result<PathBuf, String> {
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("{}: {error}", parent.display()))?;
        }
        std::fs::write(&path, bytes).map_err(|error| format!("{}: {error}", path.display()))?;
        Ok(path)
    };
    let mut written = vec![write(
        options.out.clone(),
        rendered.measured.text().as_bytes(),
    )?];
    if let Some(shots) = &options.shots {
        for (name, bytes) in &rendered.shots {
            written.push(write(shots.join(name), bytes)?);
        }
    }
    Ok(Outcome {
        renders: plan.selected.len() * plan.environments.len(),
        measured: rendered.measured,
        written,
        notes: rendered.notes,
        checks: rendered.checks,
        remarks: rendered.remarks,
    })
}
