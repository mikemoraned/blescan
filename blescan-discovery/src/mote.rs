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

pub struct MoteScanner {
    adapter: Adapter,
}

impl MoteScanner {
    pub async fn new() -> Result<MoteScanner, Box<dyn Error>> {
        let manager = Manager::new().await?;
        let mut adapter_list = manager.adapters().await?;
        if adapter_list.is_empty() {
            eprintln!("No Bluetooth adapters found");
        }
        let adapter = adapter_list.pop().unwrap();
        Ok(MoteScanner { adapter })
    }
}

#[async_trait]
impl Scanner for MoteScanner {
    async fn scan(&mut self) -> Result<Vec<DiscoveryEvent>, Box<dyn Error>> {
        let events = vec![];
        // TODO:
        // 1. Find all first Motes (BLE devices) which have the MOTE_SERVICE_UUID Service *and* have the MOTE_DISCOVERED_DEVICES_CHARACTERISTIC_UUID Characteristic
        // 2. For each, parse the JSON into a `DiscoveredDevice`
        // 3. Collect all `Signature` into a `DiscoveryEvent`
        // For each call of `scan` these events should have the same `date_time` which is time the scan was called
        // Any errors should lead to the device being skipped and it's data ignored
        Ok(events)
    }
}
