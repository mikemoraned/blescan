//! Shared definitions for BLE Mote GATT services and characteristics

pub mod device_tracker;

/// GATT Service UUID for the Mote service
pub const MOTE_SERVICE_UUID: &str = "e595b646-b900-472f-a207-288266f05314";

/// GATT Characteristic UUID for discovered devices list
pub const MOTE_DISCOVERED_DEVICES_CHARACTERISTIC_UUID: &str = "7182a610-1d80-4079-8ab8-d069d88800b1";
