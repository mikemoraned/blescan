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

pub struct Scanner {
    scans: u16,
    state: HashMap<Signature, State>,
    adapter: Adapter
}

impl Scanner {
    pub async fn new() -> Result<Scanner, Box<dyn Error>> {
        let scans = 0;
        let state: HashMap<Signature, State> = HashMap::new();
        
        let manager = Manager::new().await?;
        let mut adapter_list = manager.adapters().await?;
        if adapter_list.is_empty() {
            eprintln!("No Bluetooth adapters found");
        }
        let adapter = adapter_list.pop().unwrap();
        Ok(Scanner {
            scans, state, adapter
        })
    }

    pub async fn scan(&mut self) -> Result<(), Box<dyn Error>> {
        self.scans += 1;        
        println!("Starting scan {} on {}...", self.scans, self.adapter.adapter_info().await?);
        self.adapter
            .start_scan(ScanFilter::default())
            .await
            .expect("Can't scan BLE adapter for connected devices...");
        time::sleep(Duration::from_secs(1)).await;
        let peripherals = self.adapter.peripherals().await?;
        if peripherals.is_empty() {
            eprintln!("->>> BLE peripheral devices were not found, sorry. Exiting...");
        } else {
            for peripheral in peripherals.iter() {
                let properties = peripheral.properties().await?.unwrap();
                if let Some(signature) = Signature::find(&properties) {
                    if let Some(rssi) = properties.rssi {
                        self.state.entry(signature)
                            .and_modify(|s: &mut State| s.update(rssi, self.scans))
                            .or_insert(State::new(rssi, self.scans));
                    }
                }
            }
        }
        self.adapter
            .stop_scan().await
            .expect("Can't stop scan");
        println!("Stopped scan {} on {}", self.scans, self.adapter.adapter_info().await?);
        println!("[{}] State:", self.scans);
        for (signature, state) in self.state.iter() {
            println!("{}: {:>4}, {:>4}, {:>5}, {:>5}", signature, state.rssi, state.velocity, state.scan, self.scans - state.scan);
        }

        Ok(())
    }
}

#[derive(Hash, Eq, PartialEq)]
struct Signature(String);

impl std::fmt::Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:>32}", self.0)
    }
}


impl Signature {
    fn find(properties: &PeripheralProperties) -> Option<Signature> {
        if let Some(local_name) = &properties.local_name {
            Some(Signature(local_name.clone()))
        } else if !&properties.manufacturer_data.is_empty() {
            let mut context = md5::Context::new();
            let mut manufacturer_ids: Vec<&u16> = properties.manufacturer_data.keys().collect();
            manufacturer_ids.sort();
            for manufacturer_id in manufacturer_ids {
                let arbitrary_data = properties.manufacturer_data[manufacturer_id].clone();
                context.consume(arbitrary_data);
            }
            let digest = context.compute();
            Some(Signature(format!("{:x}", digest)))
        }
        else {
            None
        }
    }
}

