use livekit_api::access_token;
use std::env;


use livekit::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let url = "ws://209.38.105.89:7880";
    let url = "ws://127.0.0.1:7880";
    dotenv::dotenv().ok();

    let token = create_token().expect("Failed to create access token");

    println!("Generated token: {}", token);

    println!("Connecting to LiveKit server at {}", url);

    let (room, mut room_events) = Room::connect(&url, &token, RoomOptions::default()).await?;

    while let Some(event) = room_events.recv().await {
        match event {
            RoomEvent::DataReceived { payload, topic, participant, .. } => {
                if let Some(p) = participant {
                    let from = p.identity();
                    let text = String::from_utf8_lossy(&payload);
                    println!(
                        "Received message from {} on {}: {}",
                        from,
                        topic.as_deref().unwrap_or("<none>"),
                        text
                    );
                } else {

                    println!("Received message (no participant info) {}", String::from_utf8_lossy(&payload));
                }
            }
            _ => {
                // Handle other events as needed
                println!("Received event: {:?}", event);
            }
        }
    }

    Ok(())
}

fn create_token() -> Result<String, access_token::AccessTokenError> {
    let api_key = env::var("LIVEKIT_API_KEY").expect("LIVEKIT_API_KEY is not set");
    let api_secret = env::var("LIVEKIT_API_SECRET").expect("LIVEKIT_API_SECRET is not set");

    let token = access_token::AccessToken::with_api_key(&api_key, &api_secret)
        .with_identity("rust-bot")
        .with_name("Rust Bot")
        .with_grants(access_token::VideoGrants {
            room_join: true,
            room: "test_room".to_string(),
            ..Default::default()
        })
        .to_jwt();
    return token;
}
