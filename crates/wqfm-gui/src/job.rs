//! A run of wQFM-rs in a background thread: gene trees or weighted quartets
//! in, species tree out, with progress for the window.

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use wqfm::newick::Tree;
use wqfm::{
    amalgamate, count_quartets_with_progress, counts_to_wqfm_input, parse_wqfm_input,
    read_wqfm_input, Options,
};

/// What the input file holds.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    GeneTrees,
    Quartets,
}

/// Where the input comes from.
#[derive(Clone)]
pub enum Source {
    File(PathBuf),
    /// Built-in example (name, text).
    Text(String, &'static str),
}

impl Source {
    pub fn name(&self) -> String {
        match self {
            Source::File(p) => p
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| p.display().to_string()),
            Source::Text(name, _) => name.clone(),
        }
    }

    fn text(&self) -> Result<String, String> {
        match self {
            Source::File(p) => {
                std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))
            }
            Source::Text(_, t) => Ok(t.to_string()),
        }
    }
}

pub struct Settings {
    pub beta: f64,
    pub threads: usize,
    pub keep_unresolved: bool,
}

/// The result of a finished run.
pub struct Outcome {
    pub newick: String,
    pub tree: Tree,
    pub score: f64,
    pub total_weight: f64,
    pub taxa: usize,
    pub quartets: usize,
    pub gene_trees: Option<usize>,
    /// The weighted quartets made from gene trees (to save them).
    pub quartet_text: Option<String>,
    pub unresolved: usize,
    pub java_order_exact: bool,
    pub seconds: f64,
    pub input: String,
    pub beta: f64,
}

/// Progress shared with the window.
#[derive(Default)]
pub struct Progress {
    pub done: AtomicUsize,
    pub total: AtomicUsize,
    pub stage: Mutex<String>,
}

impl Progress {
    fn stage(&self, s: &str) {
        *self.stage.lock().expect("stage") = s.to_string();
    }
}

pub struct Job {
    pub progress: Arc<Progress>,
    pub result: Receiver<Result<Outcome, String>>,
    pub started: Instant,
}

pub fn start(mode: Mode, source: Source, settings: Settings) -> Job {
    let progress = Arc::new(Progress::default());
    let (tx, rx) = channel();
    let p = Arc::clone(&progress);
    std::thread::spawn(move || {
        let _ = tx.send(run(mode, &source, &settings, &p));
    });
    Job {
        progress,
        result: rx,
        started: Instant::now(),
    }
}

fn run(mode: Mode, source: &Source, s: &Settings, p: &Progress) -> Result<Outcome, String> {
    let started = Instant::now();
    let (names, quartets, gene_trees, quartet_text, unresolved) = match mode {
        Mode::GeneTrees => {
            p.stage("Reading gene trees");
            let text = source.text()?;
            let trees = text
                .lines()
                .map(str::trim)
                .enumerate()
                .filter(|(_, l)| !l.is_empty())
                .map(|(i, l)| Tree::parse(l).map_err(|e| format!("line {}: {e}", i + 1)))
                .collect::<Result<Vec<_>, _>>()?;
            p.total.store(trees.len(), Ordering::Relaxed);
            p.stage("Counting quartets");
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(s.threads)
                .build()
                .map_err(|e| e.to_string())?;
            let counts = pool.install(|| {
                count_quartets_with_progress(&trees, &|| {
                    p.done.fetch_add(1, Ordering::Relaxed);
                })
            })?;
            let unresolved = counts.counts.values().filter(|c| c[3] > 0).count();
            let qtext = counts_to_wqfm_input(&counts, s.keep_unresolved);
            let mut names = Vec::new();
            let quartets = parse_wqfm_input(&qtext, &mut names)?;
            (names, quartets, Some(trees.len()), Some(qtext), unresolved)
        }
        Mode::Quartets => {
            p.stage("Reading weighted quartets");
            let mut names = Vec::new();
            let quartets = match source {
                Source::File(path) => {
                    let f = std::fs::File::open(path)
                        .map_err(|e| format!("{}: {e}", path.display()))?;
                    read_wqfm_input(std::io::BufReader::new(f), &mut names)
                }
                Source::Text(_, t) => parse_wqfm_input(t, &mut names),
            }?;
            (names, quartets, None, None, 0)
        }
    };
    p.stage("Building the species tree");
    let options = Options {
        beta: s.beta,
        ..Options::default()
    };
    let result = amalgamate(&names, &quartets, &options);
    let tree = Tree::parse(&result.newick).map_err(|e| e.to_string())?;
    Ok(Outcome {
        newick: result.newick,
        tree,
        score: result.score,
        total_weight: result.total_weight,
        taxa: names.len(),
        quartets: quartets.len(),
        gene_trees,
        quartet_text,
        unresolved,
        java_order_exact: result.java_order_exact,
        seconds: started.elapsed().as_secs_f64(),
        input: source.name(),
        beta: s.beta,
    })
}
