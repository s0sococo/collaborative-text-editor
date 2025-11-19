use std::sync::{Arc, Mutex};

use crate::backend_api::{DocBackend, Intent};
use eframe::{egui, egui::Context};

mod ui_panels;


pub struct AppView {
    backend: Box<dyn DocBackend>,
    status: String,
    sidebar: SidebarState,
    page: Page,
    editor: EditorState,
    livekit_events: Arc<Mutex<Vec<String>>>,
    livekit_connecting: bool,
        // new: inputs for LiveKit panel
    livekit_ws_url: String,
    livekit_token: String,
    livekit_room: String,
    livekit_admin_key: String,
    livekit_admin_secret: String,
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


#[derive(PartialEq, Eq)]
pub enum Page {
    Editor,
    LiveKit,
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
            page: Page::Editor,
            livekit_events: Arc::new(Mutex::new(Vec::new())),
            livekit_connecting: false,

             // defaults
            livekit_ws_url: "ws://209.38.105.89:7880".into(),
            livekit_token: "".into(),
            livekit_room: "test-room".into(),
            livekit_admin_key: "".into(),
            livekit_admin_secret: "".into(),
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

      /// Try to create a room using LiveKit Admin REST API (requires admin key/secret),
    /// then optionally connect. Runs in background thread (blocking reqwest).
    pub fn create_room(&mut self, host: String, room: String, admin_key: String, admin_secret: String) {
        let events = Arc::clone(&self.livekit_events);
        let mut status = self.status.clone();
        std::thread::spawn(move || {
            // use blocking reqwest inside thread
            let api_url = format!("http://{}/v1/rooms", host);
            let body = format!(r#"{{"name":"{}"}}"#, room);
            let client = reqwest::blocking::Client::new();
            match client
                .post(&api_url)
                .basic_auth(admin_key, Some(admin_secret))
                .header("Content-Type", "application/json")
                .body(body)
                .send()
            {
                Ok(resp) => {
                    let mut v = events.lock().unwrap();
                    v.push(format!("Create room HTTP status: {}", resp.status()));
                }
                Err(e) => {
                    let mut v = events.lock().unwrap();
                    v.push(format!("Create room error: {:?}", e));
                }
            }
        });
        self.status = status;
    }

      // start background livekit connection (non-blocking)
    pub fn start_livekit(&mut self, url: String, token: String) {
        if self.livekit_connecting {
            return;
        }
        self.livekit_connecting = true;
        let events = Arc::clone(&self.livekit_events);

        std::thread::spawn(move || {
            // create a tokio runtime inside the thread and run async connect
            let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
            rt.block_on(async move {
                match livekit::prelude::Room::connect(&url, &token, livekit::prelude::RoomOptions::default()).await {
                    Ok((room, mut ev_rx)) => {
                        {
                            let mut v = events.lock().unwrap();
                            v.push(format!("Connected to room: {}", room.name()));
                        }
                        while let Some(ev) = ev_rx.recv().await {
                            let mut v = events.lock().unwrap();
                            v.push(format!("Event: {:?}", ev));
                        }
                    }
                    Err(e) => {
                        let mut v = events.lock().unwrap();
                        v.push(format!("LiveKit connect error: {:?}", e));
                    }
                }
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
        // show page based on state
        if self.page == Page::Editor {
            self.editor_center(ctx);
        } else {
            // livekit panel implemented in ui_panels.rs
            self.livekit_panel(ctx);
        }
        self.status_bar(ctx);
    }
}