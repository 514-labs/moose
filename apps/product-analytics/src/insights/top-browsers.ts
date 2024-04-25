import { analyticsQuery } from "./analytics_mv";
import { analyticsPages } from "./analytics_pages";
import { eventTables } from "../app/events";
import { DateRange, rangeToNum } from "./time-query";
import { createCTE } from "./util";

const topBrowsers = (range: DateRange, table: string) => {
  return `
    select pathname, uniqMerge(visits) as visits, countMerge(hits) as hits
    from ${table}
    where
timestamp >= timestampAdd(today(), interval -${rangeToNum[range]} day)
and timestamp <= today()
    group by pathname
    order by hits desc
    limit 10`;
};

enum TopBrowserCTE {
  analytics = "analytics",
  pages = "pages",
}
export const topBrowsersQuery = (range: DateRange) => {
  const ctes = {
    [TopBrowserCTE.analytics]: analyticsQuery(eventTables[0].tableName),
    [TopBrowserCTE.pages]: analyticsPages(TopBrowserCTE.analytics),
  };

  return createCTE(ctes) + topBrowsers(range, TopBrowserCTE.pages);
};
