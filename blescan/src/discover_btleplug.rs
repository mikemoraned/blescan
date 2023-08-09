use std::error::Error;
use std::time::Duration;
use chrono::Utc;
use tokio::time;

use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::{Manager, Adapter};

use crate::discover::DiscoveryEvent;
use crate::signature::Signature;

pub struct Scanner {
    scans: u16,
    adapter: Adapter
}

impl Scanner {
    pub async fn new() -> Result<Scanner, Box<dyn Error>> {
        let scans = 0;
        
        let manager = Manager::new().await?;
        let mut adapter_list = manager.adapters().await?;
        if adapter_list.is_empty() {
            eprintln!("No Bluetooth adapters found");
        }
        let adapter = adapter_list.pop().unwrap();
        Ok(Scanner {
            scans, adapter
        })
    }

    pub async fn scan(&mut self) -> Result<Vec<DiscoveryEvent>, Box<dyn Error>> {
        self.scans += 1;        
        self.adapter
            .start_scan(ScanFilter::default())
            .await
            .expect("Can't scan BLE adapter for connected devices...");
        time::sleep(Duration::from_secs(1)).await;
        let peripherals = self.adapter.peripherals().await?;
        let mut events = vec![];
        let current_time = Utc::now();
        for peripheral in peripherals.iter() {
            let properties = peripheral.properties().await?.unwrap();
            if let Some(signature) = Signature::find(&properties) {
                if let Some(rssi) = properties.rssi {
                    events.push(DiscoveryEvent::new(current_time, signature, rssi));
                }
            }
        }
        self.adapter
            .stop_scan().await
            .expect("Can't stop scan");
        Ok(events)
    }
}
