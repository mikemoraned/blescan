use chrono::Utc;
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use tokio::time;

use btleplug::api::{Central, Manager as _, Peripheral as BtlePeripheral, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral, PeripheralId};
use uuid::Uuid;

use blescan_domain::discover::DiscoveryEvent;
use blescan_mote::device_tracker::DiscoveredDevice;

use crate::Scanner;
use async_trait::async_trait;

struct ConnectedPeripheral {
    peripheral: Peripheral,
}

pub struct MoteScanner {
    adapter: Adapter,
    connected: HashMap<PeripheralId, ConnectedPeripheral>,
}

impl MoteScanner {
    pub async fn new() -> Result<MoteScanner, Box<dyn Error>> {
        let manager = Manager::new().await?;
        let mut adapter_list = manager.adapters().await?;
        if adapter_list.is_empty() {
            eprintln!("No Bluetooth adapters found");
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
        eprintln!("[MoteScanner] Starting scan");
        let scan_time = Utc::now();

        // Parse the UUIDs we're looking for
        let service_uuid = Uuid::parse_str(blescan_mote::MOTE_SERVICE_UUID)?;
        let characteristic_uuid = Uuid::parse_str(blescan_mote::MOTE_DISCOVERED_DEVICES_CHARACTERISTIC_UUID)?;
        eprintln!("[MoteScanner] Looking for service UUID: {}", service_uuid);
        eprintln!("[MoteScanner] Looking for characteristic UUID: {}", characteristic_uuid);

        // Step 1: Remove disconnected peripherals from our connected list
        eprintln!("[MoteScanner] Checking existing connections ({} total)", self.connected.len());
        let mut to_remove = Vec::new();
        for (id, conn) in &self.connected {
            match conn.peripheral.is_connected().await {
                Ok(true) => {
                    // Still connected, keep it
                }
                Ok(false) => {
                    eprintln!("[MoteScanner] Removing disconnected peripheral");
                    to_remove.push(id.clone());
                }
                Err(e) => {
                    eprintln!("[MoteScanner] Error checking connection status: {}, removing", e);
                    to_remove.push(id.clone());
                }
            }
        }
        for id in to_remove {
            self.connected.remove(&id);
        }
        eprintln!("[MoteScanner] {} peripherals still connected", self.connected.len());

        // Step 2: Discover new peripherals via ScanFilter
        eprintln!("[MoteScanner] Starting BLE scan");
        self.adapter
            .start_scan(ScanFilter {
                services: vec![service_uuid],
            })
            .await
            .expect("Can't scan BLE adapter for devices");
        time::sleep(Duration::from_secs(1)).await;

        // Get all peripherals found during scan
        let discovered_peripherals = self.adapter.peripherals().await?;
        eprintln!("[MoteScanner] Found {} peripherals during scan", discovered_peripherals.len());

        // Step 3: Find peripherals we're not already connected to and add them
        for peripheral in discovered_peripherals {
            let peripheral_id = peripheral.id();

            // Check if we're already connected to this peripheral (fast HashMap lookup)
            if self.connected.contains_key(&peripheral_id) {
                eprintln!("[MoteScanner] Already connected to this peripheral, skipping");
                continue;
            }

            eprintln!("[MoteScanner] Connecting to new peripheral...");
            if let Err(e) = peripheral.connect().await {
                eprintln!("Failed to connect to peripheral: {}, skipping", e);
                continue;
            }
            eprintln!("[MoteScanner] Connected successfully");

            // Discover services and characteristics
            eprintln!("[MoteScanner] Discovering services...");
            if let Err(e) = peripheral.discover_services().await {
                eprintln!("Failed to discover services: {}, skipping device", e);
                let _ = peripheral.disconnect().await;
                continue;
            }
            eprintln!("[MoteScanner] Services discovered");

            // Add to our connected list using the peripheral ID as the key
            self.connected.insert(peripheral_id, ConnectedPeripheral { peripheral });
            eprintln!("[MoteScanner] Added peripheral to connected list");
        }

        eprintln!("[MoteScanner] Stopping scan");
        self.adapter.stop_scan().await.expect("Can't stop scan");
        eprintln!("[MoteScanner] Total connected peripherals: {}", self.connected.len());

        // Step 4 & 5: For each connected peripheral, read characteristics and collect DiscoveryEvents
        let mut events = vec![];
        for (idx, (_id, conn)) in self.connected.iter().enumerate() {
            eprintln!("[MoteScanner] Processing connected peripheral {}/{}", idx + 1, self.connected.len());

            // Find the MOTE_DISCOVERED_DEVICES_CHARACTERISTIC_UUID characteristic
            eprintln!("[MoteScanner] Looking for characteristic...");
            let characteristics = conn.peripheral.characteristics();
            let characteristic = characteristics
                .iter()
                .find(|c| c.uuid == characteristic_uuid);

            if let Some(characteristic) = characteristic {
                eprintln!("[MoteScanner] Found characteristic, reading data...");
                // Read the characteristic value
                match conn.peripheral.read(characteristic).await {
                    Ok(data) => {
                        eprintln!("[MoteScanner] Read {} bytes from characteristic", data.len());
                        // Parse JSON into list of DiscoveredDevices
                        match String::from_utf8(data) {
                            Ok(json_str) => {
                                eprintln!("[MoteScanner] Converted to UTF-8 string");
                                match serde_json::from_str::<serde_json::Value>(&json_str) {
                                    Ok(json_value) => {
                                        eprintln!("[MoteScanner] JSON parsed successfully");
                                        // Extract devices array from JSON response
                                        if let Some(devices) = json_value.get("devices").and_then(|d| d.as_array()) {
                                            eprintln!("[MoteScanner] Found {} devices in JSON", devices.len());
                                            // Convert each DiscoveredDevice to a DiscoveryEvent
                                            for (device_idx, device_value) in devices.iter().enumerate() {
                                                match serde_json::from_value::<DiscoveredDevice>(device_value.clone()) {
                                                    Ok(discovered_device) => {
                                                        eprintln!("[MoteScanner] Parsed device {}/{}", device_idx + 1, devices.len());
                                                        events.push(DiscoveryEvent::new(
                                                            scan_time,
                                                            discovered_device.signature,
                                                            discovered_device.rssi as i16,
                                                        ));
                                                    }
                                                    Err(e) => {
                                                        eprintln!("Failed to parse DiscoveredDevice: {}, skipping", e);
                                                    }
                                                }
                                            }
                                        } else {
                                            eprintln!("[MoteScanner] No 'devices' array found in JSON");
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to parse JSON: {}, skipping device", e);
                                        eprintln!("Received JSON (length: {}): {}", json_str.len(), json_str);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to convert characteristic data to UTF-8: {}, skipping device", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to read characteristic: {}, skipping device", e);
                    }
                }
            } else {
                eprintln!("[MoteScanner] Characteristic not found");
            }
        }

        eprintln!("[MoteScanner] Scan complete, found {} events", events.len());
        Ok(events)
    }
}
