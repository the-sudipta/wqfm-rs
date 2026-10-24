//! The divide-and-conquer driver of wQFM.

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
    pub newick: String,
    pub score: f64,
    pub total_weight: f64,
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
    let top = Level {
        taxa,
        quartets: quartets
            .iter()
            .filter(|q| q.weight.is_finite())
            .copied()
            .collect(),
    };
    let mut tree = solve(top, options, n, 0);
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
    }
}

/// One level: a star for at most three taxa or no quartets; otherwise
/// bipartition, refine, divide, solve both sides and join them. As in
/// wQFM, the dummy taxon of a level at depth d (from 1) is `n + d`, so
/// sibling levels share it.
fn solve(level: Level, options: &Options, n: u32, depth: u32) -> Node {
    if level.taxa.len() <= 3 || level.quartets.is_empty() {
        return Node::Inner(level.taxa.iter().map(|&t| Node::Leaf(t)).collect());
    }
    let max_id = *level.taxa.iter().max().expect("taxa") as usize;
    let mut side = initial_bipartition(&level, max_id);
    fm(&level, &mut side, options);
    let depth = depth + 1;
    let dummy = n + depth;
    let (left, right) = divide(level, &side, dummy);
    let left_tree = solve(left, options, n, depth);
    let right_tree = solve(right, options, n, depth);
    merge(left_tree, right_tree, dummy)
}
