//! Writes seeded weighted-quartet inputs for the comparison with the jar.
//!
//! Usage: cargo run --release --example make_inputs -- <folder> <sizes> <seeds>
//! e.g. `... -- inputs 6,8,10 1,2,3`. Three kinds per size and seed:
//! - `noisy`: every 4-set, the tree's topology weighted 0.5 to 1, the other
//!   two 0 to 0.3;
//! - `ties`: every 4-set, whole weights (true 2 or 3, others 0 to 2), so
//!   many moves tie and the tie rules decide;
//! - `sparse`: 30 % of the 4-sets, weights as in `noisy`, one of the two
//!   wrong topologies sometimes the heaviest.

use std::fmt::Write as _;

use wqfm::{random_binary_tree, to_wqfm_input, Quartet, SplitMix64, Topology};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert!(
        args.len() == 4,
        "usage: make_inputs <folder> <sizes> <seeds>"
    );
    let dir = std::path::Path::new(&args[1]);
    std::fs::create_dir_all(dir).unwrap();
    let list = |s: &str| -> Vec<u64> { s.split(',').map(|x| x.parse().unwrap()).collect() };
    let mut index = String::from("name\tkind\ttaxa\tseed\tquartets\ttrue_tree\n");
    for &m in &list(&args[2]) {
        for &seed in &list(&args[3]) {
            for kind in ["noisy", "ties", "sparse"] {
                let mut rng = SplitMix64::new(seed * 1_000_003 + m * 101 + kind.len() as u64);
                let names: Vec<String> = (0..m).map(|i| format!("t{i}")).collect();
                let truth = random_binary_tree(&names, &mut rng);
                let qs = quartets(kind, &names, &truth, &mut rng);
                let name = format!("{kind}_m{m:03}_s{seed}");
                std::fs::write(
                    dir.join(format!("{name}.wqrts")),
                    to_wqfm_input(&names, &qs),
                )
                .unwrap();
                let _ = writeln!(index, "{name}\t{kind}\t{m}\t{seed}\t{}\t{truth}", qs.len());
            }
        }
    }
    std::fs::write(dir.join("inputs.tsv"), index).unwrap();
}

fn quartets(kind: &str, names: &[String], truth: &str, rng: &mut SplitMix64) -> Vec<Quartet> {
    let topo = Topology::from_newick(truth, names).unwrap();
    let m = names.len() as u32;
    let mut out = Vec::new();
    for a in 0..m {
        for b in a + 1..m {
            for c in b + 1..m {
                for d in c + 1..m {
                    if kind == "sparse" && rng.next_f64() >= 0.3 {
                        continue;
                    }
                    for q in [
                        Quartet::new(a, b, c, d, 1.0),
                        Quartet::new(a, c, b, d, 1.0),
                        Quartet::new(a, d, b, c, 1.0),
                    ] {
                        let shown = topo.displays(&q);
                        let w = match (kind, shown) {
                            ("ties", true) => (2 + rng.next_u64() % 2) as f64,
                            ("ties", false) => (rng.next_u64() % 3) as f64,
                            ("sparse", false) if rng.next_f64() < 0.1 => 1.0 + rng.next_f64(),
                            (_, true) => 0.5 + 0.5 * rng.next_f64(),
                            (_, false) => 0.3 * rng.next_f64(),
                        };
                        if w > 0.0 {
                            out.push(Quartet { weight: w, ..q });
                        }
                    }
                }
            }
        }
    }
    out
}
