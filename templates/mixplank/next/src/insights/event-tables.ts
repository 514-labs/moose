export interface EventTable {
    eventName: string;
    tableName: string;
}

export const eventTables: EventTable[] = [{
    eventName: 'Page View',
    tableName: 'PageViewEvent_0_0_trigger'
}];
/** 
, {
    eventName: 'CTA Click',
    tableName: 'CTAClickEvent_0_0_trigger'
},
{
    eventName: 'Nav Click',
    tableName: 'NavClickEvent_0_0_trigger'
}];
*/

export const eventNameMap = eventTables.reduce((acc, cur) => ({
    ...acc,
    [cur.eventName]: cur
}), {})