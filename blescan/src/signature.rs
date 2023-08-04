use btleplug::api::PeripheralProperties;


#[derive(Hash, Eq, PartialEq)]
pub struct Signature(String);

impl std::fmt::Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:>32}", self.0)
    }
}

impl Signature {
    pub fn find(properties: &PeripheralProperties) -> Option<Signature> {
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

