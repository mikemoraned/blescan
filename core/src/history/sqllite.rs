use std::{error::Error, path::Path, sync::Arc};

use crate::discover::DiscoveryEvent;
use async_trait::async_trait;
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};

use super::EventSink;

pub struct SQLLiteEventSink {
    pool: Arc<Pool<Sqlite>>,
}

impl SQLLiteEventSink {
    pub async fn create_from_file<P>(path_arg: P) -> Result<Box<dyn EventSink>, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let url = format!("sqlite://{}?mode=rwc", path_arg.as_ref().display());
        let pool = Arc::new(SqlitePoolOptions::new().connect(&url).await.unwrap());
        let sink = SQLLiteEventSink::create_from_pool(pool.clone()).await?;
        Ok(Box::new(sink))
    }

    pub async fn create_from_pool(
        pool: Arc<Pool<Sqlite>>,
    ) -> Result<SQLLiteEventSink, Box<dyn Error>> {
        sqlx::migrate!("../migrations").run(&*pool.clone()).await?;
        Ok(SQLLiteEventSink { pool: pool.clone() })
    }
}

unsafe impl Send for SQLLiteEventSink {}

#[async_trait]
impl EventSink for SQLLiteEventSink {
    async fn save(&mut self, events: &[DiscoveryEvent]) -> Result<(), Box<dyn Error>> {
        let p = self.pool.clone();
        let mut tx = p.begin().await?;

        for e in events {
            sqlx::query(
                "
            INSERT INTO discovery_events (date_time, signature, rssi) 
            VALUES (?, ?, ?)",
            )
            .bind(e.date_time)
            .bind(format!("{}", e.signature))
            .bind(e.rssi)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }
    async fn close(mut self: Box<Self>) -> Result<(), Box<dyn Error>> {
        self.pool.close().await;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use chrono::{DateTime, TimeZone, Utc};
    use sqlx::{
        Row,
        sqlite::{SqlitePoolOptions, SqliteRow},
    };

    use crate::{discover::DiscoveryEvent, history::EventSink, signature::Signature};

    use super::SQLLiteEventSink;

    #[tokio::test]
    async fn sink_multiple_events() {
        let events = &vec![
            DiscoveryEvent::new(
                Utc.timestamp_opt(1, 0).unwrap(),
                Signature::Named("Device 1".to_string()),
                -20,
            ),
            DiscoveryEvent::new(
                Utc.timestamp_opt(2, 0).unwrap(),
                Signature::Anonymous("503eb25838435ebb288f3b657b9f9031".to_string()),
                -30,
            ),
        ];

        let pool = Arc::new(
            SqlitePoolOptions::new()
                .connect("sqlite::memory:")
                .await
                .unwrap(),
        );
        let mut sink = SQLLiteEventSink::create_from_pool(pool.clone())
            .await
            .unwrap();
        sink.save(&events).await.unwrap();
        let rows = sqlx::query("SELECT * FROM discovery_events;")
            .fetch_all(&*pool.clone())
            .await
            .unwrap();
        assert!(!rows.is_empty());
        assert_row_eq(&rows.get(0).unwrap(), &events[0]);
        assert_row_eq(&rows.get(1).unwrap(), &events[1]);
    }

    fn assert_row_eq(actual: &SqliteRow, expected: &DiscoveryEvent) {
        let actual_date_time: DateTime<Utc> = actual.get(0);
        assert_eq!(actual_date_time, expected.date_time);
        let actual_signature: String = actual.get(1);
        assert_eq!(actual_signature, format!("{}", expected.signature));
        let actual_rssi: i16 = actual.get(2);
        assert_eq!(actual_rssi, expected.rssi);
    }
}
