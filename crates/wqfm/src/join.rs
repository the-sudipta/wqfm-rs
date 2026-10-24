//! Joining two subtrees at their dummy taxon, and the unrooted output form.

use crate::graph::Graph;
use crate::node::Node;

/// The part of `tree` seen from the neighbour of leaf `dummy`.
fn side_without(tree: Node, dummy: u32) -> Node {
    let g = Graph::from_rooted(&tree);
    let d = g.leaf_node(dummy).expect("dummy leaf in subtree");
    let p = g.adj[d][0];
    g.rooted(p, Some(d))
}

/// Removes both dummies and connects their neighbours.
pub(crate) fn merge(left: Node, right: Node, dummy: u32) -> Node {
    Node::Inner(vec![side_without(left, dummy), side_without(right, dummy)])
}

/// Unrooted form: a root with two children is replaced by the multifurcation
/// of one inner child with the other.
pub(crate) fn unroot(tree: Node) -> Node {
    let g = Graph::from_rooted(&tree);
    // Root at the first inner node with at least three neighbours.
    match (0..g.adj.len()).find(|&v| g.leaf[v].is_none() && g.adj[v].len() >= 3) {
        Some(v) => g.rooted(v, None),
        None => tree,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::to_newick;

    #[test]
    fn merging_joins_the_subtrees_at_the_dummy() {
        // Left (a, b, X), right (c, d, X) give ((a,b),(c,d)).
        let left = Node::Inner(vec![Node::Leaf(0), Node::Leaf(1), Node::Leaf(9)]);
        let right = Node::Inner(vec![Node::Leaf(2), Node::Leaf(3), Node::Leaf(9)]);
        let nm: Vec<String> = (0..10).map(|i| format!("t{i}")).collect();
        let merged = merge(left, right, 9);
        assert_eq!(to_newick(&unroot(merged), &nm), "((t2,t3),t0,t1);");
    }
}
