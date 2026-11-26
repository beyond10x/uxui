//! From a recorded campaign to a ranked report: each variant's terms, the ranking, the Pareto
//! frontier and the page as shipped among them.
//!
//! The terms, in ranking order, each a count where lower is better:
//!
//! 1. `hard`: the hard rules broken, one per rule and environment: the primary action fully in
//!    view without scrolling, no horizontal scroll, contrast (WCAG 2.x 1.4.3), an accessible
//!    name on every interactive element, every section and card of the built page shown, a
//!    route for every journey.
//! 2. `words`: words of visible text above the primary action.
//! 3. `primary_px`: the top of the primary action, in page px.
//! 4. `regions`: the regions the three journeys cross, summed.
//! 5. `small_targets`: tap targets under 44 px, at the viewports [`record::TAP_WIDTH`] wide.
//! 6. `actions`: the actions of the three journeys, summed.
//!
//! `hard` is summed over the environments; every other term is the worst value of the
//! environments it is measured in. The ranking is lexicographic over the terms, ties by
//! variant id. The frontier is the Pareto frontier of the variants that break no hard rule: a
//! variant that breaks one stands on no frontier, whatever its other terms.

use super::patch;
use super::record::{self, Recorded, Render};
use super::space::{self, AXES};
use crate::rank::{self, Scored};

/// The terms' names, in ranking order.
pub const TERMS: [&str; 6] = [
    "hard",
    "words",
    "primary_px",
    "regions",
    "small_targets",
    "actions",
];

/// A variant's terms, lower is better.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Terms {
    pub hard: u64,
    pub words: u64,
    pub primary_px: u64,
    pub regions: u64,
    pub small_targets: u64,
    pub actions: u64,
}

impl Terms {
    /// The terms in ranking order ([`TERMS`]).
    pub fn values(&self) -> Vec<u64> {
        vec![
            self.hard,
            self.words,
            self.primary_px,
            self.regions,
            self.small_targets,
            self.actions,
        ]
    }
}

/// The hard rules one render breaks.
fn broken(render: &Render) -> u64 {
    [
        !render.primary_in_view,
        render.h_scroll,
        render.contrast_failures > 0,
        render.unnamed > 0,
        render.content_missing > 0,
        render.journeys.iter().any(Option::is_none),
    ]
    .into_iter()
    .map(u64::from)
    .sum()
}

