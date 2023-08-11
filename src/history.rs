use std::{path::Path, error::Error};

use crate::discover::DiscoveryEvent;

pub struct EventSink;

impl EventSink {
    pub fn to_file(path: &Path) -> EventSink {
        EventSink {}
    }

    pub fn save(&self, events: &[DiscoveryEvent]) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}