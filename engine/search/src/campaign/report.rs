//! From a recorded campaign to a ranked report: each variant's terms, the ranking, the Pareto
//! frontier and the page as shipped among them.
//!
//! The first term is `hard`: the declared hard rules a variant breaks, one per rule and
//! environment. The declared terms follow in the declaration's order
//! (`declaration::TERMS` says what each one is). `hard` is summed over the environments; every
//! other term is the worst value of the environments. `text_under_14` is a share, held in
//! thousandths rounded down and written with three places (`0.564`); every other term is a
//! count or a length in px.
//!
//! The ranking is lexicographic over the terms, ties by variant id ([`crate::rank`], the code
//! the landing campaign ranks with). The frontier is the Pareto frontier of the variants that
//! break no hard rule: a variant that breaks one stands on no frontier, whatever its other
//! terms.

use std::collections::BTreeSet;

use super::declaration::{Declaration, Environment, Variant};
use super::record::{COLUMNS, Note, Recorded, Render, Renders};
use super::run;
use crate::rank::{self, Scored};

/// The terms' names in ranking order: `hard`, then the declared terms.
pub fn term_names(declaration: &Declaration) -> Vec<&str> {
    std::iter::once("hard")
        .chain(declaration.terms.iter().map(String::as_str))
        .collect()
}

/// The declared hard rules one render breaks.
fn broken(declaration: &Declaration, render: &Render) -> u64 {
    declaration
        .hard
        .iter()
        .map(|rule| match rule.as_str() {
            "primary_in_view" => !render.primary_in_view,
            "no_h_scroll" => render.h_scroll,
            "contrast" => render.low_contrast > 0,
            "named" => render.unnamed > 0,
            _ => false,
        })
        .map(u64::from)
        .sum()
}

/// What a term where more is better is held as: this less the count, so that lower is better
/// for every term, and the worst environment is the one with the fewest. The report prints the
/// count.
const MORE_IS_BETTER: u64 = u64::MAX;

/// One term of one render. The share of small text is in thousandths, rounded down; a render
/// without text has none. `rows_in_view` is held as [`MORE_IS_BETTER`] less the count.
fn term(name: &str, render: &Render) -> u64 {
    match name {
        "primary_px" => render.primary_px,
        "small_targets" => render.small_targets,
        "regions" => render.regions,
        "text_under_14" if render.text_runs == 0 => 0,
        "text_under_14" => render.text_under_14 * 1000 / render.text_runs,
        "low_contrast" => render.low_contrast,
        "overflow" => render.overflow,
        "always_visible_misses" => render.always_visible_misses,
        "rows_in_view" => MORE_IS_BETTER - render.rows_in_view,
        "primary_bottom_px" => render.primary_bottom_px,
        _ => 0,
    }
}

/// A variant's terms in the order of [`term_names`], from its renders, one per environment.
pub fn terms(declaration: &Declaration, renders: &[Render]) -> Vec<u64> {
    let hard = renders
        .iter()
        .map(|render| broken(declaration, render))
        .sum();
    std::iter::once(hard)
        .chain(declaration.terms.iter().map(|name| {
            renders
                .iter()
                .map(|render| term(name, render))
                .max()
                .unwrap_or(0)
        }))
        .collect()
}

