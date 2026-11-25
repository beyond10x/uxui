//! Ranking and the frontier. Every term is a count where lower is better.

/// One evaluated variant as the ranking sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scored {
    /// The variant's id; it breaks ties.
    pub id: String,
    /// Its terms in ranking order, lower is better.
    pub terms: Vec<u64>,
}

/// The indices of `scored`, best first: lexicographic over the terms, ties broken by the id.
pub fn rank(scored: &[Scored]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..scored.len()).collect();
    order.sort_by(|&a, &b| {
        scored[a]
            .terms
            .cmp(&scored[b].terms)
            .then_with(|| scored[a].id.cmp(&scored[b].id))
    });
    order
}

/// Whether terms `a` dominate terms `b`: no term of `a` is worse and at least one is better.
pub fn dominates(a: &[u64], b: &[u64]) -> bool {
    let pairs = || a.iter().zip(b);
    a.len() == b.len() && pairs().all(|(a, b)| a <= b) && pairs().any(|(a, b)| a < b)
}

/// The Pareto frontier: the indices of the variants no other variant dominates, in rank order.
/// Neither of two variants with equal terms dominates the other, so ties stand on it together.
pub fn frontier(scored: &[Scored]) -> Vec<usize> {
    rank(scored)
        .into_iter()
        .filter(|&index| {
            !scored
                .iter()
                .any(|other| dominates(&other.terms, &scored[index].terms))
        })
        .collect()
}
