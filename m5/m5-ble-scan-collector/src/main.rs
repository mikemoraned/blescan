//! BLE Scanner Server for M5StickC PLUS2
//!
//! This application:
//! 1. Passively scans for BLE devices in the environment
//! 2. Exposes the scan results via a BLE GATT service with notifications
//!
//! Clients can subscribe to the scan results characteristic to receive
//! periodic updates about discovered BLE devices.

use esp32_nimble::{
    uuid128, BLEAdvertisementData, BLEDevice, BLEScan,
    NimbleProperties,
};
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::task::block_on;
use esp_idf_svc::log::EspLogger;
use esp_idf_sys as _;
use log::{info, warn};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Maximum number of devices to track
const MAX_DEVICES: usize = 20;

/// Scan duration in milliseconds for each scan cycle
const SCAN_DURATION_MS: u32 = 800;

/// Delay between scan cycles in milliseconds
const SCAN_INTERVAL_MS: u32 = 200;

/// Device name for BLE advertising
const DEVICE_NAME: &str = "BLEScanServer";

/// Custom UUIDs for our BLE service
/// Service UUID: identifies our BLE scanner service
const SERVICE_UUID: &str = "12345678-1234-5678-1234-56789abcdef0";
/// Characteristic UUID: provides the scan results
const SCAN_RESULTS_UUID: &str = "12345678-1234-5678-1234-56789abcdef1";

/// Represents a discovered BLE device
#[derive(Clone, Debug)]
struct DiscoveredDevice {
    /// Device name (if available)
    name: String,
    /// BLE MAC address as string
    address: String,
    /// Received Signal Strength Indicator in dBm
    rssi: i32,
    /// When this device was last seen
    last_seen: Instant,
}

impl DiscoveredDevice {
    fn new(name: String, address: String, rssi: i32) -> Self {
        Self {
            name,
            address,
            rssi,
            last_seen: Instant::now(),
        }
    }

    /// Returns age in seconds since last seen
    fn age_secs(&self) -> u64 {
        self.last_seen.elapsed().as_secs()
    }

    /// Serialize to a compact JSON-like format for BLE transmission
    /// Format: {"n":"name","a":"addr","r":-50,"t":2}
    fn to_json(&self) -> String {
        // Use short keys to minimize payload size
        // n = name, a = address, r = rssi, t = time since last seen (seconds)
        format!(
            r#"{{"n":"{}","a":"{}","r":{},"t":{}}}"#,
            self.name.chars().take(20).collect::<String>(), // Truncate long names
            self.address,
            self.rssi,
            self.age_secs()
        )
    }
}

/// Thread-safe collection of discovered devices
struct DeviceTracker {
    devices: Vec<DiscoveredDevice>,
    /// Sequence number that increments on each update
    sequence: u32,
}

impl DeviceTracker {
    fn new() -> Self {
        Self {
            devices: Vec::with_capacity(MAX_DEVICES),
            sequence: 0,
        }
    }

    /// Update or add a device to the tracker
    fn update(&mut self, name: String, address: String, rssi: i32) {
        // Look for existing device by address
        if let Some(device) = self.devices.iter_mut().find(|d| d.address == address) {
            device.rssi = rssi;
            device.last_seen = Instant::now();
            // Update name if we now have a better one
            if !name.is_empty() && device.name.is_empty() {
                device.name = name;
            }
        } else {
            // Add new device
            if self.devices.len() >= MAX_DEVICES {
                // Remove oldest device
                if let Some(oldest_idx) = self
                    .devices
                    .iter()
                    .enumerate()
                    .max_by_key(|(_, d)| d.age_secs())
                    .map(|(i, _)| i)
                {
                    self.devices.remove(oldest_idx);
                }
            }
            self.devices.push(DiscoveredDevice::new(name, address, rssi));
        }
        self.sequence = self.sequence.wrapping_add(1);
    }

    /// Remove devices not seen for more than the specified duration
    fn prune_old(&mut self, max_age: Duration) {
        let before_len = self.devices.len();
        self.devices.retain(|d| d.last_seen.elapsed() < max_age);
        if self.devices.len() != before_len {
            self.sequence = self.sequence.wrapping_add(1);
        }
    }

    /// Get a sorted list of devices (by RSSI, strongest first)
    fn get_sorted(&self) -> Vec<DiscoveredDevice> {
        let mut devices = self.devices.clone();
        devices.sort_by(|a, b| b.rssi.cmp(&a.rssi));
        devices
    }

