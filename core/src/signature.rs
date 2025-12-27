use btleplug::api::PeripheralProperties;
use serde::{Serialize, Deserialize};

#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum Signature {
    Named(String),
    Anonymous(String)
}

impl Ord for Signature {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_s = self.normalised_string();
        let other_s = other.normalised_string();
        self_s.cmp(&other_s)
    }
}

impl PartialOrd for Signature {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Signature {
    fn normalised_string(&self) -> String {
        use Signature::{Anonymous, Named};
        match self {
            Named(n) => format!("Named:{n}"),
            Anonymous(d) => format!("Anonymous:{d}")
        }
    }
}

impl std::fmt::Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Signature::{Anonymous, Named};
        match self {
            Named(n) => write!(f, "{n:>32}")?,
            Anonymous(d) => write!(f, "{d}")?
        }
        write!(f, "")
    }
}

impl Signature {
    #[must_use] pub fn find(properties: &PeripheralProperties) -> Option<Signature> {
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
        }
        else {
            None
        }
    }
}

