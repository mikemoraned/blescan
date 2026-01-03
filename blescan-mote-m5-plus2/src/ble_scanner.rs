//! BLE passive scanning functionality

use blescan_domain::peripheral::Peripheral;
use blescan_mote::device_tracker::DeviceTracker;
use esp32_nimble::{BLEDevice, BLEScan};
use log::info;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Scan duration in milliseconds for each scan cycle
pub const SCAN_DURATION_MS: u32 = 800;

/// Delay between scan cycles in milliseconds
pub const SCAN_INTERVAL_MS: u32 = 200;

/// BLE scan interval in 0.625ms units (100 = 62.5ms)
/// TODO: Consider consolidating SCAN_INTERVAL_MS and BLE_SCAN_INTERVAL_UNITS into one shared constant
pub const BLE_SCAN_INTERVAL_UNITS: u16 = 100;

/// BLE scan window in 0.625ms units (50 = 31.25ms)
pub const BLE_SCAN_WINDOW_UNITS: u16 = 50;

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
