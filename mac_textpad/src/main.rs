mod backend_api;
mod ui;
use crate::backend_api::MockBackend;
use crate::ui::AppView;
use eframe::NativeOptions;
use egui::vec2;


use livekit::prelude::*;

use tokio_tungstenite::connect_async;
use tungstenite::http::Request;

#[tokio::main]
async fn main() {
    let url = "wss://inzynierka-28c03xlk.livekit.cloud";
    let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE3NjM1OTc1MDcsImlkZW50aXR5IjoicGFydGljaXBhbnQtNTE1YWYwOGU5NjlkIiwiaXNzIjoiQVBJQXJFOGU1U1pyQTVKIiwibmFtZSI6InBhcnRpY2lwYW50LTUxNWFmMDhlOTY5ZCIsIm5iZiI6MTc2MzU5NzIwNywic3ViIjoicGFydGljaXBhbnQtNTE1YWYwOGU5NjlkIiwidmlkZW8iOnsiY2FuVXBkYXRlT3duTWV0YWRhdGEiOnRydWUsInJvb20iOiJyb29tLTIwMjUxMTIwMDAwNjM4Iiwicm9vbUFkbWluIjp0cnVlLCJyb29tQ3JlYXRlIjp0cnVlLCJyb29tSm9pbiI6dHJ1ZSwicm9vbUxpc3QiOnRydWUsInJvb21SZWNvcmQiOnRydWV9fQ.CEyNUxahgvtUHJFw_cV0BUsmvz5arVYZXKZbIrYo_RE";

    let request = Request::builder()
        .uri(url)
        .header("Authorization", format!("Bearer {}", token))
        .body(())
        .unwrap();

    match connect_async(request).await {
        Ok((ws_stream, _)) => {
            println!("Connected to LiveKit!");
            // Handle WebSocket stream
        }
        Err(e) => {
            println!("WebSocket connection error: {:?}", e);
        }
    }
}