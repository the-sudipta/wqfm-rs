# Changelog

## Unreleased

- Library `wqfm`: initial bipartition, FM refinement, division with dummy
  taxa, joining, exhaustive oracle (3–10 taxa), quartet score, Newick and
  wQFM input format.
- Command `wqfm-rs amalgamate` and `wqfm-rs score`.
- `gt2wq`: gene trees to weighted quartets, byte-identical to wQFM's
  `generate_wqrts.py`; parallel; unresolved 4-sets left out unless `--keep-unresolved`.
- Quartet files are read with the jar's rules (three brackets a side per
  line; stop at the first empty line).
- Java `HashMap` iteration order and wQFM's dummy ids, so near-ties go as
  in the jar (decisions 10 to 12).
- Full model of Java's `HashMap` order, including tree buckets, checked
  against OpenJDK 21 on 3,000 cases; `Amalgamation::java_order_exact` and a
  warning when the jar's own order is not deterministic.
- Weights that are not finite are refused; `--beta` checked against the
  jar; 12 unusual inputs identical to the jar (`tests/edge.rs`).
- Lower memory: quartet files read line by line (`read_wqfm_input`), the
  top level borrows the quartets.
- `docs/USAGE.md`: manual with a worked example, options and messages;
  every public item documented.
- `wqfm-rs compare`: RF and normalised RF between two trees.
- Jar comparison: 202 of 203 inputs identical in topology and score
  (`validation/`).
- Packaging: release workflow (5 targets, draft release with SHA-256
  sums), Dockerfile, CITATION.cff, crates.io metadata (the command crate is
  now named `wqfm-rs`), Bioconda recipe draft.
- Golden tests: optimum on noise-free inputs, noisy inputs within 10 % of the
  optimum, input round trip.
