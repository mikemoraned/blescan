use esp32_nimble::{uuid128, BLEAdvertisementData, BLEDevice, NimbleProperties};
use esp_idf_hal::delay::FreeRtos;
use esp_idf_sys as _;
use log::info;

fn main() {
    // Initialize ESP-IDF
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Starting M5StickC PLUS2 BLE Server...");

    // Take ownership of the BLE device (initializes NimBLE stack)
    let ble_device = BLEDevice::take();

    // Get the advertising handle
    let ble_advertiser = ble_device.get_advertising();

    // Get the server handle
    let server = ble_device.get_server();

    // Configure connection callbacks
    server.on_connect(|server, client_desc| {
        info!("Client connected! Handle: {}", client_desc.conn_handle());

        // Update connection parameters for better performance
        // min_interval, max_interval, latency, timeout (in units of 1.25ms, 1.25ms, intervals, 10ms)
        if let Err(e) = server.update_conn_params(client_desc.conn_handle(), 24, 48, 0, 60) {
            info!("Failed to update connection params: {:?}", e);
        }
    });

    server.on_disconnect(|_desc, reason| {
        info!("Client disconnected! Reason: {:?}", reason);
        info!("Back to advertising...");
    });

    // Create a custom service with a UUID
    // You can generate your own UUID at https://www.uuidgenerator.net/
    let my_service = server.create_service(uuid128!("12345678-1234-1234-1234-123456789abc"));

    // Create a characteristic within the service
    let my_characteristic = my_service.lock().create_characteristic(
        uuid128!("87654321-4321-4321-4321-cba987654321"),
        NimbleProperties::READ | NimbleProperties::NOTIFY,
    );

    // Set an initial value for the characteristic
    my_characteristic.lock().set_value(b"Hello from M5StickC!");

    // Configure what we advertise
    ble_advertiser
        .lock()
        .set_data(
            BLEAdvertisementData::new()
                .name("M5StickC-Hello")
                .add_service_uuid(uuid128!("12345678-1234-1234-1234-123456789abc")),
        )
        .unwrap();

    // Start advertising
    ble_advertiser.lock().start().unwrap();
    info!("BLE Server is now advertising as 'M5StickC-Hello'");
    info!("Service UUID: 12345678-1234-1234-1234-123456789abc");
    info!("Characteristic UUID: 87654321-4321-4321-4321-cba987654321");

    // Main loop - update the characteristic value periodically
    let mut counter: u32 = 0;
    loop {
        // Update the characteristic with a counter value
        let message = format!("Hello #{}", counter);
        my_characteristic.lock().set_value(message.as_bytes());

        // If there are connected clients, notify them of the change
        my_characteristic.lock().notify();

        info!("Updated value: {}", message);
        counter = counter.wrapping_add(1);

        // Wait 2 seconds before next update
        FreeRtos::delay_ms(2000);
    }
}
