pub mod local;

use async_trait::async_trait;
use blescan_domain::discover::DiscoveryEvent;
use std::error::Error;

#[async_trait]
pub trait Scanner {
    async fn scan(&mut self) -> Result<Vec<DiscoveryEvent>, Box<dyn Error>>;
}

pub enum ScanMode {
    Local,
}

impl ScanMode {
    pub async fn create_scanner(self) -> Result<Box<dyn Scanner>, Box<dyn Error>> {
        match self {
            ScanMode::Local => {
                let local = local::LocalScanner::new().await?;
                Ok(Box::new(local))
            }
        }
    }
}
