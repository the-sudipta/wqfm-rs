# Design decisions

1. **Source of the logic.** The algorithm is the `wQFM-rs` module of Q-MAWS
   v1.0.0 (`crates/qmaws-core/src/amalgamate.rs`), which matched the wQFM v1.4
   jar in topology and score on all 23 test inputs. It is split into one
   file per feature with no change of behaviour.
2. **Java tie rules.** Gains are compared with `f64::total_cmp`, the order of
   Java's `Double` in a `TreeMap` (so −0 < +0). Ties go to the move with the
   most satisfied quartets, then to the last taxon. The initial bipartition
   sorts by weight with a stable sort.
3. **Dummy quartets.** Equal quartets created by a division are merged with
   the mean weight, as in wQFM v1.4.
4. **Taxa without quartets** are attached at the root, so the output has
   every input taxon.
5. **Shared tree builder.** The exhaustive oracle and random trees use one
   stepwise-addition builder (`edge_tree.rs`), giving the same Newick as the
   Q-MAWS code.
6. **`gt2wq` follows `generate_wqrts.py` byte for byte.** For every gene
   tree, each 4-set of its taxa gets its induced topology (four-point
   condition on path lengths); counts are summed over trees. Lines are
   written as the script writes them: each pair in text order, the pair
   with the smaller first name first, lines sorted as text, `key count`.
   Checked on 7 inputs (2 from wQFM, the avian biological set, 4
   synthetic); 6 are byte-identical, the 7th differs only by item 8.
7. **Unresolved 4-sets.** For a 4-set that a gene tree leaves unresolved,
   the helper `triplets.soda2103` prints the four taxa without `|`, so the
   script writes `((a,b,c,d)); n` (names in text order). wQFM v1.4 strips
   the brackets and reads this line as the quartet ab|cd (`Quartet.java`,
   `Quartet(String)`), i.e. polytomies add quartets that no gene tree shows.
   `gt2wq` writes these lines by default, so the pipeline stays identical to
   the original; `--skip-unresolved` leaves them out.
8. **Blank lines** in the gene tree file are skipped. The script turns a
   blank line into the line `(()); 1`, which wQFM cannot read.
9. **Parallelism.** Gene trees are counted in parallel (rayon) and the
   counts are summed; the output is sorted, so it does not depend on the
   number of threads.
