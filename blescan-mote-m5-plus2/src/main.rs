//! BLE Mote GATT Server for M5StickC PLUS2
//!
//! This application exposes a BLE GATT service with a characteristic
//! that provides a simple "hello world" message.

use esp32_nimble::{uuid128, BLEAdvertisementData, BLEDevice, NimbleProperties};
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::task::block_on;
use esp_idf_svc::log::EspLogger;
use log::info;

/// Device name for BLE advertising
const DEVICE_NAME: &str = "blescan-mote";

fn main() {
    // Initialize the ESP-IDF runtime
    esp_idf_svc::sys::link_patches();

    // Initialize logging
    EspLogger::initialize_default();
    info!("Starting BLE Mote Server for M5StickC PLUS2");

    // Run the main async logic
    block_on(run_ble_mote_server());
}

async fn run_ble_mote_server() {
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

    // Create the hello world characteristic with READ and NOTIFY properties
    let hello_characteristic = service.lock().create_characteristic(
        uuid128!(blescan_mote::MOTE_HELLO_CHARACTERISTIC_UUID),
        NimbleProperties::READ | NimbleProperties::NOTIFY,
    );

    // Set initial value
    hello_characteristic
        .lock()
        .set_value(b"hello world");

    info!("Created hello world characteristic");

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
        "Hello Characteristic UUID: {}",
        blescan_mote::MOTE_HELLO_CHARACTERISTIC_UUID
    );

    // Main loop: periodically update the hello world message
    let mut counter: u32 = 0;
    loop {
        FreeRtos::delay_ms(5000);

        counter += 1;
        let message = format!("hello world {}", counter);
        hello_characteristic.lock().set_value(message.as_bytes());
        hello_characteristic.lock().notify();

        info!("Updated message: {}", message);
    }
}
