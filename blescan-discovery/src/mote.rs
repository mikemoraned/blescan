use chrono::Utc;
use std::error::Error;
use std::time::Duration;
use tokio::time;

use btleplug::api::{Central, Manager as _, Peripheral as BtlePeripheral, ScanFilter};
use btleplug::platform::{Adapter, Manager};
use uuid::Uuid;

use blescan_domain::discover::DiscoveryEvent;
use blescan_mote::device_tracker::DiscoveredDevice;

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
        eprintln!("[MoteScanner] Starting scan");
        let mut events = vec![];
        let scan_time = Utc::now();

        // Parse the UUIDs we're looking for
        let service_uuid = Uuid::parse_str(blescan_mote::MOTE_SERVICE_UUID)?;
        let characteristic_uuid = Uuid::parse_str(blescan_mote::MOTE_DISCOVERED_DEVICES_CHARACTERISTIC_UUID)?;
        eprintln!("[MoteScanner] Looking for service UUID: {}", service_uuid);
        eprintln!("[MoteScanner] Looking for characteristic UUID: {}", characteristic_uuid);

        // Start scanning for BLE devices
        eprintln!("[MoteScanner] Starting BLE scan");
        self.adapter
            .start_scan(ScanFilter {
                services: vec![service_uuid],
            })
            .await
            .expect("Can't scan BLE adapter for devices");
        time::sleep(Duration::from_secs(1)).await;

        // Get all peripherals found during scan
        let peripherals = self.adapter.peripherals().await?;
        eprintln!("[MoteScanner] Found {} peripherals", peripherals.len());

        for (idx, peripheral) in peripherals.iter().enumerate() {
            eprintln!("[MoteScanner] Processing peripheral {}/{}", idx + 1, peripherals.len());

            // Connect to the peripheral to access its services and characteristics
            eprintln!("[MoteScanner] Connecting to peripheral...");
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

            // Check if this device has the MOTE_SERVICE_UUID service
            let has_service = peripheral.services().iter().any(|s| s.uuid == service_uuid);
            eprintln!("[MoteScanner] Has mote service: {}", has_service);
            if !has_service {
                let _ = peripheral.disconnect().await;
                continue;
            }

            // Find the MOTE_DISCOVERED_DEVICES_CHARACTERISTIC_UUID characteristic
            eprintln!("[MoteScanner] Looking for characteristic...");
            let characteristics = peripheral.characteristics();
            let characteristic = characteristics
                .iter()
                .find(|c| c.uuid == characteristic_uuid);

            if let Some(characteristic) = characteristic {
                eprintln!("[MoteScanner] Found characteristic, reading data...");
                // Read the characteristic value
                match peripheral.read(characteristic).await {
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

            // Disconnect from the peripheral
            eprintln!("[MoteScanner] Disconnecting from peripheral");
            let _ = peripheral.disconnect().await;
        }

        eprintln!("[MoteScanner] Stopping scan");
        self.adapter.stop_scan().await.expect("Can't stop scan");
        eprintln!("[MoteScanner] Scan complete, found {} events", events.len());
        Ok(events)
    }
}
