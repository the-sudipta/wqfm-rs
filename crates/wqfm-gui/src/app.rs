//! The window: input and settings on the left, the result on the right.

use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::time::Duration;

use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke, Vec2};

use crate::job::{self, Job, Mode, Outcome, Settings, Source};
use crate::shot::Shot;
use crate::{theme, tree_view};

const EXAMPLE: &str = include_str!("../assets/example_genetrees.tre");
const EXAMPLE_NAME: &str = "Example: 11 taxa, 5 gene trees (from wQFM)";

pub struct App {
    mode: Mode,
    source: Option<Source>,
    beta: f64,
    threads: usize,
    keep_unresolved: bool,
    job: Option<Job>,
    outcome: Option<Outcome>,
    error: Option<String>,
    notice: Option<String>,
    logo: Option<egui::TextureHandle>,
    pub shot: Option<Shot>,
}

impl App {
    pub fn new(ctx: &egui::Context) -> Self {
        let logo = crate::icon::rgba().map(|(rgba, w, h)| {
            let image = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
            ctx.load_texture("wqfm-rs-logo", image, egui::TextureOptions::LINEAR)
        });
        Self {
            mode: Mode::GeneTrees,
            source: None,
            beta: 1.0,
            threads: 0,
            keep_unresolved: false,
            job: None,
            outcome: None,
            error: None,
            notice: None,
            logo,
            shot: None,
        }
    }

    pub fn use_example(&mut self) {
        self.mode = Mode::GeneTrees;
        self.source = Some(Source::Text(EXAMPLE_NAME.to_string(), EXAMPLE));
    }

    pub fn run(&mut self) {
        let Some(source) = self.source.clone() else {
            return;
        };
        self.error = None;
        self.notice = None;
        self.outcome = None;
        self.job = Some(job::start(
            self.mode,
            source,
            Settings {
                beta: self.beta,
                threads: self.threads,
                keep_unresolved: self.keep_unresolved,
            },
        ));
    }

    fn poll(&mut self, ctx: &egui::Context) {
        if let Some(job) = &self.job {
            match job.result.try_recv() {
                Ok(Ok(outcome)) => {
                    self.outcome = Some(outcome);
                    self.job = None;
                }
                Ok(Err(e)) => {
                    self.error = Some(e);
                    self.job = None;
                }
                Err(_) => ctx.request_repaint_after(Duration::from_millis(80)),
            }
        }
    }

    fn dropped_file(&mut self, ctx: &egui::Context) {
        let dropped: Option<PathBuf> =
            ctx.input(|i| i.raw.dropped_files.first().map(|f| f.path().to_path_buf()));
        if let Some(p) = dropped {
            self.set_file(p);
        }
    }

    fn set_file(&mut self, p: PathBuf) {
        // A first guess of the mode from the first line: quartets look like
        // ((a,b),(c,d)); w.
        if let Ok(text) = std::fs::read_to_string(&p) {
            let first = text.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
            let quartet_like = first.trim_start().starts_with("((")
                && first.contains(");")
                && first.matches('(').count() == 3;
            self.mode = if quartet_like {
                Mode::Quartets
            } else {
                Mode::GeneTrees
            };
        }
        self.source = Some(Source::File(p));
        self.error = None;
    }

    // ----- Left panel ----------------------------------------------------

