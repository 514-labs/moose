export const analyticsPages = (table: string) => {
  return `SELECT
    toDate(timestamp) AS timestamp,
    device,
    browser,
    location,
    pathname,
    uniqState(session_id) AS visits,
    countState() AS hits
FROM ${table}
GROUP BY timestamp, device, browser, location, pathname`;
};
