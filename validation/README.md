# Validation records

## Phase 3: wqfm-rs against the wQFM v1.4 jar (2026-10-24)

`phase3_jar_comparison.tsv`: one row per input; pass = identical topology
(RF 0) and identical score.

- Jar: `wQFM-v1.4.jar` (commit 7bfdf8e), OpenJDK 21.0.12.1, Ubuntu 24.04
  (WSL), `-Xmx512m`, default settings. wqfm-rs from this repository.
- `inputs` (23): the Q-MAWS M7 set, made with `qmaws wqfm-export --seed 1`
  (Q-MAWS v1.0.0): the worksheet example, 10 noise-free and 10 noisy inputs
  (6 to 50 taxa), and the Fish mtDNA W2c quartets with and without the
  strand filter.
- `golden` (180): `cargo run --release --example make_inputs -- <dir>
  6,8,10,12,15,20,25,30,35,40 1,2,3,4,5,6`, kinds `noisy`, `ties` (whole
  weights, many ties) and `sparse` (30 % of the 4-sets).
- Script: `bash scripts/jar_compare.sh <dir> <wQFM-2020 folder> 512m`.

Result: 202 of 203 pass; 0 differ. `noisy_09_m50` (690,873 quartets) did not
finish in the jar within 512 MB; it is rerun with more memory once the Q-MAWS
benchmark has finished. The first run found one difference (`real_01`, same
score, RF 4), fixed by decision 10 in `docs/decisions.md`; all rows here are
from the fixed version, compared with the saved jar trees.

## Phase 3: Java 17 against Java 21 (2026-10-24)

`phase3_java17_vs_java21.tsv`: the jar on Eclipse Temurin OpenJDK 17.0.21
(SHA-256 checked against Adoptium) and on OpenJDK 21.0.12.1, same 202
inputs: identical trees on all 202 (RF 0), and wqfm-rs identical to the
Java 17 jar on all 202. (`noisy_09_m50` again ran out of the 512 MB cap.)

## Phase 3: wQFM's own example files and the full pipeline (2026-10-24)

`phase3_wqfm_examples.tsv`: the 10 weighted-quartet files in wQFM's
`WQFM/input_files` (up to 583,740 quartets, avian biological set) and 3
pipeline inputs (`gt2wq` on wQFM's gene-tree files, byte-identical to
`generate_wqrts.py`). 12 identical in topology and score. `weird_dataset_1`
has a 4-taxon line; the jar refuses it ("Can't handle polytomy") and so does
wqfm-rs, with an error naming the line.

## Cross-OS determinism

`determinism.sha256`: SHA-256 of wqfm-rs outputs on 24 generated inputs and
of `gt2wq` on the test gene trees (`scripts/determinism.sh`). CI
(`.github/workflows/ci.yml`) rebuilds them on Linux, Windows and macOS and
requires the same list.

## Phase 3: β other than 1 (2026-10-25)

`phase3_beta.tsv`: the jar (`-beta B`) and wqfm-rs (`--beta B`) at
β = 0.25, 0.5, 1.5 and 2 on 10 inputs (noisy, ties and sparse kinds, 10 to
40 taxa, and the Fish mtDNA quartets without the strand filter): 40 of 40
identical in topology and score.

## Phase 3: unusual inputs (2026-10-25)

`phase3_edge_cases.tsv` and `crates/wqfm/tests/data/edge/`: 13 hand-written
inputs (duplicate lines, zero and negative weights, three equal topologies
of a 4-set, 4 and 5 taxa, unusual names, number formats, CRLF, spacing, a
repeated taxon, one quartet written in several orders, an infinite
weight). 12 identical to the jar; the infinite weight is refused by
wqfm-rs, while the jar returns a meaningless tree. The test `tests/edge.rs`
repeats the 12 comparisons on every build.

## Phase 3: the model of Java's HashMap (2026-10-25)

`examples/java_order_check.rs` writes 3,000 seeded insertion sequences
(crowded buckets, real wQFM quartet hash codes, small integers; up to 401
keys) with the model's iteration orders for the map and for
`new HashMap<>(map)`; `scripts/JavaOrderCheck.java` prints the orders of the
real `java.util.HashMap` (OpenJDK 21.0.12.1). All 3,000 identical, 820 of
them with buckets turned into trees. CI repeats the check on Linux on every
push. With the full model, all 214 jar comparisons above still pass and
none is reported as uncertain.
