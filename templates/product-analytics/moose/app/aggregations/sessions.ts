// This is query is meant to summarize data that is recorded on an individual session basis.
export default {
  select: `
    SELECT 
      session_id,
      minSimpleState(timestamp) AS first_hit,
      maxSimpleState(timestamp) AS latest_hit,
      maxSimpleState(timestamp) - minSimpleState(timestamp) AS duration,
      count() AS hits,
      anyState(hostname) AS host,
      groupArray(href) AS user_journey,
      anyLastSimpleState(href) AS last_page 
    FROM PageViewProcessed
    WHERE hostname != 'development' 
    GROUP BY session_id`,
  orderBy: "first_hit",
};
