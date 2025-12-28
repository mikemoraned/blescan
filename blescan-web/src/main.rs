use std::error::Error;

use blescan_discovery::discover_btleplug::Scanner;
use blescan_domain::state::State;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut scanner = Scanner::new().await?;
    let mut state = State::default();
    loop {
        let events = scanner.scan().await?;
        state.discover(&events);
        let current_snapshot = state.snapshot();
        println!("Snapshot: {}", current_snapshot);
    }
}
