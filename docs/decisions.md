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
   script writes `((a,b,c,d)); n` (names in text order). The wQFM v1.4 jar
   refuses such a line: it needs exactly three `(` and three `)` per line,
   prints "Found STAR ... Can't handle polytomy for now" and exits
   (`FMRunner.populatePerInputLine`; seen on wQFM's own `weird_dataset_1`).
   So the original pipeline stops on any gene tree with a polytomy. `gt2wq`
   therefore leaves unresolved 4-sets out by default, and
   `--keep-unresolved` writes them byte for byte as the script (used to
   check identity). `parse_wqfm_input` refuses these lines like the jar.
   (Corrected 2026-10-24: an earlier note said the jar reads the line as
   ab|cd; its `Quartet(String)` would, but the check above runs first.)
8. **Blank lines** in the gene tree file are skipped. The script turns a
   blank line into the line `(()); 1`, which wQFM cannot read.
9. **Parallelism.** Gene trees are counted in parallel (rayon) and the
   counts are summed; the output is sorted, so it does not depend on the
   number of threads.
10. **Java `HashMap` order.** wQFM v1.4 takes two orders from Java hash
    maps: the taxa of a new level (keys of the bipartition map) and the
    merged dummy quartets (keys of a map of quartets). The quartet order
    sets the order of floating-point sums, so gains that are equal in exact
    arithmetic differ in the last bits, and the near-ties go another way.
    `java_order.rs` reproduces both orders (buckets `spread(hash) & (n-1)`,
    insertion order inside a bucket, table sizes of `new HashMap<>()` and
    `new HashMap<>(map)`, `Quartet.hashCode` of wQFM). Found on the Fish
    mtDNA input without the strand filter: same score as the jar, RF 4,
    before; identical move by move (taxon and gain bits, 529 moves) after.
    Buckets that Java turns into trees (9 or more entries in a table of at
    least 64) are not modelled; a debug assertion reports them, and none
    occurred on the test inputs.
11. **Dummy ids** are `n + depth` (depth 1 at the top level), as
    `TaxaUtils.getDummyTaxonName`; sibling levels share an id.
12. **Java version.** OpenJDK 19 changed the table size of
    `new HashMap<>(map)` (`ceil(k / 0.75)` instead of `k / 0.75 + 1`), so in
    principle the jar's tree can depend on the Java version. Measured: the
    jar on OpenJDK 17.0.21 and 21.0.12.1 gave the same tree on all 202
    comparison inputs. wqfm-rs follows OpenJDK 19 and later.
13. **Empty lines in quartet files.** The jar stops reading at the first
    empty line. `parse_wqfm_input` does the same but returns an error if
    quartets follow, since the jar would silently ignore them.
