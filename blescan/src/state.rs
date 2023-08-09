use std::collections::HashMap;

use crate::{signature::Signature, discover::DiscoveryEvent, snapshot::Snapshot, device_state::DeviceState};

pub struct State {
    state: HashMap<Signature, DeviceState>
}

impl Default for State {
    fn default() -> State {
        State { 
            state: HashMap::default()
        }
    }
}

impl State {
    pub fn snapshot(&self) -> Snapshot {
        let mut s : Vec<(Signature, DeviceState)> = self.state.clone().into_iter().collect();
        s.sort_by(|(a,_),(b,_)| a.partial_cmp(b).unwrap());
        Snapshot(s.into_iter().map(|(_,v)| v.clone()).collect())
    }

    pub fn discover(&mut self, events: Vec<DiscoveryEvent>) {
        for event in events {
            self.state.entry(event.signature.clone())
                .and_modify(|s: &mut DeviceState| s.update(&event))
                .or_insert(DeviceState::from_event(&event));
        }
    }
}

#[cfg(test)]
mod test {
    use chrono::{Utc, TimeZone};

    use crate::{signature::Signature, state::{DeviceState, Snapshot}};

    use super::{State, DiscoveryEvent};

    #[test]
    fn starting_state() {
        let state = State::default();
        assert_eq!(state.snapshot(), Snapshot(vec![]));
    }

    #[test]
    fn initial_discovery() {
        let mut state = State::default();
        let start = Utc.timestamp_opt(0, 0).unwrap();
        state.discover(
            vec![DiscoveryEvent::new(start, Signature::Named("Device 1".to_string()), -10)]
        );
        assert_eq!(state.snapshot(), 
            Snapshot(vec![DeviceState::new(start, Signature::Named("Device 1".to_string()), -10)])
        );
    }

    #[test]
    fn updated_state() {
        let mut state = State::default();
        let start = Utc.timestamp_opt(0, 0).unwrap();
        state.discover(
            vec![DiscoveryEvent::new(start, Signature::Named("Device 1".to_string()), -10)]
        );
        let later = Utc.timestamp_opt(1, 0).unwrap();
        state.discover(
            vec![DiscoveryEvent::new(later, Signature::Named("Device 1".to_string()), -20)]
        );
        assert_eq!(state.snapshot(), 
            Snapshot(vec![DeviceState::new(later, Signature::Named("Device 1".to_string()), -20)]));
    }
}