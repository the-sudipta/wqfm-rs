//! Unusual inputs against the trees of the wQFM v1.4 jar (data/edge, see
//! data/README.md): duplicate lines, zero and negative weights, equal
//! weights, 4 and 5 taxa, unusual names, number formats, CRLF, spacing, a
//! repeated taxon, the same quartet written in several orders.

use wqfm::newick::{nrf, Tree};
use wqfm::{amalgamate, parse_wqfm_input, Options, Topology};

#[test]
fn edge_cases_match_the_jar() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/edge/");
    let mut checked = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let Some(stem) = name.strip_suffix(".wqrts") else {
            continue;
        };
        let text = std::fs::read_to_string(&path).unwrap();
        let mut names = Vec::new();
        if stem == "e13_overflow_weight" {
            assert!(parse_wqfm_input(&text, &mut names).is_err());
            continue;
        }
        let quartets = parse_wqfm_input(&text, &mut names).unwrap();
        let ours = amalgamate(&names, &quartets, &Options::default());
        let jar = std::fs::read_to_string(format!("{dir}{stem}.jar.tre")).unwrap();
        let jar = jar.trim();
        let (rf, _) = nrf(
            &Tree::parse(&ours.newick).unwrap(),
            &Tree::parse(jar).unwrap(),
        );
        assert_eq!(rf, 0, "{stem}: topology differs from the jar");
        let (jar_score, _) = Topology::from_newick(jar, &names).unwrap().score(&quartets);
        assert_eq!(ours.score, jar_score, "{stem}: score differs from the jar");
        checked += 1;
    }
    assert_eq!(checked, 12);
}
