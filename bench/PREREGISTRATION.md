# Benchmark pre-registration: wQFM-rs

Written 2026-10-24, before any benchmark run. The date is the commit date of
this file. Changes after this date are made only as dated amendments at the
end, each giving the reason; nothing above the amendments is edited.

## 1. Questions and hypotheses

wQFM-rs is an exact re-implementation of wQFM v1.4. Its accuracy is by
design that of wQFM, so the benchmark asks about identity and cost, and
places wQFM among current methods only descriptively.

- **H1 (identity).** On every input on which both finish, wQFM-rs returns
  the same topology (RF 0) and the same weighted quartet score as the wQFM
  v1.4 jar, and `wqfm-rs gt2wq` writes the same bytes as `generate_wqrts.py`
  (with `--keep-unresolved`). Any difference is a defect; none is allowed.
- **H2 (cost of the amalgamation).** On weighted-quartet inputs with at least
  100,000 quartets, wQFM-rs is at least 5 times faster (median of the
  per-input wall-time ratios) **or** uses at least 3 times less peak memory
  (median ratio), **or** finishes inputs on which the jar fails within the
  limits of section 5. This is also the go/no-go gate of the plan: if H2
  fails, the paper is not written as planned and the result is reported in
  the log.
- **H3 (cost of the pipeline).** From gene trees to species tree,
  `gt2wq` + `amalgamate` is faster than `generate_wqrts.py` + the jar on
  every gene-tree dataset both finish (median ratio reported).
- **D (descriptive, no hypothesis).** Accuracy (nRF to the true species
  tree), wall time and peak memory of wQFM-rs next to wQFM-TREE, ASTRAL-III,
  ASTRAL-IV, wASTRAL and TREE-QMC on the gene-tree datasets.

Scope stated in advance: the wQFM pipeline lists every 4-set of taxa, so its
input grows as n⁴. The benchmark therefore expects wQFM (Java and Rust) to
stop being feasible on this computer somewhere between 100 and 200 taxa,
while wQFM-TREE, ASTRAL and TREE-QMC go further. That boundary is measured
and reported, not avoided. wQFM-rs is aimed at weighted-quartet inputs that
do not come from gene trees (for example alignment-free quartets such as
those of Q-MAWS), where wQFM-TREE does not apply.

## 2. Datasets

Every file is downloaded once, its SHA-256 written to `bench/datasets.tsv`
with the source URL before use. Replicates are taken in their numbered
order; the counts below are fixed now.

| Id | Data | Taxa | Use |
|---|---|---|---|
| S1 | Generated weighted quartets (`make_inputs`, seeds 1 to 5): kind `noisy` (all three topologies of every 4-set) at 25, 50, 75, 100, 125, 150 taxa; kind `sparse` (30 % of the 4-sets) at the same sizes and 200 taxa (`noisy` at 200 taxa would be about 2 × 10⁸ quartets, beyond this computer's disk and memory) | 25–200 | H1, H2 |
| S2 | wQFM's own weighted-quartet files (`WQFM/input_files`, commit 7bfdf8e) | 11–48 | H1, H2 |
| S3 | Q-MAWS v1.0.0 W2c quartets of Fish mtDNA, strand filter on and off | 25 | H1, H2 |
| G1 | 37-taxon mammalian simulation (Mirarab et al. 2014), estimated gene trees, every model condition in the package, replicates 1–10 | 37 | H1, H3, D |
| G2 | 48-taxon avian simulation (Mirarab et al. 2014), estimated gene trees, every model condition, replicates 1–10 | 48 | H1, H3, D |
| G3 | ASTRAL-II simulation (Mirarab and Warnow 2015, Dryad doi:10.6076/D10C7C), the series varying the number of taxa, estimated gene trees, replicates 1–10 | 10–1,000 | H3, D |

Sources: G1 and G2 as listed for the wQFM and ASTRAL papers (IDEALS, Dryad
doi:10.5061/dryad.wstqjq2wn); G3 from Dryad. If a listed dataset or model
condition cannot be obtained, an amendment records it before any run of
that dataset.

## 3. Methods and versions

| Method | Version (pinned) | Input |
|---|---|---|
| wQFM-rs | the commit tagged for the benchmark, recorded in `bench/tools.tsv` before the first run | quartets; gene trees via `gt2wq` |
| wQFM (Java) | `wQFM-v1.4.jar`, commit 7bfdf8e | quartets |
| generate_wqrts.py | commit 7bfdf8e with `triplets.soda2103` | gene trees |
| wQFM-TREE | release v1.1 (`wQFM-TREE-v1.1.zip`) | gene trees |
| ASTRAL-III | `Astral.5.7.8.zip` from smirarab/ASTRAL | gene trees |
| ASTRAL-IV, wASTRAL | ASTER release v1.25 | gene trees |
| TREE-QMC | release v3.0.4 | gene trees |

Every method runs with its default settings; wQFM-rs and the jar with
β = 1. Java: OpenJDK 21.0.12.1. SHA-256 of every binary or archive goes to
`bench/tools.tsv` at download.

## 4. Measures

- **Identity (H1):** RF between the wQFM-rs and jar trees; both scores with
  `wqfm-rs score`; `cmp` for `gt2wq` against the script.
- **Wall time:** seconds from process start to exit. For H2 on S1 to S3,
  three repeats per input and method, median used. Other runs once.
- **Peak memory:** maximum resident set size from GNU `time -v`.
- **Accuracy (D):** nRF between the output and the true species tree
  (`wqfm-rs compare`).
- **Threads:** the amalgamation is single-threaded in both wQFM programs;
  `gt2wq` is timed with 1 and 12 threads; other methods with their default
  thread count, recorded.

## 5. Computer and limits

The development laptop: Intel Core 5 210H (12 threads), 8 GB memory,
Windows 11, all methods in the same WSL Ubuntu 24.04 environment (wQFM-rs
built for Linux there), plugged in, Windows power mode "Best performance",
sleep off, one method at a time, nothing else running. Limits per run: 6
hours wall time and the memory of the machine (Java with `-Xmx6g`). A run
that exceeds a limit or fails is recorded as such with its error and is
never left out. Runs are only started after the Q-MAWS benchmark has
finished.

## 6. Analysis

- H1: count of differences; must be 0. Each difference, if any, is
  investigated and reported, and H1 is reported as failed.
- H2: per-input ratios jar / wQFM-rs for wall time and memory on inputs with
  at least 100,000 quartets; medians with the interquartile range; the
  number of inputs on which only one program finishes.
- H3: per-dataset ratios for the pipeline; median and range.
- D: mean nRF per model condition and method, with Wilcoxon signed-rank
  tests of wQFM-rs against each other method over replicates and Holm
  correction per dataset; times and memory per condition. Conditions where
  wQFM cannot run are shown as such.
- Everything is computed by scripts committed in `bench/` from the raw
  results, which are sealed with SHA-256 as in the Q-MAWS benchmark.

## 7. Amendments

None yet.
