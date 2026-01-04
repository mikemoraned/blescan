use chrono::Utc;
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use tokio::time;
use tracing::{error, trace};

use btleplug::api::{Central, Manager as _, Peripheral as BtlePeripheral, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral, PeripheralId};
use uuid::Uuid;

use blescan_domain::discover::DiscoveryEvent;
use blescan_mote::device_tracker::DiscoveredDevice;

use crate::Scanner;
use async_trait::async_trait;

struct Mote {
    peripheral: Peripheral,
}

impl Mote {
    async fn collect(
        &self,
        scan_time: chrono::DateTime<Utc>,
        characteristic_uuid: Uuid,
    ) -> Result<Vec<DiscoveryEvent>, Box<dyn Error>> {
        // Find the MOTE_DISCOVERED_DEVICES_CHARACTERISTIC_UUID characteristic
        trace!("[Mote] Looking for characteristic...");
        let characteristics = self.peripheral.characteristics();
        let characteristic = characteristics
            .iter()
            .find(|c| c.uuid == characteristic_uuid)
            .ok_or("Characteristic not found")?;

        trace!("[Mote] Found characteristic, reading data...");
        // Read the characteristic value
        let data = self.peripheral.read(characteristic).await?;
        trace!("[Mote] Read {} bytes from characteristic", data.len());

        // Parse JSON into list of DiscoveredDevices
        let json_str = String::from_utf8(data)?;
        trace!("[Mote] Converted to UTF-8 string");

        let json_value = serde_json::from_str::<serde_json::Value>(&json_str)?;
        trace!("[Mote] JSON parsed successfully");

        // Extract devices array from JSON response
        let devices = json_value
            .get("devices")
            .and_then(|d| d.as_array())
            .ok_or("No 'devices' array found in JSON")?;

        trace!("[Mote] Found {} devices in JSON", devices.len());

        // Convert each DiscoveredDevice to a DiscoveryEvent
        let mut events = Vec::new();
        for (device_idx, device_value) in devices.iter().enumerate() {
            match serde_json::from_value::<DiscoveredDevice>(device_value.clone()) {
                Ok(discovered_device) => {
                    trace!("[Mote] Parsed device {}/{}", device_idx + 1, devices.len());
                    events.push(DiscoveryEvent::new(
                        scan_time,
                        discovered_device.signature,
                        discovered_device.rssi as i16,
                    ));
                }
                Err(e) => {
                    error!("Failed to parse DiscoveredDevice: {}, skipping", e);
                }
            }
        }

        Ok(events)
    }
}

pub struct MoteScanner {
    adapter: Adapter,
    connected: HashMap<PeripheralId, Mote>,
}

impl MoteScanner {
    pub async fn new() -> Result<MoteScanner, Box<dyn Error>> {
        let manager = Manager::new().await?;
        let mut adapter_list = manager.adapters().await?;
        if adapter_list.is_empty() {
            error!("No Bluetooth adapters found");
        }
        let adapter = adapter_list.pop().unwrap();
        Ok(MoteScanner {
            adapter,
            connected: HashMap::new(),
        })
    }
}

#[async_trait]
impl Scanner for MoteScanner {
    async fn scan(&mut self) -> Result<Vec<DiscoveryEvent>, Box<dyn Error>> {
        trace!("[MoteScanner] Starting scan");
        let scan_time = Utc::now();

        // Parse the UUIDs we're looking for
        let service_uuid = Uuid::parse_str(blescan_mote::MOTE_SERVICE_UUID)?;
        let characteristic_uuid = Uuid::parse_str(blescan_mote::MOTE_DISCOVERED_DEVICES_CHARACTERISTIC_UUID)?;
        trace!("[MoteScanner] Looking for service UUID: {}", service_uuid);
        trace!("[MoteScanner] Looking for characteristic UUID: {}", characteristic_uuid);

        // Step 1: Remove disconnected motes from our connected list
        trace!("[MoteScanner] Checking existing connections ({} total)", self.connected.len());
        let mut to_remove = Vec::new();
        for (id, conn) in &self.connected {
            match conn.peripheral.is_connected().await {
                Ok(true) => {
                    // Still connected, keep it
                }
                Ok(false) => {
                    trace!("[MoteScanner] Removing disconnected mote");
                    to_remove.push(id.clone());
                }
                Err(e) => {
                    error!("[MoteScanner] Error checking connection status: {}, removing", e);
                    to_remove.push(id.clone());
                }
            }
        }
        for id in to_remove {
            self.connected.remove(&id);
        }
        trace!("[MoteScanner] {} motes still connected", self.connected.len());

        // Step 2: Discover new motes via ScanFilter
        trace!("[MoteScanner] Starting BLE scan");
        self.adapter
            .start_scan(ScanFilter {
                services: vec![service_uuid],
            })
            .await
            .expect("Can't scan BLE adapter for devices");
        time::sleep(Duration::from_secs(1)).await;

        // Get all motes found during scan
        let discovered_motes = self.adapter.peripherals().await?;
        trace!("[MoteScanner] Found {} motes during scan", discovered_motes.len());

        // Step 3: Find motes we're not already connected to and add them
        for mote in discovered_motes {
            let mote_id = mote.id();

            // Check if we're already connected to this mote (fast HashMap lookup)
            if self.connected.contains_key(&mote_id) {
                trace!("[MoteScanner] Already connected to this mote, skipping");
                continue;
            }

            trace!("[MoteScanner] Connecting to new mote...");
            if let Err(e) = mote.connect().await {
                error!("Failed to connect to mote: {}, skipping", e);
                continue;
            }
            trace!("[MoteScanner] Connected successfully");

            // Discover services and characteristics
            trace!("[MoteScanner] Discovering services...");
            if let Err(e) = mote.discover_services().await {
                error!("Failed to discover services: {}, skipping device", e);
                let _ = mote.disconnect().await;
                continue;
            }
            trace!("[MoteScanner] Services discovered");

            // Add to our connected list using the mote ID as the key
            self.connected.insert(mote_id, Mote { peripheral: mote });
            trace!("[MoteScanner] Added mote to connected list");
        }

        trace!("[MoteScanner] Stopping scan");
        self.adapter.stop_scan().await.expect("Can't stop scan");
        trace!("[MoteScanner] Total connected motes: {}", self.connected.len());

        // Step 4 & 5: For each connected mote, read characteristics and collect DiscoveryEvents
        let mut events = vec![];
        let mut failed_motes = vec![];

        for (idx, (id, mote)) in self.connected.iter().enumerate() {
            trace!("[MoteScanner] Processing connected mote {}/{}", idx + 1, self.connected.len());

            match mote.collect(scan_time, characteristic_uuid).await {
                Ok(mut mote_events) => {
                    events.append(&mut mote_events);
                }
                Err(e) => {
                    error!("[MoteScanner] Failed to collect from mote: {}, removing from connected list", e);
                    failed_motes.push(id.clone());
                }
            }
        }

        // Remove failed motes from connected list
        for id in failed_motes {
            self.connected.remove(&id);
        }

        trace!("[MoteScanner] Scan complete, found {} events", events.len());
        Ok(events)
    }
}
