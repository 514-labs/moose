export default {
  select: `
    SELECT
        hostname,
        pathname,
        count(pathname) AS hits
    FROM PageViewProcessed_0_0
    WHERE hostname != 'development'
    GROUP BY hostname, pathname
    ORDER BY hits DESC
  `,
  orderBy: "hits",
};
