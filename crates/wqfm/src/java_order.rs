//! The iteration order of Java's `HashMap`, which wQFM v1.4 uses for two
//! orders that change its results: the taxa of a new level (keys of the
//! bipartition map) and the merged dummy quartets (keys of a map of
//! quartets). Floating-point sums follow the quartet order, so near-ties
//! are broken as in the jar only if both orders are the same.
//!
//! `JavaMap` replays the insertions of distinct keys into a `HashMap`
//! (OpenJDK 19 and later) and reports the iteration order. It follows the
//! documented behaviour of that class: a power-of-two table, bucket
//! `spread(hash) & (n - 1)`, buckets kept as lists in insertion order, a
//! bucket with more than 8 entries turned into a red-black tree ordered by
//! hash (or the table doubled while it is smaller than 64), the tree's root
//! moved to the front of its bucket, buckets split in order when the table
//! doubles and trees of 6 or fewer entries turned back into lists.
//! Iteration visits the buckets in order and each bucket in its list order.
//!
//! One case cannot be reproduced: two different keys with the same hash in
//! one tree. Java then orders them by `System.identityHashCode`, which
//! changes from run to run, so the jar itself is not deterministic; the
//! simulation reports it.

use crate::quartet::Quartet;

const LOAD_FACTOR: f64 = 0.75;
const TREEIFY_THRESHOLD: usize = 8;
const UNTREEIFY_THRESHOLD: usize = 6;
const MIN_TREEIFY_CAPACITY: usize = 64;

/// Java's `HashMap.hash`: the hash code with its high half folded in.
fn spread(h: i32) -> i32 {
    h ^ ((h as u32) >> 16) as i32
}

fn threshold_of(capacity: usize) -> usize {
    (capacity as f64 * LOAD_FACTOR) as usize
}

/// How a replay went.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Replay {
    /// Some bucket became a tree.
    pub(crate) tree: bool,
    /// Two different keys had the same hash inside a tree, where Java's own
    /// order is not deterministic.
    pub(crate) tie: bool,
}

/// A replay of a Java `HashMap` that only tracks the order of its keys.
/// Keys are `0..hashes.len()`, all distinct.
struct JavaMap<'a> {
    hashes: &'a [i32],
    capacity: usize,
    threshold: usize,
    size: usize,
    /// Each bucket's keys in list (iteration) order.
    buckets: Vec<Vec<usize>>,
    is_tree: Vec<bool>,
    parent: Vec<Option<usize>>,
    left: Vec<Option<usize>>,
    right: Vec<Option<usize>>,
    red: Vec<bool>,
    replay: Replay,
}

impl<'a> JavaMap<'a> {
    /// `new HashMap<>()` (capacity 16 at the first insertion), or the
    /// presized table of `new HashMap<>(m)` for a map of `copy_of` entries
    /// (OpenJDK 19+: `tableSizeFor(ceil(k / 0.75))`).
    fn new(hashes: &'a [i32], copy_of: Option<usize>) -> Self {
        let capacity = match copy_of {
            Some(k) if k > 0 => ((k as f64 / LOAD_FACTOR).ceil() as usize).next_power_of_two(),
            _ => 16,
        };
        let n = hashes.len();
        Self {
            hashes,
            capacity,
            threshold: threshold_of(capacity),
            size: 0,
            buckets: vec![Vec::new(); capacity],
            is_tree: vec![false; capacity],
            parent: vec![None; n],
            left: vec![None; n],
            right: vec![None; n],
            red: vec![false; n],
            replay: Replay::default(),
        }
    }

    fn bucket_of(&self, k: usize, capacity: usize) -> usize {
        self.hashes[k] as u32 as usize & (capacity - 1)
    }

    /// Inserts a key that is not yet in the map.
    fn insert(&mut self, k: usize) {
        let b = self.bucket_of(k, self.capacity);
        if self.is_tree[b] {
            self.tree_insert(b, k);
        } else {
            let before = self.buckets[b].len();
            self.buckets[b].push(k);
            if before >= TREEIFY_THRESHOLD {
                if self.capacity < MIN_TREEIFY_CAPACITY {
                    self.resize();
                } else {
                    self.treeify(b);
                }
            }
        }
        self.size += 1;
        if self.size > self.threshold {
            self.resize();
        }
    }

