use std::collections::HashMap;

use crate::{signature::Signature, discover::DiscoveryEvent, snapshot::Snapshot};

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
                .and_modify(|s: &mut DeviceState| s.update(event.rssi))
                .or_insert(DeviceState::from_event(&event));
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct DeviceState {
    pub signature: Signature,
    pub rssi: i16,
}

impl DeviceState {
    pub fn new(signature: Signature, rssi: i16) -> DeviceState {
        DeviceState { signature, rssi }
    }

    pub fn from_event(event: &DiscoveryEvent) -> DeviceState {
        DeviceState { 
            signature: event.signature.clone(), 
            rssi: event.rssi
        }
    }

    pub fn update(&mut self, rssi: i16) {
        self.rssi = rssi;
    }
}


#[cfg(test)]
mod test {
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
        state.discover(
            vec![DiscoveryEvent::new(Signature::Named("Device 1".to_string()), -10)]
        );
        assert_eq!(state.snapshot(), 
            Snapshot(vec![DeviceState::new(Signature::Named("Device 1".to_string()), -10)])
        );
    }

    #[test]
    fn updated_state() {
        let mut state = State::default();
        state.discover(
            vec![DiscoveryEvent::new(Signature::Named("Device 1".to_string()), -10)]
        );
        state.discover(
            vec![DiscoveryEvent::new(Signature::Named("Device 1".to_string()), -20)]
        );
        assert_eq!(state.snapshot(), 
            Snapshot(vec![DeviceState::new(Signature::Named("Device 1".to_string()), -20)]));
    }
}