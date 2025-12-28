use axum::{Router, response::IntoResponse, routing::get};
use axum_typed_websockets::{Message, WebSocket, WebSocketUpgrade};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services::{ServeDir, ServeFile};
use ts_rs::TS;

use blescan_discovery::discover_btleplug::Scanner;
use blescan_domain::{snapshot::Snapshot, state::State};

// Server -> Client messages
#[derive(Serialize, TS)]
#[ts(export)]
#[serde(tag = "type", content = "data")]
pub enum ServerMsg {
    Heartbeat { seq: u32 },
    NewSnapshot { snapshot: Snapshot },
}

// Client -> Server messages
#[derive(Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type", content = "data")]
pub enum ClientMsg {}

async fn ws_handler(ws: WebSocketUpgrade<ServerMsg, ClientMsg>) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket<ServerMsg, ClientMsg>) {
    let mut seq = 0u32;
    loop {
        seq += 1;
        if socket
            .send(Message::Item(ServerMsg::Heartbeat { seq }))
            .await
            .is_err()
        {
            println!("Client disconnected");
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut scanner = Scanner::new().await?;
    let state = Arc::new(RwLock::new(State::default()));

    let state_clone = Arc::clone(&state);
    tokio::spawn(async move {
        loop {
            let scan_result = scanner.scan().await.map_err(|e| e.to_string());
            match scan_result {
                Ok(events) => {
                    let mut state = state_clone.write().await;
                    state.discover(&events);
                }
                Err(error_msg) => {
                    eprintln!("Scanner error: {}", error_msg);
                }
            }
        }
    });

    let serve_dir =
        ServeDir::new("./frontend").not_found_service(ServeFile::new("./frontend/index.html"));

    let app = Router::new()
        // .route("/ws", get(ws_handler))
        .nest_service("/bindings", ServeDir::new("bindings"))
        .fallback_service(serve_dir);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    // loop {
    //     let current_snapshot = {
    //         let state = state.read().await;
    //         state.snapshot()
    //     };
    //     println!("Snapshot: {}", current_snapshot);

    //     tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    // }

    Ok(())
}
