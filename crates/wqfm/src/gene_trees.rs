//! Gene trees to weighted quartets: for every 4-set of taxa, how many gene
//! trees show each of its three topologies (wQFM's `generate_wqrts.py`).

use std::collections::HashMap;

use rayon::prelude::*;

use crate::newick::Tree;
use crate::node::Node;
use crate::topology::Topology;

/// Quartet counts over taxon ids: sorted 4-set a < b < c < d -> counts of
/// ab|cd, ac|bd, ad|bc and of trees that leave the 4-set unresolved.
pub struct QuartetCounts {
    pub names: Vec<String>,
    pub counts: HashMap<[u32; 4], [u32; 4]>,
}

/// Counts the quartets of every gene tree; taxa may be missing from some
/// trees. A 4-set that a gene tree leaves unresolved (a polytomy) is counted
/// in the fourth slot.
pub fn count_quartets(trees: &[Tree]) -> Result<QuartetCounts, String> {
    let mut names: Vec<String> = Vec::new();
    let mut index: HashMap<String, u32> = HashMap::new();
    let mut leaf_ids: Vec<Vec<u32>> = Vec::with_capacity(trees.len());
    for (i, tree) in trees.iter().enumerate() {
        let mut ids = Vec::new();
        for name in tree.leaf_names() {
            let next = names.len() as u32;
            let id = *index.entry(name.clone()).or_insert_with(|| {
                names.push(name.clone());
                next
            });
            ids.push(id);
        }
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        if sorted.windows(2).any(|w| w[0] == w[1]) {
            return Err(format!("gene tree {}: a taxon appears twice", i + 1));
        }
        leaf_ids.push(sorted);
    }
    let m = names.len();
    let counts = trees
        .par_iter()
        .zip(&leaf_ids)
        .map(|(tree, ids)| one_tree(tree, ids, &index, m))
        .reduce(HashMap::new, merge);
    Ok(QuartetCounts { names, counts })
}

/// The quartets of one gene tree whose leaves are `ids` (sorted).
fn one_tree(
    tree: &Tree,
    ids: &[u32],
    index: &HashMap<String, u32>,
    m: usize,
) -> HashMap<[u32; 4], [u32; 4]> {
    fn convert(tree: &Tree, n: usize, index: &HashMap<String, u32>) -> Node {
        if tree.is_leaf(n) {
            let name = tree.nodes[n].label.clone().unwrap_or_default();
            return Node::Leaf(index[&name]);
        }
        Node::Inner(
            tree.nodes[n]
                .children
                .iter()
                .map(|&c| convert(tree, c, index))
                .collect(),
        )
    }
    let topo = Topology::from_node(&convert(tree, tree.root, index), m);
    let mut out = HashMap::new();
    let k = ids.len();
    for i in 0..k {
        for j in i + 1..k {
            for x in j + 1..k {
                for y in x + 1..k {
                    let q = [ids[i], ids[j], ids[x], ids[y]];
                    let t = topo.induced(q.map(|v| v as usize)).unwrap_or(3);
                    out.entry(q).or_insert([0; 4])[t] += 1;
                }
            }
        }
    }
    out
}

fn merge(
    mut a: HashMap<[u32; 4], [u32; 4]>,
    b: HashMap<[u32; 4], [u32; 4]>,
) -> HashMap<[u32; 4], [u32; 4]> {
    if a.len() < b.len() {
        return merge(b, a);
    }
    for (q, c) in b {
        let e = a.entry(q).or_insert([0; 4]);
        for t in 0..4 {
            e[t] += c[t];
        }
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(trees: &[&str]) -> Vec<Tree> {
        trees.iter().map(|t| Tree::parse(t).unwrap()).collect()
    }

    #[test]
    fn counts_each_topology_once_per_tree() {
        let c = count_quartets(&parse(&[
            "((a,b),(c,d));",
            "((a,b),c,d);",
            "((a,c),(b,d));",
            "(a,b,c,d);",
        ]))
        .unwrap();
        assert_eq!(c.names, ["a", "b", "c", "d"]);
        assert_eq!(c.counts[&[0, 1, 2, 3]], [2, 1, 0, 1]);
    }

    #[test]
    fn missing_taxa_and_duplicates() {
        let c = count_quartets(&parse(&["((a,b),(c,d),e);", "((a,e),(c,d));"])).unwrap();
        assert_eq!(c.counts.len(), 5);
        assert!(count_quartets(&parse(&["((a,b),(a,d));"])).is_err());
    }
}
