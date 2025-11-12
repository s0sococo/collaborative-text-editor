use crate::backend_api::{DocBackend, Intent};
use eframe::{egui, egui::Context};

mod ui_panels;


pub struct AppView {
    backend: Box<dyn DocBackend>,
    status: String,
    sidebar: SidebarState,
    editor: EditorState,
}
struct SidebarState {
    visible: bool,
    default_width: f32,
    docs: Vec<String>,
    selected: usize,
}

struct EditorState {
    text: String,
    cursor: usize,
    max_width: f32,
}

impl AppView {

    pub fn new(backend: Box<dyn DocBackend>) -> Self {
        let text_cache = backend.render_text();
        Self {
            backend,
            status: "Ready".into(),
            sidebar: SidebarState {
                visible: false,
                default_width: 260.0,
                docs: vec!["test_doc.txt".into(), "notes.md".into()],
                selected: 0,
            },
            editor: EditorState {
                text: text_cache,
                cursor: 0,
                max_width: 1500.0,
            },
        }
    }

    // replace all text
    fn handle_intent(&mut self, intent: Intent) {
        println!("Handling intent: {:?}", intent);
        let update = self.backend.apply_intent(intent);
        if let Some(new_text) = update.full_text {
            self.editor.text = new_text;
        }
    }
}

/// Najwaznijesza metoda eframe app - update UI
///  cała logika UI jest tutaj
// eframe  trait dla AppView
impl eframe::App for AppView {
    // glowna metoda aktualizacji UI, wywolywana w petli eventow
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.top_bar(ctx);
        self.sidebar_panel(ctx);
        self.editor_center(ctx);
        self.status_bar(ctx);
    }
}
