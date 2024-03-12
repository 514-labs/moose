import { EventTable } from "./event-tables";
import { DateRange, createDateStub } from "./time-query";

export function tableQuery(events: EventTable[], dateRange: DateRange) {
    const all_tables = events.map(event => `SELECT  '${event.eventName}' AS event_name, time, distinct_id FROM ${event.tableName} ${createDateStub(dateRange)}`)
        .join('\nUNION ALL\n')
    return `WITH CombinedEvents AS (
            ${all_tables}
        ) FROM CombinedEvents
        SELECT distinct_id, time, event_name
    ORDER BY time;`
}