    /// The direction of `k` from tree node `p`: hash order, ties unordered.
    fn direction(&mut self, k: usize, p: usize) -> bool {
        let (h, ph) = (self.hashes[k], self.hashes[p]);
        if ph == h {
            self.replay.tie = true;
        }
        // Java goes left when the node's hash is larger (and, on a tie, by
        // identity hash, which cannot be known: left is taken here).
        ph >= h
    }

    fn root_of(&self, mut x: usize) -> usize {
        while let Some(p) = self.parent[x] {
            x = p;
        }
        x
    }

    /// Adds `k` to the tree of bucket `b`: as a leaf under the node where the
    /// search ends, placed right after that node in the bucket's list.
    fn tree_insert(&mut self, b: usize, k: usize) {
        let root = self.root_of(self.buckets[b][0]);
        let mut p = root;
        loop {
            let go_left = self.direction(k, p);
            let child = if go_left { self.left[p] } else { self.right[p] };
            match child {
                Some(c) => p = c,
                None => {
                    if go_left {
                        self.left[p] = Some(k);
                    } else {
                        self.right[p] = Some(k);
                    }
                    self.parent[k] = Some(p);
                    let at = self.buckets[b].iter().position(|&x| x == p).expect("node");
                    self.buckets[b].insert(at + 1, k);
                    let root = self.balance_insertion(root, k);
                    self.move_root_to_front(b, root);
                    return;
                }
            }
        }
    }

    /// Builds the tree of bucket `b` from its list, in list order.
    fn treeify(&mut self, b: usize) {
        self.replay.tree = true;
        self.is_tree[b] = true;
        let keys = self.buckets[b].clone();
        let mut root: Option<usize> = None;
        for &x in &keys {
            self.left[x] = None;
            self.right[x] = None;
            match root {
                None => {
                    self.parent[x] = None;
                    self.red[x] = false;
                    root = Some(x);
                }
                Some(r) => {
                    let mut p = r;
                    loop {
                        let go_left = self.direction(x, p);
                        let child = if go_left { self.left[p] } else { self.right[p] };
                        match child {
                            Some(c) => p = c,
                            None => {
                                self.parent[x] = Some(p);
                                if go_left {
                                    self.left[p] = Some(x);
                                } else {
                                    self.right[p] = Some(x);
                                }
                                root = Some(self.balance_insertion(r, x));
                                break;
                            }
                        }
                    }
                }
            }
        }
        if let Some(r) = root {
            self.move_root_to_front(b, r);
        }
    }

    fn move_root_to_front(&mut self, b: usize, root: usize) {
        if self.buckets[b][0] != root {
            let at = self.buckets[b]
                .iter()
                .position(|&x| x == root)
                .expect("root");
            self.buckets[b].remove(at);
            self.buckets[b].insert(0, root);
        }
    }

    fn rotate_left(&mut self, mut root: usize, p: usize) -> usize {
        if let Some(r) = self.right[p] {
            let rl = self.left[r];
            self.right[p] = rl;
            if let Some(rl) = rl {
                self.parent[rl] = Some(p);
            }
            let pp = self.parent[p];
            self.parent[r] = pp;
            match pp {
                None => {
                    root = r;
                    self.red[r] = false;
                }
                Some(pp) if self.left[pp] == Some(p) => self.left[pp] = Some(r),
                Some(pp) => self.right[pp] = Some(r),
            }
            self.left[r] = Some(p);
            self.parent[p] = Some(r);
        }
        root
    }

    fn rotate_right(&mut self, mut root: usize, p: usize) -> usize {
        if let Some(l) = self.left[p] {
            let lr = self.right[l];
            self.left[p] = lr;
            if let Some(lr) = lr {
                self.parent[lr] = Some(p);
            }
            let pp = self.parent[p];
            self.parent[l] = pp;
            match pp {
                None => {
                    root = l;
                    self.red[l] = false;
                }
                Some(pp) if self.right[pp] == Some(p) => self.right[pp] = Some(l),
                Some(pp) => self.left[pp] = Some(l),
            }
            self.right[l] = Some(p);
            self.parent[p] = Some(l);
        }
        root
    }