    fn sidebar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if let Some(t) = &self.logo {
                ui.add(egui::Image::new(t).fit_to_exact_size(Vec2::splat(40.0)));
            }
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("wQFM-rs")
                        .size(22.0)
                        .strong()
                        .color(theme::INK),
                );
                ui.label(
                    RichText::new(format!("version {}", env!("CARGO_PKG_VERSION")))
                        .size(12.0)
                        .color(theme::MUTED),
                );
            });
        });
        ui.label(
            RichText::new(
                "Species trees from weighted quartets or gene trees, exactly as wQFM v1.4.",
            )
            .size(13.0)
            .color(theme::MUTED),
        );
        ui.add_space(10.0);

        step(ui, "1", "Input");
        ui.horizontal(|ui| {
            ui.radio_value(&mut self.mode, Mode::GeneTrees, "Gene trees")
                .on_hover_text("A file of gene trees in Newick, one per line. wQFM-rs first counts, for every four taxa, how many gene trees show each quartet (as wQFM's generate_wqrts.py), then builds the species tree.");
            ui.radio_value(&mut self.mode, Mode::Quartets, "Weighted quartets")
                .on_hover_text("A file of weighted quartets in wQFM's format, one ((a,b),(c,d)); weight per line, for example from Q-MAWS or from generate_wqrts.py.");
        });
        let running = self.job.is_some();
        ui.horizontal(|ui| {
            if ui
                .add_enabled(!running, egui::Button::new("Choose file..."))
                .on_hover_text("Open a gene-tree or quartet file. You can also drop a file anywhere on this window.")
                .clicked()
            {
                if let Some(p) = rfd::FileDialog::new()
                    .add_filter("Trees and quartets", &["tre", "nwk", "newick", "txt", "wqrts", "tree", "trees"])
                    .add_filter("All files", &["*"])
                    .pick_file()
                {
                    self.set_file(p);
                }
            }
            if ui
                .add_enabled(!running, egui::Button::new("Use example"))
                .on_hover_text("wQFM's own example: 5 estimated gene trees on 11 taxa. A quick way to see the app work.")
                .clicked()
            {
                self.use_example();
            }
        });
        let (text, color) = match &self.source {
            Some(s) => (s.name(), theme::INK),
            None => (
                "No file yet: choose one, drop one here, or use the example".to_string(),
                theme::MUTED,
            ),
        };
        egui::Frame::new()
            .fill(theme::SUNKEN)
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new(text).size(13.0).color(color));
            })
            .response
            .on_hover_text(match &self.source {
                Some(Source::File(p)) => p.display().to_string(),
                _ => "The input of the next run.".to_string(),
            });
        ui.add_space(10.0);

        step(ui, "2", "Settings");
        ui.horizontal(|ui| {
            ui.label("Beta").on_hover_text(
                "Weight of violated quartets in the partition score: score = satisfied - beta x violated. wQFM's default is 1; this is the jar's -beta option.",
            );
            ui.add_enabled(
                !running,
                egui::DragValue::new(&mut self.beta).speed(0.05).range(0.0..=10.0).max_decimals(2),
            )
            .on_hover_text("Default 1. Drag or type a value.");
        });
        if self.mode == Mode::GeneTrees {
            let cores = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1);
            ui.horizontal(|ui| {
                ui.label("Threads").on_hover_text(
                    "How many processor cores count quartets at once. 0 means all cores. The result is the same for any number.",
                );
                ui.add_enabled(!running, egui::Slider::new(&mut self.threads, 0..=cores))
                    .on_hover_text("0 = all cores");
            });
            ui.add_enabled(!running, egui::Checkbox::new(&mut self.keep_unresolved, "Keep unresolved quartets"))
                .on_hover_text("Off (default): four-taxon sets that a gene tree leaves unresolved (a polytomy) are left out. On: they are written as ((a,b,c,d)); n, exactly as generate_wqrts.py does; the wQFM jar refuses such lines, and so does wQFM-rs, so the run will stop with an error if there are any.");
        }
        ui.add_space(12.0);

        step(ui, "3", "Run");
        let ready = self.source.is_some() && !running;
        let run = ui
            .add_enabled(
                ready,
                egui::Button::new(RichText::new("Build species tree").size(16.0).strong().color(Color32::WHITE))
                    .fill(theme::ORANGE)
                    .corner_radius(CornerRadius::same(10))
                    .min_size(Vec2::new(ui.available_width(), 42.0)),
            )
            .on_hover_text("Runs wQFM-rs on the input with these settings. The window stays usable while it works.");
        if run.clicked() {
            self.run();
        }
        if let Some(job) = &self.job {
            let p = &job.progress;
            let total = p.total.load(Ordering::Relaxed);
            let done = p.done.load(Ordering::Relaxed);
            let stage = p.stage.lock().map(|s| s.clone()).unwrap_or_default();
            let fraction = if total > 0 && stage == "Counting quartets" {
                done as f32 / total as f32
            } else {
                0.0
            };
            ui.add(
                egui::ProgressBar::new(fraction)
                    .animate(fraction == 0.0)
                    .fill(theme::TEAL)
                    .text(if stage == "Counting quartets" {
                        format!("{stage}: {done} of {total} gene trees")
                    } else {
                        stage
                    }),
            );
            ui.label(
                RichText::new(format!("{:.1} s", job.started.elapsed().as_secs_f64()))
                    .size(12.0)
                    .color(theme::MUTED),
            );
        }
        if let Some(e) = &self.error {
            message(
                ui,
                theme::ERROR_BG,
                theme::ERROR_INK,
                "Something is wrong with the input",
                e,
                wqfm::error_hint(e),
            );
        }
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.label(
                RichText::new("Apache-2.0. wQFM: Mahbub et al., Bioinformatics 2021.")
                    .size(11.0)
                    .color(theme::MUTED),
            );
        });
    }

    // ----- Result --------------------------------------------------------

    fn result(&mut self, ui: &mut egui::Ui) {
        let Some(o) = &self.outcome else {
            self.welcome(ui);
            return;
        };
        ui.label(
            RichText::new("Species tree")
                .heading()
                .strong()
                .color(theme::INK),
        );
        ui.label(
            RichText::new(format!("from {}", o.input))
                .size(13.0)
                .color(theme::MUTED),
        );
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            stat(
                ui,
                "Taxa",
                &o.taxa.to_string(),
                "Number of taxa (leaves) in the species tree.",
            );
            stat(
                ui,
                "Quartets",
                &thousands(o.quartets),
                "Number of weighted quartets the tree was built from.",
            );
            if let Some(g) = o.gene_trees {
                stat(
                    ui,
                    "Gene trees",
                    &thousands(g),
                    "Number of gene trees read.",
                );
            }
            stat(
                ui,
                "Quartet score",
                &format!(
                    "{:.2} %",
                    100.0 * o.score / o.total_weight.max(f64::MIN_POSITIVE)
                ),
                &format!(
                    "Share of the total quartet weight that the tree agrees with: {} of {}.",
                    o.score, o.total_weight
                ),
            );
            stat(
                ui,
                "Time",
                &seconds(o.seconds),
                "Time for the whole run on this computer.",
            );
        });
        if !o.java_order_exact {
            message(
                ui,
                theme::WARN_BG,
                theme::WARN_INK,
                "Note",
                "On this input the wQFM jar's own hash-map order is not certain, so near-ties may go differently from a given run of the jar. The tree is still a valid wQFM result.",
                None,
            );
        }
        if o.unresolved > 0 {
            message(
                ui,
                theme::WARN_BG,
                theme::WARN_INK,
                "Unresolved quartets",
                &format!(
                    "{} four-taxon sets were left unresolved by at least one gene tree (a polytomy); they were {}.",
                    thousands(o.unresolved),
                    if self.keep_unresolved { "kept" } else { "left out, as the wQFM jar cannot read them" }
                ),
                None,
            );
        }
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if ui
                .button("Copy Newick")
                .on_hover_text("Copies the tree in Newick format, for a tree viewer such as iTOL, FigTree or Dendroscope.")
                .clicked()
            {
                ui.ctx().copy_text(o.newick.clone());
                self.notice = Some("The tree is on the clipboard.".into());
            }
            if ui.button("Save tree...").on_hover_text("Saves the tree as a Newick file (.nwk).").clicked() {
                self.notice = save(&format!("{}\n", o.newick), "species_tree.nwk", "Newick", &["nwk", "tre"]);
            }
            if let Some(q) = &o.quartet_text {
                if ui
                    .button("Save quartets...")
                    .on_hover_text("Saves the weighted quartets made from the gene trees, in wQFM's format (the same file as `wqfm-rs gt2wq`).")
                    .clicked()
                {
                    self.notice = save(q, "quartets.txt", "Weighted quartets", &["txt", "wqrts"]);
                }
            }
            if ui
                .button("Save summary...")
                .on_hover_text("Saves the numbers above and the tree as a short text file, for your notes.")
                .clicked()
            {
                self.notice = save(&summary(o), "wqfm-rs_summary.txt", "Text", &["txt"]);
            }
        });
        if let Some(n) = &self.notice {
            ui.label(RichText::new(n).size(13.0).color(theme::TEAL));
        }
        ui.add_space(6.0);
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            egui::ScrollArea::vertical()
                .max_height((ui.available_height() - 80.0).max(200.0))
                .auto_shrink([false, true])
                .show(ui, |ui| tree_view::show(ui, &o.tree));
        });
        ui.add_space(6.0);
        egui::CollapsingHeader::new(RichText::new("Newick").color(theme::MUTED))
            .default_open(false)
            .show(ui, |ui| {
                let mut text = o.newick.clone();
                ui.add(
                    egui::TextEdit::multiline(&mut text)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(2),
                );
            });
    }

    fn welcome(&mut self, ui: &mut egui::Ui) {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            if let Some(t) = &self.logo {
                ui.add(egui::Image::new(t).fit_to_exact_size(Vec2::splat(120.0)));
            }
            ui.add_space(8.0);
            ui.label(RichText::new("Build a species tree").size(26.0).strong().color(theme::INK));
            ui.label(
                RichText::new("wQFM-rs gives exactly the trees and scores of wQFM v1.4, faster and without Java.")
                    .size(14.5)
                    .color(theme::MUTED),
            );
            ui.add_space(18.0);
            for (n, t) in [
                ("1", "Choose a file of gene trees or weighted quartets, or drop one on the window."),
                ("2", "Keep the settings (wQFM's defaults) or change them; every control explains itself on hover."),
                ("3", "Press Build species tree, then view, copy or save the result."),
            ] {
                ui.label(RichText::new(format!("{n}.  {t}")).size(14.5).color(theme::INK));
            }
            ui.add_space(18.0);
            if self.source.is_none()
                && ui
                    .add(egui::Button::new(RichText::new("Try the example").strong()).min_size(Vec2::new(180.0, 36.0)))
                    .on_hover_text("Loads wQFM's 11-taxon example and runs it.")
                    .clicked()
            {
                self.use_example();
                self.run();
            }
        });
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.poll(&ctx);
        self.dropped_file(&ctx);
        egui::Panel::left("input")
            .resizable(false)
            .exact_size(330.0)
            .show_separator_line(false)
            .frame(
                egui::Frame::new()
                    .fill(theme::SIDEBAR)
                    .stroke(Stroke::new(1.0, theme::BORDER))
                    .inner_margin(Margin {
                        left: 18,
                        right: 18,
                        top: 18,
                        bottom: 12,
                    }),
            )
            .show(ui, |ui| self.sidebar(ui));
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(theme::CANVAS)
                    .inner_margin(Margin::symmetric(28, 22)),
            )
            .show(ui, |ui| self.result(ui));
        if let Some(shot) = &mut self.shot {
            shot.step(&ctx, self.outcome.is_some());
        }
    }
}

