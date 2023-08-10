use std::error::Error;

use blescan::{discover_btleplug::Scanner, state::State};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let mut scanner = Scanner::new().await?;
    let mut state = State::default();
    loop {
        println!("{}", state.snapshot());
        let events = scanner.scan().await?;
        state.discover(events);
    }
}

