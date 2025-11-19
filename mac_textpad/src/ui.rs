use std::sync::{Arc, Mutex};

use crate::backend_api::{DocBackend, Intent};
use eframe::{egui, egui::Context};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

mod ui_panels;

use livekit::prelude::*;

pub struct AppView {
    backend: Box<dyn DocBackend>,
    status: String,
    sidebar: SidebarState,
    page: Page,
    editor: EditorState,
    livekit_events: Arc<Mutex<Vec<String>>>,
    livekit_connecting: bool,
    // LiveKit panel inputs
    livekit_ws_url: String,
    livekit_identity: String,
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

#[derive(Debug, Serialize, Deserialize)]
struct VideoGrants {
    #[serde(default)]
    room_join: bool,
    #[serde(default)]
    room: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LiveKitClaims {
    iss: String,
    sub: String,
    name: String,
    iat: u64,
    exp: u64,
    nbf: u64,
    video: VideoGrants,
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
            livekit_ws_url: "ws://209.38.105.89:7880".into(),
            livekit_token: "".into(),
            livekit_room: "test-room".into(),
            livekit_admin_key: "devkey".into(),
            livekit_admin_secret: "devsecret".into(),
            livekit_identity: "rust-user".into(),
        }
    }

    fn handle_intent(&mut self, intent: Intent) {
        println!("Handling intent: {:?}", intent);
        let update = self.backend.apply_intent(intent);
        if let Some(new_text) = update.full_text {
            self.editor.text = new_text;
        }
    }

    pub fn generate_token(&self, identity: &str) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let exp = now + 3600; // valid for 1 hour

        let claims = LiveKitClaims {
            iss: self.livekit_admin_key.clone(), // API key
            sub: identity.to_string(),           // user identity
            name: identity.to_string(),          // user name
            iat: now,
            exp,
            nbf: now,
            video: VideoGrants {
                room_join: true,
                room: self.livekit_room.clone(),
            },
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.livekit_admin_secret.as_ref()),
        )
        .unwrap_or_else(|_| "".to_string())
    }

    pub fn start_livekit(&mut self, url: String, token: String) {
        if self.livekit_connecting {
            return;
        }
        self.livekit_connecting = true;
        let events = Arc::clone(&self.livekit_events);

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
            rt.block_on(async move {
                match Room::connect(&url, &token, RoomOptions::default()).await {
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

// eframe trait for AppView
impl eframe::App for AppView {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.top_bar(ctx);
        self.sidebar_panel(ctx);
        if self.page == Page::Editor {
            self.editor_center(ctx);
        } else {
            self.livekit_panel(ctx);
        }
        self.status_bar(ctx);
    }
}
