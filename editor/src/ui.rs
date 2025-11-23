use std::{
    env,
    sync::{Arc, Mutex},
};

use crate::backend_api::{DocBackend, Intent};
use eframe::{egui, egui::Context};
use jsonwebtoken::{encode, EncodingKey, Header};
use livekit_api::access_token;
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

    // Connected LiveKit room state
    livekit_events: Arc<Mutex<Vec<String>>>,
    livekit_connected: bool,
    livekit_connecting: bool,
    // LiveKit panel inputs
    livekit_ws_url: String,
    livekit_identity: String,
    // shared token storage so background threads can set the generated token for the UI/connection
    // editable token field for the UI (user can paste or modify)
    livekit_token: String,
    livekit_room: String,
    livekit_message: String,
     // Channel to send messages to the background LiveKit task
    livekit_command_sender: Option<tokio::sync::mpsc::UnboundedSender<String>>,
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
        let host = std::env::var("LIVEKIT_URL").unwrap_or_else(|_| "127.0.0.1:7880".to_string());
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
            livekit_connected: false,
            livekit_connecting: false,
            livekit_ws_url: web_socket_url.into(),
            livekit_identity: "".into(),
            livekit_token: "".into(),
            livekit_room: "".into(),
            livekit_message: "".into(),
            livekit_command_sender: None,
        }
    }

    fn handle_intent(&mut self, intent: Intent) {
        println!("Handling intent: {:?}", intent);
        let update = self.backend.apply_intent(intent);
        if let Some(new_text) = update.full_text {
            self.editor.text = new_text;
        }
    }

    fn create_token(
        room_name: &str,
        identity: &str,
    ) -> Result<String, access_token::AccessTokenError> {
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
    // ...existing code...
    pub fn connect_or_create_to_room(&mut self) {
       if self.livekit_connected {
            return;
        }
        self.livekit_connecting = true;

        println!("Connecting to LiveKit room...");

        println!("Generating token...");
        let token = match Self::create_token(&self.livekit_room, &self.livekit_identity) {
            Ok(t) => t,
            Err(e) => {
                let mut guard = self.livekit_events.lock().unwrap();
                guard.push(format!("Token generation error: {}", e));
                self.livekit_connecting = false;
                return;
            }
        };

        println!("Token generated: {}", token);
        self.livekit_token = token.clone();
        println!("Connecting to LiveKit room at {}", self.livekit_ws_url);

        let url = self.livekit_ws_url.clone();
        let events_log = self.livekit_events.clone();
        
        // Create a channel to send messages from UI to the background task
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        self.livekit_command_sender = Some(tx);

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let (room, mut room_events) = match Room::connect(&url, &token, RoomOptions::default()).await {
                    Ok(res) => res,
                    Err(e) => {
                        events_log.lock().unwrap().push(format!("Connection failed: {}", e));
                        return;
                    }
                };
                
                let room = Arc::new(room);
                events_log.lock().unwrap().push("Connected to Room".to_string());

                loop {
                    tokio::select! {
                        Some(event) = room_events.recv() => {
                            match event {
                                RoomEvent::DataReceived { payload, participant, .. } => {
                                    let text = String::from_utf8_lossy(&payload);
                                    let sender = participant.map(|p| p.name().to_string()).unwrap_or("Unknown".to_string());
                                    events_log.lock().unwrap().push(format!("[{}] {}", sender, text));
                                }
                                RoomEvent::Disconnected { reason } => {
                                     events_log.lock().unwrap().push(format!("Disconnected: {:?}", reason));
                                     break;
                                }
                                _ => {}
                            }
                        }
                        Some(msg) = rx.recv() => {
                            // Send message to others
                            let res = room.local_participant()
                                .publish_data(DataPacket {
                                    payload: msg.as_bytes().to_vec(),
                                    reliable: true,
                                    ..Default::default()
                                })
                                .await;
                            
                            if let Err(e) = res {
                                events_log.lock().unwrap().push(format!("Failed to send: {}", e));
                            } else {
                                events_log.lock().unwrap().push(format!("[You] {}", msg));
                            }
                        }
                    }
                }
                
                room.close().await.ok();
            });
        });

        self.livekit_connecting = false;
        self.livekit_connected = true;
    }

    pub fn send_livekit_message(&mut self, message: String) {
        if !self.livekit_connected {
            return;
        }
        if let Some(sender) = &self.livekit_command_sender {
            if let Err(e) = sender.send(message) {
                let mut guard = self.livekit_events.lock().unwrap();
                guard.push(format!("Failed to enqueue message: {}", e));
            }
        }
    }
    // ...existing code...
}

// eframe trait for AppView
impl eframe::App for AppView {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // If background thread wrote a token into the shared slot, copy it into the editable input
        // ...existing code in impl eframe::App for AppView, inside update() ...
        // If background thread wrote a token into the shared slot, copy it into the editable input

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
