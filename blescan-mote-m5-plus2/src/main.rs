//! BLE Mote GATT Server for M5StickC PLUS2
//!
//! This application:
//! 1. Passively scans for BLE devices in the environment
//! 2. Exposes the scan results via a BLE GATT service with notifications
//! 3. Provides a hello world characteristic for testing

mod ble_scanner;
mod server;

use esp_idf_hal::task::block_on;
use esp_idf_svc::log::EspLogger;
use log::info;

fn main() {
    // Initialize the ESP-IDF runtime
    esp_idf_svc::sys::link_patches();

    // Initialize logging
    EspLogger::initialize_default();
    info!("Starting BLE Mote Server for M5StickC PLUS2");

    // Run the main async logic
    block_on(server::run_ble_mote_server());
}
