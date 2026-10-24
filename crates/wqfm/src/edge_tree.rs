//! Unrooted binary trees as edge lists, built by stepwise addition.
//!
//! Leaves are nodes `0..m`; inner node `m + k` is added with taxon `k + 2`.

/// Inserts taxon `t` (3 ≤ t) on edge `i` of `edges`.
pub(crate) fn insert(edges: &mut Vec<(usize, usize)>, m: usize, t: usize, i: usize) {
    let inner = m + (t - 2);
    let (u, v) = edges[i];
    edges[i] = (u, inner);
    edges.push((inner, v));
    edges.push((t, inner));
}

/// The star on taxa 0, 1 and 2.
pub(crate) fn start(m: usize) -> Vec<(usize, usize)> {
    vec![(0, m), (1, m), (2, m)]
}

/// The tree as Newick, rooted at the first inner node.
pub(crate) fn to_newick(edges: &[(usize, usize)], m: usize, names: &[String]) -> String {
    let mut adj = vec![Vec::new(); 2 * m - 2];
    for &(u, v) in edges {
        adj[u].push(v);
        adj[v].push(u);
    }
    fn write(
        v: usize,
        from: usize,
        adj: &[Vec<usize>],
        m: usize,
        names: &[String],
        out: &mut String,
    ) {
        if v < m {
            out.push_str(&names[v]);
            return;
        }
        out.push('(');
        let mut first = true;
        for &w in &adj[v] {
            if w != from {
                if !first {
                    out.push(',');
                }
                first = false;
                write(w, v, adj, m, names, out);
            }
        }
        out.push(')');
    }
    let mut out = String::new();
    write(m, usize::MAX, &adj, m, names, &mut out);
    out.push(';');
    out
}