/// The terms as the report writes them: `hard=0 low_contrast=2 text_under_14=0.564 ...`.
pub fn terms_text(declaration: &Declaration, terms: &[u64]) -> String {
    term_names(declaration)
        .iter()
        .zip(terms)
        .map(|(name, value)| match *name {
            "text_under_14" => format!("{name}={}.{:03}", value / 1000, value % 1000),
            "rows_in_view" => format!("{name}={}", MORE_IS_BETTER - value),
            _ => format!("{name}={value}"),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// A ranked campaign.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ranked {
    /// Every variant's id and terms, in the record's order.
    pub scored: Vec<Scored>,
    /// The indices of `scored`, best first.
    pub ranking: Vec<usize>,
    /// The indices of `scored` on the frontier, in rank order.
    pub frontier: Vec<usize>,
}

/// Ranks a recorded campaign.
pub fn rank(recorded: &Recorded) -> Ranked {
    let scored: Vec<Scored> = recorded
        .variants
        .iter()
        .map(|(id, renders)| Scored {
            id: id.clone(),
            terms: terms(&recorded.declaration, renders),
        })
        .collect();
    let sound: Vec<usize> = (0..scored.len())
        .filter(|&index| scored[index].terms[0] == 0)
        .collect();
    let among: Vec<Scored> = sound.iter().map(|&index| scored[index].clone()).collect();
    let frontier = rank::frontier(&among)
        .into_iter()
        .map(|index| sound[index])
        .collect();
    Ranked {
        ranking: rank::rank(&scored),
        frontier,
        scored,
    }
}

/// How many different sets of terms the variants at `indices` have.
fn sets(ranked: &Ranked, indices: &[usize]) -> usize {
    indices
        .iter()
        .map(|&index| &ranked.scored[index].terms)
        .collect::<BTreeSet<_>>()
        .len()
}

/// A ranked record with what both forms of the report say about it.
struct Read {
    variants: Vec<Variant>,
    environments: Vec<Environment>,
    ranked: Ranked,
    /// The line of the page as shipped, after `shipped `.
    shipped: String,
}

impl Read {
    fn place(&self, index: usize) -> usize {
        self.ranked
            .ranking
            .iter()
            .position(|&ranked| ranked == index)
            .map_or(0, |at| at + 1)
    }
}

fn read(recorded: &Recorded) -> Result<Read, String> {
    recorded.check()?;
    let declaration = &recorded.declaration;
    let variants = declaration.variants();
    let ranked = rank(recorded);
    let shipped = declaration.shipped();
    let at = variants
        .iter()
        .position(|variant| *variant == shipped)
        .ok_or("the space holds no shipped variant")?;
    let own = &ranked.scored[at].terms;
    let before = ranked
        .scored
        .iter()
        .filter(|other| other.terms < *own)
        .count();
    let equal = ranked
        .scored
        .iter()
        .filter(|other| other.terms == *own)
        .count()
        - 1;
    let mut read = Read {
        environments: declaration.environments(),
        variants,
        ranked,
        shipped: String::new(),
    };
    read.shipped = format!(
        "{}: rank {} of {}, {before} variants before it on their terms, {equal} equal to it, {} \
         the frontier",
        shipped.id,
        read.place(at),
        read.variants.len(),
        if read.ranked.frontier.contains(&at) {
            "on"
        } else {
            "off"
        }
    );
    Ok(read)
}

fn environment_names(environments: &[Environment]) -> String {
    environments
        .iter()
        .map(Environment::name)
        .collect::<Vec<_>>()
        .join(", ")
}

/// The report: the frontier first, each of its variants with its patch, then the full ranking.
/// It refuses a record that is not of its declaration ([`Recorded::check`]).
pub fn report(recorded: &Recorded) -> Result<String, String> {
    let read = read(recorded)?;
    let declaration = &recorded.declaration;
    let ranked = &read.ranked;
    let axes: Vec<String> = declaration
        .axes
        .iter()
        .map(|axis| {
            let ids: Vec<&str> = axis.values.iter().map(|value| value.id.as_str()).collect();
            format!("{} {} ({})", axis.name, ids.len(), ids.join(", "))
        })
        .collect();
    let mut out = format!(
        "# ux-search campaign: the page {} on the browser adapter\n",
        declaration.page.path
    );
    out.push_str(&format!(
        "# adapter {}, browser {}\n",
        recorded.adapter, recorded.browser
    ));
    out.push_str(&format!("# declaration {}\n", declaration.digest));
    out.push_str(&format!(
        "# axes: {} = {} variants\n",
        axes.join(" x "),
        read.variants.len()
    ));
    out.push_str(&format!(
        "# environments {}: {}; {} renders\n",
        read.environments.len(),
        environment_names(&read.environments),
        read.variants.len() * read.environments.len()
    ));
    out.push_str(&format!(
        "# hard rules: {}\n",
        if declaration.hard.is_empty() {
            "none".to_owned()
        } else {
            declaration.hard.join(", ")
        }
    ));
    out.push_str(&format!(
        "# terms, lexicographic, lower is better{}: {}; hard summed over the environments, each \
         other term the worst of them; ties by variant id\n",
        if declaration.terms.iter().any(|term| term == "rows_in_view") {
            " (rows_in_view: more is better, the worst is the fewest)"
        } else {
            ""
        },
        term_names(declaration).join(" ")
    ));
    out.push_str(
        "# frontier: of the variants with hard=0, those no other of them beats on one term \
         without losing on another; equal terms stand on it together\n",
    );
    out.push_str(&format!("shipped {}\n", read.shipped));

    out.push_str(&format!(
        "frontier {}, sets of terms {}\n",
        ranked.frontier.len(),
        sets(ranked, &ranked.frontier)
    ));
    for &index in &ranked.frontier {
        let scored = &ranked.scored[index];
        out.push_str(&format!(
            "{:>4} {} {}\n",
            read.place(index),
            scored.id,
            terms_text(declaration, &scored.terms)
        ));
        for line in declaration.patch_lines(&read.variants[index]) {
            out.push_str(&format!("     {line}\n"));
        }
    }
    out.push_str(&format!(
        "ranking {}, sets of terms {}\n",
        ranked.ranking.len(),
        sets(ranked, &ranked.ranking)
    ));
    for (at, &index) in ranked.ranking.iter().enumerate() {
        let scored = &ranked.scored[index];
        out.push_str(&format!(
            "{:>4} {} {}\n",
            at + 1,
            scored.id,
            terms_text(declaration, &scored.terms)
        ));
    }
    Ok(out)
}

/// One variant's notes as (term, element) with the environments it is named in, in the order
/// each is first named.
fn named_in(notes: &[Note]) -> Vec<((&str, &str), Vec<usize>)> {
    let mut named: Vec<((&str, &str), Vec<usize>)> = Vec::new();
    for note in notes {
        for element in &note.elements {
            let key = (note.term.as_str(), element.as_str());
            match named.iter_mut().find(|(known, _)| *known == key) {
                Some((_, environments)) => environments.push(note.environment),
                None => named.push((key, vec![note.environment])),
            }
        }
    }
    named
}

/// The lines of one variant's notes: one per element, `<environments> <term>: <element>`, where
/// `<environments>` names each environment it is named in, or says `all <n> environments`. An
/// element of `common` is left out: it is printed once for every variant.
fn note_lines(
    recorded: &Recorded,
    read: &Read,
    index: usize,
    common: &[(&str, &str)],
    out: &mut String,
) {
    let notes = recorded
        .notes
        .get(index)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let named = named_in(notes);
    let mut printed = 0;
    for ((term, element), environments) in &named {
        if common.contains(&(*term, *element)) {
            continue;
        }
        printed += 1;
        let at = if environments.len() == read.environments.len() && environments.len() > 1 {
            format!("all {} environments", environments.len())
        } else {
            environments
                .iter()
                .map(|&at| read.environments[at].name())
                .collect::<Vec<_>>()
                .join(", ")
        };
        out.push_str(&format!("     {at} {term}: {element}\n"));
    }
    if printed == 0 && !named.is_empty() {
        out.push_str("     only the elements of every variant, above\n");
    }
    if notes.is_empty() {
        let counted = recorded.variants[index].1.iter().any(|render| {
            render.low_contrast
                + render.overflow
                + render.small_targets
                + render.always_visible_misses
                > 0
        });
        out.push_str(if counted {
            "     the record names no element for it\n"
        } else {
            "     nothing to name: low_contrast, overflow, small_targets and \
             always_visible_misses are 0 in every environment\n"
        });
    }
}

/// What `--notes` adds after the report: for the page as shipped and for each frontier
/// variant, the first elements behind `low_contrast`, `overflow`, `small_targets` and
/// `always_visible_misses`, as the run recorded them. A count says how many; these lines say
/// which, so a reader can look at the element before taking the number.
///
/// An element is printed once where it is the same: once before the variants when every
/// variant printed names it in every environment (with more than one variant), else once per
/// variant with the environments it is named in.
pub fn notes(recorded: &Recorded) -> Result<String, String> {
    let read = read(recorded)?;
    let shipped = recorded.declaration.shipped();
    let at = read
        .variants
        .iter()
        .position(|variant| *variant == shipped)
        .ok_or("the space holds no shipped variant")?;
    let mut out = format!(
        "notes: the first elements behind each count, at most {} per term and environment, of \
         the page as shipped and of the frontier\n",
        super::record::NOTE_ELEMENTS
    );
    let mut printed = vec![at];
    printed.extend(read.ranked.frontier.iter().filter(|&&index| index != at));
    let everywhere = |index: usize| -> Vec<(&str, &str)> {
        let notes = recorded
            .notes
            .get(index)
            .map(Vec::as_slice)
            .unwrap_or_default();
        named_in(notes)
            .into_iter()
            .filter(|(_, environments)| {
                (0..read.environments.len()).all(|at| environments.contains(&at))
            })
            .map(|(key, _)| key)
            .collect()
    };
    let common: Vec<(&str, &str)> = if printed.len() > 1 {
        let others: Vec<Vec<(&str, &str)>> = printed[1..]
            .iter()
            .map(|&index| everywhere(index))
            .collect();
        everywhere(at)
            .into_iter()
            .filter(|key| others.iter().all(|other| other.contains(key)))
            .collect()
    } else {
        Vec::new()
    };
    if !common.is_empty() {
        out.push_str(&format!(
            "the same in every environment of every variant below, {} variants in {} \
             environments, {} renders:\n",
            printed.len(),
            read.environments.len(),
            printed.len() * read.environments.len()
        ));
        for (term, element) in &common {
            out.push_str(&format!("     {term}: {element}\n"));
        }
    }
    out.push_str(&format!("shipped {}\n", shipped.id));
    note_lines(recorded, &read, at, &common, &mut out);
    for &index in &read.ranked.frontier {
        out.push_str(&format!(
            "{:>4} {}\n",
            read.place(index),
            read.ranked.scored[index].id
        ));
        if index == at {
            out.push_str("     the page as shipped, above\n");
        } else {
            note_lines(recorded, &read, index, &common, &mut out);
        }
    }
    Ok(out)
}

/// The modes and the viewports whose renders measure the same numbers as another one's for
/// every variant, one line each: a mode that measures in every viewport what an earlier mode
/// measures there, a viewport that measures in every mode what an earlier viewport measures.
/// Such an environment costs its renders and changes no term.
pub fn idle_environments(recorded: &Recorded) -> Vec<String> {
    let declaration = &recorded.declaration;
    let (viewports, modes) = (declaration.viewports.len(), declaration.modes.len());
    // The environments are each viewport in each mode: the mode changes fastest.
    let same = |a: usize, b: usize| {
        recorded
            .variants
            .iter()
            .all(|(_, renders)| renders.get(a) == renders.get(b))
    };
    let mut lines = Vec::new();
    for mode in 1..modes {
        if let Some(earlier) = (0..mode)
            .find(|&earlier| (0..viewports).all(|v| same(v * modes + mode, v * modes + earlier)))
        {
            lines.push(format!(
                "mode {}: every variant measures the same numbers as in the mode {}, at every \
                 viewport",
                declaration.modes[mode].name, declaration.modes[earlier].name
            ));
        }
    }
    let size = |at: usize| {
        let (width, height) = declaration.viewports[at];
        format!("{width}x{height}")
    };
    for viewport in 1..viewports {
        if let Some(earlier) = (0..viewport)
            .find(|&earlier| (0..modes).all(|m| same(viewport * modes + m, earlier * modes + m)))
        {
            lines.push(format!(
                "viewport {}: every variant measures the same numbers as at {}, in every mode",
                size(viewport),
                size(earlier)
            ));
        }
    }
    lines
}

/// The renders of a restricted run as the run prints them: per render one line of its numbers
/// by column, `<variant> <environment>: <column>=<number> ...`, then its notes, one line per
/// element.
pub fn renders(renders: &Renders) -> String {
    let environments = renders.declaration.environments();
    let mut out = format!(
        "# ux-search campaign: a restricted run on the page {} (variant {}, environment {}): \
         {} renders, not ranked\n",
        renders.declaration.page.path,
        renders.variant.as_deref().unwrap_or("every one"),
        renders.environment.as_deref().unwrap_or("every one"),
        renders.variants.len() * renders.environments.len()
    );
    for ((id, measured), notes) in renders.variants.iter().zip(&renders.notes) {
        for (&environment, render) in renders.environments.iter().zip(measured) {
            let numbers: Vec<String> = COLUMNS
                .iter()
                .zip(render.row())
                .map(|(column, number)| format!("{column}={number}"))
                .collect();
            out.push_str(&format!(
                "{id} {}: {}\n",
                environments[environment].name(),
                numbers.join(" ")
            ));
            for note in notes.iter().filter(|note| note.environment == environment) {
                for element in &note.elements {
                    out.push_str(&format!("     {}: {element}\n", note.term));
                }
            }
        }
    }
    out
}

/// The run and the frontier as two Markdown tables, for a document that keeps the campaign's
/// numbers: what ran, then each frontier variant with its terms and a link to each of its
/// screenshots (`links` before the file name `run::screenshot_name` gives).
pub fn markdown(recorded: &Recorded, links: &str) -> Result<String, String> {
    let read = read(recorded)?;
    let declaration = &recorded.declaration;
    let ranked = &read.ranked;
    let axes: Vec<String> = declaration
        .axes
        .iter()
        .map(|axis| format!("{} {}", axis.name, axis.values.len()))
        .collect();
    let breaking = ranked
        .scored
        .iter()
        .filter(|scored| scored.terms[0] > 0)
        .count();
    let rows = [
        ("page", format!("`{}`", declaration.page.path)),
        (
            "variants",
            format!("{}: {}", read.variants.len(), axes.join(" x ")),
        ),
        (
            "environments",
            format!(
                "{}: {}",
                read.environments.len(),
                environment_names(&read.environments)
            ),
        ),
        (
            "renders measured",
            (read.variants.len() * read.environments.len()).to_string(),
        ),
        ("variants that break a hard rule", breaking.to_string()),
        (
            "frontier",
            format!(
                "variants {}, sets of terms {}",
                ranked.frontier.len(),
                sets(ranked, &ranked.frontier)
            ),
        ),
        (
            "the page as shipped",
            match read.shipped.split_once(": ") {
                Some((id, rest)) => format!("`{id}`: {rest}"),
                None => read.shipped.clone(),
            },
        ),
        ("browser", format!("`{}`", recorded.browser)),
        ("adapter", format!("`{}`", recorded.adapter)),
        ("declaration", format!("`{}`", declaration.digest)),
    ];
    let mut out = String::from("| | |\n|---|---|\n");
    for (name, value) in rows {
        out.push_str(&format!("| {name} | {value} |\n"));
    }

    let shot: Vec<&Environment> = run::screenshots_of(declaration)
        .into_iter()
        .map(|at| &read.environments[at])
        .collect();
    out.push_str("\n| rank | variant | terms |");
    for environment in &shot {
        out.push_str(&format!(" {} |", environment.name()));
    }
    out.push_str(&format!("\n|---|---|---|{}\n", "---|".repeat(shot.len())));
    for &index in &ranked.frontier {
        let scored = &ranked.scored[index];
        out.push_str(&format!(
            "| {} | `{}` | `{}` |",
            read.place(index),
            scored.id,
            terms_text(declaration, &scored.terms)
        ));
        for environment in &shot {
            out.push_str(&format!(
                " [png]({links}{}) |",
                run::screenshot_name(&scored.id, environment)
            ));
        }
        out.push('\n');
    }
    Ok(out)
}
