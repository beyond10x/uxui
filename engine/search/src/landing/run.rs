//! The campaign in Chromium, through the browser adapter (`ux_browser`).
//!
//! One browser, one page per environment. The page is the built landing page with its own
//! scripts off: the static HTML a visitor's first paint shows. In it the probe
//! (`probes/landing.js`) renders each variant from the built page: the hero actions and the
//! section order as the variant's `product.json` has them, and the variant's `landing.css`
//! after the built style sheets. So 144 variants in 6 environments are 864 renders in 6 page
//! loads, not 864 builds of the site.
//!
//! What keeps a render honest:
//!
//! - the variant the sources are (the page as shipped) must render to the built page: the same
//!   markup and the same numbers, in every environment;
//! - the first variant is measured again after the last and must give the same numbers, so no
//!   render leaves a trace in the next;
//! - a variant's `product.json` may differ from the sources only in what the probe renders
//!   (`/product/actions` and the order of `/sections`);
//! - each screenshot is taken on a page loaded anew, and the variant must measure there what
//!   it measured in the campaign;
//! - with `rebuilt`, a site built from one variant's patched sources must show, in every
//!   environment, the markup and the numbers the probe rendered for that variant.
//!
//! The page reaches nothing but the server of the built site (`ux_browser::cdp::Guard`), with
//! `GET` and `HEAD` only.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use ux_browser::cdp::{Connection, Guard};
use ux_browser::render::Viewport;
use ux_browser::session::{Browser, Conn, Page, Setup};
use ux_browser::ws::WebSocket;

use super::patch::{self, Value};
use super::record::{self, Environment, Recorded, Render, Route};
use super::report;
use super::space;

/// The probe, evaluated once in each page.
const PROBE: &str = include_str!("../probes/landing.js");

/// From this width on the documentation pages show their sidebar; below it the sidebar opens
/// with one more action (the breakpoint of the site's theme).
const WIDE_FROM: i64 = 997;
const SIDEBAR_ACTIONS: i64 = 1;

/// The environments a frontier variant's screenshot is taken in, as indices of
/// [`record::ENVIRONMENTS`]: 1440x900 light and 390x844 dark.
pub const SCREENSHOT_ENVIRONMENTS: [usize; 2] = [0, 5];

/// What to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// The built site, `website/build`.
    pub build: PathBuf,
    /// `website/product.json`.
    pub product: PathBuf,
    /// `website/src/css/landing.css`.
    pub css: PathBuf,
    /// The directory the measurements and the screenshots are written to.
    pub out: PathBuf,
    /// The directory the browser's profile is made in and removed from.
    pub profile: PathBuf,
    /// How many pages are measured at once.
    pub jobs: usize,
    /// A variant's id and a site built from that variant's patched sources, to check the
    /// probe's render of it against.
    pub rebuilt: Option<(String, PathBuf)>,
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
    /// What broke a hard rule, one line per environment and finding, with the number of
    /// variants it was found in and the first of them.
    pub notes: Vec<String>,
    /// The checks the run made beside the record, one line each.
    pub checks: Vec<String>,
}

/// The file name of a variant's screenshot in an environment.
pub fn screenshot_name(id: &str, environment: &Environment) -> String {
    format!(
        "{id}.{}x{}-{}.png",
        environment.width, environment.height, environment.mode
    )
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
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

fn list<'a>(value: &'a Value, path: &[&str]) -> Result<&'a [Value], String> {
    let mut at = value;
    for key in path {
        at = at
            .get(key)
            .ok_or_else(|| format!("product.json: no `{}`", path.join(".")))?;
    }
    match at {
        Value::List(items) => Ok(items),
        _ => Err(format!("product.json: `{}` is not a list", path.join("."))),
    }
}

/// What the probe renders of a variant: its actions, its sections and its style sheet.
fn spec(product: &Value, css: &str) -> Result<Value, String> {
    let field = |item: &Value, key: &str| text(item.get_str(key).unwrap_or_default());
    let actions = list(product, &["product", "actions"])?
        .iter()
        .map(|action| {
            map(vec![
                ("label", field(action, "label")),
                ("href", field(action, "href")),
                ("kind", field(action, "kind")),
            ])
        })
        .collect();
    let sections = list(product, &["sections"])?
        .iter()
        .map(|section| {
            map(vec![
                ("kind", field(section, "kind")),
                ("eyebrow", field(section, "eyebrow")),
            ])
        })
        .collect();
    Ok(map(vec![
        ("actions", Value::List(actions)),
        ("sections", Value::List(sections)),
        ("css", text(css)),
    ]))
}

