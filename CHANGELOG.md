# Changelog

## Unreleased

- Library `wqfm`: initial bipartition, FM refinement, division with dummy
  taxa, joining, exhaustive oracle (3–10 taxa), quartet score, Newick and
  wQFM input format.
- Command `wqfm-rs amalgamate` and `wqfm-rs score`.
- `gt2wq`: gene trees to weighted quartets, byte-identical to wQFM's
  `generate_wqrts.py`; parallel; `--skip-unresolved` for polytomies.
- Golden tests: optimum on noise-free inputs, noisy inputs within 10 % of the
  optimum, input round trip.
