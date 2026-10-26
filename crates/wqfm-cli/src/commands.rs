//! The subcommands. Results go to files or standard output in the same
//! formats as before; messages for people go through `ui` to standard error.

use std::fs;
use std::path::Path;
use std::time::Instant;

use wqfm::newick::{nrf, Tree};
use wqfm::{
    amalgamate, count_quartets_with_progress, counts_to_wqfm_input, read_wqfm_input, Options,
    Quartet, Topology,
};

use crate::args::Command;
use crate::ui;

pub fn run(command: Command) -> Result<(), String> {
    let started = Instant::now();
    match command {
        Command::Amalgamate {
            input,
            output,
            beta,
        } => {
            ui::header("amalgamate");
            let (names, quartets) = read_quartets(&input)?;
            ui::line(
                "read",
                &format!(
                    "{} quartets on {} taxa  ({})",
                    ui::thousands(quartets.len()),
                    names.len(),
                    input.display()
                ),
            );
            let options = Options {
                beta,
                ..Options::default()
            };
            let result = amalgamate(&names, &quartets, &options);
            if !result.java_order_exact {
                ui::warning(
                    "the wQFM jar's own hash-map order is not certain on this input; \
                     near-ties may go differently from the jar",
                );
            }
            match &output {
                Some(path) => fs::write(path, format!("{}\n", result.newick))
                    .map_err(|e| format!("{}: {e}", path.display()))?,
                None => println!("{}", result.newick),
            }
            if let Some(path) = &output {
                ui::line("tree", &format!("written to {}", path.display()));
            }
            ui::line(
                "score",
                &format!(
                    "{} of {} quartet weight  ({:.2} %)",
                    ui::weight(result.score),
                    ui::weight(result.total_weight),
                    100.0 * ratio(result.score, result.total_weight)
                ),
            );
            if beta != 1.0 {
                ui::line("beta", &beta.to_string());
            }
            ui::done(started);
            Ok(())
        }
        Command::Gt2wq {
            input,
            output,
            keep_unresolved,
            threads,
        } => {
            ui::header("gt2wq");
            let text =
                fs::read_to_string(&input).map_err(|e| format!("{}: {e}", input.display()))?;
            let trees = text
                .lines()
                .map(str::trim)
                .enumerate()
                .filter(|(_, l)| !l.is_empty())
                .map(|(i, l)| Tree::parse(l).map_err(|e| format!("line {}: {e}", i + 1)))
                .collect::<Result<Vec<_>, _>>()?;
            ui::line(
                "read",
                &format!(
                    "{} gene trees  ({})",
                    ui::thousands(trees.len()),
                    input.display()
                ),
            );
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .map_err(|e| e.to_string())?;
            ui::line("threads", &pool.current_num_threads().to_string());
            let progress = ui::Progress::new("counting", trees.len());
            let counts = pool.install(|| count_quartets_with_progress(&trees, &|| progress.tick()));
            progress.finish();
            let counts = counts?;
            let out = counts_to_wqfm_input(&counts, keep_unresolved);
            let unresolved = counts.counts.values().filter(|c| c[3] > 0).count();
            match &output {
                Some(path) => {
                    fs::write(path, &out).map_err(|e| format!("{}: {e}", path.display()))?
                }
                None => print!("{out}"),
            }
            ui::line(
                "quartets",
                &format!(
                    "{} lines on {} taxa ({} four-taxon sets)",
                    ui::thousands(out.lines().count()),
                    counts.names.len(),
                    ui::thousands(counts.counts.len())
                ),
            );
            if unresolved > 0 {
                ui::line(
                    "unresolved",
                    &format!(
                        "{} four-taxon sets left unresolved by some gene tree ({})",
                        ui::thousands(unresolved),
                        if keep_unresolved {
                            "written as ((a,b,c,d)); n"
                        } else {
                            "left out"
                        }
                    ),
                );
            }
            if let Some(path) = &output {
                ui::line("written", &path.display().to_string());
            }
            ui::done(started);
            Ok(())
        }
        Command::Compare { a, b } => {
            let read = |p: &Path| -> Result<Tree, String> {
                let text = fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))?;
                Tree::parse(text.trim()).map_err(|e| format!("{}: {e}", p.display()))
            };
            let (ta, tb) = (read(&a)?, read(&b)?);
            let (mut la, mut lb) = (ta.leaf_names(), tb.leaf_names());
            la.sort();
            lb.sort();
            if la != lb {
                return Err("the trees have different taxa".into());
            }
            let (rf, nrf) = nrf(&ta, &tb);
            println!("{rf}\t{nrf:.6}");
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
    let file = fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut names = Vec::new();
    let quartets = read_wqfm_input(std::io::BufReader::new(file), &mut names)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    Ok((names, quartets))
}

fn ratio(score: f64, total: f64) -> f64 {
    if total > 0.0 {
        score / total
    } else {
        0.0
    }
}