/// The document without what the probe renders: no actions, the sections in the order of their
/// kinds. Two documents that differ only in what the probe renders give the same value.
fn outside_the_probe(product: &Value) -> Result<Value, String> {
    list(product, &["product", "actions"])?;
    list(product, &["sections"])?;
    let Value::Map(entries) = product else {
        return Err("product.json is not an object".to_owned());
    };
    let entries = entries
        .iter()
        .map(|(key, value)| match (key.as_str(), value) {
            ("product", Value::Map(inner)) => (
                key.clone(),
                Value::Map(
                    inner
                        .iter()
                        .filter(|(key, _)| key != "actions")
                        .cloned()
                        .collect(),
                ),
            ),
            ("sections", Value::List(sections)) => {
                let mut sections = sections.clone();
                sections.sort_by_key(|section| section.get_str("kind").map(str::to_owned));
                (key.clone(), Value::List(sections))
            }
            _ => (key.clone(), value.clone()),
        })
        .collect();
    Ok(Value::Map(entries))
}

/// Everything a page needs to measure the variants, made before a browser starts.
struct Plan {
    ids: Vec<String>,
    /// Each variant's spec as JSON text, in the order of `ids`.
    specs: Vec<String>,
    /// The index of the variant the sources are.
    shipped: usize,
    /// The probe's configuration as JSON text.
    config: String,
}

fn plan(product: &str, css: &str) -> Result<Plan, String> {
    let variants = space::variants();
    let shipped = variants
        .iter()
        .position(space::Variant::is_shipped)
        .ok_or("the space holds no shipped variant")?;
    let (same_product, same_css) = variants[shipped].patch().apply(product, css)?;
    if same_product != product || same_css != css {
        return Err(format!(
            "the sources are not the variant {}: its patch changes them, so the built page is not \
             a variant of the space",
            variants[shipped].id
        ));
    }
    let document = patch::parse(product).map_err(|why| format!("product.json: {why}"))?;
    let rest = outside_the_probe(&document)?;
    let mut specs = Vec::with_capacity(variants.len());
    for variant in &variants {
        let patch = variant.patch();
        let (_, patched_css) = patch
            .apply(product, css)
            .map_err(|why| format!("{}: {why}", variant.id))?;
        let patched = patch
            .product(product)
            .map_err(|why| format!("{}: {why}", variant.id))?;
        if outside_the_probe(&patched)? != rest {
            return Err(format!(
                "{}: its patch changes more of product.json than /product/actions and the order \
                 of /sections, which is all the probe renders",
                variant.id
            ));
        }
        specs.push(patch::compact(&spec(&patched, &patched_css)?));
    }
    let journeys = record::JOURNEYS
        .iter()
        .map(|journey| {
            let paths = |paths: &[&str]| Value::List(paths.iter().map(|path| text(path)).collect());
            map(vec![
                ("name", text(journey.name)),
                ("targets", paths(journey.targets)),
                ("via", paths(journey.via)),
            ])
        })
        .collect();
    let config = patch::compact(&map(vec![
        ("base_path", text(record::PAGE)),
        ("journeys", Value::List(journeys)),
        ("wide_from", Value::Int(WIDE_FROM)),
        ("sidebar_actions", Value::Int(SIDEBAR_ACTIONS)),
    ]));
    Ok(Plan {
        ids: variants.into_iter().map(|variant| variant.id).collect(),
        specs,
        shipped,
        config,
    })
}

/// A served build: the URL of the landing page and the guard that keeps a page on its server.
struct Site {
    url: String,
    guard: Guard,
}

