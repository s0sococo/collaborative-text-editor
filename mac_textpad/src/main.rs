mod backend_api;
mod ui;
use crate::backend_api::MockBackend;
use crate::ui::AppView;
use eframe::NativeOptions;
use egui::vec2;


fn main() -> eframe::Result<()> {
    let mut native_options = NativeOptions::default();
    native_options.centered = true;
    eframe::run_native(
        "Collaborative Text Editor",
        native_options,
        Box::new(|_cc| Ok(Box::new(AppView::new(Box::new(MockBackend::default()))))),
    )
}
