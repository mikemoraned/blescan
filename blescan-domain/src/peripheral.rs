use std::collections::HashMap;
use crate::signature::Signature;
use xxhash_rust::xxh3::xxh3_64;

pub struct Peripheral {
    pub local_name: Option<String>,
    pub manufacturer_data: HashMap<u16, Vec<u8>>,
}

impl Peripheral {
    pub fn new(local_name: Option<String>, manufacturer_data: HashMap<u16, Vec<u8>>) -> Self {
        Self {
            local_name,
            manufacturer_data,
        }
    }

    pub fn try_into_signature(&self) -> Option<Signature> {
        if let Some(local_name) = &self.local_name {
            Some(Signature::Named(local_name.clone()))
        } else if !self.manufacturer_data.is_empty() {
            // Collect and sort manufacturer data for consistent hashing
            let mut manufacturer_ids: Vec<&u16> = self.manufacturer_data.keys().collect();
            manufacturer_ids.sort();

            // Concatenate all manufacturer data in sorted order
            let mut data_to_hash = Vec::new();
            for manufacturer_id in manufacturer_ids {
                data_to_hash.extend_from_slice(&manufacturer_id.to_le_bytes());
                data_to_hash.extend_from_slice(&self.manufacturer_data[manufacturer_id]);
            }

            // Hash with xxh3 and encode as base62
            let hash = xxh3_64(&data_to_hash);
            let encoded = base62::encode(hash);

            Some(Signature::Anonymous(encoded))
        } else {
            None
        }
    }
}
