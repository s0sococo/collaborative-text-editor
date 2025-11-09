mod backend_api;
mod ui;

use crate::backend_api::{DocBackend, FrontendUpdate, Intent};

struct MockBackend {
    text: String,
}

impl Default for MockBackend {
    fn default() -> Self {
        Self {
            text: "Hello, world!".into(),
        }
    }
}

impl DocBackend for MockBackend {
    fn apply_intent(&mut self, intent: Intent) -> FrontendUpdate {
        match intent {
            Intent::ReplaceAll { text } => self.text = text,
            _ => {}
        }
        FrontendUpdate {
            full_text: Some(self.text.clone()),
            remote_cursors: vec![],
        }
    }

    fn render_text(&self) -> String {
        self.text.clone()
    }
}

use crate::ui::AppView;
use eframe::NativeOptions;

fn main() -> eframe::Result<()> {
    let native_options = NativeOptions::default();
    eframe::run_native(
        "Collaborative Text Editor (skeleton)",
        native_options,
        Box::new(|_cc| Ok(Box::new(AppView::new(Box::new(MockBackend::default()))))),
    )
}
