pub mod noop;
pub mod sqllite;
use std::{
    error::Error,
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use sqlx::sqlite::SqlitePoolOptions;

use crate::{discover::DiscoveryEvent, history::sqllite::SQLLiteEventSink};

#[derive(PartialEq, Debug)]
#[allow(non_camel_case_types)]
pub enum EventSinkFormat {
    SQLITE(PathBuf),
}

impl EventSinkFormat {
    pub fn create_from_file<P>(path_arg: P) -> Result<EventSinkFormat, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let path = path_arg.as_ref();
        if Some(OsStr::new("sqlite")) == path.extension() {
            Ok(EventSinkFormat::SQLITE(path.to_path_buf()))
        } else {
            Err(format!("unknown type: {}", path.display()).into())
        }
    }

    pub async fn to_sink(&self) -> Result<Box<dyn EventSink>, Box<dyn Error>> {
        use EventSinkFormat::*;
        match self {
            SQLITE(path_buf) => {
                let url = format!("sqlite://{}?mode=rwc", path_buf.display());
                let pool = Arc::new(SqlitePoolOptions::new().connect(&url).await.unwrap());
                let sink = SQLLiteEventSink::create_from_pool(pool.clone()).await?;
                Ok(Box::new(sink))
            }
        }
    }
}

#[async_trait]
pub trait EventSink: Send {
    async fn save(&mut self, events: &[DiscoveryEvent]) -> Result<(), Box<dyn Error>>;
    async fn close(mut self: Box<Self>) -> Result<(), Box<dyn Error>>;
}

#[cfg(test)]
mod test {
    use super::EventSinkFormat;

    #[test]
    fn sqlite_format_matching() {
        let valid = "foop.sqlite";

        assert_eq!(
            EventSinkFormat::create_from_file(valid).unwrap(),
            EventSinkFormat::SQLITE(valid.into())
        );
    }

    #[test]
    fn format_not_matching() {
        let invalid = vec!["foop.json", "farp", "feep.txt"];

        for i in invalid {
            assert!(EventSinkFormat::create_from_file(i).is_err());
        }
    }
}
