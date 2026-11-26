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
use super::record::{Recorded, Render};
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

/// One term of one render. The share of small text is in thousandths, rounded down; a render
/// without text has none.
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
        "# terms, lexicographic, lower is better: {}; hard summed over the environments, each \
         other term the worst of them; ties by variant id\n",
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

/// The lines of one variant's notes: one per element, `<environment> <term>: <element>`.
fn note_lines(recorded: &Recorded, read: &Read, index: usize, out: &mut String) {
    let notes = recorded
        .notes
        .get(index)
        .map(Vec::as_slice)
        .unwrap_or_default();
    for note in notes {
        let environment = read.environments[note.environment].name();
        for element in &note.elements {
            out.push_str(&format!("     {environment} {}: {element}\n", note.term));
        }
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
/// `always_visible_misses`, per environment, as the run recorded them. A count says how many;
/// these lines say which, so a reader can look at the element before taking the number.
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
    out.push_str(&format!("shipped {}\n", shipped.id));
    note_lines(recorded, &read, at, &mut out);
    for &index in &read.ranked.frontier {
        out.push_str(&format!(
            "{:>4} {}\n",
            read.place(index),
            read.ranked.scored[index].id
        ));
        if index == at {
            out.push_str("     the page as shipped, above\n");
        } else {
            note_lines(recorded, &read, index, &mut out);
        }
    }
    Ok(out)
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
