use std::{path::Path, error::Error, io::Write, fs::OpenOptions};

use crate::discover::DiscoveryEvent;

pub struct EventSink<W> {
    writer: W
}

impl <W> EventSink<W> 
where
    W: Write
{
    // pub fn to_file<P>(path_arg: P) -> Result<EventSink<W>, Box<dyn Error>> 
    //     where P: AsRef<Path>
    // {
    //     let path = path_arg.as_ref();
    //     let writer: W = OpenOptions::new()
    //         .append(true)
    //         .open(path)?;
    //     Ok(EventSink { writer })
    // }

    fn to_writer(writer: W) -> EventSink<W> {
        EventSink {
            writer
        }
    }

    pub fn save(&self, _events: &[DiscoveryEvent]) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use chrono::{Utc, TimeZone};

    use crate::{discover::DiscoveryEvent, signature::Signature};

    use super::EventSink;


    #[test]
    fn sink_single_event() {
        let events = &vec![
            DiscoveryEvent::new(Utc.timestamp_opt(1, 0).unwrap(), Signature::Named("Device 1".to_string()), -20)
        ];
        let mut buf = Cursor::new(Vec::new());
        let sink = EventSink::to_writer(&mut buf);

        sink.save(&events).unwrap();

        assert_eq!(buf.get_ref().is_empty(), false);
    }
}