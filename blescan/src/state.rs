use std::collections::HashMap;

use crate::signature::Signature;

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
    pub fn snapshot(&self) -> Vec<DeviceState> {
        let mut s : Vec<(Signature, DeviceState)> = self.state.clone().into_iter().collect();
        s.sort_by(|(a,_),(b,_)| a.partial_cmp(b).unwrap());
        s.into_iter().map(|(_,v)| v.clone()).collect()
    }

    pub fn discover(&mut self, events: Vec<DiscoveryEvent>) {
        for event in events {
            self.state.entry(event.signature.clone())
                            .or_insert(DeviceState::new(event.signature.clone()));
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct DeviceState {
    pub signature: Signature
}

impl DeviceState {
    pub fn new(signature: Signature) -> DeviceState {
        DeviceState { signature }
    }
}

pub struct DiscoveryEvent {
    pub signature: Signature
}

impl DiscoveryEvent {
    pub fn new(signature: Signature) -> DiscoveryEvent {
        DiscoveryEvent { signature }
    }
}

#[cfg(test)]
mod test {
    use crate::{signature::Signature, state::DeviceState};

    use super::{State, DiscoveryEvent};

    #[test]
    fn starting_state() {
        let state = State::default();
        assert_eq!(state.snapshot(), vec![]);
    }

    #[test]
    fn initial_discovery() {
        let mut state = State::default();
        state.discover(vec![DiscoveryEvent::new(Signature::Named("Device 1".to_string()))]);
        assert_eq!(state.snapshot(), vec![DeviceState::new(Signature::Named("Device 1".to_string()))]);
    }

    #[test]
    fn updated_state() {
        let mut state = State::default();
        state.discover(vec![DiscoveryEvent::new(Signature::Named("Device 1".to_string()))]);
        assert_eq!(state.snapshot(), vec![DeviceState::new(Signature::Named("Device 1".to_string()))]);
    }
}