    /// Red-black repair after inserting `x`; returns the root.
    fn balance_insertion(&mut self, mut root: usize, mut x: usize) -> usize {
        self.red[x] = true;
        loop {
            let Some(mut xp) = self.parent[x] else {
                self.red[x] = false;
                return x;
            };
            let Some(xpp) = self.parent[xp].filter(|_| self.red[xp]) else {
                return root;
            };
            if self.left[xpp] == Some(xp) {
                match self.right[xpp].filter(|&u| self.red[u]) {
                    Some(uncle) => {
                        self.red[uncle] = false;
                        self.red[xp] = false;
                        self.red[xpp] = true;
                        x = xpp;
                    }
                    None => {
                        let mut xpp_opt = Some(xpp);
                        if self.right[xp] == Some(x) {
                            x = xp;
                            root = self.rotate_left(root, x);
                            xp = self.parent[x].expect("parent after rotation");
                            xpp_opt = self.parent[xp];
                        }
                        self.red[xp] = false;
                        if let Some(g) = xpp_opt {
                            self.red[g] = true;
                            root = self.rotate_right(root, g);
                        }
                    }
                }
            } else {
                match self.left[xpp].filter(|&u| self.red[u]) {
                    Some(uncle) => {
                        self.red[uncle] = false;
                        self.red[xp] = false;
                        self.red[xpp] = true;
                        x = xpp;
                    }
                    None => {
                        let mut xpp_opt = Some(xpp);
                        if self.left[xp] == Some(x) {
                            x = xp;
                            root = self.rotate_right(root, x);
                            xp = self.parent[x].expect("parent after rotation");
                            xpp_opt = self.parent[xp];
                        }
                        self.red[xp] = false;
                        if let Some(g) = xpp_opt {
                            self.red[g] = true;
                            root = self.rotate_left(root, g);
                        }
                    }
                }
            }
        }
    }

    /// Doubles the table; each bucket splits into a low and a high bucket,
    /// both in list order.
    fn resize(&mut self) {
        let old = self.capacity;
        let new = old * 2;
        let mut buckets = vec![Vec::new(); new];
        let mut is_tree = vec![false; new];
        let old_buckets = std::mem::take(&mut self.buckets);
        let old_tree = std::mem::take(&mut self.is_tree);
        for (j, keys) in old_buckets.into_iter().enumerate() {
            let (lo, hi): (Vec<usize>, Vec<usize>) = keys
                .into_iter()
                .partition(|&k| self.hashes[k] as u32 as usize & old == 0);
            let both = !lo.is_empty() && !hi.is_empty();
            buckets[j] = lo;
            buckets[j + old] = hi;
            if old_tree[j] {
                for idx in [j, j + old] {
                    let n = buckets[idx].len();
                    if n == 0 {
                        continue;
                    }
                    // Small parts become lists; a part that kept every key
                    // keeps its tree as it is; otherwise the tree is rebuilt.
                    is_tree[idx] = n > UNTREEIFY_THRESHOLD;
                }
                self.buckets = std::mem::take(&mut buckets);
                self.is_tree = std::mem::take(&mut is_tree);
                for idx in [j, j + old] {
                    if self.is_tree[idx] && both {
                        self.treeify(idx);
                    }
                }
                buckets = std::mem::take(&mut self.buckets);
                is_tree = std::mem::take(&mut self.is_tree);
            }
        }
        self.buckets = buckets;
        self.is_tree = is_tree;
        self.capacity = new;
        self.threshold = threshold_of(new);
    }

    fn order(&self) -> Vec<usize> {
        self.buckets.iter().flatten().copied().collect()
    }
}

/// The iteration order of distinct keys with hash codes `hashes`, inserted
/// in index order into `new HashMap<>()` (or, with `copy_of`, into the
/// presized table of a copy).
fn replay(hashes: &[i32], insert_order: &[usize], copy_of: Option<usize>) -> (Vec<usize>, Replay) {
    let spread_hashes: Vec<i32> = hashes.iter().map(|&h| spread(h)).collect();
    let mut map = JavaMap::new(&spread_hashes, copy_of);
    for &k in insert_order {
        map.insert(k);
    }
    (map.order(), map.replay)
}

