import { EventTable } from "./event-tables";
import { DateRange, createDateStub } from "./time-query";

export function tableQuery(events: EventTable[], dateRange: DateRange) {
    const all_tables = events.map(event => `SELECT  '${event.eventName}' AS event_name, timestamp, session_id, pathname FROM ${event.tableName} ${createDateStub(dateRange)}`)
        .join('\nUNION ALL\n')
    return `WITH CombinedEvents AS (
            ${all_tables}
        ) FROM CombinedEvents
        SELECT session_id, timestamp, event_name, pathname
    ORDER BY timestamp;`
}
