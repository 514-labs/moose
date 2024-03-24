export const analyticsPages = (table: string) => {
  return `SELECT
    toDate(timestamp) AS date,
    device,
    browser,
    location,
    pathname,
    uniqState(session_id) AS visits,
    countState() AS hits
FROM ${table}
GROUP BY date, device, browser, location, pathname`;
};
