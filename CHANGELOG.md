# Changelog

## Unreleased

- Library `wqfm`: initial bipartition, FM refinement, division with dummy
  taxa, joining, exhaustive oracle (3–10 taxa), quartet score, Newick and
  wQFM input format.
- Command `wqfm-rs amalgamate` and `wqfm-rs score`.
- `gt2wq`: gene trees to weighted quartets, byte-identical to wQFM's
  `generate_wqrts.py`; parallel; `--skip-unresolved` for polytomies.
- Java `HashMap` iteration order and wQFM's dummy ids, so near-ties go as
  in the jar (decisions 10 to 12).
- `wqfm-rs compare`: RF and normalised RF between two trees.
- Jar comparison: 202 of 203 inputs identical in topology and score
  (`validation/`).
- Golden tests: optimum on noise-free inputs, noisy inputs within 10 % of the
  optimum, input round trip.
