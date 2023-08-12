use std::{path::{Path, PathBuf}, error::Error, io::{Write, BufWriter}, fs::OpenOptions, ffi::OsStr};

use crate::discover::DiscoveryEvent;

#[derive(PartialEq, Debug)]
pub enum EventSinkFormat {
    JSONL(PathBuf)
}

impl EventSinkFormat {
    pub fn create_from_file<P>(path_arg: P) -> Result<EventSinkFormat, Box<dyn Error>> 
        where P: AsRef<Path>
    {
        let path = path_arg.as_ref().clone();
        if Some(OsStr::new("jsonl")) == path.extension() {
            Ok(EventSinkFormat::JSONL(path.to_path_buf()))
        }
        else {
            Err(format!("unknown type: {}", path.display()).into())
        }
    }

    pub fn to_sink(&self) -> Result<impl EventSink, Box<dyn Error>>  {
        match self {
            EventSinkFormat::JSONL(path_buf) => {
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path_buf)?;
                let buf_writer = BufWriter::new(file);
                Ok(JsonLinesEventSink::create_from_writer(buf_writer))
            }
        }
    }
}


pub trait EventSink {
    fn save(&mut self, events: &[DiscoveryEvent]) -> Result<(), Box<dyn Error>>;
}

pub struct JsonLinesEventSink<'a> {
    writer: Box<dyn std::io::Write + 'a>
}

impl<'a> JsonLinesEventSink<'a> {
    pub fn create_from_file<P>(path_arg: P) -> Result<JsonLinesEventSink<'a>, Box<dyn Error>> 
        where P: AsRef<Path> + 'a
    {
        let path = path_arg.as_ref();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        let buf_writer = BufWriter::new(file);
        Ok(JsonLinesEventSink::create_from_writer(buf_writer))
    }
    
    pub fn create_from_writer(writer: impl Write + 'a) -> JsonLinesEventSink<'a> {
        JsonLinesEventSink {
            writer: Box::new(writer)
        }
    }
}

impl<'a> EventSink for JsonLinesEventSink<'a> {
    fn save(&mut self, events: &[DiscoveryEvent]) -> Result<(), Box<dyn Error>> {
        for event in events {
            serde_json::to_writer(&mut self.writer, event)?;
            writeln!(&mut self.writer, "")?;
        }
        self.writer.flush()?;
        Ok(())
    }
}

pub struct NoopEventSink;

impl NoopEventSink {
    pub fn new() -> impl EventSink {
        NoopEventSink
    }
}

impl EventSink for NoopEventSink {
    fn save(&mut self, _: &[DiscoveryEvent]) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use chrono::{Utc, TimeZone};

    use crate::{discover::DiscoveryEvent, signature::Signature, history::EventSink};

    use super::{JsonLinesEventSink, EventSinkFormat};

    #[test]
    fn jsonl_format_matching() {
        let valid = "foop.jsonl";

        assert_eq!(EventSinkFormat::create_from_file(valid).unwrap(), EventSinkFormat::JSONL(valid.into()));        
    }

    #[test]
    fn jsonl_format_not_matching() {
        let invalid = vec!["foop.json", "farp", "feep.sqllite"];

        for i in invalid {
            assert!(EventSinkFormat::create_from_file(i).is_err());
        }
    }

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
            let mut sink = JsonLinesEventSink::create_from_writer(&mut buf);
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