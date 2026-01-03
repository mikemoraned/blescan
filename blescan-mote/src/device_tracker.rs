//! Device tracker for managing discovered BLE devices

use blescan_domain::{peripheral::Peripheral, signature::Signature};
use serde::Serialize;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Maximum number of devices to track
const MAX_DEVICES: usize = 20;

/// Represents a discovered BLE device with its signature
#[derive(Clone, Debug, Serialize)]
pub struct DiscoveredDevice {
    /// Device signature (Named or Anonymous)
    pub signature: Signature,
    /// Received Signal Strength Indicator in dBm
    pub rssi: i32,
    /// Age in seconds since last seen
    #[serde(skip)]
    last_seen: Instant,
}

impl DiscoveredDevice {
    pub fn new(signature: Signature, rssi: i32) -> Self {
        Self {
            signature,
            rssi,
            last_seen: Instant::now(),
        }
    }

    /// Returns age in seconds since last seen
    pub fn age_secs(&self) -> u64 {
        self.last_seen.elapsed().as_secs()
    }

    /// Update the device's RSSI and last seen time
    pub fn update(&mut self, rssi: i32) {
        self.rssi = rssi;
        self.last_seen = Instant::now();
    }
}

/// Response structure for device list
#[derive(Serialize)]
pub struct DeviceListResponse {
    pub seq: u32,
    pub count: usize,
    pub devices: Vec<DiscoveredDevice>,
}

/// Thread-safe collection of discovered devices
pub struct DeviceTracker {
    /// Map from signature to device info
    devices: HashMap<Signature, DiscoveredDevice>,
    /// Sequence number that increments on each update
    sequence: u32,
}

impl DeviceTracker {
    pub fn new() -> Self {
        Self {
            devices: HashMap::with_capacity(MAX_DEVICES),
            sequence: 0,
        }
    }

    /// Update or add a device to the tracker
    pub fn update(&mut self, peripheral: Peripheral, rssi: i32) {
        // Convert peripheral to signature
        if let Some(signature) = peripheral.try_into_signature() {
            // Look for existing device by signature
            if let Some(device) = self.devices.get_mut(&signature) {
                device.update(rssi);
            } else {
                // Add new device
                if self.devices.len() >= MAX_DEVICES {
                    // Remove oldest device
                    if let Some(oldest_sig) = self
                        .devices
                        .iter()
                        .max_by_key(|(_, d)| d.age_secs())
                        .map(|(sig, _)| sig.clone())
                    {
                        self.devices.remove(&oldest_sig);
                    }
                }
                self.devices
                    .insert(signature.clone(), DiscoveredDevice::new(signature, rssi));
            }
            self.sequence = self.sequence.wrapping_add(1);
        }
    }

    /// Remove devices not seen for more than the specified duration
    pub fn prune_old(&mut self, max_age: Duration) {
        let before_len = self.devices.len();
        self.devices
            .retain(|_, d| d.last_seen.elapsed() < max_age);
        if self.devices.len() != before_len {
            self.sequence = self.sequence.wrapping_add(1);
        }
    }

    /// Get a sorted list of devices (by RSSI, strongest first)
    pub fn get_sorted(&self) -> Vec<DiscoveredDevice> {
        let mut devices: Vec<DiscoveredDevice> = self.devices.values().cloned().collect();
        devices.sort_by(|a, b| b.rssi.cmp(&a.rssi));
        devices
    }

    /// Serialize to JSON for BLE transmission
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let devices = self.get_sorted();
        let response = DeviceListResponse {
            seq: self.sequence,
            count: self.devices.len(),
            devices: devices.into_iter().take(10).collect(),
        };
        serde_json::to_string(&response)
    }

    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    pub fn get_sequence(&self) -> u32 {
        self.sequence
    }
}
