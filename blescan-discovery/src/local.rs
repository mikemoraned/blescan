use chrono::Utc;
use std::error::Error;
use std::time::Duration;
use tokio::time;

use btleplug::api::{Central, Manager as _, Peripheral as BtlePeripheral, ScanFilter};
use btleplug::platform::{Adapter, Manager};

use blescan_domain::discover::DiscoveryEvent;
use blescan_domain::peripheral::Peripheral;

use crate::Scanner;
use async_trait::async_trait;

pub struct LocalScanner {
    adapter: Adapter,
}

impl LocalScanner {
    pub async fn new() -> Result<LocalScanner, Box<dyn Error>> {
        let manager = Manager::new().await?;
        let mut adapter_list = manager.adapters().await?;
        if adapter_list.is_empty() {
            eprintln!("No Bluetooth adapters found");
        }
        let adapter = adapter_list.pop().unwrap();
        Ok(LocalScanner { adapter })
    }
}

#[async_trait]
impl Scanner for LocalScanner {
    async fn scan(&mut self) -> Result<Vec<DiscoveryEvent>, Box<dyn Error>> {
        self.adapter
            .start_scan(ScanFilter::default())
            .await
            .expect("Can't scan BLE adapter for connected devices...");
        time::sleep(Duration::from_secs(1)).await;
        let peripherals = self.adapter.peripherals().await?;
        let mut events = vec![];
        let current_time = Utc::now();
        for peripheral in &peripherals {
            let properties = peripheral.properties().await?.unwrap();
            let peripheral_info = Peripheral::new(
                properties.local_name.clone(),
                properties.manufacturer_data.clone(),
            );
            if let Some(signature) = peripheral_info.try_into_signature()
                && let Some(rssi) = properties.rssi
            {
                events.push(DiscoveryEvent::new(current_time, signature, rssi));
            }
        }
        self.adapter.stop_scan().await.expect("Can't stop scan");
        Ok(events)
    }
}
