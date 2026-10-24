//! `gt2wq` against the output of wQFM's `generate_wqrts.py` (see data/README.md).

use wqfm::newick::Tree;
use wqfm::{count_quartets, counts_to_wqfm_input};

fn check(name: &str) {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/");
    let trees: Vec<Tree> = std::fs::read_to_string(format!("{dir}{name}.tre"))
        .unwrap()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(|l| Tree::parse(l).unwrap())
        .collect();
    let expected = std::fs::read_to_string(format!("{dir}{name}.wqrts")).unwrap();
    let got = counts_to_wqfm_input(&count_quartets(&trees).unwrap(), true);
    assert!(
        got == expected,
        "{name}: output differs from generate_wqrts.py"
    );
}

#[test]
fn identical_to_generate_wqrts() {
    for name in [
        "gtree_11tax_est_5genes_R1",
        "gtree_15tax_100g_100b_R1",
        "s1_12tax",
        "s4_8tax_many",
    ] {
        check(name);
    }
}
