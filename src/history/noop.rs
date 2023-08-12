use std::error::Error;

use async_trait::async_trait;

use crate::discover::DiscoveryEvent;

use super::EventSink;

#[derive(Default)]
pub struct NoopEventSink;

#[async_trait]
impl EventSink for NoopEventSink {
    async fn save(&mut self, _: &[DiscoveryEvent]) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
