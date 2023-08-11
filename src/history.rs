use std::{path::Path, error::Error, io::{Write, BufWriter}, fs::OpenOptions};

use crate::discover::DiscoveryEvent;

pub struct EventSink<'a> {
    writer: Box<dyn std::io::Write + 'a>
}

impl<'a> EventSink<'a> {
    pub fn create_from_file<P>(path_arg: P) -> Result<EventSink<'a>, Box<dyn Error>> 
        where P: AsRef<Path> + 'a
    {
        let path = path_arg.as_ref();
        let file = OpenOptions::new()
            .append(true)
            .open(path)?;
        let buf_writer = BufWriter::new(file);
        Ok(EventSink::create_from_writer(buf_writer))
    }
    
    pub fn create_from_writer(writer: impl Write + 'a) -> EventSink<'a> {
        EventSink {
            writer: Box::new(writer)
        }
    }

    pub fn save(&mut self, events: &[DiscoveryEvent]) -> Result<(), Box<dyn Error>> {
        for event in events {
            serde_json::to_writer(&mut self.writer, event)?;
            writeln!(&mut self.writer, "")?;
        }
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
    fn sink_multiple_events() {
        let events = &vec![
            DiscoveryEvent::new(
                Utc.timestamp_opt(1, 0).unwrap(), 
                Signature::Named("Device 1".to_string()), 
                -20),
            DiscoveryEvent::new(
                Utc.timestamp_opt(2, 0).unwrap(), 
                Signature::Anonymous("503eb25838435ebb288f3b657b9f9031".to_string()), 
                -30)
        ];
        let mut buf = Cursor::new(Vec::new());
        {
            let mut sink = EventSink::create_from_writer(&mut buf);
            sink.save(&events).unwrap();
        }

        assert_eq!(buf.get_ref().is_empty(), false);
        let expected = concat!(
            "{\"date_time\":\"1970-01-01T00:00:01Z\",\"signature\":{\"Named\":\"Device 1\"},\"rssi\":-20}\n",
            "{\"date_time\":\"1970-01-01T00:00:02Z\",\"signature\":{\"Anonymous\":\"503eb25838435ebb288f3b657b9f9031\"},\"rssi\":-30}\n"
        );
        let actual = String::from_utf8(buf.get_ref().to_vec()).unwrap();
        assert_eq!(actual, expected);
    }
}