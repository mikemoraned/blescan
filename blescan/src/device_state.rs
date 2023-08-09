use crate::{signature::Signature, discover::DiscoveryEvent};

#[derive(PartialEq, Debug, Clone)]
pub struct DeviceState {
    pub signature: Signature,
    pub rssi: i16,
}

impl DeviceState {
    pub fn new(signature: Signature, rssi: i16) -> DeviceState {
        DeviceState { signature, rssi }
    }

    pub fn from_event(event: &DiscoveryEvent) -> DeviceState {
        DeviceState { 
            signature: event.signature.clone(), 
            rssi: event.rssi
        }
    }

    pub fn update(&mut self, rssi: i16) {
        self.rssi = rssi;
    }
}