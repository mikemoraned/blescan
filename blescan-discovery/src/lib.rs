pub mod local;

use std::error::Error;

use blescan_domain::discover::DiscoveryEvent;

#[allow(async_fn_in_trait)]
pub trait Scanner {
    async fn scan(&mut self) -> Result<Vec<DiscoveryEvent>, Box<dyn Error>>;
}
