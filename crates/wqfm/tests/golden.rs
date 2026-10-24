//! Golden tests ported from Q-MAWS (M7): optimality on noise-free inputs,
//! closeness to the optimum on noisy inputs, and the input round trip.

use wqfm::newick::{nrf, Tree};
use wqfm::{
    amalgamate, exhaustive_best, induced_quartets, parse_wqfm_input, random_binary_tree,
    to_wqfm_input, Options, Quartet, SplitMix64, Topology,
};

fn names(m: usize) -> Vec<String> {
    (0..m).map(|i| format!("t{i}")).collect()
}

/// The linear congruential generator of the Q-MAWS tests (same streams).
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }

    fn unit(&mut self) -> f64 {
        self.next() as f64 / (1u64 << 31) as f64
    }
}

/// A random unrooted binary tree as Newick (random stepwise addition).
fn random_tree(names: &[String], rng: &mut Lcg) -> String {
    let m = names.len();
    let mut edges: Vec<(usize, usize)> = vec![(0, m), (1, m), (2, m)];
    for t in 3..m {
        let inner = m + (t - 2);
        let i = (rng.next() as usize) % edges.len();
        let (u, v) = edges[i];
        edges[i] = (u, inner);
        edges.push((inner, v));
        edges.push((t, inner));
    }
    let mut adj = vec![Vec::new(); 2 * m - 2];
    for &(u, v) in &edges {
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

/// Every quartet induced by a tree, with random positive weights.
fn induced(tree: &str, names: &[String], rng: &mut Lcg) -> Vec<Quartet> {
    let topo = Topology::from_newick(tree, names).unwrap();
    let m = names.len() as u32;
    let mut out = Vec::new();
    for a in 0..m {
        for b in a + 1..m {
            for c in b + 1..m {
                for d in c + 1..m {
                    for q in [
                        Quartet::new(a, b, c, d, 1.0),
                        Quartet::new(a, c, b, d, 1.0),
                        Quartet::new(a, d, b, c, 1.0),
                    ] {
                        if topo.displays(&q) {
                            out.push(Quartet {
                                weight: 1.0 + 9.0 * rng.unit(),
                                ..q
                            });
                        }
                    }
                }
            }
        }
    }
    out
}

fn same_topology(a: &str, b: &str) -> bool {
    nrf(&Tree::parse(a).unwrap(), &Tree::parse(b).unwrap()).0 == 0
}

#[test]
fn g8_noise_free_inputs_reach_the_optimum() {
    // On quartets induced by a tree, wQFM-rs returns a tree with the
    // exhaustive optimum score (all the weight), i.e. the tree itself.
    let mut rng = Lcg(8);
    for m in 4..=8 {
        for _ in 0..12 {
            let nm = names(m);
            let truth = random_tree(&nm, &mut rng);
            let qs = induced(&truth, &nm, &mut rng);
            let result = amalgamate(&nm, &qs, &Options::default());
            let (best, best_score) = exhaustive_best(&nm, &qs);
            assert!(
                (result.score - best_score).abs() < 1e-9,
                "m {m}: wQFM-rs {} ({}) vs optimum {best_score} ({best})",
                result.score,
                result.newick
            );
            assert!(
                same_topology(&result.newick, &truth),
                "{} vs {truth}",
                result.newick
            );
        }
    }
}

#[test]
fn noisy_inputs_stay_close_to_the_optimum() {
    let mut rng = Lcg(81);
    let mut worst: f64 = 1.0;
    for m in 5..=8 {
        for _ in 0..10 {
            let nm = names(m);
            let truth = random_tree(&nm, &mut rng);
            let mut qs = induced(&truth, &nm, &mut rng);
            // Noise: every 4-set also gets its two other topologies with
            // smaller random weights.
            let extra: Vec<Quartet> = qs
                .iter()
                .flat_map(|q| {
                    let [a, b] = q.l;
                    let [c, d] = q.r;
                    [Quartet::new(a, c, b, d, 0.0), Quartet::new(a, d, b, c, 0.0)]
                })
                .collect();
            for mut q in extra {
                q.weight = 6.0 * rng.unit();
                qs.push(q);
            }
            let result = amalgamate(&nm, &qs, &Options::default());
            let (_, best_score) = exhaustive_best(&nm, &qs);
            assert!(result.score <= best_score + 1e-9);
            worst = worst.min(result.score / best_score);
        }
    }
    println!("noisy inputs: worst wQFM-rs score / optimum = {worst:.6}");
    assert!(worst > 0.9, "worst ratio {worst}");
}

#[test]
fn taxa_without_quartets_are_kept() {
    let nm = names(6);
    let qs = vec![Quartet::new(0, 1, 2, 3, 1.0), Quartet::new(0, 1, 2, 4, 1.0)];
    let r = amalgamate(&nm, &qs, &Options::default());
    assert_eq!(Tree::parse(&r.newick).unwrap().leaf_names().len(), 6);
    assert_eq!(r.score, 2.0);
}

#[test]
fn input_round_trip_and_induced_quartets() {
    let nm = names(6);
    let mut rng = SplitMix64::new(4);
    let tree = random_binary_tree(&nm, &mut rng);
    let topo = Topology::from_newick(&tree, &nm).unwrap();
    let qs = induced_quartets(&topo);
    assert_eq!(qs.len(), 15, "one topology per 4-set");
    assert!(qs.iter().all(|q| topo.displays(q)));
    let text = to_wqfm_input(&nm, &qs);
    let mut names_back = Vec::new();
    let back = parse_wqfm_input(&text, &mut names_back).unwrap();
    assert_eq!(back.len(), qs.len());
    let to_names = |q: &Quartet, n: &[String]| {
        let s = |t: u32| n[t as usize].clone();
        let mut l = [s(q.l[0]), s(q.l[1])];
        let mut r = [s(q.r[0]), s(q.r[1])];
        l.sort();
        r.sort();
        let mut pair = [l, r];
        pair.sort();
        (pair, q.weight)
    };
    for (a, b) in qs.iter().zip(&back) {
        assert_eq!(to_names(a, &nm), to_names(b, &names_back));
    }
}
