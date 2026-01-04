//! BLE GATT server functionality

use crate::ble_scanner;
use blescan_mote::device_tracker::DeviceTracker;
use esp_idf_hal::delay::FreeRtos;
use esp32_nimble::{BLEAdvertisementData, BLEDevice, BLEScan, NimbleProperties, uuid128};
use log::{info, warn};
use std::sync::{Arc, Mutex};

/// Device name for BLE advertising
const DEVICE_NAME: &str = "blescan-mote";

const MAX_DEVICES: usize = 20;

pub async fn run_ble_mote_server() {
    info!("Initializing BLE...");

    // Initialize BLE device
    let ble_device = BLEDevice::take();

    // Get handles for server and advertising
    let server = ble_device.get_server();
    let advertising = ble_device.get_advertising();

    // Configure server callbacks
    server.on_connect(|_server, desc| {
        info!("Client connected: {:?}", desc.address());
    });

    server.on_disconnect(|desc, reason| {
        info!(
            "Client disconnected: {:?}, reason: {:?}",
            desc.address(),
            reason
        );
    });

    // Create our custom service using the shared UUID
    let service = server.create_service(uuid128!(blescan_mote::MOTE_SERVICE_UUID));

    // Create the discovered devices characteristic with READ and NOTIFY properties
    let devices_characteristic = service.lock().create_characteristic(
        uuid128!(blescan_mote::MOTE_DISCOVERED_DEVICES_CHARACTERISTIC_UUID),
        NimbleProperties::READ | NimbleProperties::NOTIFY,
    );

    // Set initial value
    devices_characteristic
        .lock()
        .set_value(b"{\"seq\":0,\"count\":0,\"devices\":[]}");

    info!("Created discovered devices characteristic");

    // Track subscriptions for devices characteristic
    let has_subscribers = Arc::new(Mutex::new(false));
    let has_subscribers_clone = has_subscribers.clone();

    devices_characteristic
        .lock()
        .on_subscribe(move |_, _, sub| {
            let subscribed = !sub.is_empty();
            if let Ok(mut hs) = has_subscribers_clone.lock() {
                *hs = subscribed;
            }
            info!(
                "Devices subscription changed: {}",
                if subscribed {
                    "subscribed"
                } else {
                    "unsubscribed"
                }
            );
        });

    // Configure advertising data
    let mut ad_data = BLEAdvertisementData::new();
    ad_data
        .name(DEVICE_NAME)
        .add_service_uuid(uuid128!(blescan_mote::MOTE_SERVICE_UUID));

    advertising.lock().set_data(&mut ad_data).unwrap();

    // Start advertising
    advertising.lock().start().unwrap();
    info!("BLE Server started, advertising as '{}'", DEVICE_NAME);
    info!("Service UUID: {}", blescan_mote::MOTE_SERVICE_UUID);
    info!(
        "Discovered Devices Characteristic UUID: {}",
        blescan_mote::MOTE_DISCOVERED_DEVICES_CHARACTERISTIC_UUID
    );

    // Create shared device tracker
    let tracker = Arc::new(Mutex::new(DeviceTracker::new(MAX_DEVICES)));

    // Create scan instance and configure it
    let mut ble_scan = BLEScan::new();
    ble_scanner::configure_scanner(&mut ble_scan);

    info!("Starting continuous BLE scanning...");

    let mut last_sequence: u32 = 0;

    // Main loop: scan -> update characteristics -> repeat
    loop {
        // Perform scan cycle
        match ble_scanner::scan_cycle(&mut ble_scan, &ble_device, tracker.clone()).await {
            Ok(_) => {
                // Scan completed successfully
            }
            Err(e) => {
                warn!("Scan error: {}", e);
                FreeRtos::delay_ms(100);
                continue;
            }
        }

        // Prune old devices
        ble_scanner::prune_old_devices(tracker.clone());

        // Check if we have new device data to notify
        let (should_notify, json_data, device_count) = {
            if let Ok(t) = tracker.lock() {
                let current_seq = t.get_sequence();
                let changed = current_seq != last_sequence;
                last_sequence = current_seq;
                let json = t.to_json().unwrap_or_else(|e| {
                    warn!("JSON serialization error: {}", e);
                    String::from("{\"seq\":0,\"count\":0,\"devices\":[]}")
                });
                (changed, json, t.device_count())
            } else {
                (false, String::new(), 0)
            }
        };

        // Update devices characteristic and notify subscribers
        if should_notify {
            let has_subs = has_subscribers.lock().map(|h| *h).unwrap_or(false);

            // Always update the value (for reads)
            devices_characteristic
                .lock()
                .set_value(json_data.as_bytes());

            // Notify if we have subscribers
            if has_subs {
                devices_characteristic.lock().notify();
                info!(
                    "Notified subscribers: {} devices ({} bytes)",
                    device_count,
                    json_data.len()
                );
            }
        }

        // Brief delay between scan cycles
        FreeRtos::delay_ms(ble_scanner::SCAN_CYCLE_DELAY_MS);
    }
}
