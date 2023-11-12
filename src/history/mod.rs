pub mod sqllite;
pub mod noop;
pub mod jsonl;
use std::{path::{Path, PathBuf}, error::Error, io::BufWriter, fs::OpenOptions, ffi::OsStr, sync::Arc};

use async_trait::async_trait;
use gzp::Compression;
use sqlx::sqlite::SqlitePoolOptions;

use crate::{discover::DiscoveryEvent, history::sqllite::SQLLiteEventSink};

use self::jsonl::JsonLinesEventSink;

#[derive(PartialEq, Debug)]
#[allow(non_camel_case_types)]
pub enum EventSinkFormat {
    JSONL(PathBuf),
    JSONL_GZIP(PathBuf),
    SQLITE(PathBuf)
}

impl EventSinkFormat {
    pub fn create_from_file<P>(path_arg: P) -> Result<EventSinkFormat, Box<dyn Error>> 
        where P: AsRef<Path>
    {
        let path = path_arg.as_ref();
        if Some(OsStr::new("jsonl")) == path.extension() {
            Ok(EventSinkFormat::JSONL(path.to_path_buf()))
        }
        else if Some(OsStr::new("gz")) == path.extension() {
            if Some(OsStr::new("jsonl")) == Path::new(path.file_stem().unwrap()).extension() {
                Ok(EventSinkFormat::JSONL_GZIP(path.to_path_buf()))
            }
            else {
                Err(format!("unknown type: {}", path.display()).into())
            }
        }
        else if Some(OsStr::new("sqlite")) == path.extension() {
            Ok(EventSinkFormat::SQLITE(path.to_path_buf()))
        }
        else {
            Err(format!("unknown type: {}", path.display()).into())
        }
    }

    pub async fn to_sink(&self) -> Result<Box<dyn EventSink>, Box<dyn Error>>  {
        use EventSinkFormat::*;
        match self {
            JSONL(path_buf) => {
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path_buf)?;
                let buf_writer = BufWriter::new(file);
                Ok(Box::new(JsonLinesEventSink::create_from_writer(Box::new(buf_writer))))
            },
            JSONL_GZIP(path_buf) => {
                use gzp::{deflate::Gzip, ZBuilder};
                
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path_buf)?;
                let buf_writer = BufWriter::new(file);
                let compressed_writer = ZBuilder::<Gzip, _>::new()
                    .compression_level(Compression::best())
                    .from_writer(buf_writer);
                Ok(Box::new(JsonLinesEventSink::create_from_writer(Box::new(compressed_writer))))
            },
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
pub trait EventSink : Send {
    async fn save(&mut self, events: &[DiscoveryEvent]) -> Result<(), Box<dyn Error>>;
    async fn close(mut self: Box<Self>) -> Result<(), Box<dyn Error>>;
}

#[cfg(test)]
mod test {
    use super::EventSinkFormat;

    #[test]
    fn jsonl_format_matching() {
        let valid = "foop.jsonl";

        assert_eq!(EventSinkFormat::create_from_file(valid).unwrap(), EventSinkFormat::JSONL(valid.into()));        
    }

    #[test]
    fn jsonl_gzip_format_matching() {
        let valid = "foop.jsonl.gz";

        assert_eq!(EventSinkFormat::create_from_file(valid).unwrap(), EventSinkFormat::JSONL_GZIP(valid.into()));        
    }

    #[test]
    fn sqlite_format_matching() {
        let valid = "foop.sqlite";

        assert_eq!(EventSinkFormat::create_from_file(valid).unwrap(), EventSinkFormat::SQLITE(valid.into()));        
    }

    #[test]
    fn format_not_matching() {
        let invalid = vec!["foop.json", "farp", "feep.txt"];

        for i in invalid {
            assert!(EventSinkFormat::create_from_file(i).is_err());
        }
    }

}