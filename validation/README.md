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
