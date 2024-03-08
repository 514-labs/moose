import { EventTable } from "./event-tables";

export function tableQuery(events: EventTable[]) {
    const all_tables = events.map(event => `SELECT  '${event.eventName}' AS event_name, time, distinct_id, city FROM ${event.tableName}`)
        .join('\nUNION ALL\n')
    return `WITH CombinedEvents AS (
            ${all_tables}
        ) FROM CombinedEvents
        SELECT distinct_id, time, event_name, city
    ORDER BY time;`
}
