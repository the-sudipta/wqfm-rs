# wqfm

Library of [wQFM-rs](https://github.com/the-sudipta/wqfm-rs): an exact Rust
implementation of wQFM (Mahbub et al., *Bioinformatics* 37:3734–3743, 2021),
the weighted quartet amalgamation for species trees, with gene trees to
weighted quartets (`count_quartets`), the quartet consistency score and an
exhaustive oracle for small inputs. It returns the same tree and score as the
wQFM v1.4 jar, including its tie-breaking.

```rust
use wqfm::{amalgamate, Options, Quartet};
let names: Vec<String> = ["a", "b", "c", "d"].map(String::from).to_vec();
let tree = amalgamate(&names, &[Quartet::new(0, 1, 2, 3, 1.0)], &Options::default());
assert_eq!(tree.score, 1.0);
```

Licence: Apache-2.0.
