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
//!   and must still be there at every measurement.
//!
//! The page reaches nothing but its own origin, with `GET` and `HEAD` only
//! (`ux_browser::cdp::Guard`). A request the guard failed fails the run and is named: a page
//! that was denied something is not the page a visitor sees. An app that reads its fixtures
//! from another origin or with another method is served so that it does not, or is not
//! measured.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use ux_browser::cdp::{Connection, Guard};
use ux_browser::render::Viewport;
use ux_browser::session::{Browser, Conn, Page, Setup};
use ux_browser::target::Site;
use ux_browser::ws::WebSocket;

use super::declaration::{Declaration, Environment, Mode};
use super::record::{Recorded, Render};
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
}

/// A finished run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// What was measured.
    pub recorded: Recorded,
    /// The files written, in the order written.
    pub written: Vec<PathBuf>,
    /// The renders measured for the record.
    pub renders: usize,
    /// What broke a declared hard rule, one line per environment and finding, with the number
    /// of variants it was found in and the first of them.
    pub notes: Vec<String>,
    /// The checks the run made beside the record, one line each.
    pub checks: Vec<String>,
}

/// The environments a frontier variant's screenshot is taken in, as indices of the
/// declaration's environments: the first and the last.
pub fn screenshot_environments(environments: usize) -> Vec<usize> {
    match environments {
        0 => Vec::new(),
        1 => vec![0],
        count => vec![0, count - 1],
    }
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
    environments: Vec<Environment>,
    ids: Vec<String>,
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
        ])));
        urls.push(declaration.url(&options.site, &mode.name)?);
    }
    Ok(Plan {
        file: options.declaration.display().to_string(),
        environments: declaration.environments(),
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

/// One render as the probe returns it, and what it says broke a hard rule.
fn render_of(value: &Value) -> Result<(Render, Vec<String>), String> {
    let notes = value
        .get("notes")
        .map(Value::list)
        .unwrap_or_default()
        .iter()
        .filter_map(|note| note.str().map(str::to_owned))
        .collect();
    Ok((
        Render {
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
        },
        notes,
    ))
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

/// Opens the declared page in an environment, waits until it is ready, and prepares the probe
/// in it.
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
    let items = found("always_visible");
    for (index, item) in declaration.always_visible.iter().enumerate() {
        matched(
            plan,
            &format!("always_visible[{index}].selector"),
            &item.selector,
            items.list().get(index).unwrap_or(&Value::Null),
            url,
        )?;
    }
    Ok(page)
}

/// How often a render is measured again while a font it uses is still loading, and the wait
/// between two tries.
const FONT_TRIES: usize = 200;
const FONT_WAIT_MS: u64 = 25;

/// Renders a variant (`null`: the page with no sheet injected) and measures it. Refused: a
/// property of the sheet that is not in force, and a mode's attribute the page took away.
fn measure(
    conn: &mut Conn,
    page: &Page,
    plan: &Plan,
    mode: usize,
    spec: &str,
) -> Result<(Render, Vec<String>), String> {
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
                    "the page has `{name}` {} when it is measured, not `{expected}`: it sets \
                     the attribute itself",
                    value
                        .get_str("attribute")
                        .map_or("unset".to_owned(), |held| format!("`{held}`"))
                ),
            ));
        }
        return render_of(&value);
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

/// One environment's renders, in the order of the plan's variants, with each one's notes.
type Measured = Vec<(Render, Vec<String>)>;

