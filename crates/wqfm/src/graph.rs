//! An undirected view of a rooted tree, used to reroot and join subtrees.

use crate::node::Node;

/// Undirected tree as adjacency lists; leaves carry taxon ids.
pub(crate) struct Graph {
    pub(crate) adj: Vec<Vec<usize>>,
    pub(crate) leaf: Vec<Option<u32>>,
}

impl Graph {
    /// The tree with inner nodes of degree two suppressed.
    pub(crate) fn from_rooted(root: &Node) -> Self {
        let mut g = Graph {
            adj: Vec::new(),
            leaf: Vec::new(),
        };
        g.add(root, None);
        g.suppress_degree_two();
        g
    }

    fn add(&mut self, n: &Node, parent: Option<usize>) {
        let id = self.adj.len();
        self.adj.push(Vec::new());
        self.leaf.push(match n {
            Node::Leaf(t) => Some(*t),
            Node::Inner(_) => None,
        });
        if let Some(p) = parent {
            self.adj[p].push(id);
            self.adj[id].push(p);
        }
        if let Node::Inner(children) = n {
            for c in children {
                self.add(c, Some(id));
            }
        }
    }

    /// Removes inner nodes with exactly two neighbours by joining them.
    fn suppress_degree_two(&mut self) {
        while let Some(v) =
            (0..self.adj.len()).find(|&v| self.leaf[v].is_none() && self.adj[v].len() == 2)
        {
            let (a, b) = (self.adj[v][0], self.adj[v][1]);
            for (x, y) in [(a, b), (b, a)] {
                let pos = self.adj[x].iter().position(|&z| z == v).expect("edge");
                self.adj[x][pos] = y;
            }
            self.adj[v].clear();
        }
    }

    /// The tree rooted at `root`, not going towards `from`.
    pub(crate) fn rooted(&self, root: usize, from: Option<usize>) -> Node {
        if let Some(t) = self.leaf[root] {
            return Node::Leaf(t);
        }
        Node::Inner(
            self.adj[root]
                .iter()
                .filter(|&&c| Some(c) != from)
                .map(|&c| self.rooted(c, Some(root)))
                .collect(),
        )
    }

    /// The vertex of leaf `t`, if it is in the tree.
    pub(crate) fn leaf_node(&self, t: u32) -> Option<usize> {
        (0..self.leaf.len()).find(|&v| self.leaf[v] == Some(t) && !self.adj[v].is_empty())
    }
}