    /// Serialize all devices to JSON for BLE transmission
    /// Format: {"seq":123,"count":5,"devices":[...]}
    fn to_json(&self) -> String {
        let devices = self.get_sorted();
        let device_json: Vec<String> = devices.iter().take(10).map(|d| d.to_json()).collect();
        format!(
            r#"{{"seq":{},"count":{},"devices":[{}]}}"#,
            self.sequence,
            devices.len(),
            device_json.join(",")
        )
    }

    fn device_count(&self) -> usize {
        self.devices.len()
    }

    fn get_sequence(&self) -> u32 {
        self.sequence
    }
}

fn main() {
    // Initialize the ESP-IDF runtime
    esp_idf_svc::sys::link_patches();

    // Initialize logging
    EspLogger::initialize_default();
    info!("Starting BLE Scanner Server for M5StickC PLUS2");

    // Create shared device tracker
    let tracker = Arc::new(Mutex::new(DeviceTracker::new()));

    // Run the main async logic
    block_on(run_ble_scanner_server(tracker));
}

async fn run_ble_scanner_server(tracker: Arc<Mutex<DeviceTracker>>) {
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

    // Create our custom service
    let service = server.create_service(uuid128!(SERVICE_UUID));

    // Create the scan results characteristic with READ and NOTIFY
    let scan_characteristic = service.lock().create_characteristic(
        uuid128!(SCAN_RESULTS_UUID),
        NimbleProperties::READ | NimbleProperties::NOTIFY,
    );

    // Set initial value
    scan_characteristic
        .lock()
        .set_value(b"{\"seq\":0,\"count\":0,\"devices\":[]}");

    // Track subscriptions
    let has_subscribers = Arc::new(Mutex::new(false));
    let has_subscribers_clone = has_subscribers.clone();

    scan_characteristic.lock().on_subscribe(move |_, _, sub| {
        let subscribed = !sub.is_empty();
        if let Ok(mut hs) = has_subscribers_clone.lock() {
            *hs = subscribed;
        }
        info!(
            "Subscription changed: {}",
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
        .add_service_uuid(uuid128!(SERVICE_UUID));

    advertising.lock().set_data(&mut ad_data).unwrap();

    // Start advertising
    advertising.lock().start().unwrap();
    info!("BLE Server started, advertising as '{}'", DEVICE_NAME);
    info!("Service UUID: {}", SERVICE_UUID);
    info!("Scan Results Characteristic UUID: {}", SCAN_RESULTS_UUID);

    // Create scan instance
    let mut ble_scan = BLEScan::new();

    // Configure passive scanning
    // Passive scan doesn't send scan requests - just listens for advertisements
    ble_scan
        .active_scan(false) // Passive scanning
        .interval(100) // Scan interval in 0.625ms units (100 = 62.5ms)
        .window(50); // Scan window in 0.625ms units (50 = 31.25ms)

    info!("Starting continuous BLE scanning...");

    let mut last_sequence: u32 = 0;

    // Main loop: scan -> update characteristic -> repeat
    loop {
        // Clone tracker for the scan callback
        let tracker_for_scan = tracker.clone();

        // Start a scan for SCAN_DURATION_MS
        match ble_scan
            .start(&ble_device, SCAN_DURATION_MS as i32, |device, data| {
                let name = data.name().map(|n| n.to_string()).unwrap_or_default();
                let address = format!("{}", device.addr());
                let rssi = device.rssi() as i32;

                if let Ok(mut t) = tracker_for_scan.lock() {
                    t.update(name, address, rssi);
                }
                None::<()>
            })
            .await
        {
            Ok(_) => {
                // Scan completed successfully
            }
            Err(e) => {
                warn!("Scan error: {:?}", e);
                FreeRtos::delay_ms(100);
                continue;
            }
        }

        // Prune devices not seen in the last 30 seconds
        if let Ok(mut t) = tracker.lock() {
            t.prune_old(Duration::from_secs(30));
        }

        // Check if we have new data to notify
        let (should_notify, json_data, device_count) = {
            if let Ok(t) = tracker.lock() {
                let current_seq = t.get_sequence();
                let changed = current_seq != last_sequence;
                last_sequence = current_seq;
                (changed, t.to_json(), t.device_count())
            } else {
                (false, String::new(), 0)
            }
        };

        // Update characteristic and notify subscribers
        if should_notify {
            let has_subs = has_subscribers.lock().map(|h| *h).unwrap_or(false);

            // Always update the value (for reads)
            scan_characteristic
                .lock()
                .set_value(json_data.as_bytes());

            // Notify if we have subscribers
            if has_subs {
                scan_characteristic.lock().notify();
                info!(
                    "Notified subscribers: {} devices ({} bytes)",
                    device_count,
                    json_data.len()
                );
            }
        }

        // Brief delay between scan cycles
        FreeRtos::delay_ms(SCAN_INTERVAL_MS);
    }
}