//! Writes test cases for the Java `HashMap` model and the model's answers.
//!
//! Usage: java_order_check <cases file> <expected file> <number of cases>
//! Each case is a line of distinct hash codes, inserted in that order. The
//! expected line is the iteration order of the map, `|`, and the order of a
//! copy made with `new HashMap<>(map)`; cases where two hashes tie inside a
//! tree (Java's own order is then not deterministic) are not written.
//! `scripts/JavaOrderCheck.java` prints Java's answers for the same cases.

use std::collections::HashSet;
use std::fmt::Write as _;

use wqfm::{java_hashmap_orders, SplitMix64};

/// wQFM's `Quartet.hashCode` for ab|cd (a < b, c < d, a < c).
fn quartet_hash(a: i32, b: i32, c: i32, d: i32) -> i32 {
    let arr = |x: i32, y: i32| 31i32.wrapping_mul(31i32.wrapping_add(x)).wrapping_add(y);
    97i32
        .wrapping_mul(97i32.wrapping_mul(3).wrapping_add(arr(a, b)))
        .wrapping_add(arr(c, d))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert!(
        args.len() == 4,
        "usage: java_order_check <cases> <expected> <n>"
    );
    let n: usize = args[3].parse().unwrap();
    let mut rng = SplitMix64::new(20261025);
    let (mut cases, mut expected) = (String::new(), String::new());
    let (mut written, mut with_tree) = (0, 0);
    while written < n {
        let size = 2 + (rng.next_u64() % 400) as usize;
        let mut seen = HashSet::new();
        let mut hashes = Vec::with_capacity(size);
        let kind = rng.next_u64() % 3;
        let stride = [16i32, 32, 64, 128, 256][(rng.next_u64() % 5) as usize];
        let taxa = 6 + (rng.next_u64() % 60) as i32;
        let dummy = taxa + 1 + (rng.next_u64() % 10) as i32;
        let mut tries = 0;
        while hashes.len() < size && tries < 20 * size {
            tries += 1;
            let h = match kind {
                // Crowded buckets: multiples of a stride plus a little noise.
                0 => (rng.next_u64() % 64) as i32 * stride + (rng.next_u64() % 3) as i32,
                // wQFM dummy quartets: three real taxa and the dummy.
                1 => {
                    let mut t: Vec<i32> = (0..3)
                        .map(|_| (rng.next_u64() % taxa as u64) as i32)
                        .collect();
                    t.sort();
                    t.dedup();
                    if t.len() < 3 {
                        continue;
                    }
                    match rng.next_u64() % 3 {
                        0 => quartet_hash(t[0], t[1], t[2], dummy),
                        1 => quartet_hash(t[0], t[2], t[1], dummy),
                        _ => quartet_hash(t[1], t[2], t[0], dummy),
                    }
                }
                // Small integers (taxon ids of a level).
                _ => (rng.next_u64() % (4 * size as u64 + 8)) as i32,
            };
            if seen.insert(h) {
                hashes.push(h);
            }
        }
        let (first, copy, tie, tree) = java_hashmap_orders(&hashes);
        if tie {
            continue;
        }
        with_tree += usize::from(tree);
        let line = |v: &[usize]| {
            v.iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        };
        let _ = writeln!(
            cases,
            "{}",
            hashes
                .iter()
                .map(|h| h.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        );
        let _ = writeln!(expected, "{} | {}", line(&first), line(&copy));
        written += 1;
    }
    eprintln!("{written} cases, {with_tree} with a tree bucket");
    std::fs::write(&args[1], cases).unwrap();
    std::fs::write(&args[2], expected).unwrap();
}
