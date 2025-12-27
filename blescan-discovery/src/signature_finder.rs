use btleplug::api::PeripheralProperties;
use blescan_domain::signature::Signature;

pub fn find_signature(properties: &PeripheralProperties) -> Option<Signature> {
    if let Some(local_name) = &properties.local_name {
        Some(Signature::Named(local_name.clone()))
    } else if !&properties.manufacturer_data.is_empty() {
        let mut context = md5::Context::new();
        let mut manufacturer_ids: Vec<&u16> = properties.manufacturer_data.keys().collect();
        manufacturer_ids.sort();
        for manufacturer_id in manufacturer_ids {
            let arbitrary_data = properties.manufacturer_data[manufacturer_id].clone();
            context.consume(arbitrary_data);
        }
        let digest = context.compute();
        Some(Signature::Anonymous(format!("{digest:x}")))
    } else {
        None
    }
}
