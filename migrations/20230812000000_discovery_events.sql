CREATE TABLE IF NOT EXISTS discovery_events
(
    date_time DATETIME            NOT NULL,
    signature TEXT                NOT NULL,
    rssi      INTEGER             NOT NULL
);