/// The key order of wQFM's bipartition map for a level whose taxa list is
/// `taxa`: filled in that order into `new HashMap<>()`, then copied (FM
/// always passes on a copy). A copy of a copy has the same order unless a
/// bucket is a tree, which the replay reports.
pub(crate) fn bipartition_key_order(taxa: &[u32]) -> (Vec<u32>, Replay) {
    let hashes: Vec<i32> = taxa.iter().map(|&t| t as i32).collect();
    let all: Vec<usize> = (0..taxa.len()).collect();
    let (first, r1) = replay(&hashes, &all, None);
    let (copied, r2) = replay(&hashes, &first, Some(taxa.len()));
    let replay = Replay {
        tree: r1.tree || r2.tree,
        tie: r1.tie || r2.tie,
    };
    (copied.into_iter().map(|i| taxa[i]).collect(), replay)
}

/// Java's `Quartet.hashCode` in wQFM v1.4.
fn quartet_hash(q: &Quartet) -> i32 {
    let arr = |a: [u32; 2]| {
        31i32
            .wrapping_mul(31i32.wrapping_add(a[0] as i32))
            .wrapping_add(a[1] as i32)
    };
    let h = 97i32.wrapping_mul(3).wrapping_add(arr(q.l));
    97i32.wrapping_mul(h).wrapping_add(arr(q.r))
}

/// The iteration order of distinct quartets inserted in the given order
/// into `new HashMap<>()`, as indices into `quartets`.
pub(crate) fn quartet_key_order(quartets: &[Quartet]) -> (Vec<usize>, Replay) {
    let hashes: Vec<i32> = quartets.iter().map(quartet_hash).collect();
    let all: Vec<usize> = (0..quartets.len()).collect();
    replay(&hashes, &all, None)
}

/// The iteration orders of a `new HashMap<>()` filled with distinct keys
/// whose hash codes are `hashes` (in index order), and of
/// `new HashMap<>(thatMap)`, with whether two hashes tied inside a tree
/// and whether any bucket became a tree. Used to check the model against
/// Java (`examples/java_order_check.rs`).
#[doc(hidden)]
pub fn java_hashmap_orders(hashes: &[i32]) -> (Vec<usize>, Vec<usize>, bool, bool) {
    let all: Vec<usize> = (0..hashes.len()).collect();
    let (first, r1) = replay(hashes, &all, None);
    let (copy, r2) = replay(hashes, &first, Some(hashes.len()));
    (first, copy, r1.tie || r2.tie, r1.tree || r2.tree)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_ids_stay_ascending_and_large_ones_wrap() {
        assert_eq!(bipartition_key_order(&[3, 1, 2]).0, vec![1, 2, 3]);
        // 10 keys: table 16, so 17 sits in bucket 1 after 0 and 16 in bucket 0.
        let taxa = [0, 1, 2, 3, 4, 5, 6, 7, 16, 17];
        assert_eq!(
            bipartition_key_order(&taxa).0,
            vec![0, 16, 1, 17, 2, 3, 4, 5, 6, 7]
        );
    }

    #[test]
    fn quartet_hash_matches_java() {
        // Arrays.hashCode({1, 2}) = 994, ({3, 4}) = 1058;
        // 97 * (97 * 3 + 994) + 1058 = 125703.
        assert_eq!(quartet_hash(&Quartet::new(1, 2, 3, 4, 1.0)), 125_703);
    }

    #[test]
    fn crowded_small_table_doubles_instead_of_tree() {
        // Nine keys in bucket 0 of a 16-table: the table doubles to 32, then
        // 64 is not reached, so no tree.
        let hashes: Vec<i32> = (0..9).map(|i| i * 16).collect();
        let (order, r) = replay(&hashes, &(0..9).collect::<Vec<_>>(), None);
        assert!(!r.tree);
        assert_eq!(order, vec![0, 2, 4, 6, 8, 1, 3, 5, 7]);
    }
}
