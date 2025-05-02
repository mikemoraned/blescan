use std::{error::Error, io::Write};

use async_trait::async_trait;
use gzp::ZWriter;

use crate::discover::DiscoveryEvent;

use super::EventSink;
pub struct JsonLinesEventSink<'a> {
    writer: Writer<'a>
}

pub enum Writer<'a> {
    PLAIN(Box<dyn Write + 'a>),
    COMPRESSED(Box<dyn ZWriter + 'a>)
}

impl<'a> JsonLinesEventSink<'a> {
    pub fn create_from_writer(writer: Box<dyn Write + 'a>) -> JsonLinesEventSink<'a> {
        JsonLinesEventSink {
            writer: Writer::PLAIN(writer)
        }
    }

    pub fn create_from_zwriter(writer: Box<dyn ZWriter + 'a>) -> JsonLinesEventSink<'a> {
        JsonLinesEventSink {
            writer: Writer::COMPRESSED(writer)
        }
    }
}

unsafe impl<'a> Send for JsonLinesEventSink<'a> {}

#[async_trait]
impl<'a> EventSink for JsonLinesEventSink<'a> {
    async fn save(&mut self, events: &[DiscoveryEvent]) -> Result<(), Box<dyn Error>> {
        let writer = &mut self.writer;
        match writer {
            Writer::PLAIN(w) => {
                for event in events {
                    serde_json::to_writer(&mut *w, event)?;
                    writeln!(w)?;
                }
                w.flush()?;
            },
            Writer::COMPRESSED(w) => {
                for event in events {
                    serde_json::to_writer(&mut *w, event)?;
                    writeln!(w)?;
                }
                w.flush()?;
            },
        }
        Ok(())
    }
    async fn close(mut self: Box<Self>) -> Result<(), Box<dyn Error>> {
        match self.writer {
            Writer::PLAIN(_) => Ok(()),
            Writer::COMPRESSED(ref mut w) => {
                w.flush()?;
                Ok(w.finish()?)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use chrono::{Utc, TimeZone};

    use crate::{discover::DiscoveryEvent, signature::Signature, history::EventSink};

    use super::JsonLinesEventSink;

    #[tokio::test]
    async fn sink_multiple_events() {
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
            let mut sink = JsonLinesEventSink::create_from_writer(Box::new(&mut buf));
            sink.save(&events).await.unwrap();
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