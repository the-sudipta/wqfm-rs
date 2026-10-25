//! `wqfm`: an exact Rust implementation of wQFM (Mahbub, Wahab, Reaz, Rahman
//! and Bayzid, Bioinformatics 37:3734–3743, 2021; official code
//! github.com/Mahim1997/wQFM-2020, version 1.4, Apache License 2.0), the
//! weighted quartet amalgamation, with an exhaustive tree oracle and the
//! weighted quartet consistency score.
//!
//! ```
//! use wqfm::{amalgamate, Options, Quartet};
//! let names: Vec<String> = ["a", "b", "c", "d"].map(String::from).to_vec();
//! let quartets = [Quartet::new(0, 1, 2, 3, 1.0)];
//! let tree = amalgamate(&names, &quartets, &Options::default());
//! assert_eq!(tree.score, 1.0);
//! ```

#![deny(missing_docs)]

mod amalgamate;
mod bipartition;
mod divide;
mod edge_tree;
mod fm;
mod gene_trees;
mod graph;
mod io;
mod java_order;
mod join;
mod level;
pub mod newick;
mod node;
mod options;
mod oracle;
mod partition;
mod quartet;
mod random;
mod topology;

pub use amalgamate::{amalgamate, Amalgamation};
pub use gene_trees::{count_quartets, QuartetCounts};
pub use io::{counts_to_wqfm_input, parse_wqfm_input, read_wqfm_input, to_wqfm_input};
#[doc(hidden)]
pub use java_order::java_hashmap_orders;
pub use options::Options;
pub use oracle::{all_binary_trees, exhaustive_best};
pub use quartet::{quartets_of, Quartet};
pub use random::{random_binary_tree, SplitMix64};
pub use topology::{induced_quartets, Topology};
