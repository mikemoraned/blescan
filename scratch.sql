SELECT *
FROM discovery_events
ORDER BY date_time DESC
LIMIT 10

SELECT signature, group_concat(rssi ORDER BY date_time DESC)
FROM discovery_events
GROUP BY signature 

WITH recent_events AS (
  SELECT *, strftime('%s', 'now') - strftime('%s', date_time) AS age_in_seconds
  FROM discovery_events
  WHERE age_in_seconds < 60
)
SELECT signature, group_concat(rssi ORDER BY date_time DESC)
FROM recent_events
GROUP BY signature 

