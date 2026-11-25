//! The ranking and the frontier, on hand-built terms.

use ux_search::rank::{Scored, dominates, frontier, rank};

fn scored(id: &str, terms: &[u64]) -> Scored {
    Scored {
        id: id.to_owned(),
        terms: terms.to_vec(),
    }
}

fn ids(scored: &[Scored], order: &[usize]) -> Vec<String> {
    order
        .iter()
        .map(|&index| scored[index].id.clone())
        .collect()
}

#[test]
fn the_ranking_is_lexicographic_over_the_terms_then_by_id() {
    let all = [
        scored("d", &[0, 2, 0]),
        scored("c", &[1, 0, 0]),
        scored("b", &[0, 2, 0]),
        scored("a", &[0, 3, 0]),
        scored("e", &[0, 0, 9]),
    ];
    assert_eq!(ids(&all, &rank(&all)), ["e", "b", "d", "a", "c"]);
}

#[test]
fn an_empty_list_ranks_to_nothing() {
    assert!(rank(&[]).is_empty());
    assert!(frontier(&[]).is_empty());
}

#[test]
fn a_variant_dominates_only_when_no_term_is_worse_and_one_is_better() {
    assert!(dominates(&[0, 1], &[0, 2]));
    assert!(dominates(&[0, 1], &[1, 1]));
    assert!(!dominates(&[0, 1], &[0, 1]), "equal terms dominate nothing");
    assert!(
        !dominates(&[0, 2], &[1, 1]),
        "a trade-off dominates nothing"
    );
    assert!(!dominates(&[1, 1], &[0, 2]));
}

#[test]
fn the_frontier_is_the_non_dominated_set_in_rank_order_with_ties_kept() {
    let all = [
        scored("worse", &[1, 5, 5]),
        scored("trade-b", &[0, 4, 1]),
        scored("trade-a", &[0, 1, 6]),
        scored("tie-2", &[0, 1, 6]),
        scored("dominated", &[0, 4, 2]),
    ];
    assert_eq!(ids(&all, &frontier(&all)), ["tie-2", "trade-a", "trade-b"]);
}
