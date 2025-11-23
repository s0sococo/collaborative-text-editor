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
    // shared token storage so background threads can set the generated token for the UI/connection
    livekit_token_shared: Arc<Mutex<Option<String>>>,
    // editable token field for the UI (user can paste or modify)
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
        let web_socket_url = if host.starts_with("ws://") || host.starts_with("wss://") {
        host
        } else if host.starts_with("http://") {
            host.replacen("http://", "ws://", 1)
        } else if host.starts_with("https://") {
            host.replacen("https://", "wss://", 1)
        } else {
            format!("ws://{}", host)
        };
        
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
            livekit_ws_url: web_socket_url.into(),
            livekit_identity: "".into(),
            livekit_token_shared: Arc::new(Mutex::new(None)),
            livekit_token: "".into(),
            livekit_room: "".into(),
        }
    }

    fn handle_intent(&mut self, intent: Intent) {
        println!("Handling intent: {:?}", intent);
        let update = self.backend.apply_intent(intent);
        if let Some(new_text) = update.full_text {
            self.editor.text = new_text;
        }
    }

  fn create_token(room_name: &str, identity: &str) -> Result<String, access_token::AccessTokenError> {
    let api_key = env::var("LIVEKIT_API_KEY").expect("LIVEKIT_API_KEY is not set");
    let api_secret = env::var("LIVEKIT_API_SECRET").expect("LIVEKIT_API_SECRET is not set");

    access_token::AccessToken::with_api_key(&api_key, &api_secret)
        .with_identity(identity)
        .with_name(identity)
        .with_grants(access_token::VideoGrants {
            room_join: true,
            room: room_name.to_string(),
            can_publish: true,
            can_publish_data: true, // Required to send chat messages
            ..Default::default()
        })
        .to_jwt()
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
                // include the token as a query parameter in case the websocket upgrade does not forward
                // the Authorization header; the LiveKit server accepts access_token in the URL.
                let connect_url = format!("{}?access_token={}", url, token);

                match Room::connect(&connect_url, &token, RoomOptions::default()).await {
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
    // ...existing code...
    pub fn create_room(&mut self) {
            let (room, mut room_events) =
        Room::connect(&self.livekit_ws_url, &token, RoomOptions::default()).await?;
    }
    // ...existing code...
}

// eframe trait for AppView
impl eframe::App for AppView {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // If background thread wrote a token into the shared slot, copy it into the editable input
        // ...existing code in impl eframe::App for AppView, inside update() ...
        // If background thread wrote a token into the shared slot, copy it into the editable input
        let mut should_connect = false;
        let mut url = String::new();
        let mut token = String::new();

        if let Ok(mut slot) = self.livekit_token_shared.lock() {
            if slot.is_some() {
                if let Some(tok) = slot.take() {
                    self.livekit_token = tok;
                    if let Ok(mut v) = self.livekit_events.lock() {
                        v.push("Received token from background".into());
                    }
                    // Prepare to AUTO-CONNECT here (only after token delivered)
                    if !self.livekit_connecting {
                        url = self.livekit_ws_url.clone();
                        token = self.livekit_token.clone();
                        // should_connect = true;
                    }
                }
            }
        }
        // if should_connect {
        //     self.start_livekit(url, token);
        // }

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
