use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

use blescan_discovery::discover_btleplug::Scanner;
use blescan_domain::state::State;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut scanner = Scanner::new().await?;
    let state = Arc::new(RwLock::new(State::default()));

    // Background loop: owns Scanner, updates state
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

    // Main loop: read-only access to state, prints snapshots
    loop {
        let current_snapshot = {
            let state = state.read().await;
            state.snapshot()
        };
        println!("Snapshot: {}", current_snapshot);

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
