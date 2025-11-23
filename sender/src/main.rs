use livekit_api::services::room::{CreateRoomOptions, RoomClient};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let host = std::env::var("LIVEKIT_URL").unwrap_or_else(|_| "127.0.0.1:7880".to_string());

    // add "http://" prefix if missing
    let http_url = if !host.starts_with("http://") && !host.starts_with("https://") {
        format!("http://{}", host)
    } else {
        host
    };

    let room_service = match RoomClient::new(&http_url) {
        Ok(svc) => svc,
        Err(e) => {
            eprintln!("Failed to create RoomClient: {}. Ensure LIVEKIT_API_KEY and LIVEKIT_API_SECRET environment variables are set, or provide credentials programmatically.", e);
            return;
        }
    };

    let room_options = CreateRoomOptions {
        // Enable message sending by allowing data channels
        // (Assuming the livekit_api supports this option; adjust as needed)
        ..Default::default()
    };

    let room = match room_service.create_room("test_room", room_options).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to create room: {}", e);
            return;
        }
    };

    println!("Created room: {:?}", room);

    // Send a text message to the room

    let data = b"Hello, LiveKit room!".to_vec();
    let options = livekit_api::services::room::SendDataOptions {
        ..Default::default()
    };
    room_service
        .send_data(&room.name, data, options)
        .await
        .unwrap();

    println!("Sent message to room: {}", room.name);

    // press enter to send another message
    use std::io::{self, BufRead};

    println!("Type a message and press Enter to send. Type EXIT to quit.");

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let msg = match line {
            Ok(m) => m,
            Err(_) => break,
        };
        if msg.trim().eq_ignore_ascii_case("EXIT") {
            println!("Exiting application.");
            break;
        }
        let data = msg.into_bytes();
        let options = livekit_api::services::room::SendDataOptions {
            ..Default::default()
        };
        if let Err(e) = room_service.send_data(&room.name, data, options).await {
            eprintln!("Failed to send message: {}", e);
        } else {
            println!("Sent message to room: {}", room.name);
        }
    }
}
