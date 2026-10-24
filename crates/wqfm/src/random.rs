//! A seeded random generator and random binary trees, for tests and benchmarks.

use crate::edge_tree;

/// SplitMix64 (Steele, Lea and Flood 2014), the generator Q-MAWS uses.
pub struct SplitMix64(u64);

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// A uniform number in [0, 1).
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }
}

/// A random unrooted binary tree on `names` by random stepwise addition
/// (each new taxon on a uniformly chosen edge), as Newick.
pub fn random_binary_tree(names: &[String], rng: &mut SplitMix64) -> String {
    let m = names.len();
    assert!(m >= 3, "a tree needs at least 3 taxa");
    let mut edges = edge_tree::start(m);
    for t in 3..m {
        let i = (rng.next_u64() % edges.len() as u64) as usize;
        edge_tree::insert(&mut edges, m, t, i);
    }
    edge_tree::to_newick(&edges, m, names)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitmix_is_reproducible() {
        let (mut a, mut b) = (SplitMix64::new(7), SplitMix64::new(7));
        assert_eq!(a.next_u64(), b.next_u64());
        let x = a.next_f64();
        assert!((0.0..1.0).contains(&x));
    }
}