fn measure_environment(
    conn: &mut Conn,
    plan: &Plan,
    environment: &Environment,
) -> Result<Measured, String> {
    let mode = plan.mode(environment);
    let page = open(conn, plan, environment)?;
    let (built, _) = measure(conn, &page, plan, mode, "null")?;
    let specs = &plan.specs[mode];
    let mut measured = Vec::with_capacity(specs.len());
    for (index, spec) in specs.iter().enumerate() {
        let id = &plan.ids[index];
        let (render, notes) = measure(conn, &page, plan, mode, spec)
            .map_err(|why| format!("{why} (variant {id})"))?;
        if index == plan.shipped && render != built {
            return Err(format!(
                "{id}: the variant of the shipped values measures {render:?} with its sheet \
                 injected, the page without a sheet {built:?}"
            ));
        }
        measured.push((render, notes));
    }
    let (again, _) = measure(conn, &page, plan, mode, &specs[0])?;
    if again != measured[0].0 {
        return Err(format!(
            "{}: measured again after the last variant it gives {again:?}, at first {:?}: a \
             render leaves a trace, or the page changes while it is measured",
            plan.ids[0], measured[0].0
        ));
    }
    unblocked(conn, plan)?;
    page.close(conn)?;
    Ok(measured)
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
    let (render, _) = measure(conn, &page, plan, mode, &plan.specs[mode][index])?;
    if render != *recorded {
        return Err(format!(
            "{}: on a page loaded anew it measures {render:?}, in the campaign {recorded:?}",
            plan.ids[index]
        ));
    }
    let shot = if shoot {
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
fn notes(plan: &Plan, measured: &[Measured]) -> Vec<String> {
    let mut lines = Vec::new();
    for (environment, renders) in plan.environments.iter().zip(measured) {
        let mut found: BTreeMap<&str, (usize, &str)> = BTreeMap::new();
        for (id, (_, notes)) in plan.ids.iter().zip(renders) {
            for note in notes {
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

/// Everything the browser is asked: the record, the frontier's screenshots, the checks.
struct Rendered {
    recorded: Recorded,
    shots: Vec<(String, Vec<u8>)>,
    notes: Vec<String>,
    checks: Vec<String>,
}

fn render(browser: &Browser, options: &Options, plan: &Plan) -> Result<Rendered, String> {
    let ws_url = browser.process.ws_url.as_str();
    let environments = &plan.environments;
    let measured: Vec<Measured> = ux_browser::pool::run(
        options.jobs,
        environments.len(),
        || connect(ws_url),
        |conn: &mut Conn, at| {
            measure_environment(conn, plan, &environments[at])
                .map_err(|why| format!("{why} ({})", environments[at].name()))
        },
        close,
    )?;
    let recorded = Recorded {
        adapter: format!("ux-browser {}", ux_browser::VERSION),
        browser: browser.product.clone(),
        declaration: plan.declaration.clone(),
        variants: plan
            .ids
            .iter()
            .enumerate()
            .map(|(index, id)| {
                let renders = measured.iter().map(|renders| renders[index].0).collect();
                (id.clone(), renders)
            })
            .collect(),
    };
    let mut checks = vec![
        format!(
            "{}: with its sheet injected it measures what the page measures without one, in {} \
             environments",
            plan.ids[plan.shipped],
            environments.len()
        ),
        format!(
            "{}: measured again after the last variant, the same numbers in {} environments",
            plan.ids[0],
            environments.len()
        ),
        format!(
            "every property a sheet sets is in force on every element its selector matches, in \
             {} renders",
            plan.ids.len() * environments.len()
        ),
        format!(
            "no request left {} or had another method than GET and HEAD",
            plan.guard.origin
        ),
    ];

    let frontier = report::rank(&recorded).frontier;
    let shot_in = screenshot_environments(environments.len());
    let shoot = options.shots.is_some();
    let shots: Vec<Option<Vec<u8>>> = ux_browser::pool::run(
        options.jobs,
        frontier.len() * shot_in.len(),
        || connect(ws_url),
        |conn: &mut Conn, at| {
            let index = frontier[at / shot_in.len()];
            let environment = shot_in[at % shot_in.len()];
            measure_anew(
                conn,
                plan,
                &environments[environment],
                index,
                &measured[environment][index].0,
                shoot,
            )
            .map_err(|why| format!("{why} ({})", environments[environment].name()))
        },
        close,
    )?;
    checks.push(format!(
        "{} frontier variants: each measures on a page loaded anew what it measured in the \
         campaign, in {} environments",
        frontier.len(),
        shot_in.len()
    ));
    let mut files = Vec::new();
    for (at, shot) in shots.into_iter().enumerate() {
        if let Some(bytes) = shot {
            let index = frontier[at / shot_in.len()];
            let environment = &environments[shot_in[at % shot_in.len()]];
            files.push((screenshot_name(&plan.ids[index], environment), bytes));
        }
    }
    Ok(Rendered {
        recorded,
        shots: files,
        notes: notes(plan, &measured),
        checks,
    })
}

/// Runs the campaign: every variant in every environment, then the frontier on pages loaded
/// anew. A declaration that is refused starts no browser. Nothing is written unless all of it
/// succeeds.
pub fn run(options: &Options) -> Result<Outcome, String> {
    let plan = plan(options)?;
    let chrome = ux_browser::chrome::find(None)?;
    let browser = Browser::start_binary(&chrome, &options.profile, &[])?;
    let rendered = render(&browser, options, &plan);
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
        rendered.recorded.text().as_bytes(),
    )?];
    if let Some(shots) = &options.shots {
        for (name, bytes) in &rendered.shots {
            written.push(write(shots.join(name), bytes)?);
        }
    }
    Ok(Outcome {
        renders: rendered.recorded.variants.len() * plan.environments.len(),
        recorded: rendered.recorded,
        written,
        notes: rendered.notes,
        checks: rendered.checks,
    })
}
