use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use tokio::time;

use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::{Manager, Adapter};

use crate::signature::Signature;

pub struct State {
    pub rssi: i16,
    pub scan: u16,
    pub velocity: i16
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
    pub state: HashMap<Signature, State>,
    scans: u16,
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
        // println!("Starting scan {} on {}...", self.scans, self.adapter.adapter_info().await?);
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
        // println!("Stopped scan {} on {}", self.scans, self.adapter.adapter_info().await?);
        

        Ok(())
    }
}

impl std::fmt::Display for Scanner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] Named:", self.scans)?;
        for (signature, state) in self.state.iter() {
            if let Signature::Named(_) = signature {
                self.fmt_row(signature, state, f)?;
            }
        }
        write!(f, "[{}] Anonymous:", self.scans)?;
        for (signature, state) in self.state.iter() {
            if let Signature::Anonymous(_) = signature {
                self.fmt_row(signature, state, f)?;
            }
        }
        write!(f, "")
    }
}

impl Scanner {
    fn fmt_row(&self, signature: &Signature, state: &State, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:>4}, {:>4}, {:>5}, {:>5}\n", signature, state.rssi, state.velocity, state.scan, self.scans - state.scan)
    }
}