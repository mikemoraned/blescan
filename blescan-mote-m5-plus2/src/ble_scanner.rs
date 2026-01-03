//! BLE passive scanning functionality

use blescan_domain::peripheral::Peripheral;
use blescan_mote::device_tracker::DeviceTracker;
use esp32_nimble::{BLEDevice, BLEScan};
use log::info;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// BLE timing unit: 0.625ms per unit
const BLE_TIME_UNIT_MS: f32 = 0.625;

/// Scan duration in milliseconds for each scan cycle
pub const SCAN_DURATION_MS: u32 = 800;

/// Delay between scan cycles in milliseconds
pub const SCAN_CYCLE_DELAY_MS: u32 = 200;

/// BLE scan interval in milliseconds (62.5ms)
const BLE_SCAN_INTERVAL_MS: f32 = 62.5;

/// BLE scan interval in BLE time units (0.625ms units)
pub const BLE_SCAN_INTERVAL_UNITS: u16 = (BLE_SCAN_INTERVAL_MS / BLE_TIME_UNIT_MS) as u16;

/// BLE scan window in milliseconds (31.25ms)
const BLE_SCAN_WINDOW_MS: f32 = 31.25;

/// BLE scan window in BLE time units (0.625ms units)
pub const BLE_SCAN_WINDOW_UNITS: u16 = (BLE_SCAN_WINDOW_MS / BLE_TIME_UNIT_MS) as u16;

/// Maximum age for devices before pruning (30 seconds)
pub const MAX_DEVICE_AGE: Duration = Duration::from_secs(30);

/// Performs a single scan cycle and updates the device tracker
pub async fn scan_cycle(
    ble_scan: &mut BLEScan,
    ble_device: &BLEDevice,
    tracker: Arc<Mutex<DeviceTracker>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let tracker_for_scan = tracker.clone();

    // Start a scan for SCAN_DURATION_MS
    ble_scan
        .start(ble_device, SCAN_DURATION_MS as i32, |device, data| {
            let name = data.name().map(|n| n.to_string());
            let rssi = device.rssi() as i32;

            // Extract manufacturer data
            let mut manufacturer_data = HashMap::new();
            if let Some(mfg) = data.manufacture_data() {
                manufacturer_data.insert(mfg.company_identifier, mfg.payload.to_vec());
            }

            // Create Peripheral and update tracker
            let peripheral = Peripheral::new(name, manufacturer_data);
            if let Ok(mut t) = tracker_for_scan.lock() {
                t.update(peripheral, rssi);
            }

            None::<()>
        })
        .await
        .map_err(|e| format!("Scan error: {:?}", e))?;

    Ok(())
}

/// Prunes old devices from the tracker
pub fn prune_old_devices(tracker: Arc<Mutex<DeviceTracker>>) {
    if let Ok(mut t) = tracker.lock() {
        t.prune_old(MAX_DEVICE_AGE);
    }
}

/// Configure a BLE scanner for passive scanning
pub fn configure_scanner(ble_scan: &mut BLEScan) {
    ble_scan
        .active_scan(false) // Passive scanning
        .interval(BLE_SCAN_INTERVAL_UNITS)
        .window(BLE_SCAN_WINDOW_UNITS);

    info!("BLE scanner configured for passive scanning");
}
