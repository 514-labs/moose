import { MetricList } from "@/app/types";
import { QueryFormData } from "@/components/query-form/query-form";

export async function getMetricList() {
  const response = await fetch("http://localhost:4000/consumption/metric_list");

  if (!response.ok) {
    throw new Error("Network response was not ok");
  }
  const data = (await response.json()) as MetricList;

  return data.map(({ metricName }) => ({
    val: metricName,
    label: metricName,
  }));
}

export interface MetricAttributes {
  [key: string]: string[];
}

export async function getMetricAttributes(metricName: string) {
  const response = await fetch(
    "http://localhost:4000/consumption/metric_attributes?metricName=" +
      metricName,
  );

  if (!response.ok) {
    throw new Error("Network response was not ok");
  }
  return (await response.json()) as MetricAttributes;
}

export interface MetricQuery {
  query: QueryFormData;
}

export async function getMetric({ query }: MetricQuery) {
  const encodedQuery = encodeURIComponent(JSON.stringify(query));

  const response = await fetch(
    "http://localhost:4000/consumption/metric_query?query=" + encodedQuery,
  );

  if (!response.ok) {
    throw new Error("Network response was not ok");
  }
  return await response.json();
}

export interface QueryParams {
  metricName: string;
  propertyName: string;
  filter: string;
}

export async function getMetricCommonProperties({
  metricName,
  propertyName,
  filter,
}: QueryParams) {
  const response = await fetch(
    `http://localhost:4000/consumption/metric_common_properties?metricName=${metricName}&propertyName=${propertyName}&filters=${filter}`,
  );

  if (!response.ok) {
    throw new Error("Network response was not ok");
  }
  return await response.json();
}

export interface MetricQuery {
  query: QueryFormData;
}

export async function getMetricTimeSeries({ query }: MetricQuery) {
  const encodedQuery = encodeURIComponent(JSON.stringify(query));

  const response = await fetch(
    "http://localhost:4000/consumption/metric_timeseries?query=" + encodedQuery,
  );

  if (!response.ok) {
    throw new Error("Network response was not ok");
  }
  const data = await response.json();

  return data.map((d: any) => ({
    ...d,
    timeseries: d.timeseries.map((point) => [
      new Date(point[0]),
      parseInt(point[1]),
    ]),
  }));

  /*
  return data.map((d: any) => ({
    hits: parseInt(d.hits),
    timestamp: new Date(d.timestamp),
  }));
  */
}
