# Download list for the benchmark

Prepared 2026-10-25 from the repositories' own metadata; approved by the
owner and downloaded 2026-10-26 (unmetered connection, capped at 4 MB/s,
while the Q-MAWS benchmark ran its tuning runs, never its speed runs).
SHA-256 of every file: `bench/datasets.tsv`, `bench/tools.tsv`.

Everything goes to `D:\_external\wqfm_bench\` (never C:), is
only downloaded now and unpacked after the Q-MAWS benchmark, and gets its
SHA-256 in `bench/datasets.tsv` or `bench/tools.tsv`.

## Datasets

| # | File | Size | Source | Licence |
|---|---|---|---|---|
| 1 | `estimated-gene-trees.tar.bz` (ASTRAL-II, 11 model conditions, 10 to 1,000 taxa, 50 replicates, 1,000 FastTree gene trees each) | 1,957.8 MB | Dryad doi:10.6076/D10C7C (SHA-256 published, prefix 0600b554…) | CC0 |
| 2 | `true-specis-trees.tar.bz` (ASTRAL-II true species trees) | 2.3 MB | same (SHA-256 prefix f5c8e6c1…) | CC0 |
| 3 | `README.md` (ASTRAL-II layout) | 7 KB | same | CC0 |
| 4 | `mammalian-0.5X-estimated-genetrees.tar.bz2` | 33.8 MB | IDEALS item 55769 (Mirarab et al. 2014) | public deposit, cite the DOI |
| 5 | `mammalian-1X-estimated-genetrees.tar.bz2` | 669.6 MB | IDEALS item 55768, doi:10.13012/C5KW5CXW | same |
| 6 | `mammalian-2X-estimated-genetrees.tar.bz2` | 33.8 MB | IDEALS item 55770 | same |
| 7 | `mammalian-model-species.tre` | < 1 KB | IDEALS item 55763 | same |
| 8 | `avian-0.5X-estimated-genetrees.tar.bz2` | 291.2 MB | IDEALS item 55766 | same |
| 9 | `avian-1X-estimated-genetrees.tar.bz2` | 1,124.3 MB | IDEALS item 55764 | same |
| 10 | `avian-2X-estimated-genetrees.tar.bz2` | 294.5 MB | IDEALS item 55767 | same |
| 11 | `avian-model-species.tre` | < 1 KB | IDEALS item 55762 | same |
| 12 | `README for Mirarab et al_fixed.html.txt` (layout of the IDEALS files) | < 1 KB | IDEALS | same |

Not needed and not downloaded: alignments, true gene trees, the 3 GB
`Data.zip` of doi:10.5061/dryad.wstqjq2wn.

## Tools

| # | File | Size | Source | Licence |
|---|---|---|---|---|
| 13 | `wQFM-TREE-v1.1.zip` | 3.5 MB | github.com/abdur-rafi/wQFM-TREE release v1.1 | Apache-2.0 |
| 14 | `Astral.5.7.8.zip` | 5.1 MB | github.com/smirarab/ASTRAL (file in master) | Apache-2.0 |
| 15 | ASTER v1.25 source archive (no binaries published; built in WSL after the Q-MAWS benchmark) | about 1 MB | github.com/chaoszhang/ASTER tag v1.25 | AGPL-3.0 |
| 16 | `tree-qmc-v3.0.4-b5bfe82-linux64` and the v3.0.4 source archive | 0.4 MB + about 1 MB | github.com/molloy-lab/TREE-QMC release v3.0.4 | MIT |

## Totals

About 4.4 GB of network traffic and of disk on D: (59 GB free). Unpacked
ASTRAL-II gene trees are larger; only the conditions used are unpacked,
after the Q-MAWS benchmark.

## How

Dryad and GitHub files with `curl` straight to D:. IDEALS refuses
command-line clients, so its signed S3 links are read in the browser and
fetched with `curl` straight to D: (the browser's own download folder is on
C:). Dryad's published SHA-256 values are checked.

## Status (2026-10-26)

- Downloaded: items 4 to 11 (IDEALS) and 13 to 16 (tools). IDEALS refuses
  command-line clients on its pages, but its files are plain S3 objects:
  their signed links (valid 15 minutes) were read in the browser and
  fetched with `curl`. Sizes match the repository's records. The README
  (item 12) is served by IDEALS itself and was skipped; it only describes
  the file layout.
- The ASTER and TREE-QMC source archives are 54 MB and 70 MB (with their
  test data), not about 1 MB as estimated above.
- Not yet downloaded: items 1 to 3 (Dryad). Dryad now protects downloads
  with bot detection, which is not circumvented; the owner downloads these
  three files in a browser into `D:\_external\wqfm_bench\datasets\astral2\`,
  and their SHA-256 is then checked against Dryad's published values
  (in `bench/datasets.tsv`).
