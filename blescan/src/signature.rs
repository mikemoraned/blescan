use btleplug::api::PeripheralProperties;


#[derive(Hash, Eq, PartialEq)]
pub enum Signature {
    Named(String),
    Anonymous(md5::Digest)
}

impl std::fmt::Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Signature::*;
        match self {
            Named(n) => write!(f, "{:>32}", n)?,
            Anonymous(d) => write!(f, "{:x}", d)?
        }
        write!(f, "")
    }
}

impl Signature {
    pub fn find(properties: &PeripheralProperties) -> Option<Signature> {
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
            Some(Signature::Anonymous(digest))
        }
        else {
            None
        }
    }
}

