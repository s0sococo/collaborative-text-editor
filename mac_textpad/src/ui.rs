use crate::backend_api::{DocBackend, Intent};
use eframe::{egui, egui::Context};

pub struct AppView {
    backend: Box<dyn DocBackend>,
    text_cache: String,
    cursor: usize,
    status: String,
}

impl AppView {
    pub fn new(backend: Box<dyn DocBackend>) -> Self {
        let text_cache = backend.render_text();
        Self {
            backend,
            text_cache: String::new(),
            cursor: 0,
            status: "Ready".to_string(),
        }
    }

    // replace all text
    fn handle_intent(&mut self, intent: Intent) {
        let update = self.backend.apply_intent(intent);
        if let Some(new_text) = update.full_text {
            self.text_cache = new_text;
        }
    }
}

impl eframe::App for AppView {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // layput: top bar + central editor box
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.heading("Collaborative Text Editor (skeleton)");
            if ui.button("Clear text").clicked() {
                self.handle_intent(Intent::ReplaceAll {
                    text: String::new(),
                });
                self.status = "Cleared".into();
            }
        });
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.label(&self.status);
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            
            let resp = ui.add(
                egui::TextEdit::multiline(&mut self.text_cache)
                .desired_rows(24)
                .lock_focus(true)
                .hint_text("Start typing...."),
            );

            if resp.changed() {
                println!("Text changed: {}", self.text_cache);
                self.handle_intent(Intent::ReplaceAll {
                    text: self.text_cache.clone(),
                });
                self.status = "Text updated".into();
            }
            
        });
    }
}
