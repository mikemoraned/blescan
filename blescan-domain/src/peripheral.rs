use std::collections::HashMap;
use crate::signature::Signature;

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
        if !self.manufacturer_data.is_empty() {
            // Collect and sort manufacturer data for consistent hashing
            let mut context = md5::Context::new();
            let mut manufacturer_ids: Vec<&u16> = self.manufacturer_data.keys().collect();
            manufacturer_ids.sort();
            for manufacturer_id in manufacturer_ids {
                context.consume(manufacturer_id.to_le_bytes());
                let arbitrary_data = self.manufacturer_data[manufacturer_id].clone();
                context.consume(arbitrary_data);
            }
            let digest = context.compute();
            let id = format!("{digest:x}")[..8].to_string();

            if let Some(local_name) = &self.local_name {
                Some(Signature::Named { name: local_name.clone(), id })
            } else {
                Some(Signature::Anonymous { id })
            }
        } else {
            None
        }
    }
}
