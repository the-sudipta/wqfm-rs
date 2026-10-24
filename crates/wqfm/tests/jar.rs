//! wqfm-rs against a tree of the wQFM v1.4 jar (see data/README.md).

use wqfm::newick::{nrf, Tree};
use wqfm::{amalgamate, parse_wqfm_input, Options, Topology};

#[test]
fn fish_no_strand_matches_the_jar() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/");
    let text = std::fs::read_to_string(format!("{dir}fish_no_strand.wqrts")).unwrap();
    let mut names = Vec::new();
    let quartets = parse_wqfm_input(&text, &mut names).unwrap();
    let ours = amalgamate(&names, &quartets, &Options::default());
    let jar = std::fs::read_to_string(format!("{dir}fish_no_strand.jar.tre")).unwrap();
    let jar = jar.trim();
    let (rf, _) = nrf(&Tree::parse(&ours.newick).unwrap(), &Tree::parse(jar).unwrap());
    assert_eq!(rf, 0, "topology differs from the jar");
    let (jar_score, _) = Topology::from_newick(jar, &names).unwrap().score(&quartets);
    assert_eq!(ours.score, jar_score);
}
