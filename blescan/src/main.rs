use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use tokio::time;

use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let manager = Manager::new().await?;
    let adapter_list = manager.adapters().await?;
    if adapter_list.is_empty() {
        eprintln!("No Bluetooth adapters found");
    }
    let adapter = &adapter_list[0];

    let mut state = HashMap::new();
    let mut scans = 0;
    loop {
        scans += 1;        
        println!("Starting scan {} on {}...", scans, adapter.adapter_info().await?);
        adapter
            .start_scan(ScanFilter::default())
            .await
            .expect("Can't scan BLE adapter for connected devices...");
        time::sleep(Duration::from_secs(1)).await;
        let peripherals = adapter.peripherals().await?;
        if peripherals.is_empty() {
            eprintln!("->>> BLE peripheral devices were not found, sorry. Exiting...");
        } else {
            for peripheral in peripherals.iter() {
                let properties = peripheral.properties().await?.unwrap();
                if let Some(local_name) = properties.local_name {
                    if let Some(rssi) = properties.rssi {
                        let signature = local_name.clone();
                        state.entry(signature).and_modify(|r| *r = rssi).or_insert(rssi);
                    }
                }
            }
        }
        adapter
            .stop_scan().await
            .expect("Can't stop scan");
        println!("Stopped scan {} on {}", scans, adapter.adapter_info().await?);
        println!("[{}] State:", scans);
        for (signature, rssi) in state.iter() {
            println!("\t{}: {}", signature, rssi);
        }
    }
}