import { EventTable } from "./event-tables";

interface FunnelQuery {
    events: EventTable[]
}

export function createCombinedEvents(events: EventTable[]) {
    return events.map(event => `SELECT distinct_id, time, '${event.eventName}' AS event_name FROM ${event.tableName}`)
        .join('\nUNION ALL\n')
}

function windowFunnelEventsList(events: EventTable[]) {
    return events.map(event => `event_name = '${event.eventName}'`).join(',')
}

export const createFunnelQuery = (queryParams: FunnelQuery) => {
    if (queryParams.events.length == 0) {
        return null;
    }
    return `
        WITH CombinedEvents AS (
            ${createCombinedEvents(queryParams.events)}
        ),
        LevelCounts AS ( 
            SELECT distinct_id,
                windowFunnel(60*60*24*30, 'strict_increase')(time,
                    ${windowFunnelEventsList(queryParams.events)}
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
