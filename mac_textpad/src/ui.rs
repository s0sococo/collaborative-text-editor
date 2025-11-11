use crate::backend_api::{DocBackend, Intent};
use eframe::{egui, egui::Context};

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
        let update = self.backend.apply_intent(intent);
        if let Some(new_text) = update.full_text {
            self.editor.text = new_text;
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            if i.modifiers.command && i.key_pressed(egui::Key::Backslash) {
                self.sidebar.visible = !self.sidebar.visible;
            }
            if i.modifiers.command && i.key_pressed(egui::Key::O) {
                // self.open_file();
            }
            if i.modifiers.command && i.key_pressed(egui::Key::S) {
                // self.save();
            }
        });
    }

    fn top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("topbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("☰ Menu").clicked() {
                    self.sidebar.visible = !self.sidebar.visible;
                }
                ui.separator();

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("Cursor: {}", self.editor.cursor));
                    ui.separator();
                    ui.label("Collaborative Editor");
                });
            });
        });
    }

    fn sidebar_panel(&mut self, ctx: &egui::Context) {
        if !self.sidebar.visible {
            return;
        }
        egui::SidePanel::left("sidebar")
            .resizable(true)
            .default_width(self.sidebar.default_width)
            .show(ctx, |ui| {
                ui.heading("Documents");
                ui.separator();
                if ui.button("+ New").clicked() {
                    self.handle_intent(Intent::ReplaceAll {
                        text: String::new(),
                    });
                    self.editor.text.clear();
                    self.editor.cursor = 0;
                    self.status = "New document".into();
                    self.sidebar.docs.push("untitled.txt".into());
                    self.sidebar.selected = self.sidebar.docs.len() - 1;
                }
                ui.add_space(8.0);

                for (i, name) in self.sidebar.docs.iter().enumerate() {
                    let selected = self.sidebar.selected == i;
                    if ui.selectable_label(selected, name).clicked() {
                        self.sidebar.selected = i;
                        // Hook up: load different doc later
                    }
                }

                ui.add_space(8.0);
                // ui.horizontal(|ui| {
                //     if ui.button("Share…").clicked() { /* later */ }
                //     if ui.button("Delete").clicked() { /* later */ }
                // });
            });
    }

    fn editor_center(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // keep shortcuts here so they work even when sidebar hidden
            self.handle_shortcuts(ctx);

            // centered column
            let available = ui.available_size();
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                egui::Frame::group(ui.style())
                    .show(ui, |ui| {
                        // ui.set_min_width(available.x);
                        // ui.set_max_width(available.x);

                        let output = egui::TextEdit::multiline(&mut self.editor.text)
                            .desired_rows(available.y as usize / 20) // Adjust desired rows based on available height
                            .lock_focus(true)
                            .hint_text("Start typing…")
                            .show(ui);

                        if output.response.changed() {
                            self.handle_intent(Intent::ReplaceAll {
                                text: self.editor.text.clone(),
                            });
                        }

                        if let Some(cr) = output.cursor_range {
                            let pos = cr.primary.index;
                            if pos != self.editor.cursor {
                                self.editor.cursor = pos;
                                let _ = self.backend.apply_intent(Intent::MoveCursor { pos });
                            }
                        }
                    });
            });
        });
    }

    fn status_bar(&mut self, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(&self.status);
            ui.separator();
            ui.label(format!("Length: {}", self.editor.text.chars().count()));
            ui.separator();
            ui.label(if self.sidebar.visible { "Sidebar: visible" } else { "Sidebar: hidden" });
        });
    });
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
