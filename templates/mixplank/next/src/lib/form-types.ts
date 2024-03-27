export enum MetricOptions {
  Total_Events = "total_events",
  Total_Sessions = "total_sessions",
  Aggregated_Property = "aggregated_property",
}

export enum AggregateFunctions {
  Sum = "sum",
  Min = "min",
}

interface AggregatedProperty {
  aggregatedProperty: AggregateFunctions;
}

type Metric =
  | MetricOptions.Total_Events
  | MetricOptions.Total_Sessions
  | AggregatedProperty;

export interface MetricForm {
  event_name: string;
  metric: Metric;
}
