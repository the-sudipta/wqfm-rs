# Test data for `gt2wq`

Each `X.tre` (gene trees) has `X.wqrts`, the output of wQFM's
`generate_wqrts.py` (commit 7bfdf8e, with `triplets.soda2103`, Python 3.12.3,
pandas 2.1.4, Ubuntu 24.04) on it. `wqfm-rs gt2wq` must give the same bytes.

- `gtree_11tax_est_5genes_R1.tre`, `gtree_15tax_100g_100b_R1.tre`: from
  wQFM's `WQFM/input_files` (Apache-2.0, see NOTICE), line endings LF.
- `s1_12tax.tre`, `s4_8tax_many.tre`: made by `make_synthetic.py`
  (`s4` has polytomies, missing taxa, branch lengths and support values).

# Test data for the jar comparison

- `fish_no_strand.wqrts`: the W2c weighted quartets of Q-MAWS v1.0.0 on
  Fish mtDNA without the strand filter (25 taxa, 32,564 quartets; input
  `real_01` of the Phase 3 comparison). `fish_no_strand.jar.tre` is the tree
  of `wQFM-v1.4.jar` on it (OpenJDK 21.0.12.1). Before Java's `HashMap` order
  was modelled, wqfm-rs gave a tree with the same score but RF 4 here.
