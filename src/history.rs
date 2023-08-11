use std::{path::Path, error::Error, io::Write, fs::OpenOptions};

use crate::discover::DiscoveryEvent;

pub struct EventSink(Box<dyn Write>);

impl EventSink {
    pub fn to_file<P>(path_arg: P) -> Result<EventSink, Box<dyn Error>> 
        where P: AsRef<Path>
    {
        let path = path_arg.as_ref();
        let w = OpenOptions::new()
            .append(true)
            .open(path)?;
        Ok(EventSink(Box::new(w)))
    }

    // fn to_writer(w: dyn Write) -> EventSink {
    //     EventSink(Box::new(w))
    // }

    pub fn save(&self, _events: &[DiscoveryEvent]) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn sink_single_event() {

    }
}