use std::error::Error;
use blescan::scanner::Scanner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let mut scanner = Scanner::new().await?;
    loop {
        scanner.scan().await?;
    }
}
