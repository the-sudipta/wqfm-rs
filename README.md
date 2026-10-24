# wQFM-rs

An exact Rust implementation of **wQFM**, the weighted quartet amalgamation
method of Mahbub, Wahab, Reaz, Rahman and Bayzid (*Bioinformatics*
37:3734–3743, 2021). It follows the official Java code (version 1.4) rule by
rule, including its tie-breaking, so it returns the same tree and score.

## Build

```bash
cargo build --release
```

The binary is `target/release/wqfm-rs`.

## Use

Input is wQFM's format: one weighted quartet per line.

```text
((a,b),(c,d)); 0.75
((a,c),(b,e)); 0.20
```

```bash
wqfm-rs amalgamate -i quartets.txt -o tree.nwk
wqfm-rs score -i quartets.txt -t tree.nwk
```

`amalgamate` takes `--beta` (default 1), the weight of violated quartets in
the partition score. `score` prints the satisfied weight, the total weight and
their ratio.

## Library

```rust
use wqfm::{amalgamate, Options, Quartet};
let names: Vec<String> = ["a", "b", "c", "d"].map(String::from).to_vec();
let tree = amalgamate(&names, &[Quartet::new(0, 1, 2, 3, 1.0)], &Options::default());
println!("{}", tree.newick);
```

## Layout

| Path | Contents |
|---|---|
| `crates/wqfm` | the library, one feature per file |
| `crates/wqfm-cli` | the `wqfm-rs` command |
| `docs/decisions.md` | design decisions |

## Licence

Apache-2.0. wQFM is © its authors under Apache-2.0; see `NOTICE`. No Java
code is included.
