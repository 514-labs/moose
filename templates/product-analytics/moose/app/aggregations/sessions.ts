// This is query is meant to summarize data that is recorded on an individual session basis.
export default {
  select: `
    SELECT 
      session_id,
      minSimpleState(timestamp) AS first_hit,
      maxSimpleState(timestamp) AS latest_hit,
      maxSimpleState(timestamp) - minSimpleState(timestamp) AS duration,
      countState() AS hits,
      anyState(hostname) AS host,
      case when min(timestamp) = max(timestamp) then 1 else 0 end as is_bounce,
      groupArray(href) AS user_journey,
      anyLastSimpleState(href) AS last_page 
    FROM PageViewProcessed_0_0
    WHERE hostname != 'development' 
    GROUP BY session_id`,
  orderBy: "first_hit",
};
