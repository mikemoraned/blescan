use crate::{ signature::Signature, device_state::DeviceState};

#[derive(PartialEq, Debug)]
pub struct Snapshot(pub Vec<DeviceState>);

impl std::fmt::Display for Snapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Named:")?;
        for state in &self.0 {
            if let Signature::Named(_) = state.signature {
                writeln!(f, "{:>4}, {:>4}", state.signature, state.rssi)?;
            }
        }
        writeln!(f, "Anonymous:")?;
        for state in &self.0 {
            if let Signature::Anonymous(_) = state.signature {
                writeln!(f, "{:>4}, {:>4}", state.signature, state.rssi)?;
            }
        }
        write!(f, "")
    }
}

impl Snapshot {
    #[must_use] pub fn order_by_age_oldest_last(&self) -> Snapshot {
        let mut ordered_by_age : Vec<DeviceState> = self.0.clone();
        ordered_by_age.sort_by(
            |a, b| b.date_time.cmp(&a.date_time));
        Snapshot(ordered_by_age)
    }

    #[must_use] pub fn compared_to(&self, baseline: chrono::DateTime<chrono::Utc>) 
        -> Vec<(DeviceState, Comparison)> {
        self.0.iter().map(|d| {
            (
                d.clone(), 
                Comparison { relative_age: d.date_time - baseline }
            )
        }).collect()
    }
}

#[derive(PartialEq, Debug)]
pub struct Comparison {
    pub relative_age: chrono::Duration
}

#[cfg(test)]
mod test {
    use chrono::{Utc, TimeZone, Duration};

    use crate::{device_state::DeviceState, signature::Signature, snapshot::Comparison};

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

    #[test]
    fn device_with_comparison() {
        let snapshot = 
            Snapshot(vec![
                DeviceState::new(Utc.timestamp_opt(1, 0).unwrap(), Signature::Named("1".to_string()), -1),
                DeviceState::new(Utc.timestamp_opt(2, 0).unwrap(), Signature::Named("2".to_string()), -1),
                DeviceState::new(Utc.timestamp_opt(3, 0).unwrap(), Signature::Named("3".to_string()), -1),
            ]);
        let baseline = Utc.timestamp_opt(0, 0).unwrap();
        let expected_comparisons 
            = vec![
                (snapshot.0[0].clone(), Comparison { relative_age: Duration::seconds(1) }),
                (snapshot.0[1].clone(), Comparison { relative_age: Duration::seconds(2) }),
                (snapshot.0[2].clone(), Comparison { relative_age: Duration::seconds(3) }),
            ];
        let actual_comparisons 
            = snapshot.compared_to(baseline);
        assert_eq!(actual_comparisons, expected_comparisons);
    }
}