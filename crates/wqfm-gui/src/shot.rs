//! Screenshots for the documentation: with `WQFM_GUI_SHOT=<file.png>` the
//! window opens off the screen, runs the built-in example, saves a
//! screenshot when the result is shown and closes.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use eframe::egui;

pub struct Shot {
    pub path: PathBuf,
    started: Option<Instant>,
    requested: bool,
}

impl Shot {
    pub fn from_env() -> Option<Self> {
        std::env::var_os("WQFM_GUI_SHOT").map(|p| Self {
            path: PathBuf::from(p),
            started: None,
            requested: false,
        })
    }

    /// Call every frame once the result is on screen.
    pub fn step(&mut self, ctx: &egui::Context, ready: bool) {
        let image = ctx.input(|i| {
            i.raw.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(image) = image {
            if let Err(e) = save(&self.path, &image) {
                eprintln!("the screenshot was not saved: {e}");
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        if ready {
            let started = *self.started.get_or_insert_with(Instant::now);
            if !self.requested && started.elapsed() >= Duration::from_millis(600) {
                self.requested = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            }
        }
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}

fn save(path: &PathBuf, image: &egui::ColorImage) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut enc = png::Encoder::new(
        std::io::BufWriter::new(file),
        image.size[0] as u32,
        image.size[1] as u32,
    );
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut w = enc.write_header().map_err(|e| e.to_string())?;
    let data: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
    w.write_image_data(&data).map_err(|e| e.to_string())
}
