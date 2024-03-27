export const analyticsSessions = (table: string) => {
  return `SELECT
    toStartOfHour(timestamp) AS date,
    session_id,
    anySimpleState(device) AS device,
    anySimpleState(browser) AS browser,
    anySimpleState(location) AS location,
    minSimpleState(timestamp) AS first_hit,
    maxSimpleState(timestamp) AS latest_hit,
    countState() AS hits
FROM ${table}
GROUP BY date, session_id`;
};
