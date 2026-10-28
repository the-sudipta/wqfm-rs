//! `wqfm-rs-gui`: the desktop app of wQFM-rs.

// No console window behind the app on Windows (release builds).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod icon;
mod job;
mod shot;
mod theme;
mod tree_view;

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let shot = shot::Shot::from_env();
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("wQFM-rs")
        .with_app_id("wqfm-rs")
        .with_inner_size([1180.0, 760.0])
        .with_min_inner_size([900.0, 600.0]);
    if shot.is_some() {
        // A documentation screenshot opens off the screen, without focus and
        // outside the taskbar, so it never disturbs the person at the computer.
        viewport = viewport
            .with_position([-20000.0, -20000.0])
            .with_active(false)
            .with_taskbar(false);
    }
    if let Some((rgba, width, height)) = icon::rgba() {
        viewport = viewport.with_icon(egui::IconData {
            rgba,
            width,
            height,
        });
    }
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "wQFM-rs",
        options,
        Box::new(move |cc| {
            theme::install(&cc.egui_ctx);
            let mut app = app::App::new(&cc.egui_ctx);
            if shot.is_some() {
                app.use_example();
                app.run();
            }
            app.shot = shot;
            Ok(Box::new(app))
        }),
    )
}
