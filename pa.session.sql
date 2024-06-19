SELECT 
  host,
  toStartOfMinute(first_hit) AS minute,
  uniq(session_id) as visits,
  sum(hits) AS total_hits,
  avg(duration) AS avg_session_length,
  100 * countIf(hits = 1) / count() AS bounce_rate
FROM sessions
GROUP BY minute, host
ORDER BY minute ASC

select * FROM sessions

SELECT 
      session_id,
      minSimpleState(timestamp) AS first_hit,
      maxSimpleState(timestamp) AS latest_hit,
      maxSimpleState(timestamp) - minSimpleState(timestamp) AS duration,
      countState() AS hits,
      anyState(hostname) AS host,
      groupArray(href) AS user_journey,
      anyLastSimpleState(href) AS last_page 
    FROM PageViewProcessed
    WHERE hostname != 'development' 
    GROUP BY session_id