fn step(ui: &mut egui::Ui, n: &str, title: &str) {
    ui.horizontal(|ui| {
        egui::Frame::new()
            .fill(theme::NAVY)
            .corner_radius(CornerRadius::same(11))
            .inner_margin(Margin::symmetric(7, 1))
            .show(ui, |ui| {
                ui.label(RichText::new(n).size(12.5).strong().color(Color32::WHITE))
            });
        ui.label(RichText::new(title).size(16.0).strong().color(theme::INK));
    });
}

fn stat(ui: &mut egui::Ui, label: &str, value: &str, help: &str) {
    theme::card()
        .inner_margin(Margin::symmetric(16, 10))
        .show(ui, |ui| {
            ui.set_min_width(110.0);
            ui.vertical(|ui| {
                ui.label(RichText::new(label).size(12.0).color(theme::MUTED));
                ui.label(RichText::new(value).size(22.0).strong().color(theme::INK));
            });
        })
        .response
        .on_hover_text(help);
}

fn message(
    ui: &mut egui::Ui,
    bg: Color32,
    ink: Color32,
    title: &str,
    text: &str,
    hint: Option<&str>,
) {
    egui::Frame::new()
        .fill(bg)
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(title).strong().color(ink));
            ui.label(RichText::new(text).size(13.0).color(ink));
            if let Some(h) = hint {
                ui.label(
                    RichText::new(format!("What to do: {h}"))
                        .size(13.0)
                        .color(theme::INK),
                );
            }
        });
}

