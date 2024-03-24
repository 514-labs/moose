import { EventTable } from "./event-tables";

export const analyticsQuery = (events: EventTable[]) => {
  return `SELECT
    timestamp,
    session_id,
    location,
    referrer,
    pathname,
    href,
    case
        when match(user_agent, 'wget|ahrefsbot|curl|urllib|bitdiscovery|https://|googlebot')
        then 'bot'
        when match(user_agent, 'android')
        then 'mobile-android'
        when match(user_agent, 'ipad|iphone|ipod')
        then 'mobile-ios'
        else 'desktop'
    END as device,
    case
        when match(user_agent, 'firefox')
        then 'firefox'
        when match(user_agent, 'chrome|crios')
        then 'chrome'
        when match(user_agent, 'opera')
        then 'opera'
        when match(user_agent, 'msie|trident')
        then 'ie'
        when match(user_agent, 'iphone|ipad|safari')
        then 'safari'
        else 'Unknown'
    END as browser
    FROM ${events[0].tableName}`;
};
