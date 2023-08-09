use chrono::{DateTime, Utc};

use crate::signature::Signature;

pub struct DiscoveryEvent {
    pub date_time: DateTime<Utc>,
    pub signature: Signature,
    pub rssi: i16,
}

impl DiscoveryEvent {
    pub fn new(date_time: DateTime<Utc>, signature: Signature, rssi: i16) -> DiscoveryEvent {
        DiscoveryEvent { date_time, signature, rssi }
    }
}