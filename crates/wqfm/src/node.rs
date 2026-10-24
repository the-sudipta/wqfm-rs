//! Rooted trees built during the recursion, and their Newick form.

/// A rooted tree on taxon ids.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Node {
    Leaf(u32),
    Inner(Vec<Node>),
}

/// The tree as Newick with taxon `t` written as `names[t]`.
pub(crate) fn to_newick(tree: &Node, names: &[String]) -> String {
    fn write(n: &Node, names: &[String], out: &mut String) {
        match n {
            Node::Leaf(t) => out.push_str(&names[*t as usize]),
            Node::Inner(children) => {
                out.push('(');
                for (i, c) in children.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write(c, names, out);
                }
                out.push(')');
            }
        }
    }
    let mut out = String::new();
    write(tree, names, &mut out);
    out.push(';');
    out
}
