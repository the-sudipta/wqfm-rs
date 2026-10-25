//! The divide-and-conquer driver of wQFM.

use std::borrow::Cow;

use crate::bipartition::initial_bipartition;
use crate::divide::divide;
use crate::fm::fm;
use crate::join::{merge, unroot};
use crate::level::Level;
use crate::node::{to_newick, Node};
use crate::options::Options;
use crate::quartet::Quartet;
use crate::topology::Topology;

/// The tree found by `wQFM-rs` as Newick (unrooted, with a basal
/// multifurcation), and its weighted quartet consistency score.
#[derive(Clone, Debug, PartialEq)]
pub struct Amalgamation {
    /// The tree in Newick, taxa written with their names.
    pub newick: String,
    /// Total weight of the input quartets the tree agrees with.
    pub score: f64,
    /// Total weight of all input quartets.
    pub total_weight: f64,
    /// False if, somewhere in the recursion, the order of Java's `HashMap`
    /// is not certain (two quartet hashes tied inside a tree bucket, where
    /// the jar's own order varies between runs, or a copied bipartition map
    /// with a tree bucket); near-ties may then go differently from the jar.
    pub java_order_exact: bool,
}

/// Runs `wQFM-rs` on weighted quartets over taxa `0..names.len()`.
pub fn amalgamate(names: &[String], quartets: &[Quartet], options: &Options) -> Amalgamation {
    let n = names.len() as u32;
    let mut used = vec![false; names.len()];
    for q in quartets {
        for t in q.key() {
            used[t as usize] = true;
        }
    }
    // Taxa that appear in no quartet are attached at the root (wQFM only
    // knows the taxa of its input).
    let taxa: Vec<u32> = (0..n).filter(|&t| used[t as usize]).collect();
    let missing: Vec<u32> = (0..n).filter(|&t| !used[t as usize]).collect();
    // Quartets with weights that are not finite are left out of the search
    // (the file reader refuses them); otherwise the caller's slice is used
    // as it is, without a copy.
    let quartets_used: Cow<'_, [Quartet]> = if quartets.iter().all(|q| q.weight.is_finite()) {
        Cow::Borrowed(quartets)
    } else {
        Cow::Owned(
            quartets
                .iter()
                .filter(|q| q.weight.is_finite())
                .copied()
                .collect(),
        )
    };
    let top = Level {
        taxa,
        quartets: quartets_used,
    };
    let mut exact = true;
    let mut tree = solve(top, options, n, 0, &mut exact);
    if !missing.is_empty() {
        let mut children = match tree {
            Node::Inner(c) => c,
            leaf => vec![leaf],
        };
        children.extend(missing.into_iter().map(Node::Leaf));
        tree = Node::Inner(children);
    }
    let newick = to_newick(&unroot(tree), names);
    let parsed = Topology::from_newick(&newick, names).expect("own Newick parses");
    let (score, total_weight) = parsed.score(quartets);
    Amalgamation {
        newick,
        score,
        total_weight,
        java_order_exact: exact,
    }
}

/// One level: a star for at most three taxa or no quartets; otherwise
/// bipartition, refine, divide, solve both sides and join them. As in
/// wQFM, the dummy taxon of a level at depth d (from 1) is `n + d`, so
/// sibling levels share it.
fn solve(level: Level<'_>, options: &Options, n: u32, depth: u32, exact: &mut bool) -> Node {
    if level.taxa.len() <= 3 || level.quartets.is_empty() {
        return Node::Inner(level.taxa.iter().map(|&t| Node::Leaf(t)).collect());
    }
    let max_id = *level.taxa.iter().max().expect("taxa") as usize;
    let mut side = initial_bipartition(&level, max_id);
    fm(&level, &mut side, options);
    let depth = depth + 1;
    let dummy = n + depth;
    let (left, right, divided_exactly) = divide(level, &side, dummy);
    *exact &= divided_exactly;
    let left_tree = solve(left, options, n, depth, exact);
    let right_tree = solve(right, options, n, depth, exact);
    merge(left_tree, right_tree, dummy)
}
