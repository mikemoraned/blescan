use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use tokio::time;

use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter, PeripheralProperties};
use btleplug::platform::{Manager, Adapter};

struct State {
    rssi: i16,
    scan: u16,
    velocity: i16
}

impl State {
    fn new(rssi: i16, scan: u16) -> State {
        State { rssi, scan, velocity: 0 }
    }

    fn update(&mut self, rssi: i16, scan: u16) {
        let velocity = rssi - self.rssi;
        self.rssi = rssi;
        self.scan = scan;
        self.velocity = velocity;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let adapter = get_adapter().await?;

    let mut state: HashMap<String, State> = HashMap::new();
    let mut scans = 0;
    loop {
        scan(&mut scans, &mut state, &adapter).await?;
    }
}

async fn get_adapter() -> Result<Adapter, Box<dyn Error>> {
    let manager = Manager::new().await?;
    let mut adapter_list = manager.adapters().await?;
    if adapter_list.is_empty() {
        eprintln!("No Bluetooth adapters found");
    }
    Ok(adapter_list.pop().unwrap())
}

async fn scan(scans: &mut u16, state: &mut HashMap<String, State>, adapter: &Adapter) -> Result<(), Box<dyn Error>> {
    *scans += 1;        
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
            if let Some(signature) = find_signature(&properties) {
                if let Some(rssi) = properties.rssi {
                    state.entry(signature)
                        .and_modify(|s: &mut State| s.update(rssi, *scans))
                        .or_insert(State::new(rssi, *scans));
                }
            }
        }
    }
    adapter
        .stop_scan().await
        .expect("Can't stop scan");
    println!("Stopped scan {} on {}", scans, adapter.adapter_info().await?);
    println!("[{}] State:", scans);
    for (signature, state) in state.iter() {
        println!("{:>32}: {:>4}, {:>4}, {:>5}, {:>5}", signature, state.rssi, state.velocity, state.scan, *scans - state.scan);
    }

    Ok(())
}

fn find_signature(properties: &PeripheralProperties) -> Option<String> {
    if let Some(local_name) = &properties.local_name {
        Some(local_name.clone())
    } else if !&properties.manufacturer_data.is_empty() {
        let mut context = md5::Context::new();
        let mut manufacturer_ids: Vec<&u16> = properties.manufacturer_data.keys().collect();
        manufacturer_ids.sort();
        for manufacturer_id in manufacturer_ids {
            let arbitrary_data = properties.manufacturer_data[manufacturer_id].clone();
            context.consume(arbitrary_data);
        }
        let digest = context.compute();
        Some(format!("{:x}", digest))
    }
    else {
        None
    }
}