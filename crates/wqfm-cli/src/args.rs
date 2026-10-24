//! Command-line arguments.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Exact Rust implementation of wQFM (weighted quartet amalgamation).
#[derive(Parser)]
#[command(name = "wqfm-rs", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Builds a species tree from weighted quartets in wQFM's format.
    Amalgamate {
        /// Weighted quartets, one `((a,b),(c,d)); w` per line.
        #[arg(short, long)]
        input: PathBuf,
        /// Output Newick file (standard output if left out).
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Weight of violated quartets in the partition score.
        #[arg(long, default_value_t = 1.0)]
        beta: f64,
    },
    /// Turns gene trees into weighted quartets (as wQFM's generate_wqrts.py).
    Gt2wq {
        /// Gene trees in Newick, one per line.
        #[arg(short, long)]
        input: PathBuf,
        /// Output file of weighted quartets (standard output if left out).
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Leave out 4-sets that a gene tree leaves unresolved. By default
        /// they are written as the original script writes them.
        #[arg(long)]
        skip_unresolved: bool,
        /// Worker threads (0 = all cores).
        #[arg(short = 't', long, default_value_t = 0)]
        threads: usize,
    },
    /// Scores a tree against weighted quartets (satisfied / total weight).
    Score {
        /// Weighted quartets, one `((a,b),(c,d)); w` per line.
        #[arg(short, long)]
        input: PathBuf,
        /// Newick tree on the same taxa.
        #[arg(short, long)]
        tree: PathBuf,
    },
}
