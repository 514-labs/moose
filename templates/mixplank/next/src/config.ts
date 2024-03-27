import { MetricForm } from "./lib/form-types";

export interface EventTable {
  eventName: string;
  tableName: string;
  modelName: string;
}

export const pageViewEvent = {
  eventName: "Page View",
  tableName: "PageViewEvent_0_0_trigger",
  modelName: "PageViewEvent",
};

const virtualEvents = [
  {
    eventName: "Session Start",
    tableName: "virtual_session",
    modelName: "Session",
  },
];

export const eventTables: EventTable[] = [
  pageViewEvent,
  {
    eventName: "Add To Cart",
    tableName: "AddToCartEvent_0_0_trigger",
    modelName: "AddToCartEvent",
  },
  ...virtualEvents,
];

export const eventNameMap = eventTables.reduce(
  (acc, cur) => ({
    ...acc,
    [cur.eventName]: cur,
  }),
  {}
);

function isDefined<T>(argument: T | undefined): argument is T {
  return argument !== undefined;
}

export function eventConfigFromNames(eventNames: MetricForm[]) {
  const validEvents = eventNames
    .map((ev) => eventTables.find((table) => table.tableName == ev.event_name))
    .filter(isDefined);
  return validEvents;
}
