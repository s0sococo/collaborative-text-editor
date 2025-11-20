use super::*;
use eframe::{egui, egui::Context};
use egui::Key;

impl AppView {
    pub fn handle_shortcuts(&mut self, ctx: &egui::Context) {
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

    pub fn top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("topbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("☰ Menu").clicked() {
                    self.sidebar.visible = !self.sidebar.visible;
                }

                // ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                //     ui.label(format!("Cursor Position: {}", self.editor.cursor));
                // });
            });
        });
    }

    pub fn sidebar_panel(&mut self, ctx: &egui::Context) {
        if !self.sidebar.visible {
            return;
        }
        egui::SidePanel::left("sidebar")
            .resizable(true)
            .default_width(self.sidebar.default_width)
            .show(ctx, |ui| {
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

                // new: open LiveKit page
                if ui.button("Open LiveKit").clicked() {
                    self.page = Page::LiveKit;
                }

                for (i, name) in self.sidebar.docs.iter().enumerate() {
                    let selected = self.sidebar.selected == i;
                    if ui.selectable_label(selected, name).clicked() {
                        self.sidebar.selected = i;
                        // Hook up: load different doc later
                    }
                }
            });
    }

    pub fn livekit_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if ui.button("Back to Editor").clicked() {
                        self.page = Page::Editor;
                    }
                    ui.label(if self.livekit_connecting {
                        "Connecting..."
                    } else {
                        "LiveKit"
                    });
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("WS URL:");
                    ui.text_edit_singleline(&mut self.livekit_ws_url);
                });
                ui.horizontal(|ui| {
                    ui.label("Room:");
                    ui.text_edit_singleline(&mut self.livekit_room);
                });
                ui.horizontal(|ui| {
                    ui.label("Identity:");
                    ui.text_edit_singleline(&mut self.livekit_identity);
                });

                ui.separator();

                // Create room via Admin API (Cloud / Enterprise only)
                if ui.button("Create Room (Admin)").clicked() {
                    self.create_room();
                }

                ui.horizontal(|ui| {
                    // if ui.button("Auto Generate Token").clicked() {
                    //     // generate synchronously for UI and store in shared slot as well
                    //     let tok = self.generate_token(&self.livekit_identity);
                    //     // put into visible/editable field
                    //     self.livekit_token = tok.clone();
                    //     // tok already stored in shared slot by generate_token (if implemented that way)
                    //     let mut v = self.livekit_events.lock().unwrap();
                    //     v.push("Token auto-generated (dev-only).".into());
                    // }
                    if ui.button("Connect with Token").clicked() {
                        let token = self.livekit_token.clone();
                        if token.trim().is_empty() {
                            let mut v = self.livekit_events.lock().unwrap();
                            v.push("No token available. Generate or paste a token first.".into());
                        } else {
                            let url = self.livekit_ws_url.clone();
                            self.start_livekit(url, token);
                        }
                    }
                });

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Token:");
                    ui.text_edit_singleline(&mut self.livekit_token);
                });

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Token (shared):");
                    let token_display = {
                        self.livekit_token_shared
                            .lock()
                            .unwrap()
                            .clone()
                            .unwrap_or_default()
                    };
                    ui.add(egui::Label::new(token_display).wrap());
                });
                ui.separator();
                ui.heading("Events:");
                let events = {
                    let guard = self.livekit_events.lock().unwrap();
                    guard.clone()
                };
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for ev in events.iter().rev() {
                        ui.label(ev);
                    }
                });
            });
        });
    }
    pub fn editor_center(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // keep shortcuts here so they work even when sidebar hidden
            self.handle_shortcuts(ctx);

            // centered column
            let available = ui.available_size();
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                // display the document text with a visible cursor
                let mut display_text = self.editor.text.clone();
                let cursor_pos = self.editor.cursor;
                if cursor_pos <= display_text.len() {
                    display_text.insert_str(cursor_pos, "|"); // Use "|" as a cursor indicator
                }

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add(egui::Label::new(display_text).wrap());
                    });

                // invisible capture buffer to keep keyboard focus and receive events
                let mut _capture = String::new();
                let output = egui::TextEdit::multiline(&mut _capture)
                    .desired_rows((available.y / 18.0) as usize)
                    .lock_focus(true)
                    .desired_width(self.editor.max_width)
                    .show(ui);

                // helpers for char-boundary navigation/removal
                fn prev_char_idx(s: &str, idx: usize) -> usize {
                    if idx == 0 {
                        return 0;
                    }
                    let mut i = idx;
                    // step back one UTF-8 codepoint
                    while !s.is_char_boundary(i) {
                        i -= 1;
                    }
                    // now find previous char boundary
                    let mut j = i;
                    loop {
                        if j == 0 {
                            return 0;
                        }
                        j -= 1;
                        if s.is_char_boundary(j) {
                            return j;
                        }
                    }
                }
                fn next_char_idx(s: &str, idx: usize) -> usize {
                    if idx >= s.len() {
                        return s.len();
                    }
                    let mut i = idx;
                    // step forward to next char boundary
                    i += 1;
                    while i < s.len() && !s.is_char_boundary(i) {
                        i += 1;
                    }
                    i.min(s.len())
                }

                // process low-level input events and turn them into intents
                ctx.input(|input| {
                    for event in input.events.iter() {
                        match event {
                            egui::Event::Text(text) => {
                                // insert text at cursor
                                if !text.is_empty() {
                                    let mut new_text = self.editor.text.clone();
                                    new_text.insert_str(self.editor.cursor, text);
                                    self.handle_intent(Intent::ReplaceAll { text: new_text });
                                    // advance cursor by bytes of inserted text
                                    self.editor.cursor += text.len();
                                    let _ = self.backend.apply_intent(Intent::MoveCursor {
                                        pos: self.editor.cursor,
                                    });
                                }
                            }
                            egui::Event::Key {
                                key, pressed: true, ..
                            } => {
                                match key {
                                    Key::Backspace => {
                                        if self.editor.cursor > 0 {
                                            let prev = prev_char_idx(
                                                &self.editor.text,
                                                self.editor.cursor,
                                            );
                                            // use handle_intent so editor.text gets updated from backend response
                                            self.handle_intent(Intent::DeleteRange {
                                                start: prev,
                                                end: self.editor.cursor,
                                            });
                                            self.editor.cursor = prev;
                                            // notify backend about cursor move
                                            self.handle_intent(Intent::MoveCursor {
                                                pos: self.editor.cursor,
                                            });
                                        }
                                    }
                                    Key::ArrowLeft => {
                                        if self.editor.cursor > 0 {
                                            let prev = prev_char_idx(
                                                &self.editor.text,
                                                self.editor.cursor,
                                            );
                                            self.editor.cursor = prev;
                                            let _ = self.backend.apply_intent(Intent::MoveCursor {
                                                pos: self.editor.cursor,
                                            });
                                        }
                                    }
                                    Key::ArrowRight => {
                                        if self.editor.cursor < self.editor.text.len() {
                                            let next = next_char_idx(
                                                &self.editor.text,
                                                self.editor.cursor,
                                            );
                                            self.editor.cursor = next;
                                            let _ = self.backend.apply_intent(Intent::MoveCursor {
                                                pos: self.editor.cursor,
                                            });
                                        }
                                    }
                                    Key::Enter => {
                                        // insert newline using Intent::InsertAt
                                        self.handle_intent(Intent::InsertAt {
                                            pos: self.editor.cursor,
                                            text: "\n".into(),
                                        });
                                        self.editor.cursor += 1;
                                        let _ = self.backend.apply_intent(Intent::MoveCursor {
                                            pos: self.editor.cursor,
                                        });
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                });
            });
        });
    }

    pub fn status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            // ui.horizontal_wrapped(|ui| {
            //     ui.label(&self.status);
            //     ui.separator();
            //     ui.label(format!("Length: {}", self.editor.text.chars().count()));
            //     ui.separator();
            //     ui.label(if self.sidebar.visible {
            //         "Sidebar: visible"
            //     } else {
            //         "Sidebar: hidden"
            //     });
            // });
        });
    }
}
