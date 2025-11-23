use livekit::prelude::*;
use livekit_api::access_token;
use livekit_api::services::room::{CreateRoomOptions, RoomClient};
use std::env;
use std::io::BufRead;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    // 1. Configuration
    let host = std::env::var("LIVEKIT_URL").unwrap_or_else(|_| "127.0.0.1:7880".to_string());
    // add "http://" prefix if missing
    // let http_url = if !host.starts_with("http://") && !host.starts_with("https://") {
    //     format!("http://{}", host)
    // } else {
    //     host.clone()
    // };

    let web_socket_url = if host.starts_with("ws://") || host.starts_with("wss://") {
        host
    } else if host.starts_with("http://") {
        host.replacen("http://", "ws://", 1)
    } else if host.starts_with("https://") {
        host.replacen("https://", "wss://", 1)
    } else {
        format!("ws://{}", host)
    };

    // 2. User Inputs
    println!("Enter room name:");
    let mut room_name = String::new();
    std::io::stdin().read_line(&mut room_name)?;
    let room_name = room_name.trim().to_string();

    println!("Enter your username:");
    let mut username = String::new();
    std::io::stdin().read_line(&mut username)?;
    let username = username.trim().to_string();

    if room_name.is_empty() || username.is_empty() {
        eprintln!("Room name and username cannot be empty.");
        return Ok(());
    }

    // 3. Create Token
    let token = create_token(&room_name, &username).expect("Failed to create access token");

    println!(
        "Connecting to {} in room '{}' as '{}'...",
        web_socket_url, room_name, username
    );

    // 4. Connect to Room
    let (room, mut room_events) =
        Room::connect(&web_socket_url, &token, RoomOptions::default()).await?;
    let room = Arc::new(room);

    println!("Connected! Type a message and press Enter to send. Type EXIT to quit.");

    // 5. Spawn Event Listener (Receiver)
    // We clone the room handle just to keep it alive if needed, though strictly not used inside the loop here
    let _room_handle = room.clone();
    tokio::spawn(async move {
        while let Some(event) = room_events.recv().await {
            match event {
                RoomEvent::DataReceived {
                    payload,
                    participant,
                    ..
                } => {
                    let text = String::from_utf8_lossy(&payload);
                    let sender = participant.map(|p| p.name());

                    let sender = sender.as_deref().unwrap_or("Unknown");

                    // Print received message
                    // \r helps handle the CLI cursor if needed, but simple println is fine
                    println!("\r[{}] {}", sender, text);
                }
                _ => {}
            }
        }
    });

    // 6. Main Loop (Sender)

    let local_participant = room.local_participant();

    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let msg = match line {
            Ok(m) => m,
            Err(_) => break,
        };
        if msg.trim().eq_ignore_ascii_case("EXIT") {
            println!("Exiting application.");
            break;
        }
        // 3. Publish the data
        local_participant
            .publish_data(DataPacket {
                payload: msg.into_bytes(),
                reliable: true,
                ..Default::default()
            })
            .await
            .unwrap();
    }

    // Cleanup
    room.close().await.ok();
    Ok(())
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
