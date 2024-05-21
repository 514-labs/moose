export default {
  select: `
  SELECT 
  host,
  toStartOfMinute(first_hit) AS minute,
  uniq(session_id) as visits,
  sum(hits) AS total_hits,
  avg(duration) AS avg_session_length,
  100 * countIf(hits = 1) / count() AS bounce_rate
FROM sessions
GROUP BY minute, host
ORDER BY minute ASC`,
  orderBy: "minute",
};
