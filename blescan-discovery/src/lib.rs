pub mod local;
pub mod mote;

use async_trait::async_trait;
use blescan_domain::discover::DiscoveryEvent;
use std::error::Error;

#[async_trait]
pub trait Scanner {
    async fn scan(&mut self) -> Result<Vec<DiscoveryEvent>, Box<dyn Error>>;
}

#[derive(Debug, Clone, Copy)]
pub enum ScanMode {
    Local,
    Mote
}

impl std::str::FromStr for ScanMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "local" => Ok(ScanMode::Local),
            "mote" => Ok(ScanMode::Mote),
            _ => Err(format!("Invalid scan mode: {}. Must be 'local' or 'mote'", s)),
        }
    }
}

impl ScanMode {
    pub async fn create_scanner(self) -> Result<Box<dyn Scanner>, Box<dyn Error>> {
        match self {
            ScanMode::Local => {
                let local = local::LocalScanner::new().await?;
                Ok(Box::new(local))
            },
            ScanMode::Mote => {
                let mote = mote::MoteScanner::new().await?;
                Ok(Box::new(mote))
            }
        }
    }
}