fn save(text: &str, name: &str, kind: &str, ext: &[&str]) -> Option<String> {
    let path = rfd::FileDialog::new()
        .set_file_name(name)
        .add_filter(kind, ext)
        .save_file()?;
    Some(match std::fs::write(&path, text) {
        Ok(()) => format!("Saved {}", path.display()),
        Err(e) => format!("Could not save {}: {e}", path.display()),
    })
}

fn summary(o: &Outcome) -> String {
    let mut s = format!(
        "wQFM-rs {}\ninput: {}\ntaxa: {}\nweighted quartets: {}\n",
        env!("CARGO_PKG_VERSION"),
        o.input,
        o.taxa,
        o.quartets
    );
    if let Some(g) = o.gene_trees {
        s += &format!("gene trees: {g}\n");
    }
    s += &format!(
        "beta: {}\nquartet score: {} of {} ({:.4} %)\ntime: {:.3} s\ntree: {}\n",
        o.beta,
        o.score,
        o.total_weight,
        100.0 * o.score / o.total_weight.max(f64::MIN_POSITIVE),
        o.seconds,
        o.newick
    );
    s
}

fn thousands(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn seconds(s: f64) -> String {
    if s < 10.0 {
        format!("{s:.2} s")
    } else {
        format!("{s:.1} s")
    }
}
