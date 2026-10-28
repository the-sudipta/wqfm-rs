<img src="../assets/icon/wqfm-rs-icon.svg" width="64" align="right" alt="">

# Using wQFM-rs

wQFM-rs builds a species tree from weighted quartets with the wQFM method,
and returns exactly the tree and score of the wQFM v1.4 Java program. It can
also make those quartets from gene trees.

## Install

See the README. After the first release: a binary from the GitHub release,
`cargo install wqfm-rs`, or `conda install -c bioconda wqfm-rs`.

## From gene trees to a species tree (worked example)

Gene trees, one Newick tree per line (branch lengths and support values are
allowed and ignored; taxa may be missing from some trees):

```text
((A,B),(C,(D,E)));
((A,B),((C,D),E));
(((A,C),B),(D,E));
```

```bash
wqfm-rs gt2wq -i genes.nwk -o quartets.txt
wqfm-rs amalgamate -i quartets.txt -o species.nwk
wqfm-rs score -i quartets.txt -t species.nwk
```

`gt2wq` writes how many gene trees show each quartet topology:

```text
((A,B),(C,D)); 2
((A,B),(C,E)); 2
...
```

`amalgamate` writes the species tree to `species.nwk` and shows a short
summary on standard error:

```text
wqfm-rs 0.1.0  amalgamate
  read     555 quartets on 11 taxa  (quartets.txt)
  tree     written to species.nwk
  score    1,382 of 1,650 quartet weight  (83.76 %)
  done     in 0.01 s
```

The score is the weight of the quartets the tree agrees with, of the total.
On a terminal the summary is in colour, and long `gt2wq` runs show a
progress bar; `-q` (`--quiet`) turns both off. Colours are also off when
standard error is redirected or `NO_COLOR` is set. `score` prints the same three numbers for any tree, tab-separated.

## Commands

### `wqfm-rs amalgamate`

| Option | Meaning |
|---|---|
| `-i, --input FILE` | weighted quartets, one `((a,b),(c,d)); weight` per line |
| `-o, --output FILE` | the species tree (Newick); standard output if left out |
| `--beta B` | weight of violated quartets in the partition score, default 1 (the jar's `-beta B`) |

The output is unrooted, written with a basal multifurcation. Taxon names
are written as given.

### `wqfm-rs gt2wq`

| Option | Meaning |
|---|---|
| `-i, --input FILE` | gene trees, one Newick tree per line |
| `-o, --output FILE` | weighted quartets; standard output if left out |
| `--keep-unresolved` | also write 4-sets that a gene tree leaves unresolved (see below) |
| `-t, --threads N` | worker threads, default all cores; the output does not depend on it |

The output is byte for byte what wQFM's `generate_wqrts.py` writes, except
that 4-sets a gene tree leaves unresolved (a polytomy) are left out: the
script writes them as `((a,b,c,d)); n`, and the wQFM jar refuses such lines
and stops. With `--keep-unresolved` they are written exactly as the script
does.

### `wqfm-rs score`

`-i FILE` (weighted quartets) and `-t FILE` (a tree on the same taxa).
Prints `satisfied weight`, `total weight` and their ratio.

### `wqfm-rs compare`

Two tree files on the same taxa. Prints the Robinson–Foulds distance and the
normalised distance (divided by 2(n − 3)).

## Desktop app

`wqfm-rs-gui` runs the same two steps in a window, with the same results:

1. **Input:** choose *Gene trees* or *Weighted quartets*, then *Choose file...*,
   drop a file on the window, or *Use example* (wQFM's 11-taxon example). The
   kind of file is guessed from its first line and can be changed.
2. **Settings:** *Beta* (as `--beta`), and for gene trees *Threads* and *Keep
   unresolved quartets* (as `--keep-unresolved`). Hover over any control for
   what it does and its default.
3. **Run:** *Build species tree*. The window stays usable; a bar shows the
   progress of counting quartets.

The result shows the number of taxa, quartets and gene trees, the quartet
score, the time and the tree. *Copy Newick* puts the tree on the clipboard
for a tree viewer; *Save tree...*, *Save quartets...* and *Save summary...*
write files. Errors are shown in plain words with what to do, as in the
command line.

## Input format of weighted quartets

One quartet per line, as wQFM reads it: `((a,b),(c,d)); w`. Spaces are
allowed anywhere; the weight may be written as `3`, `0.5` or `2e-3`; zero
and negative weights are used as given, as by wQFM. Names may contain any
character except brackets, commas, semicolons and spaces. The same quartet
may appear more than once; each line counts.

The file is read with wQFM's rules:

- each line must have exactly three `(` and three `)`; a line such as
  `((a,b,c,d)); 1` is an error;
- reading stops at the first empty line; if quartets follow it, wQFM-rs
  reports an error (wQFM would ignore them silently);
- weights must be finite numbers (`1e309` or `NaN` are errors).

## Messages

Errors start with `error:`, followed where possible by a `hint:` line
saying what to do. The exit code is 1.

| Message | Meaning and what to do |
|---|---|
| `line N: not a quartet ((a,b),(c,d)); w (an unresolved 4-set?)` | the line is not a quartet; if the file came from `generate_wqrts.py`, re-make it with `wqfm-rs gt2wq` |
| `line N: expected ((a,b),(c,d)); w` | the line does not have four names and a weight |
| `line N: weight X is not a number` / `not a finite number` | fix the weight |
| `line N: quartets after an empty line (wQFM would ignore them)` | remove the empty line |
| `gene tree N: a taxon appears twice` | a gene tree repeats a leaf name |
| `invalid Newick at character K: ...` | a syntax error in a tree, at the given place |
| `the trees have different taxa` | `compare` needs two trees on the same taxa |
| `warning: the wQFM jar's own hash-map order is not certain on this input` | two quartets on this input have the same Java hash code; the jar's own result can then differ between runs, and wQFM-rs's may differ from a given jar run on a near-tie. The tree is still a valid wQFM result. |

## Exactness

wQFM-rs reproduces wQFM v1.4 rule by rule, including its tie-breaking and
the iteration order of Java's `HashMap`, which decides near-ties through the
order of floating-point sums. The checks behind this claim are in
`validation/` and `docs/decisions.md`.

## Library

The crate `wqfm` exposes the same functions (`amalgamate`, `count_quartets`,
`parse_wqfm_input`, `Topology::score`, ...); see its documentation on
docs.rs and the example in the README.
