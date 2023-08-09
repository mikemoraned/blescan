use crate::signature::Signature;

pub struct DiscoveryEvent {
    pub signature: Signature,
    pub rssi: i16,
}

impl DiscoveryEvent {
    pub fn new(signature: Signature, rssi: i16) -> DiscoveryEvent {
        DiscoveryEvent { signature, rssi }
    }
}