/// A variant's terms from its renders, one per environment in the order of
/// [`record::ENVIRONMENTS`]. A journey without a route adds nothing to `regions` and
/// `actions`; `hard` carries it.
pub fn terms(renders: &[Render]) -> Terms {
    let worst = |value: &dyn Fn(&Render) -> u64| renders.iter().map(value).max().unwrap_or(0);
    let routes = |render: &Render, value: fn(&record::Route) -> u64| -> u64 {
        render.journeys.iter().flatten().map(value).sum()
    };
    Terms {
        hard: renders.iter().map(broken).sum(),
        words: worst(&|render| render.words),
        primary_px: worst(&|render| render.primary_px),
        regions: worst(&|render| routes(render, |route| route.regions)),
        small_targets: renders
            .iter()
            .zip(record::ENVIRONMENTS)
            .filter(|(_, environment)| environment.width == record::TAP_WIDTH)
            .map(|(render, _)| render.small_targets)
            .max()
            .unwrap_or(0),
        actions: worst(&|render| routes(render, |route| route.actions)),
    }
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
            terms: terms(renders).values(),
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

/// The terms as the report writes them: `hard=0 words=56 ...`.
pub fn terms_text(terms: &[u64]) -> String {
    TERMS
        .iter()
        .zip(terms)
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// A patch as the report prints it under a frontier variant, one line per source file and
/// rule, without the indent: the operations on `product.json`, then the custom properties of
/// each rule of `landing.css`.
pub fn patch_lines(patch: &space::Patch) -> Vec<String> {
    let mut lines = vec![format!("product.json {}", patch::ops_json(&patch.product))];
    for rule in &patch.css {
        let values: Vec<String> = rule
            .values
            .iter()
            .map(|(property, value)| format!("{property}: {value}"))
            .collect();
        lines.push(format!(
            "landing.css {} {{ {} }}",
            rule.selector,
            values.join("; ")
        ));
    }
    lines
}

/// The variants of the space, when the record is of them: the 144 variants of
/// [`space::variants`], in their order, each with one render per environment. Any other record
/// is refused.
pub fn check(recorded: &Recorded) -> Result<Vec<space::Variant>, String> {
    let variants = space::variants();
    if recorded.variants.len() != variants.len() {
        return Err(format!(
            "the record holds {} variants, the space {}",
            recorded.variants.len(),
            variants.len()
        ));
    }
    for (variant, (id, renders)) in variants.iter().zip(&recorded.variants) {
        if *id != variant.id {
            return Err(format!(
                "the record holds `{id}` where the space has `{}`",
                variant.id
            ));
        }
        if renders.len() != record::ENVIRONMENTS.len() {
            return Err(format!(
                "{id}: {} renders, not one per environment ({})",
                renders.len(),
                record::ENVIRONMENTS.len()
            ));
        }
    }
    Ok(variants)
}

/// The report: the frontier first, each of its variants with its patch, then the full ranking.
/// It refuses a record [`check`] refuses.
pub fn report(recorded: &Recorded) -> Result<String, String> {
    let variants = check(recorded)?;
    let ranked = rank(recorded);
    let place = |index: usize| {
        ranked
            .ranking
            .iter()
            .position(|&ranked| ranked == index)
            .map_or(0, |at| at + 1)
    };

    let axes: Vec<String> = AXES
        .iter()
        .map(|axis| {
            format!(
                "{} {} ({})",
                axis.name,
                axis.ids.len(),
                axis.labels.join(", ")
            )
        })
        .collect();
    let environments: Vec<String> = record::ENVIRONMENTS
        .iter()
        .map(record::Environment::name)
        .collect();
    let journeys: Vec<&str> = record::JOURNEYS
        .iter()
        .map(|journey| journey.name)
        .collect();
    let mut out = format!(
        "# ux-search landing: the documentation landing page {} on the browser adapter\n",
        record::PAGE
    );
    out.push_str(&format!(
        "# adapter {}, browser {}\n",
        recorded.adapter, recorded.browser
    ));
    out.push_str(&format!(
        "# sources: product.json {} landing.css {}\n",
        recorded.product_digest, recorded.css_digest
    ));
    out.push_str(&format!(
        "# axes: {} = {} variants\n",
        axes.join(" x "),
        variants.len()
    ));
    out.push_str(&format!(
        "# environments {}: {}; {} renders\n",
        environments.len(),
        environments.join(", "),
        variants.len() * environments.len()
    ));
    out.push_str(&format!(
        "# journeys {}: {}\n",
        journeys.len(),
        journeys.join(", ")
    ));
    out.push_str(&format!(
        "# terms, lexicographic, lower is better: {}; hard summed over the environments, each \
         other term the worst of them, small_targets of those {} px wide; ties by variant id\n",
        TERMS.join(" "),
        record::TAP_WIDTH
    ));
    out.push_str(
        "# frontier: of the variants with hard=0, those no other of them beats on one term \
         without losing on another; equal terms stand on it together\n",
    );

    let shipped = space::shipped();
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
    out.push_str(&format!(
        "shipped {}: rank {} of {}, {before} variants before it on their terms, {equal} equal \
         to it, {} the frontier\n",
        shipped.id,
        place(at),
        variants.len(),
        if ranked.frontier.contains(&at) {
            "on"
        } else {
            "off"
        }
    ));

    out.push_str(&format!("frontier {}\n", ranked.frontier.len()));
    for &index in &ranked.frontier {
        let scored = &ranked.scored[index];
        out.push_str(&format!(
            "{:>4} {} {}\n",
            place(index),
            scored.id,
            terms_text(&scored.terms)
        ));
        for line in patch_lines(&variants[index].patch()) {
            out.push_str(&format!("     {line}\n"));
        }
    }
    out.push_str(&format!("ranking {}\n", ranked.ranking.len()));
    for (at, &index) in ranked.ranking.iter().enumerate() {
        let scored = &ranked.scored[index];
        out.push_str(&format!(
            "{:>4} {} {}\n",
            at + 1,
            scored.id,
            terms_text(&scored.terms)
        ));
    }
    Ok(out)
}
