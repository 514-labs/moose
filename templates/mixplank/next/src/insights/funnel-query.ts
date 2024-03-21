import { EventTable } from "./event-tables";
import { DateRange, createDateStub } from "./time-query";

interface FunnelQuery {
    events: EventTable[]
}

export function createCombinedEvents(events: EventTable[], dateRange: DateRange) {
    return events.map(event => `SELECT session_id, timestamp, '${event.eventName}' AS event_name FROM ${event.tableName} ${createDateStub(dateRange)}`)
        .join('\nUNION ALL\n')
}

function windowFunnelEventsList(events: EventTable[]) {
    return events.map(event => `event_name = '${event.eventName}'`).join(',')
}

export const createFunnelQuery = (queryParams: FunnelQuery, dateRange: DateRange) => {
    const cleanEvents = queryParams.events.filter((ev) => ev.eventName != null)
    if (!cleanEvents || cleanEvents.length == 0) {
        return '';
    }
    return `
        WITH CombinedEvents AS (
            ${createCombinedEvents(cleanEvents, dateRange)}
        ),
        LevelCounts AS ( 
            SELECT session_id,
                windowFunnel(60*60*24*30, 'strict_increase')(timestamp,
                    ${windowFunnelEventsList(cleanEvents)}
                ) AS level
            FROM CombinedEvents
            GROUP BY ALL
            HAVING level >= 1
        )
        SELECT level, sum(count()) OVER (ORDER BY level DESC) AS count
        FROM LevelCounts
        GROUP BY level
        ORDER BY level ASC;`
};
