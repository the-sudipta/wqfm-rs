//! The exhaustive oracle: every unrooted binary tree on up to ten taxa.

use crate::edge_tree;
use crate::quartet::Quartet;
use crate::topology::Topology;

/// Every unrooted binary tree on `m` taxa (3 ≤ m ≤ 10), by stepwise
/// addition of taxa to every edge, as Newick strings with `names`.
pub fn all_binary_trees(names: &[String]) -> Vec<String> {
    let m = names.len();
    assert!(
        (3..=10).contains(&m),
        "exhaustive enumeration needs 3 to 10 taxa"
    );
    let mut trees: Vec<Vec<(usize, usize)>> = vec![edge_tree::start(m)];
    for t in 3..m {
        let mut next = Vec::with_capacity(trees.len() * (2 * t - 3));
        for edges in &trees {
            for i in 0..edges.len() {
                let mut e = edges.clone();
                edge_tree::insert(&mut e, m, t, i);
                next.push(e);
            }
        }
        trees = next;
    }
    trees
        .iter()
        .map(|edges| edge_tree::to_newick(edges, m, names))
        .collect()
}

/// The best tree of the exhaustive search (the first on equal scores) and
/// its score.
pub fn exhaustive_best(names: &[String], quartets: &[Quartet]) -> (String, f64) {
    let mut best: Option<(String, f64)> = None;
    for nwk in all_binary_trees(names) {
        let t = Topology::from_newick(&nwk, names).expect("own Newick");
        let (s, _) = t.score(quartets);
        if best.as_ref().is_none_or(|b| s > b.1) {
            best = Some((nwk, s));
        }
    }
    best.expect("at least one tree")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::newick::Tree;

    #[test]
    fn enumeration_counts_trees() {
        // (2m − 5)!! unrooted binary trees.
        for (m, count) in [(3, 1), (4, 3), (5, 15), (6, 105), (7, 945), (8, 10_395)] {
            let nm: Vec<String> = (0..m).map(|i| format!("t{i}")).collect();
            let all = all_binary_trees(&nm);
            assert_eq!(all.len(), count);
            let mut distinct = std::collections::BTreeSet::new();
            for t in &all {
                let mut splits = Tree::parse(t).unwrap().splits();
                splits.sort();
                distinct.insert(format!("{splits:?}"));
                assert_eq!(Topology::from_newick(t, &nm).unwrap().taxa(), m);
            }
            assert_eq!(distinct.len(), count, "trees for m = {m} are distinct");
        }
    }
}
