// mod backend_api;
// mod ui;
// use livekit_api::access_token;
// use std::env;

// // use crate::backend_api::MockBackend;
// // use crate::ui::AppView;
// // use eframe::NativeOptions;
// // use egui::vec2;

// // use livekit::prelude::*;
// // fn main() -> eframe::Result<()> {
// //     let mut native_options = NativeOptions::default();
// //     native_options.centered = true;
// //     eframe::run_native(
// //         "Collaborative Text Editor",
// //         native_options,
// //         Box::new(|_cc| Ok(Box::new(AppView::new(Box::new(MockBackend::default()))))),
// //     )
// // }

use livekit_api::services::room::{CreateRoomOptions, RoomClient};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    // let api_host = "https://inzynierka-28c03xlk.livekit.cloud";
    let api_host = "http://209.38.105.89:7880";

    let room_service = match RoomClient::new(api_host) {
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

    let stdin = io::stdin();
    let _ = stdin.lock().lines().next();
    let data = b"Another message from Rust client!".to_vec();
    let options = livekit_api::services::room::SendDataOptions {
        ..Default::default()
    };
    room_service
        .send_data(&room.name, data, options)
        .await
        .unwrap();
    println!("Sent another message to room: {}", room.name);

    // await for user input to exit
    use std::io::{self, BufRead};
    println!("Press Enter to exit...");
    let stdin = io::stdin();
    let _ = stdin.lock().lines().next();
}
