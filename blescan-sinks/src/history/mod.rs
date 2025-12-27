pub mod noop;
pub mod sqllite;
use std::error::Error;

use async_trait::async_trait;

use blescan_domain::discover::DiscoveryEvent;

#[async_trait]
pub trait EventSink: Send {
    async fn save(&mut self, events: &[DiscoveryEvent]) -> Result<(), Box<dyn Error>>;
    async fn close(mut self: Box<Self>) -> Result<(), Box<dyn Error>>;
}
