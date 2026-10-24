//! The iteration order of Java's `HashMap`, which wQFM v1.4 uses for two
//! orders that change its results: the taxa of a new level (keys of the
//! bipartition map) and the merged dummy quartets (keys of a map of
//! quartets). Floating-point sums follow the quartet order, so near-ties
//! are broken as in the jar only if both orders are the same.
//!
//! Model (OpenJDK 19 and later): keys sit in buckets `spread(hash) & (n - 1)`
//! of a power-of-two table; iteration visits buckets in order and each
//! bucket in insertion order; resizing keeps that relative order. So the
//! iteration order is a stable sort of the insertion order by bucket index
//! of the final table size.

use crate::quartet::Quartet;

const LOAD_FACTOR: f64 = 0.75;
/// A bucket with this many entries is turned into a tree (Java's
/// `TREEIFY_THRESHOLD`), or the table is doubled if it is smaller than 64.
const TREEIFY_THRESHOLD: usize = 8;
const MIN_TREEIFY_CAPACITY: usize = 64;

/// Java's `HashMap.hash`: the hash code with its high half folded in.
fn spread(h: i32) -> u32 {
    let h = h as u32;
    h ^ (h >> 16)
}

/// The smallest power of two at least `c` (Java's `tableSizeFor`).
fn table_size_for(c: usize) -> usize {
    c.max(1).next_power_of_two()
}

/// Table size of `new HashMap<>()` after `k` distinct insertions without
/// crowded buckets: 16, doubled while `k` exceeds three quarters of it.
fn grown_capacity(k: usize) -> usize {
    let mut n = 16;
    while k as f64 > LOAD_FACTOR * n as f64 {
        n *= 2;
    }
    n
}

/// Table size of `new HashMap<>(m)` for a map of `k` entries (OpenJDK 19+:
/// `tableSizeFor(ceil(k / 0.75))`).
fn copied_capacity(k: usize) -> usize {
    table_size_for((k as f64 / LOAD_FACTOR).ceil() as usize)
}

fn sort_by_bucket<T: Copy>(items: &mut [T], n: usize, hash: impl Fn(T) -> u32) {
    items.sort_by_key(|&x| hash(x) as usize & (n - 1));
}

/// The key order of wQFM's bipartition map for a level whose taxa list is
/// `taxa`: filled in that order into `new HashMap<>()`, then copied (FM
/// always passes on a copy).
pub(crate) fn bipartition_key_order(taxa: &[u32]) -> Vec<u32> {
    let mut keys = taxa.to_vec();
    let hash = |t: u32| spread(t as i32);
    let k = keys.len();
    sort_by_bucket(&mut keys, grown_capacity(k), hash);
    sort_by_bucket(&mut keys, copied_capacity(k), hash);
    keys
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
/// into `new HashMap<>()`, as indices into `quartets`. Also returns whether
/// a bucket became a tree, where Java's order within that bucket is not
/// modelled (it has not happened on any test input).
pub(crate) fn quartet_key_order(quartets: &[Quartet]) -> (Vec<usize>, bool) {
    let hashes: Vec<u32> = quartets.iter().map(|q| spread(quartet_hash(q))).collect();
    let mut n = 16;
    let mut fill = vec![0usize; n];
    let mut tree = false;
    let rebuild = |n: usize, k: usize| -> Vec<usize> {
        let mut f = vec![0usize; n];
        for &h in &hashes[..k] {
            f[h as usize & (n - 1)] += 1;
        }
        f
    };
    for (k, &h) in hashes.iter().enumerate() {
        let b = h as usize & (n - 1);
        let before = fill[b];
        fill[b] += 1;
        if before >= TREEIFY_THRESHOLD {
            if n < MIN_TREEIFY_CAPACITY {
                n *= 2;
                fill = rebuild(n, k + 1);
            } else {
                tree = true;
            }
        }
        if (k + 1) as f64 > LOAD_FACTOR * n as f64 {
            n *= 2;
            fill = rebuild(n, k + 1);
        }
    }
    let mut order: Vec<usize> = (0..quartets.len()).collect();
    sort_by_bucket(&mut order, n, |i| hashes[i]);
    (order, tree)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacities_follow_openjdk() {
        assert_eq!((grown_capacity(12), grown_capacity(13)), (16, 32));
        assert_eq!((copied_capacity(12), copied_capacity(13)), (16, 32));
        assert_eq!(copied_capacity(25), 64);
    }

    #[test]
    fn small_ids_stay_ascending_and_large_ones_wrap() {
        assert_eq!(bipartition_key_order(&[3, 1, 2]), vec![1, 2, 3]);
        // 10 keys: table 16, so 17 sits in bucket 1 after 0 and 16 in bucket 0.
        let taxa = [0, 1, 2, 3, 4, 5, 6, 7, 16, 17];
        assert_eq!(
            bipartition_key_order(&taxa),
            vec![0, 16, 1, 17, 2, 3, 4, 5, 6, 7]
        );
    }

    #[test]
    fn quartet_hash_matches_java() {
        // Arrays.hashCode({1, 2}) = 994, ({3, 4}) = 1058;
        // 97 * (97 * 3 + 994) + 1058 = 125703.
        assert_eq!(quartet_hash(&Quartet::new(1, 2, 3, 4, 1.0)), 125_703);
    }
}
