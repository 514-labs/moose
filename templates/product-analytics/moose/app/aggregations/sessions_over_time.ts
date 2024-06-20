export default {
  select: `
  SELECT 
  host,
  toStartOfMinute(first_hit) AS minute,
  uniq(session_id) as visits,
  countMerge(hits) AS total_hits,
  avg(duration) AS avg_session_length,
100 * count(is_bounce) / countMerge(hits) AS bounce_rate
FROM sessions
GROUP BY minute, host
ORDER BY minute ASC`,
  orderBy: "minute",
};
