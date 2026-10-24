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
