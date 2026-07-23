#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod data;
mod patch;
mod reloaded;
mod ui;

use eframe::egui;

fn main() -> eframe::Result<()> {
    if std::env::args().any(|a| a == "--apply") {
        let mut app = app::App::new();
        std::process::exit(if app.apply().is_ok() { 0 } else { 1 });
    }

    let icon = egui::IconData {
        rgba: ui::WINDOW_ICON.to_vec(),
        width: 256,
        height: 256,
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([660.0, 760.0])
            .with_min_inner_size([560.0, 420.0])
            .with_icon(std::sync::Arc::new(icon))
            .with_title("GBFRER Summon Drop Picker"),
        ..Default::default()
    };

    eframe::run_native("GBFRER Summon Drop Picker", options, Box::new(|cc| {
        ui::setup_style(&cc.egui_ctx);
        Ok(Box::new(app::App::new()))
    }))
}
