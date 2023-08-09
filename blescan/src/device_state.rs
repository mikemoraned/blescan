use chrono::{DateTime, Utc};

use crate::{signature::Signature, discover::DiscoveryEvent};

#[derive(PartialEq, Debug, Clone)]
pub struct DeviceState {
    pub date_time: DateTime<Utc>,
    pub signature: Signature,
    pub rssi: i16,
}

impl DeviceState {
    pub fn new(date_time: DateTime<Utc>, signature: Signature, rssi: i16) -> DeviceState {
        DeviceState { date_time, signature, rssi }
    }

    pub fn from_event(event: &DiscoveryEvent) -> DeviceState {
        DeviceState {
            date_time: event.date_time,
            signature: event.signature.clone(), 
            rssi: event.rssi
        }
    }

    pub fn update(&mut self, event: &DiscoveryEvent) {
        self.date_time = event.date_time;
        self.rssi = event.rssi;
    }
}