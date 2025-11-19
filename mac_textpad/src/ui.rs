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
            livekit_ws_url: "wss://inzynierka-28c03xlk.livekit.cloud".into(),
            livekit_identity: "rust-user".into(),
            livekit_token_shared: Arc::new(Mutex::new(None)),
            livekit_token: "".into(),
            livekit_room: "test-room".into(),
            livekit_admin_key: "APIArE8e5SZrA5J".into(),
            livekit_admin_secret: "r1vnQcZcahVLaH1PGorLWtUdO6w9eKZouhcxZQm23YC".into(),
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

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.livekit_admin_secret.as_ref()),
        )
        .unwrap_or_else(|_| "".to_string());

        // store token in shared slot for other threads / UI
        if !token.is_empty() {
            if let Ok(mut shared) = self.livekit_token_shared.lock() {
                *shared = Some(token.clone());
            }
        }

        token
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

       fn admin_base_from_ws(&self) -> String {
        let u = self.livekit_ws_url.trim();
        if u.starts_with("wss://") {
            u.replacen("wss://", "https://", 1)
        } else if u.starts_with("ws://") {
            u.replacen("ws://", "http://", 1)
        } else {
            u.to_string()
        }
    }

   // ...existing code...
    pub fn create_room(&mut self) {
        let admin_base = self.admin_base_from_ws();
        let room_name = self.livekit_room.clone();
        let admin_key = self.livekit_admin_key.clone();
        let admin_secret = self.livekit_admin_secret.clone();
        let events = Arc::clone(&self.livekit_events);
        let token_slot = Arc::clone(&self.livekit_token_shared);
        let ws_url = self.livekit_ws_url.clone();

        {
            let mut v = events.lock().unwrap();
            v.push(format!("Creating room '{}' via admin API at {}", room_name, admin_base));
        }

        std::thread::spawn(move || {
            // admin JWT
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let admin_claims = LiveKitClaims {
                iss: admin_key.clone(),
                sub: admin_key.clone(),
                name: "admin".into(),
                iat: now,
                exp: now + 60,
                nbf: now,
                video: VideoGrants { room_join: false, room: "".into() },
            };
            let admin_token = match encode(&Header::default(), &admin_claims, &EncodingKey::from_secret(admin_secret.as_ref())) {
                Ok(t) => t,
                Err(e) => {
                    let mut v = events.lock().unwrap();
                    v.push(format!("admin JWT encode error: {:?}", e));
                    return;
                }
            };

            // admin API call
            let api_url = format!("{}/admin/v1/rooms", admin_base.trim_end_matches('/'));
            let client = reqwest::blocking::Client::new();
            let body = serde_json::json!({ "name": room_name.clone() });

            match client.post(&api_url)
                .bearer_auth(admin_token.clone())
                .json(&body)
                .send()
            {
                Ok(resp) => {
                    let mut v = events.lock().unwrap();
                    v.push(format!("Create room HTTP status: {}", resp.status()));
                    if let Ok(t) = resp.text() { v.push(format!("Create room body: {}", t)); }
                }
                Err(e) => {
                    let mut v = events.lock().unwrap();
                    v.push(format!("Create room request error: {:?}", e));
                    return;
                }
            }

            // generate user token signed with admin_secret
            let now2 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let user_claims = LiveKitClaims {
                iss: admin_key.clone(),
                sub: "user-generated".into(),
                name: "user".into(),
                iat: now2,
                exp: now2 + 3600,
                nbf: now2,
                video: VideoGrants { room_join: true, room: room_name.clone() },
            };
            let user_token = match encode(&Header::default(), &user_claims, &EncodingKey::from_secret(admin_secret.as_ref())) {
                Ok(t) => t,
                Err(e) => {
                    let mut v = events.lock().unwrap();
                    v.push(format!("User token encode error: {:?}", e));
                    return;
                }
            };

            // store token into shared slot so UI shows it
            if let Ok(mut slot) = token_slot.lock() {
                *slot = Some(user_token.clone());
            }

            {
                let mut v = events.lock().unwrap();
                v.push(format!("User token generated (len={})", user_token.len()));
            }

            // connect with generated user token (background)
            let events2 = Arc::clone(&events);
            let ws2 = ws_url.clone();
            let token_for_connect = user_token.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
                rt.block_on(async move {
                    match livekit::prelude::Room::connect(&ws2, &token_for_connect, livekit::prelude::RoomOptions::default()).await {
                        Ok((room, mut events_rx)) => {
                            let mut v = events2.lock().unwrap();
                            v.push(format!("Connected to room: {}", room.name()));
                            drop(v);
                            while let Some(ev) = events_rx.recv().await {
                                let mut v = events2.lock().unwrap();
                                v.push(format!("room event: {:?}", ev));
                            }
                        }
                        Err(e) => {
                            let mut v = events2.lock().unwrap();
                            v.push(format!("LiveKit connect error: {:?}", e));
                        }
                    }
                });
            });
        });
    }
// ...existing code...
}

// eframe trait for AppView
impl eframe::App for AppView {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // If background thread wrote a token into the shared slot, copy it into the editable input
        if let Ok(mut slot) = self.livekit_token_shared.lock() {
            if slot.is_some() {
                if let Some(tok) = slot.take() {
                    self.livekit_token = tok;
                    if let Ok(mut v) = self.livekit_events.lock() {
                        v.push("Received token from background".into());
                    }
                }
            }
        }

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
