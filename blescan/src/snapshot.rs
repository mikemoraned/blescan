use crate::{ signature::Signature, device_state::DeviceState};

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

    pub fn order_by_age_oldest_last(&self) -> Snapshot {
        let mut ordered_by_age : Vec<DeviceState> = self.0.clone();
        ordered_by_age.sort_by(
            |a, b| b.date_time.partial_cmp(&a.date_time).unwrap());
        Snapshot(ordered_by_age)
    }
}

#[cfg(test)]
mod test {
    use chrono::{Utc, TimeZone};

    use crate::{device_state::DeviceState, signature::Signature};

    use super::Snapshot;

    #[test]
    fn order_by_age_oldest_last() {
        let initial_order = 
            Snapshot(vec![
                DeviceState::new(Utc.timestamp_opt(1, 0).unwrap(), Signature::Named("1".to_string()), -1),
                DeviceState::new(Utc.timestamp_opt(2, 0).unwrap(), Signature::Named("2".to_string()), -1),
                DeviceState::new(Utc.timestamp_opt(3, 0).unwrap(), Signature::Named("3".to_string()), -1)
            ]);
        let expected_order = 
            Snapshot(vec![
                DeviceState::new(Utc.timestamp_opt(3, 0).unwrap(), Signature::Named("3".to_string()), -1),
                DeviceState::new(Utc.timestamp_opt(2, 0).unwrap(), Signature::Named("2".to_string()), -1),
                DeviceState::new(Utc.timestamp_opt(1, 0).unwrap(), Signature::Named("1".to_string()), -1),
            ]);
        let actual_order = initial_order.order_by_age_oldest_last();
        assert_eq!(actual_order, expected_order);
    }
}