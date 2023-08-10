use std::{collections::HashMap, cmp::Ordering};

use crate::{ signature::Signature, device_state::DeviceState};

#[derive(PartialEq, Debug, Default, Clone)]
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
    #[must_use] pub fn order_by_age_and_volume(&self) -> Snapshot {
        let mut ordered : Vec<DeviceState> = self.0.clone();
        ordered.sort_by(
            |a, b| 
            if a.date_time == b.date_time {
                b.rssi.cmp(&a.rssi)
            }
            else {
                b.date_time.cmp(&a.date_time)
            }
        );
        Snapshot(ordered)
    }

    #[must_use] pub fn compared_to(&self, now: chrono::DateTime<chrono::Utc>, previous: Snapshot) 
        -> Vec<(DeviceState, Comparison)> {
        let previous_rssi: HashMap<Signature, i16> = previous.0.iter().map(|d| {
            (d.signature.clone(), d.rssi)
        }).collect();
        self.0.iter().map(|d| {
            let curr = &d.rssi;
            let rssi_comparison : RssiComparison  = match previous_rssi.get(&d.signature) {
                Some(prev) => {
                    match curr.cmp(prev) {
                        Ordering::Greater => RssiComparison::Louder,
                        Ordering::Equal => RssiComparison::Same,
                        Ordering::Less => RssiComparison::Quieter
                    }
                },
                None => RssiComparison::New
            };
            (
                d.clone(), 
                Comparison { 
                    relative_age: now - d.date_time,
                    rssi: rssi_comparison
                }
            )
        }).collect()
    }
}

#[derive(PartialEq, Debug)]
pub struct Comparison {
    pub relative_age: chrono::Duration,
    pub rssi: RssiComparison
}

#[derive(PartialEq, Debug, Clone)]
pub enum RssiComparison {
    Louder,
    Quieter,
    Same,
    New
}

#[cfg(test)]
mod test {
    use chrono::{Utc, TimeZone, Duration};

    use crate::{device_state::DeviceState, signature::Signature, snapshot::{Comparison, RssiComparison}};

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
        let actual_order = initial_order.order_by_age_and_volume();
        assert_eq!(actual_order, expected_order);
    }

    #[test]
    fn order_by_volume_when_same_age() {
        let initial_order = 
            Snapshot(vec![
                DeviceState::new(Utc.timestamp_opt(3, 0).unwrap(), Signature::Named("1".to_string()), -3),
                DeviceState::new(Utc.timestamp_opt(3, 0).unwrap(), Signature::Named("2".to_string()), -2),
                DeviceState::new(Utc.timestamp_opt(3, 0).unwrap(), Signature::Named("3".to_string()), -1)
            ]);
        let expected_order = 
            Snapshot(vec![
                DeviceState::new(Utc.timestamp_opt(3, 0).unwrap(), Signature::Named("3".to_string()), -1),
                DeviceState::new(Utc.timestamp_opt(3, 0).unwrap(), Signature::Named("2".to_string()), -2),
                DeviceState::new(Utc.timestamp_opt(3, 0).unwrap(), Signature::Named("1".to_string()), -3)
            ]);
        let actual_order = initial_order.order_by_age_and_volume();
        fn just_rssi(v: &Vec<DeviceState>) -> Vec<i16> {
            v.iter().map(|d|{ d.rssi.clone()}).collect()
        }
        assert_eq!(just_rssi(&actual_order.0), just_rssi(&expected_order.0));
        assert_eq!(actual_order, expected_order);
    }

    #[test]
    fn relative_age() {
        let snapshot = 
            Snapshot(vec![
                DeviceState::new(Utc.timestamp_opt(1, 0).unwrap(), Signature::Named("1".to_string()), -1),
                DeviceState::new(Utc.timestamp_opt(2, 0).unwrap(), Signature::Named("2".to_string()), -1),
                DeviceState::new(Utc.timestamp_opt(3, 0).unwrap(), Signature::Named("3".to_string()), -1),
            ]);
        let now = Utc.timestamp_opt(10, 0).unwrap();
        let expected_comparisons 
            = vec![
                (snapshot.0[0].clone(), Comparison { 
                    relative_age: Duration::seconds(9),
                    rssi: RssiComparison::New
                }),
                (snapshot.0[1].clone(), Comparison { 
                    relative_age: Duration::seconds(8),
                    rssi: RssiComparison::New 
                }),
                (snapshot.0[2].clone(), Comparison { 
                    relative_age: Duration::seconds(7),
                    rssi: RssiComparison::New
                }),
            ];
        let actual_comparisons 
            = snapshot.compared_to(now, Snapshot::default());
        assert_eq!(actual_comparisons, expected_comparisons);
    }

    #[test]
    fn relative_volume() {
        let previous = 
            Snapshot(vec![
                DeviceState::new(Utc.timestamp_opt(1, 0).unwrap(), Signature::Named("1".to_string()), -10),
                DeviceState::new(Utc.timestamp_opt(2, 0).unwrap(), Signature::Named("2".to_string()), -10),
                DeviceState::new(Utc.timestamp_opt(3, 0).unwrap(), Signature::Named("3".to_string()), -10),
            ]);
        let now = Utc.timestamp_opt(10, 0).unwrap();
        let current = 
            Snapshot(vec![
                DeviceState::new(Utc.timestamp_opt(1, 0).unwrap(), Signature::Named("1".to_string()), -5),
                DeviceState::new(Utc.timestamp_opt(2, 0).unwrap(), Signature::Named("2".to_string()), -15),
                DeviceState::new(Utc.timestamp_opt(3, 0).unwrap(), Signature::Named("3".to_string()), -10),
                DeviceState::new(Utc.timestamp_opt(4, 0).unwrap(), Signature::Named("4".to_string()), -10),
            ]);
        let expected_comparisons 
            = vec![
                (current.0[0].clone(), Comparison { 
                    relative_age: Duration::seconds(9),
                    rssi: RssiComparison::Louder 
                }),
                (current.0[1].clone(), Comparison { 
                    relative_age: Duration::seconds(8),
                    rssi: RssiComparison::Quieter
                }),
                (current.0[2].clone(), Comparison { 
                    relative_age: Duration::seconds(7),
                    rssi: RssiComparison::Same 
                }),
                (current.0[3].clone(), Comparison { 
                    relative_age: Duration::seconds(6),
                    rssi: RssiComparison::New 
                }),
            ];
        let actual_comparisons 
            = current.compared_to(now, previous);

        fn just_rssi(v: &Vec<(DeviceState, Comparison)>) -> Vec<RssiComparison> {
            v.iter().map(|(_,c)|{ c.rssi.clone()}).collect()
        }
        assert_eq!(just_rssi(&actual_comparisons), just_rssi(&expected_comparisons));
        assert_eq!(actual_comparisons, expected_comparisons);
    }
}