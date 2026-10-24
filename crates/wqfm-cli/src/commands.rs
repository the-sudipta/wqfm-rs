//! The subcommands.

use std::fs;
use std::path::Path;

use wqfm::{amalgamate, parse_wqfm_input, Options, Quartet, Topology};

use crate::args::Command;

pub fn run(command: Command) -> Result<(), String> {
    match command {
        Command::Amalgamate {
            input,
            output,
            beta,
        } => {
            let (names, quartets) = read_quartets(&input)?;
            let options = Options {
                beta,
                ..Options::default()
            };
            let result = amalgamate(&names, &quartets, &options);
            match output {
                Some(path) => fs::write(&path, format!("{}\n", result.newick))
                    .map_err(|e| format!("{}: {e}", path.display()))?,
                None => println!("{}", result.newick),
            }
            eprintln!(
                "score {} of {} ({:.6})",
                result.score,
                result.total_weight,
                ratio(result.score, result.total_weight)
            );
            Ok(())
        }
        Command::Score { input, tree } => {
            let (names, quartets) = read_quartets(&input)?;
            let text = fs::read_to_string(&tree).map_err(|e| format!("{}: {e}", tree.display()))?;
            let topo = Topology::from_newick(text.trim(), &names)?;
            let (score, total) = topo.score(&quartets);
            println!("{score}\t{total}\t{:.6}", ratio(score, total));
            Ok(())
        }
    }
}

fn read_quartets(path: &Path) -> Result<(Vec<String>, Vec<Quartet>), String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut names = Vec::new();
    let quartets = parse_wqfm_input(&text, &mut names)?;
    Ok((names, quartets))
}

fn ratio(score: f64, total: f64) -> f64 {
    if total > 0.0 {
        score / total
    } else {
        0.0
    }
}
