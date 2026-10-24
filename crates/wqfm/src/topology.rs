//! Quartet tests on a fixed tree, and the weighted quartet consistency score.

use std::collections::HashMap;

use crate::graph::Graph;
use crate::newick::Tree;
use crate::node::Node;
use crate::quartet::Quartet;

/// A tree on taxa `0..m` with leaf-to-leaf path lengths, for quartet tests.
pub struct Topology {
    m: usize,
    dist: Vec<u32>,
}

impl Topology {
    /// Reads a Newick tree whose leaves are exactly `names`.
    pub fn from_newick(newick: &str, names: &[String]) -> Result<Self, String> {
        let tree = Tree::parse(newick).map_err(|e| e.to_string())?;
        let index: HashMap<&str, u32> = names
            .iter()
            .enumerate()
            .map(|(i, n)| (n.as_str(), i as u32))
            .collect();
        fn convert(tree: &Tree, n: usize, index: &HashMap<&str, u32>) -> Result<Node, String> {
            if tree.is_leaf(n) {
                let name = tree.nodes[n].label.as_deref().unwrap_or("");
                return index
                    .get(name)
                    .map(|&t| Node::Leaf(t))
                    .ok_or_else(|| format!("unknown leaf {name}"));
            }
            tree.nodes[n]
                .children
                .iter()
                .map(|&c| convert(tree, c, index))
                .collect::<Result<Vec<_>, _>>()
                .map(Node::Inner)
        }
        let root = convert(&tree, tree.root, &index)?;
        Ok(Self::from_node(&root, names.len()))
    }

    pub(crate) fn from_node(root: &Node, m: usize) -> Self {
        let g = Graph::from_rooted(root);
        let mut dist = vec![u32::MAX; m * m];
        for t in 0..m as u32 {
            let Some(start) = g.leaf_node(t) else {
                continue;
            };
            // Breadth-first search over the tree.
            let mut d = vec![u32::MAX; g.adj.len()];
            d[start] = 0;
            let mut queue = std::collections::VecDeque::from([start]);
            while let Some(v) = queue.pop_front() {
                for &w in &g.adj[v] {
                    if d[w] == u32::MAX {
                        d[w] = d[v] + 1;
                        queue.push_back(w);
                    }
                }
            }
            for (v, &dv) in d.iter().enumerate() {
                if let Some(u) = g.leaf[v] {
                    if !g.adj[v].is_empty() || g.adj.len() == 1 {
                        dist[t as usize * m + u as usize] = dv;
                    }
                }
            }
        }
        Self { m, dist }
    }

    /// The number of taxa.
    pub fn taxa(&self) -> usize {
        self.m
    }

    fn d(&self, a: u32, b: u32) -> u32 {
        self.dist[a as usize * self.m + b as usize]
    }

    /// True if the tree induces ab|cd (four-point condition on path lengths;
    /// false for a star on the four taxa).
    pub fn displays(&self, q: &Quartet) -> bool {
        let [a, b] = q.l;
        let [c, d] = q.r;
        let s1 = self.d(a, b) as u64 + self.d(c, d) as u64;
        let s2 = self.d(a, c) as u64 + self.d(b, d) as u64;
        let s3 = self.d(a, d) as u64 + self.d(b, c) as u64;
        s1 < s2 && s1 < s3
    }

    /// The topology the tree induces on taxa a < b < c < d: 0 for ab|cd,
    /// 1 for ac|bd, 2 for ad|bc; `None` if the tree is a star on them.
    pub fn induced(&self, q: [usize; 4]) -> Option<usize> {
        let [a, b, c, d] = q.map(|t| t as u32);
        let s = [
            self.d(a, b) as u64 + self.d(c, d) as u64,
            self.d(a, c) as u64 + self.d(b, d) as u64,
            self.d(a, d) as u64 + self.d(b, c) as u64,
        ];
        (0..3).find(|&t| (0..3).all(|o| o == t || s[t] < s[o]))
    }

    /// Weighted quartet consistency score and total weight.
    pub fn score(&self, quartets: &[Quartet]) -> (f64, f64) {
        let mut score = 0.0;
        let mut total = 0.0;
        for q in quartets {
            total += q.weight;
            if self.displays(q) {
                score += q.weight;
            }
        }
        (score, total)
    }
}

/// The topology that `tree` induces on every 4-set of taxa, each with
/// weight 1, in rank order.
pub fn induced_quartets(tree: &Topology) -> Vec<Quartet> {
    let m = tree.m as u32;
    let mut out = Vec::new();
    for d in 3..m {
        for c in 2..d {
            for b in 1..c {
                for a in 0..b {
                    for q in [
                        Quartet::new(a, b, c, d, 1.0),
                        Quartet::new(a, c, b, d, 1.0),
                        Quartet::new(a, d, b, c, 1.0),
                    ] {
                        if tree.displays(&q) {
                            out.push(q);
                        }
                    }
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(m: usize) -> Vec<String> {
        (0..m).map(|i| format!("t{i}")).collect()
    }

    #[test]
    fn displays_follows_the_tree() {
        let nm = names(5);
        let t = Topology::from_newick("((t0,t1),t2,(t3,t4));", &nm).unwrap();
        assert!(t.displays(&Quartet::new(0, 1, 3, 4, 1.0)));
        assert!(t.displays(&Quartet::new(0, 1, 2, 3, 1.0)));
        assert!(!t.displays(&Quartet::new(0, 2, 1, 3, 1.0)));
        assert_eq!(t.induced([0, 1, 3, 4]), Some(0));
        let star = Topology::from_newick("(t0,t1,t2,t3,t4);", &nm).unwrap();
        assert!(!star.displays(&Quartet::new(0, 1, 2, 3, 1.0)));
        assert_eq!(star.induced([0, 1, 2, 3]), None);
    }

    #[test]
    fn unknown_leaves_are_errors() {
        assert!(Topology::from_newick("(t0,t1,x);", &names(3)).is_err());
    }
}