fn serve(build: &Path) -> Result<Site, String> {
    let server = ux_browser::serve::start(build, record::PAGE)?;
    Ok(Site {
        url: format!("{}{}", server.origin, record::PAGE),
        guard: Guard::new(&server.origin, &["GET".to_owned(), "HEAD".to_owned()]),
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
    let routes = match value.get("journeys") {
        Some(Value::List(routes)) if routes.len() == record::JOURNEYS.len() => routes,
        _ => return Err("the probe's `journeys` is not one route per journey".to_owned()),
    };
    let mut journeys = [None; 3];
    for (at, route) in routes.iter().enumerate() {
        journeys[at] = match route {
            Value::Null => None,
            Value::List(pair) => match pair.as_slice() {
                [Value::Int(actions), Value::Int(regions)] => Some(Route {
                    actions: u64::try_from(*actions).map_err(|_| "a route below 0")?,
                    regions: u64::try_from(*regions).map_err(|_| "a route below 0")?,
                }),
                _ => return Err("a route of the probe is not two whole numbers".to_owned()),
            },
            _ => return Err("a route of the probe is not a list or null".to_owned()),
        };
    }
    let notes = match value.get("notes") {
        Some(Value::List(notes)) => notes
            .iter()
            .filter_map(|note| note.str().map(str::to_owned))
            .collect(),
        _ => Vec::new(),
    };
    Ok((
        Render {
            primary_in_view: flag(value, "primary_in_view")?,
            h_scroll: flag(value, "h_scroll")?,
            contrast_failures: count(value, "contrast_failures")?,
            unnamed: count(value, "unnamed")?,
            content_missing: count(value, "content_missing")?,
            words: count(value, "words")?,
            primary_px: count(value, "primary_px")?,
            small_targets: count(value, "small_targets")?,
            journeys,
        },
        notes,
    ))
}

/// Opens the landing page of `site` in `environment` with its scripts off, and prepares the
/// probe in it.
fn open(
    conn: &mut Conn,
    site: &Site,
    plan: &Plan,
    environment: &Environment,
) -> Result<Page, String> {
    let viewport = Viewport {
        width: i64::from(environment.width),
        height: i64::from(environment.height),
    };
    let page = Page::open(
        conn,
        &Setup {
            viewport: &viewport,
            mode: environment.mode,
            reduced_motion: true,
            guard: Some(&site.guard),
            init_scripts: &[],
        },
    )?;
    page.call(
        conn,
        "Emulation.setScriptExecutionDisabled",
        map(vec![("value", Value::Bool(true))]),
    )?;
    let status = page.navigate(conn, &site.url)?;
    if status != 200 {
        return Err(format!("{} answered {status}", site.url));
    }
    page.eval(conn, PROBE)?;
    let prepared = page.eval(
        conn,
        &format!(
            "__uxuiLanding.prepare({}, {})",
            plan.config,
            patch::compact(&text(environment.mode))
        ),
    )?;
    let size = (count(&prepared, "width")?, count(&prepared, "height")?);
    if size != (u64::from(environment.width), u64::from(environment.height)) {
        return Err(format!(
            "the viewport is {}x{}, not {}x{}",
            size.0, size.1, environment.width, environment.height
        ));
    }
    for part in ["actions", "sections", "content"] {
        if count(&prepared, part)? == 0 {
            return Err(format!("the built page shows no {part}"));
        }
    }
    Ok(page)
}

/// How often a render is measured again while a font it uses is still loading, and the wait
/// between two tries.
const FONT_TRIES: usize = 200;
const FONT_WAIT_MS: u64 = 25;

/// Renders a variant (`null`: the built page) and measures it. With its scripts off the page
/// runs no callback, so there is no frame or font promise to wait on: the probe lays the
/// render out itself, and a render measured while a font was loading is measured again.
fn measure(conn: &mut Conn, page: &Page, spec: &str) -> Result<(Render, Vec<String>), String> {
    let expression = format!("__uxuiLanding.measure({spec})");
    for _ in 0..FONT_TRIES {
        let value = page.eval(conn, &expression)?;
        if value.get_str("fonts") == Some("loaded") {
            return render_of(&value);
        }
        page.wait(conn, FONT_WAIT_MS)?;
    }
    Err(format!(
        "a font was still loading after {FONT_TRIES} tries {FONT_WAIT_MS} ms apart"
    ))
}

fn markup(conn: &mut Conn, page: &Page) -> Result<String, String> {
    match page.eval(conn, "__uxuiLanding.html()")? {
        Value::Str(markup) => Ok(markup),
        _ => Err("the probe's markup is not text".to_owned()),
    }
}

/// One environment's renders, in the order of the plan's variants, with each one's notes.
type Measured = Vec<(Render, Vec<String>)>;

fn measure_environment(
    conn: &mut Conn,
    site: &Site,
    plan: &Plan,
    environment: &Environment,
) -> Result<Measured, String> {
    let page = open(conn, site, plan, environment)?;
    let (built, _) = measure(conn, &page, "null")?;
    let built_markup = markup(conn, &page)?;
    let mut measured = Vec::with_capacity(plan.specs.len());
    for (index, spec) in plan.specs.iter().enumerate() {
        let id = &plan.ids[index];
        let (render, notes) = measure(conn, &page, spec).map_err(|why| format!("{id}: {why}"))?;
        if index == plan.shipped {
            if markup(conn, &page)? != built_markup {
                return Err(format!(
                    "{id}: the probe's render of the variant the sources are is not the markup \
                     of the built page"
                ));
            }
            if render != built {
                return Err(format!(
                    "{id}: the probe's render of the variant the sources are measures {render:?}, \
                     the built page {built:?}"
                ));
            }
        }
        measured.push((render, notes));
    }
    let (again, _) = measure(conn, &page, &plan.specs[0])?;
    if again != measured[0].0 {
        return Err(format!(
            "{}: measured again after the last variant it gives {again:?}, at first {:?}",
            plan.ids[0], measured[0].0
        ));
    }
    page.close(conn)?;
    Ok(measured)
}

/// The screenshots of the variants at `indices` in one environment, each after the variant
/// measured there what the campaign recorded.
fn shoot_environment(
    conn: &mut Conn,
    site: &Site,
    plan: &Plan,
    environment: &Environment,
    expected: &[(usize, Render)],
) -> Result<Vec<Vec<u8>>, String> {
    let page = open(conn, site, plan, environment)?;
    let mut shots = Vec::with_capacity(expected.len());
    for (index, recorded) in expected {
        let id = &plan.ids[*index];
        let (render, _) = measure(conn, &page, &plan.specs[*index])?;
        if render != *recorded {
            return Err(format!(
                "{id}: on the page its screenshot is taken of it measures {render:?}, in the \
                 campaign {recorded:?}"
            ));
        }
        shots.push(page.screenshot(conn)?);
    }
    page.close(conn)?;
    Ok(shots)
}

/// Whether a site built from one variant's patched sources shows, in `environment`, the markup
/// and the numbers the probe rendered for that variant from the built page.
fn compare_rebuilt(
    conn: &mut Conn,
    site: &Site,
    rebuilt: &Site,
    plan: &Plan,
    index: usize,
    environment: &Environment,
) -> Result<(), String> {
    let page = open(conn, site, plan, environment)?;
    let (rendered, _) = measure(conn, &page, &plan.specs[index])?;
    let rendered_markup = markup(conn, &page)?;
    page.close(conn)?;
    let page = open(conn, rebuilt, plan, environment)?;
    let (built, _) = measure(conn, &page, "null")?;
    let built_markup = markup(conn, &page)?;
    page.close(conn)?;
    let id = &plan.ids[index];
    if rendered_markup != built_markup {
        return Err(format!(
            "{id} at {}: the site built from its sources has another markup than the probe's \
             render of it",
            environment.name()
        ));
    }
    if rendered != built {
        return Err(format!(
            "{id} at {}: the site built from its sources measures {built:?}, the probe's render \
             of it {rendered:?}",
            environment.name()
        ));
    }
    Ok(())
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

/// What broke a hard rule, grouped: one line per environment and finding.
fn notes(plan: &Plan, measured: &[Measured]) -> Vec<String> {
    let mut lines = Vec::new();
    for (environment, renders) in record::ENVIRONMENTS.iter().zip(measured) {
        let mut found: BTreeMap<&str, (usize, &str)> = BTreeMap::new();
        for (id, (_, notes)) in plan.ids.iter().zip(renders) {
            for note in notes {
                found.entry(note).or_insert((0, id)).0 += 1;
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

fn render(
    browser: &Browser,
    options: &Options,
    site: &Site,
    rebuilt: Option<&(usize, Site)>,
    plan: &Plan,
    digests: (String, String),
) -> Result<Rendered, String> {
    let ws_url = browser.process.ws_url.as_str();
    let environments = &record::ENVIRONMENTS;
    let measured: Vec<Measured> = ux_browser::pool::run(
        options.jobs,
        environments.len(),
        || connect(ws_url),
        |conn: &mut Conn, at| {
            measure_environment(conn, site, plan, &environments[at])
                .map_err(|why| format!("{}: {why}", environments[at].name()))
        },
        close,
    )?;
    let recorded = Recorded {
        adapter: format!("ux-browser {}", ux_browser::VERSION),
        browser: browser.product.clone(),
        product_digest: digests.0,
        css_digest: digests.1,
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
            "{}: the probe's render is the built page, markup and numbers, in {} environments",
            plan.ids[plan.shipped],
            environments.len()
        ),
        format!(
            "{}: measured again after the last variant, the same numbers in {} environments",
            plan.ids[0],
            environments.len()
        ),
    ];

    let frontier = report::rank(&recorded).frontier;
    let shots: Vec<Vec<Vec<u8>>> = ux_browser::pool::run(
        options.jobs,
        SCREENSHOT_ENVIRONMENTS.len(),
        || connect(ws_url),
        |conn: &mut Conn, at| {
            let environment = &environments[SCREENSHOT_ENVIRONMENTS[at]];
            let expected: Vec<(usize, Render)> = frontier
                .iter()
                .map(|&index| (index, measured[SCREENSHOT_ENVIRONMENTS[at]][index].0))
                .collect();
            shoot_environment(conn, site, plan, environment, &expected)
                .map_err(|why| format!("{}: {why}", environment.name()))
        },
        close,
    )?;
    checks.push(format!(
        "{} frontier variants: each measures on the page of its screenshot what it measured in \
         the campaign, in {} environments",
        frontier.len(),
        SCREENSHOT_ENVIRONMENTS.len()
    ));
    let mut files = Vec::new();
    for (at, &index) in frontier.iter().enumerate() {
        for (shots, &environment) in shots.iter().zip(&SCREENSHOT_ENVIRONMENTS) {
            files.push((
                screenshot_name(&plan.ids[index], &environments[environment]),
                shots[at].clone(),
            ));
        }
    }

    if let Some((index, rebuilt)) = rebuilt {
        ux_browser::pool::run(
            options.jobs,
            environments.len(),
            || connect(ws_url),
            |conn: &mut Conn, at| {
                compare_rebuilt(conn, site, rebuilt, plan, *index, &environments[at])
            },
            close,
        )?;
        checks.push(format!(
            "{}: the site built from its patched sources shows the markup and the numbers of \
             the probe's render, in {} environments",
            plan.ids[*index],
            environments.len()
        ));
    }
    Ok(Rendered {
        recorded,
        shots: files,
        notes: notes(plan, &measured),
        checks,
    })
}

/// Runs the campaign: every variant in every environment, then the frontier's screenshots.
/// Nothing is written unless all of it succeeds.
pub fn run(options: &Options) -> Result<Outcome, String> {
    let site = serve(&options.build)?;
    let product = read(&options.product)?;
    let css = read(&options.css)?;
    let plan = plan(&product, &css)?;
    let rebuilt = match &options.rebuilt {
        None => None,
        Some((id, build)) => {
            let index = plan
                .ids
                .iter()
                .position(|known| known == id)
                .ok_or_else(|| format!("`{id}` is no variant of the space"))?;
            Some((index, serve(build)?))
        }
    };
    let chrome = ux_browser::chrome::find(None)?;
    let browser = Browser::start_binary(&chrome, &options.profile, &[])?;
    let digests = (
        record::digest(product.as_bytes()),
        record::digest(css.as_bytes()),
    );
    let rendered = render(&browser, options, &site, rebuilt.as_ref(), &plan, digests);
    browser.close();
    let rendered = rendered?;

    let write = |name: &str, bytes: &[u8]| -> Result<PathBuf, String> {
        let path = options.out.join(name);
        std::fs::write(&path, bytes).map_err(|error| format!("{}: {error}", path.display()))?;
        Ok(path)
    };
    std::fs::create_dir_all(&options.out)
        .map_err(|error| format!("{}: {error}", options.out.display()))?;
    let mut written = vec![write(
        "measurements.json",
        rendered.recorded.text().as_bytes(),
    )?];
    for (name, bytes) in &rendered.shots {
        written.push(write(name, bytes)?);
    }
    Ok(Outcome {
        renders: rendered.recorded.variants.len() * record::ENVIRONMENTS.len(),
        recorded: rendered.recorded,
        written,
        notes: rendered.notes,
        checks: rendered.checks,
    })
}
