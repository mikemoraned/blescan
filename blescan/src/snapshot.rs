use crate::{state::DeviceState, signature::Signature};

#[derive(PartialEq, Debug)]
pub struct Snapshot(pub Vec<DeviceState>);

impl std::fmt::Display for Snapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Named:")?;
        for state in self.0.iter() {
            if let Signature::Named(_) = state.signature {
                self.fmt_row(state, f)?;
            }
        }
        writeln!(f, "Anonymous:")?;
        for state in self.0.iter() {
            if let Signature::Anonymous(_) = state.signature {
                self.fmt_row(state, f)?;
            }
        }
        write!(f, "")
    }
}

impl Snapshot {
    fn fmt_row(&self, state: &DeviceState, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:>4}, {:>4}", state.signature, state.rssi)
    }
}