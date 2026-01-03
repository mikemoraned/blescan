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
}

pub fn find_signature(peripheral: &Peripheral) -> Option<Signature> {
    if let Some(local_name) = &peripheral.local_name {
        Some(Signature::Named(local_name.clone()))
    } else if !peripheral.manufacturer_data.is_empty() {
        let mut context = md5::Context::new();
        let mut manufacturer_ids: Vec<&u16> = peripheral.manufacturer_data.keys().collect();
        manufacturer_ids.sort();
        for manufacturer_id in manufacturer_ids {
            let arbitrary_data = peripheral.manufacturer_data[manufacturer_id].clone();
            context.consume(arbitrary_data);
        }
        let digest = context.compute();
        Some(Signature::Anonymous(format!("{digest:x}")))
    } else {
        None
    }
}
