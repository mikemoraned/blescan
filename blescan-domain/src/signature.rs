use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(TS, Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum Signature {
    Named(String),
    Anonymous(String),
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
            Anonymous(d) => format!("Anonymous:{d}"),
        }
    }
}

impl std::fmt::Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Signature::{Anonymous, Named};
        match self {
            Named(n) => write!(f, "{n:>32}"),
            Anonymous(d) => write!(f, "{d}"),
        }
    }
}
