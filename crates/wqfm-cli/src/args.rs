//! Command-line arguments.

use std::path::PathBuf;

use clap::builder::styling::{AnsiColor, Styles};
use clap::{Parser, Subcommand};

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Cyan.on_default().bold())
    .usage(AnsiColor::Cyan.on_default().bold())
    .literal(AnsiColor::Green.on_default().bold())
    .placeholder(AnsiColor::BrightBlack.on_default())
    .error(AnsiColor::Red.on_default().bold())
    .valid(AnsiColor::Green.on_default())
    .invalid(AnsiColor::Yellow.on_default());

const EXAMPLES: &str = "Examples:
  wqfm-rs gt2wq -i genetrees.nwk -o quartets.txt
  wqfm-rs amalgamate -i quartets.txt -o species.nwk
  wqfm-rs score -i quartets.txt -t species.nwk
  wqfm-rs compare species.nwk true.nwk

Same trees and scores as wQFM v1.4. Manual: docs/USAGE.md";

/// Exact, fast Rust implementation of wQFM: weighted quartets and gene trees
/// to species trees.
#[derive(Parser)]
#[command(
    name = "wqfm-rs",
    bin_name = "wqfm-rs",
    version,
    about,
    styles = STYLES,
    after_help = EXAMPLES,
    arg_required_else_help = true
)]
pub struct Cli {
    /// No header, summary or progress bar on standard error (errors still show).
    #[arg(short, long, global = true)]
    pub quiet: bool,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Builds a species tree from weighted quartets in wQFM's format.
    #[command(after_help = "Example:
  wqfm-rs amalgamate -i quartets.txt -o species.nwk

Input lines look like ((a,b),(c,d)); 0.75. The tree goes to the output file
(or standard output); its quartet score is shown on standard error.")]
    Amalgamate {
        /// Weighted quartets, one `((a,b),(c,d)); w` per line.
        #[arg(short, long)]
        input: PathBuf,
        /// Output Newick file (standard output if left out).
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Weight of violated quartets in the partition score (as the jar's -beta).
        #[arg(long, default_value_t = 1.0)]
        beta: f64,
    },
    /// Turns gene trees into weighted quartets (as wQFM's generate_wqrts.py).
    #[command(after_help = "Example:
  wqfm-rs gt2wq -i genetrees.nwk -o quartets.txt

One Newick gene tree per line; branch lengths and support values are ignored.
Weights are the number of gene trees that show each quartet.")]
    Gt2wq {
        /// Gene trees in Newick, one per line.
        #[arg(short, long)]
        input: PathBuf,
        /// Output file of weighted quartets (standard output if left out).
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Also write 4-sets that a gene tree leaves unresolved, as
        /// `((a,b,c,d)); n`, byte for byte as generate_wqrts.py (the wQFM
        /// jar refuses such lines).
        #[arg(long)]
        keep_unresolved: bool,
        /// Worker threads (0 = all cores).
        #[arg(short = 't', long, default_value_t = 0)]
        threads: usize,
    },
    /// Robinson–Foulds distance between two trees on the same taxa.
    #[command(after_help = "Example:
  wqfm-rs compare species.nwk true.nwk

Prints the RF distance and the normalised RF distance, tab-separated.")]
    Compare {
        /// First Newick tree.
        a: PathBuf,
        /// Second Newick tree.
        b: PathBuf,
    },
    /// Scores a tree against weighted quartets (satisfied / total weight).
    #[command(after_help = "Example:
  wqfm-rs score -i quartets.txt -t species.nwk

Prints the satisfied weight, the total weight and their ratio, tab-separated.")]
    Score {
        /// Weighted quartets, one `((a,b),(c,d)); w` per line.
        #[arg(short, long)]
        input: PathBuf,
        /// Newick tree on the same taxa.
        #[arg(short, long)]
        tree: PathBuf,
    },
}
